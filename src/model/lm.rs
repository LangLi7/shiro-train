//! Das echte, trainierbare Byte-Modell auf Burn (Entscheidung **D-010**).
//!
//! Architektur gemaess `docs/byte-model-roadmap.md`: Embedding -> 12 (bzw. n)
//! Schichten aus RMSNorm + kausaler Multi-Head-Attention mit **RoPE** + SwiGLU,
//! Abschluss-Norm, Ausgabekopf mit Bias. Embeddings sind **nicht** geteilt
//! (`tie_embeddings = false`, Entscheidung o2), damit `ModelConfig::param_count()`
//! und die real allokierten Tensoren exakt uebereinstimmen.
//!
//! **Warum Burn 0.22 hier anders aussieht als in aelteren Beispielen:** seit 0.22
//! tragen Tensoren und Module **keinen Backend-Parameter** mehr. `Tensor<D, K>` und
//! `Linear` sind backendfrei; das Geraet entscheidet zur Laufzeit
//! (`Device::flex()`, `Device::cuda(0)`, `Device::default()`), und Autodiff wird als
//! Dekorator am Geraet eingeschaltet (`device.autodiff()`). Genau das ist die
//! Portabilitaet, die D-014 beschreibt.

use burn::module::Param;
use burn::nn::loss::CrossEntropyLossConfig;
use burn::nn::{Embedding, Linear, RmsNorm};
use burn::optim::GradientsParams;

use crate::train::state::TrainState;
use burn::prelude::*;
use burn::tensor::activation;

use crate::config::ModelConfig;
use crate::data::loader::Batch;
use crate::error::{Context, Result};

/// Architektur-Metadaten, die **nicht** zu den Gewichten gehoeren.
///
/// Bewusst neben dem Modul gefuehrt: Burn-Module enthalten nur Tensoren, alles
/// andere waere ein Sonderfall im `Module`-Derive.
#[derive(Debug, Clone, PartialEq)]
pub struct Arch {
    pub d_model: usize,
    pub n_layers: usize,
    pub n_heads: usize,
    pub head_dim: usize,
    pub d_ff: usize,
    pub vocab: usize,
    pub max_seq_len: usize,
    pub rope_theta: f32,
}

impl Arch {
    pub fn from_config(cfg: &ModelConfig) -> Result<Self> {
        cfg.validate()?;
        Ok(Self {
            d_model: cfg.d_model,
            n_layers: cfg.n_layers,
            n_heads: cfg.n_heads,
            head_dim: cfg.d_model / cfg.n_heads,
            d_ff: cfg.d_ff,
            vocab: cfg.vocab_size,
            max_seq_len: cfg.max_seq_len,
            rope_theta: cfg.rope_theta,
        })
    }
}

/// Vorberechnete RoPE-Faktoren fuer alle Positionen bis `max_seq_len`.
///
/// Die Faktoren haengen nur von Position und Kopfbreite ab, nicht von den Daten -
/// sie werden einmal gebaut und dann in jeden Forward-Pass hineingereicht.
/// Rotiert wird paarweise `(2j, 2j+1)`, identisch zu `ops::apply_rope_inplace`.
pub struct Rope {
    pub cos: Vec<f32>,
    pub sin: Vec<f32>,
    pub seq_len: usize,
    pub half: usize,
}

impl Rope {
    pub fn new(arch: &Arch, seq_len: usize) -> Self {
        let half = arch.head_dim / 2;
        let mut cos = vec![0.0f32; seq_len * half];
        let mut sin = vec![0.0f32; seq_len * half];
        for pos in 0..seq_len {
            for j in 0..half {
                let inv_freq = arch.rope_theta.powf(-2.0 * j as f32 / arch.head_dim as f32);
                let angle = pos as f32 * inv_freq;
                cos[pos * half + j] = angle.cos();
                sin[pos * half + j] = angle.sin();
            }
        }
        Self {
            cos,
            sin,
            seq_len,
            half,
        }
    }

    /// Materialisiert die Positionsfaktoren einmal je Forward-Pass.
    ///
    /// Dieselben Tensoren werden fuer Q und K sowie alle Schichten wiederverwendet,
    /// statt die Host-Daten pro Projektion erneut auf das Geraet zu kopieren.
    fn tensors(&self, seq_len: usize, device: &Device) -> (Tensor<4>, Tensor<4>) {
        self.tensors_at(0, seq_len, device)
    }

    /// Materialisiert einen Positionsbereich fuer die inkrementelle Generierung.
    fn tensors_at(&self, start: usize, seq_len: usize, device: &Device) -> (Tensor<4>, Tensor<4>) {
        let begin = start * self.half;
        let count = seq_len * self.half;
        let shape = [1, seq_len, 1, self.half];
        let cos = Tensor::<4>::from_data(
            TensorData::new(self.cos[begin..begin + count].to_vec(), shape),
            device,
        );
        let sin = Tensor::<4>::from_data(
            TensorData::new(self.sin[begin..begin + count].to_vec(), shape),
            device,
        );
        (cos, sin)
    }

    /// Wendet RoPE auf `[batch, seq, heads, head_dim]` an und erhaelt die
    /// benachbarte Reihenfolge jedes Rotationspaares `(2j, 2j+1)`.
    fn apply(&self, x: Tensor<4>, arch: &Arch, cos: &Tensor<4>, sin: &Tensor<4>) -> Tensor<4> {
        let [b, s, h, hd] = x.dims();
        let half = self.half;
        debug_assert_eq!(hd, arch.head_dim);
        let x5 = x.reshape([b, s, h, half, 2]);
        let even = x5.clone().slice([0..b, 0..s, 0..h, 0..half, 0..1]);
        let odd = x5.slice([0..b, 0..s, 0..h, 0..half, 1..2]);
        let even = even.reshape([b, s, h, half]);
        let odd = odd.reshape([b, s, h, half]);

        let out_even = even.clone() * cos.clone() - odd.clone() * sin.clone();
        let out_odd = even * sin.clone() + odd * cos.clone();
        let paired = Tensor::cat(
            vec![
                out_even.reshape([b, s, h, half, 1]),
                out_odd.reshape([b, s, h, half, 1]),
            ],
            4,
        );
        paired.reshape([b, s, h, hd])
    }
}

/// Eine Transformer-Schicht: Attention-Block und SwiGLU-Block, je mit RMSNorm.
#[derive(Module, Debug)]
pub struct LmLayer {
    norm_attn: RmsNorm,
    wq: Linear,
    wk: Linear,
    wv: Linear,
    wo: Linear,
    norm_ffn: RmsNorm,
    w_gate: Linear,
    w_up: Linear,
    w_down: Linear,
}

/// Das vollstaendige Modell.
#[derive(Module, Debug)]
pub struct Lm {
    emb: Embedding,
    layers: Vec<LmLayer>,
    norm_final: RmsNorm,
    head: Linear,
}

/// Ergebnis eines Forward-Passes.
pub struct ForwardOut {
    /// Logits in `[batch * seq, vocab]` - genau die Form, die der Loss braucht.
    pub logits: Tensor<2>,
}

impl Lm {
    /// Baut das Modell **deterministisch** aus dem Seed der Konfiguration.
    ///
    /// **Warum nicht `LinearConfig::init(device)`:** Burn zieht seine Startwerte aus
    /// einem eigenen Generator, der sich nicht prozessuebergreifend reproduzieren
    /// laesst. Zwei Laeufe mit demselben Seed bekamen messbar verschiedene
    /// Startloss-Werte (5,6839 gegen 5,6723, Test `same_seed_gives_same_start_loss`).
    /// Fuer dieses Projekt ist das nicht akzeptabel: Roadmap und Pruefprotokoll
    /// verlangen reproduzierbare Laeufe, und `docs/verifikation.md` vergleicht
    /// Digests. Deshalb werden die Gewichte hier aus **SplitMix64** gezogen - demselben
    /// Generator, den auch `src/data/logic.rs` benutzt.
    ///
    /// Verteilung (damit die Zahlen nachpruefbar bleiben):
    /// - Matrizen (`wq/wk/wv/wo`, `w_gate/w_up/w_down`, Ausgabekopf): gleichverteilt
    ///   in `[-k, k]` mit `k = 1/sqrt(d_in)` - dieselbe Skala, die Burn fuer `Linear`
    ///   dokumentiert.
    /// - Einbettung: gleichverteilt in `[-k, k]` mit `k = 1/sqrt(d_model)`, passend
    ///   zur backendfreien Referenz in `src/model/tiny.rs`.
    /// - RMSNorm-Gewichte: 1 (Skalierung neutral).
    /// - Ausgabe-Bias: gleichverteilt wie die Kopfmatrix.
    pub fn new(cfg: &ModelConfig, seed: u64, device: &Device) -> Result<Self> {
        let arch = Arch::from_config(cfg)?;
        let eps = cfg.rms_norm_eps;
        let mut rng = SplitMix::new(seed);

        let layers = (0..arch.n_layers)
            .map(|_| LmLayer {
                norm_attn: rms_norm(eps, arch.d_model, device),
                wq: linear(arch.d_model, arch.d_model, &mut rng, device),
                wk: linear(arch.d_model, arch.d_model, &mut rng, device),
                wv: linear(arch.d_model, arch.d_model, &mut rng, device),
                wo: linear(arch.d_model, arch.d_model, &mut rng, device),
                norm_ffn: rms_norm(eps, arch.d_model, device),
                w_gate: linear(arch.d_model, arch.d_ff, &mut rng, device),
                w_up: linear(arch.d_model, arch.d_ff, &mut rng, device),
                w_down: linear(arch.d_ff, arch.d_model, &mut rng, device),
            })
            .collect();

        let emb_k = 1.0 / (arch.d_model as f32).sqrt();
        let emb_values = rng.uniform_vec(arch.vocab * arch.d_model, emb_k);
        let head_values = rng.uniform_vec(arch.d_model * arch.vocab, emb_k);
        let bias_values = rng.uniform_vec(arch.vocab, emb_k);

        Ok(Self {
            emb: Embedding {
                weight: param2(emb_values, [arch.vocab, arch.d_model], device),
            },
            layers,
            norm_final: rms_norm(eps, arch.d_model, device),
            head: Linear {
                weight: param2(head_values, [arch.d_model, arch.vocab], device),
                bias: Some(param1(bias_values, [arch.vocab], device)),
            },
        })
    }
    /// Kausale Maske als additive Matrix: 0 auf und unter der Diagonale, sonst -1e30.
    fn causal_mask(seq: usize, device: &Device) -> Tensor<4> {
        let mut mask = vec![0.0f32; seq * seq];
        for i in 0..seq {
            for j in 0..seq {
                if j > i {
                    mask[i * seq + j] = -1.0e30;
                }
            }
        }
        Tensor::from_data(TensorData::new(mask, [1, 1, seq, seq]), device)
    }

    /// Kausale Maske, wenn bereits `past` Schluesselpositionen im KV-Cache liegen.
    fn causal_mask_with_past(seq: usize, past: usize, device: &Device) -> Tensor<4> {
        let total = past + seq;
        let mut mask = vec![0.0f32; seq * total];
        for i in 0..seq {
            for j in (past + i + 1)..total {
                mask[i * total + j] = -1.0e30;
            }
        }
        Tensor::from_data(TensorData::new(mask, [1, 1, seq, total]), device)
    }

    /// Forward-Pass fuer einen Batch: `tokens` ist `[batch, seq]`.
    pub fn forward(
        &self,
        tokens: Tensor<2, Int>,
        arch: &Arch,
        rope: &Rope,
        device: &Device,
    ) -> Tensor<2> {
        let [b, s] = tokens.dims();
        let mut x = self.emb.forward(tokens); // [b, s, d]
        let scale = 1.0 / (arch.head_dim as f32).sqrt();
        let mask = Self::causal_mask(s, device);
        let (rope_cos, rope_sin) = rope.tensors(s, device);

        for layer in &self.layers {
            // --- Attention ---
            let n = layer.norm_attn.forward(x.clone());
            let q = layer.wq.forward(n.clone());
            let k = layer.wk.forward(n.clone());
            let v = layer.wv.forward(n);
            let q = q.reshape([b, s, arch.n_heads, arch.head_dim]);
            let k = k.reshape([b, s, arch.n_heads, arch.head_dim]);
            let v = v.reshape([b, s, arch.n_heads, arch.head_dim]);
            let q = rope.apply(q, arch, &rope_cos, &rope_sin).swap_dims(1, 2); // [b, h, s, hd]
            let k = rope.apply(k, arch, &rope_cos, &rope_sin).swap_dims(1, 2);
            let v = v.swap_dims(1, 2);

            let scores = q.matmul(k.swap_dims(2, 3)) * scale + mask.clone();
            let probs = activation::softmax(scores, 3);
            let ctx = probs.matmul(v); // [b, h, s, hd]
            let ctx = ctx.swap_dims(1, 2).reshape([b, s, arch.d_model]);
            x = x + layer.wo.forward(ctx);

            // --- SwiGLU ---
            let n2 = layer.norm_ffn.forward(x.clone());
            // SwiGLU uses SiLU(x) = x * sigmoid(x) as its gate activation.
            let gate = layer.w_gate.forward(n2.clone());
            let gate = gate.clone() * activation::sigmoid(gate);
            let up = layer.w_up.forward(n2);
            x = x + layer.w_down.forward(gate * up);
        }

        let x = self.norm_final.forward(x);
        let logits = self.head.forward(x); // [b, s, vocab]
        logits.reshape([b * s, arch.vocab])
    }

    /// Inkrementeller Forward-Pass fuer die Generierung mit einem KV-Cache.
    ///
    /// Bei einem Prefill enthaelt `tokens` den ganzen Kontext. Danach kann ein
    /// einzelnes neues Token durchgereicht werden; K/V der vorherigen Positionen
    /// werden wiederverwendet statt alle Schichten fuer den Kontext erneut zu
    /// berechnen. Der Cache ist lokal pro Generierungslauf und beruehrt Gewichte
    /// oder Trainingszustand nicht.
    fn forward_cached(
        &self,
        tokens: Tensor<2, Int>,
        arch: &Arch,
        rope: &Rope,
        cache: &mut [Option<(Tensor<4>, Tensor<4>)>],
        device: &Device,
    ) -> Result<Tensor<2>> {
        if cache.len() != self.layers.len() {
            crate::error::bail!(
                "KV-Cache hat {} Schichten, Modell hat {}",
                cache.len(),
                self.layers.len()
            );
        }
        let [b, s] = tokens.dims();
        if s == 0 {
            crate::error::bail!("KV-Cache kann keinen leeren Tokenbereich verarbeiten");
        }

        let mut past_len = None;
        for entry in cache.iter() {
            match entry {
                Some((key, value)) => {
                    let key_len = key.dims()[2];
                    if value.dims()[2] != key_len {
                        crate::error::bail!("KV-Cache hat unterschiedliche Key-/Value-Laengen");
                    }
                    if past_len.is_some_and(|known| known != key_len) {
                        crate::error::bail!("KV-Cache-Schichten haben unterschiedliche Laengen");
                    }
                    past_len = Some(key_len);
                }
                None if past_len.is_some() => {
                    crate::error::bail!("KV-Cache ist nur teilweise befuellt");
                }
                None => {}
            }
        }
        let past_len = past_len.unwrap_or(0);
        if cache.iter().any(Option::is_none) && past_len > 0 {
            crate::error::bail!("KV-Cache ist nur teilweise befuellt");
        }
        if past_len + s > rope.seq_len {
            crate::error::bail!(
                "KV-Cache-Kontext {} ueberschreitet RoPE-Fenster {}",
                past_len + s,
                rope.seq_len
            );
        }

        let mut x = self.emb.forward(tokens); // [b, s, d]
        let scale = 1.0 / (arch.head_dim as f32).sqrt();
        let mask = Self::causal_mask_with_past(s, past_len, device);
        let (rope_cos, rope_sin) = rope.tensors_at(past_len, s, device);

        for (layer_index, layer) in self.layers.iter().enumerate() {
            let n = layer.norm_attn.forward(x.clone());
            let q = layer.wq.forward(n.clone());
            let k = layer.wk.forward(n.clone());
            let v = layer.wv.forward(n);
            let q = q.reshape([b, s, arch.n_heads, arch.head_dim]);
            let k = k.reshape([b, s, arch.n_heads, arch.head_dim]);
            let v = v.reshape([b, s, arch.n_heads, arch.head_dim]);
            let q = rope.apply(q, arch, &rope_cos, &rope_sin).swap_dims(1, 2);
            let k = rope.apply(k, arch, &rope_cos, &rope_sin).swap_dims(1, 2);
            let v = v.swap_dims(1, 2);

            let (all_k, all_v) = match &cache[layer_index] {
                Some((past_k, past_v)) => (
                    Tensor::cat(vec![past_k.clone(), k], 2),
                    Tensor::cat(vec![past_v.clone(), v], 2),
                ),
                None => (k, v),
            };
            cache[layer_index] = Some((all_k.clone(), all_v.clone()));

            let scores = q.matmul(all_k.swap_dims(2, 3)) * scale + mask.clone();
            let probs = activation::softmax(scores, 3);
            let ctx = probs.matmul(all_v);
            let ctx = ctx.swap_dims(1, 2).reshape([b, s, arch.d_model]);
            x = x + layer.wo.forward(ctx);

            let n2 = layer.norm_ffn.forward(x.clone());
            let gate = layer.w_gate.forward(n2.clone());
            let gate = gate.clone() * activation::sigmoid(gate);
            let up = layer.w_up.forward(n2);
            x = x + layer.w_down.forward(gate * up);
        }

        let x = self.norm_final.forward(x);
        let logits = self.head.forward(x);
        Ok(logits.reshape([b * s, arch.vocab]))
    }
}

/// Ein Trainingsschritt-Ergebnis inklusive der Kennzahlen, die der Bericht braucht.
pub struct StepOutcome {
    pub loss: f32,
    pub targets: usize,
}

/// Trainingszustand: Modell, Optimierer, Architektur, RoPE.
pub struct Trainer {
    pub model: Lm,
    /// Erste AdamW-Momentenschaetzung, **je Parameter ein Tensor auf dem Geraet**.
    ///
    /// Reihenfolge identisch zu `Lm::flat_params`. Bis Runde 317 lag der Zustand als
    /// `Vec<f32>` auf dem Host. Das erzwang je Schritt rund 6 GB PCIe-Verkehr
    /// (Gradienten herunter, Parameter herunter, Parameter hinauf) und eine
    /// einthreadige AdamW-Schleife ueber 509 Mio Werte: gemessen 5 523,6 ms je
    /// Schritt bei 15-48 % GPU-Auslastung.
    pub m: Vec<Tensor<1>>,
    /// Zweite AdamW-Momentenschaetzung, gleiche Aufteilung wie [`Trainer::m`].
    pub v: Vec<Tensor<1>>,
    pub cfg: AdamCfg,
    pub arch: Arch,
    pub rope: Rope,
    pub device: Device,
    pub step: usize,
    pub seed: u64,
}

impl Trainer {
    pub fn new(cfg: &ModelConfig, seed: u64, device: &Device, seq_len: usize) -> Result<Self> {
        // Autodiff ist in Burn 0.22 ein **Dekorator am Geraet**: nur Tensoren, die auf
        // einem Autodiff-Geraet entstehen, koennen `backward()`.
        let device = device.clone().autodiff();
        let arch = Arch::from_config(cfg)?;
        let model = Lm::new(cfg, seed, &device)?;
        let numels = param_numels(&model)?;
        let m: Vec<Tensor<1>> = numels.iter().map(|n| Tensor::zeros([*n], &device)).collect();
        let v: Vec<Tensor<1>> = numels.iter().map(|n| Tensor::zeros([*n], &device)).collect();
        Ok(Self {
            model,
            m,
            v,
            cfg: AdamCfg::default(),
            rope: Rope::new(&arch, seq_len),
            arch,
            device: device.clone(),
            step: 0,
            seed,
        })
    }

    /// Nur-Lese-Zustand fuer `chat` und `eval`: **ohne** Optimiererzustand.
    ///
    /// `generate` und `eval_step` veraendern keine Gewichte, also brauchen sie weder
    /// `m`/`v` noch einen AdamW-Zustand. Der Vektor bleibt leer statt `param_count()`
    /// gross - so kostet ein Nur-Lese-Lauf keinen zusätzlichen Speicher.
    pub fn for_inference(model: Lm, arch: Arch, rope: Rope, device: Device, seed: u64) -> Self {
        Self {
            model,
            m: Vec::new(),
            v: Vec::new(),
            cfg: AdamCfg::default(),
            arch,
            rope,
            device,
            step: 0,
            seed,
        }
    }

    /// Ein Schritt: Forward, Cross-Entropy auf den gueltigen Positionen, AdamW.
    ///
    /// Die Zielpositionen kommen aus `Batch::valid` - Auffuellbytes am Zeilenende
    /// gehen **nicht** in den Loss ein (dieselbe Regel wie `ops::cross_entropy`).
    pub fn train_step(&mut self, batch: &Batch, lr: f64) -> Result<StepOutcome> {
        let batch_size = batch.batch_size;
        let seq_len = batch.seq_len;
        let toks: Vec<i32> = batch.input.iter().map(|b| *b as i32).collect();
        let device = &self.device;
        let x = Tensor::<2, Int>::from_data(TensorData::new(toks, [batch_size, seq_len]), device);

        let logits = self.model.forward(x, &self.arch, &self.rope, device);

        let mut idx: Vec<i32> = Vec::with_capacity(batch.total_valid());
        let mut y: Vec<i32> = Vec::with_capacity(batch.total_valid());
        for b in 0..batch_size {
            for t in 0..batch.valid[b].saturating_sub(1) {
                let flat = b * seq_len + t;
                idx.push(flat as i32);
                y.push(batch.target[flat] as i32);
            }
        }
        let n = idx.len();
        if n == 0 {
            return Ok(StepOutcome {
                loss: f32::NAN,
                targets: 0,
            });
        }
        let idx = Tensor::<1, Int>::from_data(TensorData::new(idx, [n]), device);
        let y = Tensor::<1, Int>::from_data(TensorData::new(y, [n]), device);
        let sel_logits = logits.select(0, idx);
        let loss = CrossEntropyLossConfig::new()
            .init(device)
            .forward(sel_logits, y);
        let loss_value = loss.clone().into_scalar();

        // Eigener AdamW-Schritt statt Burn's Optimierer: der Zustand (m, v) liegt damit
        // in **meiner** Parameterreihenfolge und ueberlebt einen Neuaufbau des Modells
        // (Burn's OptimizerRecord indiziert ueber ParamId). Ausserdem entfaellt der
        // Modell-Klon je Schritt, den `ModuleOptimizer::step` erzwingt.
        //
        // Gerechnet wird **auf dem Geraet**. Bis Runde 317 lagen `m` und `v` auf dem
        // Host und jeder Schritt schob rund 6 GB ueber PCIe; der Host sah nur den Loss.
        let grads = GradientsParams::from_grads(loss.backward(), &self.model);
        let gs = grads_device(&self.model, &grads)?;

        // Globales Norm-Clipping wie in `adamw_step`: die Summen bleiben Tensoren,
        // gelesen wird einmal statt je Parameter.
        let scale = if self.cfg.grad_clip > 0.0 {
            let mut acc = (gs[0].clone() * gs[0].clone()).sum();
            for g in &gs[1..] {
                acc = acc + (g.clone() * g.clone()).sum();
            }
            let norm_sum: f32 = acc.into_scalar::<f32>();
            let norm = norm_sum.sqrt();
            if norm > self.cfg.grad_clip {
                self.cfg.grad_clip / (norm + 1e-6)
            } else {
                1.0
            }
        } else {
            1.0
        };

        let n_step = self.step as u64 + 1;
        let args = AdamArgs {
            scale,
            lr: lr as f32,
            beta1: self.cfg.beta1,
            beta2: self.cfg.beta2,
            eps: self.cfg.eps,
            wd: self.cfg.weight_decay,
            bc1: 1.0 - self.cfg.beta1.powi(n_step as i32),
            bc2: 1.0 - self.cfg.beta2.powi(n_step as i32),
        };
        let mut k = 0usize;
        adam_all(&mut self.model, &gs, &mut self.m, &mut self.v, &mut k, &args)?;
        if k != gs.len() || k != self.m.len() {
            crate::error::bail!(
                "AdamW: {k} Parameter aktualisiert, {} Gradienten und {} Zustandsteile erwartet",
                gs.len(),
                self.m.len()
            );
        }
        self.step += 1;

        Ok(StepOutcome {
            loss: loss_value,
            targets: n,
        })
    }
}

/// Speichern und Laden des **vollstaendigen** Modells als safetensors.
///
/// Das ist das Auslieferungsformat aus `MODEL_FORMAT.md` (Abschnitt 1): Burn
/// schreibt die Gewichte ueber seinen eigenen `SafetensorsStore`, damit die Datei
/// nicht von einer Eigenimplementierung abhaengt. Die Gegenprobe (unabhaengiges
/// Lesen) macht `src/model/safetensors.rs`.
impl Lm {
    /// Schreibt alle Gewichte nach `path` (Elternordner wird angelegt).
    pub fn save_safetensors(&self, path: &std::path::Path) -> Result<()> {
        use burn::store::{ModuleSnapshot, SafetensorsStore};

        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Ordner {} nicht anlegbar", dir.display()))?;
        }
        let mut store = SafetensorsStore::from_file(path);
        self.save_into(&mut store)
            .map_err(|e| anyhow::anyhow!("safetensors schreiben fehlgeschlagen: {e}"))?;
        Ok(())
    }

    /// Laedt Gewichte aus `path` in ein bereits gebautes Modell.
    pub fn load_safetensors(mut self, path: &std::path::Path) -> Result<Self> {
        use burn::store::{ModuleSnapshot, SafetensorsStore};

        let mut store = SafetensorsStore::from_file(path);
        self.load_from(&mut store)
            .map_err(|e| anyhow::anyhow!("safetensors lesen fehlgeschlagen: {e}"))?;
        Ok(self)
    }
}

impl Trainer {
    /// Loss **ohne** Gewichtsaktualisierung - die Grundlage jeder Messung.
    pub fn eval_step(&self, batch: &Batch) -> StepOutcome {
        let batch_size = batch.batch_size;
        let seq_len = batch.seq_len;
        let toks: Vec<i32> = batch.input.iter().map(|b| *b as i32).collect();
        let device = &self.device;
        let x = Tensor::<2, Int>::from_data(TensorData::new(toks, [batch_size, seq_len]), device);
        let logits = self.model.forward(x, &self.arch, &self.rope, device);

        let mut idx: Vec<i32> = Vec::with_capacity(batch.total_valid());
        let mut y: Vec<i32> = Vec::with_capacity(batch.total_valid());
        for b in 0..batch_size {
            for t in 0..batch.valid[b].saturating_sub(1) {
                let flat = b * seq_len + t;
                idx.push(flat as i32);
                y.push(batch.target[flat] as i32);
            }
        }
        let n = idx.len();
        if n == 0 {
            return StepOutcome {
                loss: f32::NAN,
                targets: 0,
            };
        }
        let idx = Tensor::<1, Int>::from_data(TensorData::new(idx, [n]), device);
        let y = Tensor::<1, Int>::from_data(TensorData::new(y, [n]), device);
        let loss = CrossEntropyLossConfig::new()
            .init(device)
            .forward(logits.select(0, idx), y);
        StepOutcome {
            loss: loss.into_scalar(),
            targets: n,
        }
    }

    /// Autoregressive Generierung aus einem Byte-Prompt.
    ///
    /// Verwendet einen KV-Cache, damit K/V der bisherigen Bytes pro Schicht nicht
    /// bei jeder neuen Ausgabe erneut berechnet werden. Wird das Kontextfenster
    /// verschoben, wird der Cache neu aus dem sichtbaren Fenster aufgebaut.
    /// `temperature = 0.0` bedeutet Greedy (argmax).
    pub fn generate(
        &self,
        prompt: &[u8],
        max_new: usize,
        temperature: f32,
        top_k: usize,
        seed: u64,
    ) -> Result<Vec<u8>> {
        let mut ctx: Vec<u8> = prompt.to_vec();
        if ctx.is_empty() {
            crate::error::bail!("Prompt ist leer - das Byte-Modell braucht mindestens ein Byte");
        }
        let window = self.rope.seq_len.min(self.arch.max_seq_len);
        let mut rng = crate::model::lm::SplitMix::new(seed);
        let mut cache: Vec<Option<(Tensor<4>, Tensor<4>)>> =
            (0..self.model.layers.len()).map(|_| None).collect();
        let mut cached_len = 0usize;
        let mut cached_start = usize::MAX;

        for _ in 0..max_new {
            let start = ctx.len().saturating_sub(window);
            let input = &ctx[start..];
            let append_one = cached_start == start && cached_len + 1 == input.len();
            let (tokens, len) = if append_one {
                (
                    vec![*input.last().expect("Kontext ist nicht leer") as i32],
                    1,
                )
            } else {
                for layer_cache in &mut cache {
                    *layer_cache = None;
                }
                cached_len = 0;
                (
                    input.iter().map(|b| *b as i32).collect::<Vec<_>>(),
                    input.len(),
                )
            };
            let x = Tensor::<2, Int>::from_data(TensorData::new(tokens, [1, len]), &self.device);
            let logits =
                self.model
                    .forward_cached(x, &self.arch, &self.rope, &mut cache, &self.device)?;
            let last = (len - 1) as i32;
            let row = logits.select(
                0,
                Tensor::<1, Int>::from_data(TensorData::new(vec![last], [1]), &self.device),
            );
            let values: Vec<f32> = row
                .into_data()
                .try_to_vec::<f32>()
                .map_err(|e| anyhow::anyhow!("Logits nicht lesbar: {e}"))?;
            if values.len() != self.arch.vocab {
                crate::error::bail!(
                    "Logit-Zeile hat {} Werte, erwartet {}",
                    values.len(),
                    self.arch.vocab
                );
            }
            let next = pick_next(&values, temperature, top_k, &mut rng);
            ctx.push(next);
            cached_len += len;
            cached_start = start;
        }
        Ok(ctx)
    }
}

/// Waehlt das naechste Byte: Greedy bei `temperature <= 0`, sonst Top-k-Sampling.
fn pick_next(logits: &[f32], temperature: f32, top_k: usize, rng: &mut SplitMix) -> u8 {
    if temperature <= 0.0 {
        let mut best = 0usize;
        for (i, v) in logits.iter().enumerate() {
            if *v > logits[best] {
                best = i;
            }
        }
        return best as u8;
    }
    let mut order: Vec<usize> = (0..logits.len()).collect();
    order.sort_by(|a, b| {
        logits[*b]
            .partial_cmp(&logits[*a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let k = if top_k == 0 {
        order.len()
    } else {
        top_k.min(order.len())
    };
    let max = logits[order[0]];
    let mut probs = Vec::with_capacity(k);
    let mut sum = 0.0f32;
    for &i in &order[..k] {
        let p = ((logits[i] - max) / temperature).exp();
        probs.push(p);
        sum += p;
    }
    let mut r = rng.next_f32() * sum;
    for (idx, p) in probs.iter().enumerate() {
        r -= *p;
        if r <= 0.0 {
            return order[idx] as u8;
        }
    }
    order[k - 1] as u8
}

/// SplitMix64 - derselbe Generator wie in `src/data/logic.rs`, damit Sampling
/// reproduzierbar ist, ohne eine Zufallsabhaengigkeit einzufuehren.
pub struct SplitMix {
    state: u64,
}

impl SplitMix {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Gleichverteilt in [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Gleichverteilte Startwerte in `[-k, k]` aus dem eigenen Generator.
impl SplitMix {
    fn uniform_vec(&mut self, n: usize, k: f32) -> Vec<f32> {
        (0..n).map(|_| (self.next_f32() * 2.0 - 1.0) * k).collect()
    }
}

/// Matrix-Parameter aus flachen Werten.
fn param2(values: Vec<f32>, shape: [usize; 2], device: &Device) -> Param<Tensor<2>> {
    Param::from_tensor(Tensor::from_data(TensorData::new(values, shape), device))
}

/// Vektor-Parameter aus flachen Werten.
fn param1(values: Vec<f32>, shape: [usize; 1], device: &Device) -> Param<Tensor<1>> {
    Param::from_tensor(Tensor::from_data(TensorData::new(values, shape), device))
}

/// Linear ohne Bias, gleichverteilt in `[-1/sqrt(d_in), 1/sqrt(d_in)]`.
fn linear(d_in: usize, d_out: usize, rng: &mut SplitMix, device: &Device) -> Linear {
    let k = 1.0 / (d_in as f32).sqrt();
    Linear {
        weight: param2(rng.uniform_vec(d_in * d_out, k), [d_in, d_out], device),
        bias: None,
    }
}

/// RMSNorm mit neutraler Skalierung (`gamma = 1`).
fn rms_norm(eps: f32, d_model: usize, device: &Device) -> RmsNorm {
    RmsNorm {
        gamma: param1(vec![1.0; d_model], [d_model], device),
        epsilon: eps as f64,
    }
}

/// Prüfpfad für den Gradientenvergleich (Akzeptanzkriterium aus
/// `docs/abschluss-vorbereitungsphase.md` §6).
///
/// Der Vergleich „analytisch gegen numerisch" braucht drei Dinge, die das Training
/// selbst nicht braucht: Zugriff auf einen einzelnen Parameter, die Möglichkeit ihn
/// zu verändern, und einen Loss **ohne** Gewichtsaktualisierung samt Gradienten.
impl Lm {
    /// Der Gewichtstensor des Ausgabekopfs - der Prüftensor fuer die finite Differenz.
    pub fn head_weight(&self) -> Param<Tensor<2>> {
        self.head.weight.clone()
    }

    /// Ersetzt den Gewichtstensor des Ausgabekopfs durch flache Werte.
    ///
    /// Nur fuer Prüfzwecke: das Modell behaelt seine Form, der Parameter bekommt
    /// neue Zahlen (und damit eine neue `ParamId` - deshalb muss der Gradient
    /// **vor** der ersten Störung gezogen werden).
    pub fn set_head_weight(&mut self, values: Vec<f32>, device: &Device) -> Result<()> {
        let [d_in, d_out] = self.head.weight.val().dims();
        if values.len() != d_in * d_out {
            crate::error::bail!(
                "set_head_weight: {} Werte, erwartet {}",
                values.len(),
                d_in * d_out
            );
        }
        self.head.weight = param2(values, [d_in, d_out], device);
        Ok(())
    }
}

impl Trainer {
    /// Loss **und** Gradienten fuer einen Batch, ohne Gewichte zu veraendern.
    ///
    /// Der Gradient kommt aus Burn's Autodiff; er wird hier nicht ausgewertet,
    /// sondern an den Aufrufer zurueckgegeben (Prüfpfad).
    pub fn loss_and_grads(&self, batch: &Batch) -> (f32, GradientsParams) {
        let batch_size = batch.batch_size;
        let seq_len = batch.seq_len;
        let toks: Vec<i32> = batch.input.iter().map(|b| *b as i32).collect();
        let device = &self.device;
        let x = Tensor::<2, Int>::from_data(TensorData::new(toks, [batch_size, seq_len]), device);
        let logits = self.model.forward(x, &self.arch, &self.rope, device);

        let mut idx: Vec<i32> = Vec::with_capacity(batch.total_valid());
        let mut y: Vec<i32> = Vec::with_capacity(batch.total_valid());
        for b in 0..batch_size {
            for t in 0..batch.valid[b].saturating_sub(1) {
                let flat = b * seq_len + t;
                idx.push(flat as i32);
                y.push(batch.target[flat] as i32);
            }
        }
        let n = idx.len().max(1);
        let idx = Tensor::<1, Int>::from_data(TensorData::new(idx, [n]), device);
        let y = Tensor::<1, Int>::from_data(TensorData::new(y, [n]), device);
        let loss = CrossEntropyLossConfig::new()
            .init(device)
            .forward(logits.select(0, idx), y);
        let value = loss.clone().into_scalar();
        let grads = GradientsParams::from_grads(loss.backward(), &self.model);
        (value, grads)
    }
}

/// Flache Gewichte in **fester Reihenfolge** — die Grundlage von `TrainState` (B6).
///
/// Die Reihenfolge ist identisch zu `ModelConfig::param_count()`:
/// `emb.weight`, je Schicht `norm_attn.gamma · wq · wk · wv · wo · norm_ffn.gamma ·
/// w_gate · w_up · w_down`, dann `norm_final.gamma`, `head.weight`, `head.bias`.
/// Damit ist ein Modellstand **ohne** Burn-Speicherformat les- und schreibbar — die
/// Resume-Datei haengt nicht an der Bibliothek.
impl Lm {
    /// Alle Gewichte als ein flacher Vektor.
    pub fn flat_params(&self) -> Result<Vec<f32>> {
        let mut out: Vec<f32> = Vec::new();
        read2(&self.emb.weight, &mut out)?;
        for l in &self.layers {
            read1(&l.norm_attn.gamma, &mut out)?;
            read2(&l.wq.weight, &mut out)?;
            read2(&l.wk.weight, &mut out)?;
            read2(&l.wv.weight, &mut out)?;
            read2(&l.wo.weight, &mut out)?;
            read1(&l.norm_ffn.gamma, &mut out)?;
            read2(&l.w_gate.weight, &mut out)?;
            read2(&l.w_up.weight, &mut out)?;
            read2(&l.w_down.weight, &mut out)?;
        }
        read1(&self.norm_final.gamma, &mut out)?;
        read2(&self.head.weight, &mut out)?;
        match &self.head.bias {
            Some(b) => read1(b, &mut out)?,
            None => crate::error::bail!("Ausgabekopf ohne Bias - Reihenfolge unvollstaendig"),
        }
        Ok(out)
    }

    /// Setzt alle Gewichte aus einem flachen Vektor (Gegenstueck zu [`Lm::flat_params`]).
    pub fn set_flat_params(&mut self, values: &[f32], device: &Device) -> Result<()> {
        let mut cur = 0usize;
        write2(&mut self.emb.weight, values, &mut cur, device)?;
        for l in self.layers.iter_mut() {
            write1(&mut l.norm_attn.gamma, values, &mut cur, device)?;
            write2(&mut l.wq.weight, values, &mut cur, device)?;
            write2(&mut l.wk.weight, values, &mut cur, device)?;
            write2(&mut l.wv.weight, values, &mut cur, device)?;
            write2(&mut l.wo.weight, values, &mut cur, device)?;
            write1(&mut l.norm_ffn.gamma, values, &mut cur, device)?;
            write2(&mut l.w_gate.weight, values, &mut cur, device)?;
            write2(&mut l.w_up.weight, values, &mut cur, device)?;
            write2(&mut l.w_down.weight, values, &mut cur, device)?;
        }
        write1(&mut self.norm_final.gamma, values, &mut cur, device)?;
        write2(&mut self.head.weight, values, &mut cur, device)?;
        match self.head.bias.as_mut() {
            Some(b) => write1(b, values, &mut cur, device)?,
            None => crate::error::bail!("Ausgabekopf ohne Bias"),
        }
        if cur != values.len() {
            crate::error::bail!(
                "Gewichtsvektor hat {} Werte, verbraucht wurden {}",
                values.len(),
                cur
            );
        }
        Ok(())
    }
}

fn read1(p: &Param<Tensor<1>>, out: &mut Vec<f32>) -> Result<()> {
    let v: Vec<f32> = p
        .val()
        .into_data()
        .try_to_vec::<f32>()
        .map_err(|e| crate::error::anyhow!("Tensor nicht lesbar: {e}"))?;
    out.extend_from_slice(&v);
    Ok(())
}

fn read2(p: &Param<Tensor<2>>, out: &mut Vec<f32>) -> Result<()> {
    let v: Vec<f32> = p
        .val()
        .into_data()
        .try_to_vec::<f32>()
        .map_err(|e| crate::error::anyhow!("Tensor nicht lesbar: {e}"))?;
    out.extend_from_slice(&v);
    Ok(())
}

fn write1(
    p: &mut Param<Tensor<1>>,
    values: &[f32],
    cur: &mut usize,
    device: &Device,
) -> Result<()> {
    let [n] = p.val().dims();
    if *cur + n > values.len() {
        crate::error::bail!("Gewichtsvektor zu kurz bei Offset {}", cur);
    }
    *p = param1(values[*cur..*cur + n].to_vec(), [n], device);
    *cur += n;
    Ok(())
}

fn write2(
    p: &mut Param<Tensor<2>>,
    values: &[f32],
    cur: &mut usize,
    device: &Device,
) -> Result<()> {
    let [a, b] = p.val().dims();
    let n = a * b;
    if *cur + n > values.len() {
        crate::error::bail!("Gewichtsvektor zu kurz bei Offset {}", cur);
    }
    *p = param2(values[*cur..*cur + n].to_vec(), [a, b], device);
    *cur += n;
    Ok(())
}

/// Hyperparameter des eigenen AdamW.
#[derive(Debug, Clone)]
pub struct AdamCfg {
    pub beta1: f32,
    pub beta2: f32,
    pub eps: f32,
    pub weight_decay: f32,
    pub grad_clip: f32,
}

impl Default for AdamCfg {
    fn default() -> Self {
        Self {
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.0,
            grad_clip: 1.0,
        }
    }
}

/// AdamW auf flachen Vektoren (entkoppeltes Weight Decay, Bias-Korrektur, globales
/// Norm-Clipping). `step` ist **1-basiert**.
pub fn adamw_step(
    params: &mut [f32],
    grads: &[f32],
    m: &mut [f32],
    v: &mut [f32],
    step: u64,
    lr: f32,
    cfg: &AdamCfg,
) -> Result<()> {
    let n = params.len();
    if grads.len() != n || m.len() != n || v.len() != n {
        crate::error::bail!(
            "AdamW: Params {}, Gradienten {}, m {}, v {}",
            n,
            grads.len(),
            m.len(),
            v.len()
        );
    }
    if step == 0 {
        crate::error::bail!("AdamW braucht einen 1-basierten Schrittzaehler");
    }
    let scale = if cfg.grad_clip > 0.0 {
        let norm = grads.iter().map(|g| g * g).sum::<f32>().sqrt();
        if norm > cfg.grad_clip {
            cfg.grad_clip / (norm + 1e-6)
        } else {
            1.0
        }
    } else {
        1.0
    };
    let bc1 = 1.0 - cfg.beta1.powi(step as i32);
    let bc2 = 1.0 - cfg.beta2.powi(step as i32);
    for i in 0..n {
        let g = grads[i] * scale;
        params[i] -= lr * cfg.weight_decay * params[i];
        m[i] = cfg.beta1 * m[i] + (1.0 - cfg.beta1) * g;
        v[i] = cfg.beta2 * v[i] + (1.0 - cfg.beta2) * g * g;
        let mh = m[i] / bc1;
        let vh = v[i] / bc2;
        params[i] -= lr * mh / (vh.sqrt() + cfg.eps);
    }
    Ok(())
}

/// Anzahl Werte eines Tensors.
fn numel<const D: usize>(t: &Tensor<D>) -> usize {
    t.dims().iter().product()
}

/// Anzahl Werte je Parameter, in der Reihenfolge von `Lm::flat_params`.
fn param_numels(model: &Lm) -> Result<Vec<usize>> {
    let mut out: Vec<usize> = Vec::new();
    out.push(numel(&model.emb.weight.val()));
    for l in &model.layers {
        out.push(numel(&l.norm_attn.gamma.val()));
        out.push(numel(&l.wq.weight.val()));
        out.push(numel(&l.wk.weight.val()));
        out.push(numel(&l.wv.weight.val()));
        out.push(numel(&l.wo.weight.val()));
        out.push(numel(&l.norm_ffn.gamma.val()));
        out.push(numel(&l.w_gate.weight.val()));
        out.push(numel(&l.w_up.weight.val()));
        out.push(numel(&l.w_down.weight.val()));
    }
    out.push(numel(&model.norm_final.gamma.val()));
    out.push(numel(&model.head.weight.val()));
    match &model.head.bias {
        Some(b) => out.push(numel(&b.val())),
        None => crate::error::bail!("Ausgabekopf ohne Bias - Parameterreihenfolge unvollstaendig"),
    }
    Ok(out)
}

/// Gradienten je Parameter als flacher Tensor **auf dem Geraet**, in derselben
/// Reihenfolge wie `Lm::flat_params`.
fn grads_device(model: &Lm, grads: &GradientsParams) -> Result<Vec<Tensor<1>>> {
    let mut out: Vec<Tensor<1>> = Vec::new();
    out.push(gflat(&model.emb.weight, grads)?);
    for l in &model.layers {
        out.push(gflat(&l.norm_attn.gamma, grads)?);
        out.push(gflat(&l.wq.weight, grads)?);
        out.push(gflat(&l.wk.weight, grads)?);
        out.push(gflat(&l.wv.weight, grads)?);
        out.push(gflat(&l.wo.weight, grads)?);
        out.push(gflat(&l.norm_ffn.gamma, grads)?);
        out.push(gflat(&l.w_gate.weight, grads)?);
        out.push(gflat(&l.w_up.weight, grads)?);
        out.push(gflat(&l.w_down.weight, grads)?);
    }
    out.push(gflat(&model.norm_final.gamma, grads)?);
    out.push(gflat(&model.head.weight, grads)?);
    match &model.head.bias {
        Some(b) => out.push(gflat(b, grads)?),
        None => crate::error::bail!("Ausgabekopf ohne Bias - Gradientenreihenfolge unvollstaendig"),
    }
    Ok(out)
}

fn gflat<const D: usize>(p: &Param<Tensor<D>>, grads: &GradientsParams) -> Result<Tensor<1>> {
    let n = numel(&p.val());
    let t = grads
        .get::<D>(p.id)
        .ok_or_else(|| crate::error::anyhow!("Kein Gradient fuer Parameter {}", p.id.val()))?;
    Ok(t.reshape([n]))
}

/// Hyperparameter eines AdamW-Schritts auf dem Geraet.
struct AdamArgs {
    scale: f32,
    lr: f32,
    beta1: f32,
    beta2: f32,
    eps: f32,
    wd: f32,
    bc1: f32,
    bc2: f32,
}

/// AdamW fuer **einen** Parameter, vollstaendig auf dem Geraet.
///
/// Rechnet dieselbe Formel wie [`adamw_step`] (entkoppeltes Weight Decay,
/// Bias-Korrektur), nur ohne Umweg ueber den Host.
fn adam_one<const D: usize>(
    p: &mut Param<Tensor<D>>,
    g: &Tensor<1>,
    m: &mut Tensor<1>,
    v: &mut Tensor<1>,
    a: &AdamArgs,
) -> Result<()> {
    let dims = p.val().dims();
    let n = numel(&p.val());
    if g.dims()[0] != n || m.dims()[0] != n || v.dims()[0] != n {
        crate::error::bail!(
            "AdamW: Parameter {}, Gradient {}, m {}, v {}",
            n,
            g.dims()[0],
            m.dims()[0],
            v.dims()[0]
        );
    }
    let g = g.clone() * a.scale;
    let pv = p.val().reshape([n]);
    let pv = pv.clone() - pv.clone() * (a.lr * a.wd);
    let m_new = m.clone() * a.beta1 + g.clone() * (1.0 - a.beta1);
    let v_new = v.clone() * a.beta2 + (g.clone() * g) * (1.0 - a.beta2);
    let upd = (m_new.clone() / a.bc1) / ((v_new.clone() / a.bc2).sqrt() + a.eps);
    let p_new = pv - upd * a.lr;
    *m = m_new;
    *v = v_new;
    // `detach()` ist Pflicht: `Param::from_tensor` setzt require_grad und bricht bei
    // einem verfolgten Nicht-Blatt-Tensor ab ("Can't convert a non leaf tensor into a
    // tracked tensor"). Der neue Wert ist das Ergebnis einer Rechnung, also verfolgt.
    *p = Param::from_tensor(p_new.detach().reshape(dims));
    Ok(())
}

/// Holt den naechsten Parameter aus `gs`/`m`/`v` und aktualisiert ihn.
fn adv<const D: usize>(
    p: &mut Param<Tensor<D>>,
    gs: &[Tensor<1>],
    m: &mut [Tensor<1>],
    v: &mut [Tensor<1>],
    k: &mut usize,
    a: &AdamArgs,
) -> Result<()> {
    let i = *k;
    if i >= gs.len() || i >= m.len() || i >= v.len() {
        crate::error::bail!("AdamW: Index {i} ausserhalb von Gradienten oder Zustand");
    }
    adam_one(p, &gs[i], &mut m[i], &mut v[i], a)?;
    *k += 1;
    Ok(())
}

/// AdamW ueber alle Parameter, in der Reihenfolge von `Lm::flat_params`.
fn adam_all(
    model: &mut Lm,
    gs: &[Tensor<1>],
    m: &mut [Tensor<1>],
    v: &mut [Tensor<1>],
    k: &mut usize,
    a: &AdamArgs,
) -> Result<()> {
    adv(&mut model.emb.weight, gs, m, v, k, a)?;
    for i in 0..model.layers.len() {
        let l = &mut model.layers[i];
        adv(&mut l.norm_attn.gamma, gs, m, v, k, a)?;
        adv(&mut l.wq.weight, gs, m, v, k, a)?;
        adv(&mut l.wk.weight, gs, m, v, k, a)?;
        adv(&mut l.wv.weight, gs, m, v, k, a)?;
        adv(&mut l.wo.weight, gs, m, v, k, a)?;
        adv(&mut l.norm_ffn.gamma, gs, m, v, k, a)?;
        adv(&mut l.w_gate.weight, gs, m, v, k, a)?;
        adv(&mut l.w_up.weight, gs, m, v, k, a)?;
        adv(&mut l.w_down.weight, gs, m, v, k, a)?;
    }
    adv(&mut model.norm_final.gamma, gs, m, v, k, a)?;
    adv(&mut model.head.weight, gs, m, v, k, a)?;
    match model.head.bias.as_mut() {
        Some(b) => adv(b, gs, m, v, k, a)?,
        None => crate::error::bail!("Ausgabekopf ohne Bias - Aktualisierung unvollstaendig"),
    }
    Ok(())
}

/// AdamW-Zustand als flacher Host-Vektor, fuer die Standdatei.
fn flat_host(parts: &[Tensor<1>]) -> Result<Vec<f32>> {
    let mut out: Vec<f32> = Vec::new();
    for t in parts {
        let v: Vec<f32> = t
            .clone()
            .into_data()
            .try_to_vec::<f32>()
            .map_err(|e| crate::error::anyhow!("AdamW-Zustand nicht lesbar: {e}"))?;
        out.extend_from_slice(&v);
    }
    Ok(out)
}

/// Baut den AdamW-Zustand aus einem flachen Host-Vektor; die Laengen kommen aus `templates`.
fn device_from_flat(
    templates: &[Tensor<1>],
    values: &[f32],
    device: &Device,
) -> Result<Vec<Tensor<1>>> {
    let mut cur = 0usize;
    let mut out: Vec<Tensor<1>> = Vec::with_capacity(templates.len());
    for t in templates {
        let n = t.dims()[0];
        if cur + n > values.len() {
            crate::error::bail!("AdamW-Zustand zu kurz bei Offset {cur}");
        }
        let td = TensorData::new(values[cur..cur + n].to_vec(), [n]);
        out.push(Tensor::from_data(td, device));
        cur += n;
    }
    if cur != values.len() {
        crate::error::bail!(
            "AdamW-Zustand hat {} Werte, verbraucht wurden {cur}",
            values.len()
        );
    }
    Ok(out)
}

/// Speichern und Fortsetzen (Blocker **B6**).
impl Trainer {
    /// Schreibt Gewichte **und** AdamW-Zustand **und** Schrittnummer in **eine** Datei.
    pub fn save_state(&self, path: &std::path::Path, schedule: &str, lr: f32) -> Result<()> {
        TrainState {
            step: self.step as u64,
            seed: self.seed,
            lr,
            schedule: schedule.to_string(),
            params: self.model.flat_params()?,
            // Der Zustand liegt auf dem Geraet und wird nur hier heruntergeholt:
            // einmal je Checkpoint, nicht einmal je Schritt.
            m: flat_host(&self.m)?,
            v: flat_host(&self.v)?,
        }
        .save(path)
    }

    /// Setzt einen Lauf aus einem Stand fort: Gewichte, AdamW-Zustand und Schrittnummer.
    ///
    /// Der Lernratenplan braucht keinen eigenen Zustand — `trainer::learning_rate` ist
    /// eine reine Funktion von Schrittnummer und Konfiguration, und die Schrittnummer
    /// steht im Stand.
    pub fn resume_from(
        cfg: &ModelConfig,
        device: &Device,
        seq_len: usize,
        path: &std::path::Path,
    ) -> Result<Self> {
        let state = TrainState::load(path)?;
        if state.params.len() != cfg.param_count() {
            crate::error::bail!(
                "Stand hat {} Gewichte, die Konfiguration verlangt {}",
                state.params.len(),
                cfg.param_count()
            );
        }
        let mut t = Self::new(cfg, state.seed, device, seq_len)?;
        t.model.set_flat_params(&state.params, &t.device)?;
        let m_new = device_from_flat(&t.m, &state.m, &t.device)?;
        let v_new = device_from_flat(&t.v, &state.v, &t.device)?;
        t.m = m_new;
        t.v = v_new;
        t.step = state.step as usize;
        Ok(t)
    }
}

#[cfg(test)]
mod kv_cache_tests {
    use super::*;

    fn small_config(max_seq_len: usize) -> ModelConfig {
        ModelConfig {
            name: "kv-cache-test".to_string(),
            d_model: 32,
            n_layers: 2,
            n_heads: 2,
            d_ff: 64,
            vocab_size: 256,
            max_seq_len,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            tie_embeddings: false,
            dropout: 0.0,
        }
    }

    fn forward_tokens(
        model: &Lm,
        arch: &Arch,
        rope: &Rope,
        tokens: &[u8],
        device: &Device,
    ) -> Vec<f32> {
        let values = tokens.iter().map(|token| *token as i32).collect::<Vec<_>>();
        model
            .forward(
                Tensor::<2, Int>::from_data(TensorData::new(values, [1, tokens.len()]), device),
                arch,
                rope,
                device,
            )
            .into_data()
            .try_to_vec::<f32>()
            .expect("Logits lesbar")
    }

    fn assert_close(actual: &[f32], expected: &[f32]) {
        assert_eq!(actual.len(), expected.len());
        let max_error = actual
            .iter()
            .zip(expected)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(max_error < 1e-4, "maximale Logit-Abweichung: {max_error}");
    }

    #[test]
    fn cached_forward_matches_full_forward_and_rebuilds_after_window_shift() {
        let cfg = small_config(8);
        let device = Device::default();
        let arch = Arch::from_config(&cfg).expect("Architektur");
        let rope = Rope::new(&arch, cfg.max_seq_len);
        let model = Lm::new(&cfg, 7, &device).expect("Modell");
        let all_tokens = (0u8..16).collect::<Vec<_>>();
        let mut cache = (0..arch.n_layers).map(|_| None).collect::<Vec<_>>();
        let mut cached_len = 0usize;
        let mut cached_start = usize::MAX;

        for end in 1..=all_tokens.len() {
            let visible_start = end.saturating_sub(cfg.max_seq_len);
            let visible = &all_tokens[visible_start..end];
            let append_one = cached_start == visible_start && cached_len + 1 == visible.len();
            let (input, input_len) = if append_one {
                (vec![*visible.last().expect("Nicht leer") as i32], 1)
            } else {
                cache.iter_mut().for_each(|entry| *entry = None);
                cached_len = 0;
                (
                    visible
                        .iter()
                        .map(|token| *token as i32)
                        .collect::<Vec<_>>(),
                    visible.len(),
                )
            };
            let cached_logits = model
                .forward_cached(
                    Tensor::<2, Int>::from_data(TensorData::new(input, [1, input_len]), &device),
                    &arch,
                    &rope,
                    &mut cache,
                    &device,
                )
                .expect("KV-Forward")
                .into_data()
                .try_to_vec::<f32>()
                .expect("Cache-Logits lesbar");
            let full_logits = forward_tokens(&model, &arch, &rope, visible, &device);
            let vocab = arch.vocab;
            assert_close(
                &cached_logits[(input_len - 1) * vocab..input_len * vocab],
                &full_logits[(visible.len() - 1) * vocab..visible.len() * vocab],
            );
            cached_len += input_len;
            cached_start = visible_start;
        }
    }

    #[test]
    fn cached_generation_matches_uncached_greedy_past_context_window() {
        let cfg = small_config(8);
        let device = Device::default();
        let arch = Arch::from_config(&cfg).expect("Architektur");
        let rope = Rope::new(&arch, cfg.max_seq_len);
        let model = Lm::new(&cfg, 11, &device).expect("Modell");
        let trainer = Trainer::for_inference(model, arch, rope, device.clone(), 11);
        let prompt = b"byte";

        let mut expected = prompt.to_vec();
        for _ in 0..12 {
            let start = expected.len().saturating_sub(cfg.max_seq_len);
            let values = forward_tokens(
                &trainer.model,
                &trainer.arch,
                &trainer.rope,
                &expected[start..],
                &device,
            );
            let last = &values[values.len() - trainer.arch.vocab..];
            expected.push(pick_next(last, 0.0, 0, &mut SplitMix::new(0)));
        }

        let actual = trainer
            .generate(prompt, 12, 0.0, 0, 11)
            .expect("Cache-Generierung");
        assert_eq!(actual, expected);
    }
}
