//! Deterministisches Mini-Modell fuer den Smoke-Test.
//!
//! Das ist **nicht** das Zielmodell, sondern die kuerzeste Kette, die beweist, dass
//! die Umgebung traegt: Bytes rein, Forward-Pass durch die echten Bausteine der
//! Roadmap (Byte-Embedding, RMSNorm, RoPE, kausale Attention, SwiGLU), Kreuzentropie
//! raus, ein Update, das den Loss senkt, und ein reproduzierbarer Zustand.
//!
//! Was hier absichtlich fehlt:
//!
//! - **Autograd / Backpropagation** durch alle Schichten. Aktualisiert wird nur der
//!   Ausgabekopf, mit dem exakten Gradienten `softmax(logits) - onehot(target)`.
//!   Das ist ehrlich ein Platzhalter: es zeigt, dass Loss sinkt und Checkpoints
//!   stimmen, ersetzt aber kein Training. Der echte Backward-Pass kommt mit dem
//!   Backend aus Phase 0 (Candle oder Burn).
//! - **BF16/FP16.** Alle Werte sind `f32`. Roadmap-Vorgabe ist BF16-Autocast mit
//!   FP32-Gewichten; das setzt das Backend um, nicht diese Datei.
//!
//! Speicherlayout der Attention: `[n_heads][seq][head_dim]`, damit die kausale
//! Attention in `ops` ohne Umkopieren arbeiten kann. Beim Zusammenfuegen zur
//! Ausgabeprojektion wird deshalb pro Position explizit kopiert - das ist die
//! einzige Stelle, an der umgeordnet wird.

use crate::config::ModelConfig;
use crate::data::loader::Batch;
use crate::error::{Result, bail};
use crate::model::ops;

/// Deterministische Gewichtsinitialisierung (SplitMix64, wie im Datengenerator).
#[derive(Debug, Clone)]
pub struct InitRng {
    state: u64,
}

impl InitRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Gleichverteilt in `[-scale, scale)`.
    pub fn uniform(&mut self, scale: f32) -> f32 {
        let unit = (self.next_u64() >> 11) as f32 / (1u64 << 53) as f32; // [0,1)
        (unit * 2.0 - 1.0) * scale
    }

    pub fn vec(&mut self, n: usize, scale: f32) -> Vec<f32> {
        (0..n).map(|_| self.uniform(scale)).collect()
    }
}

/// RMSNorm startet bei 1.
fn ones(n: usize) -> Vec<f32> {
    vec![1.0; n]
}

/// Gewichte einer Schicht.
#[derive(Debug, Clone)]
pub struct LayerWeights {
    pub wq: Vec<f32>,
    pub wk: Vec<f32>,
    pub wv: Vec<f32>,
    pub wo: Vec<f32>,
    pub w_gate: Vec<f32>,
    pub w_up: Vec<f32>,
    pub w_down: Vec<f32>,
    pub norm_attn: Vec<f32>,
    pub norm_ffn: Vec<f32>,
}

/// Das Mini-Modell samt aktualisierbarem Ausgabekopf.
#[derive(Debug, Clone)]
pub struct TinyModel {
    pub config: ModelConfig,
    pub seed: u64,
    pub token_embedding: Vec<f32>,
    pub w_out: Vec<f32>,
    pub b_out: Vec<f32>,
    pub layers: Vec<LayerWeights>,
    pub norm_final: Vec<f32>,
}

/// Ergebnis eines Forward-Passes auf einem Fenster.
#[derive(Debug, Clone)]
pub struct ForwardOut {
    /// `[batch * seq * vocab]`.
    pub logits: Vec<f32>,
    /// Eingabe des Ausgabekopfs (normierter Reststrom), `[batch * seq * d_model]`.
    ///
    /// Wird zurueckgegeben, damit der Gradientenschritt den Forward-Pass nicht
    /// wiederholen muss. Ohne Backend gibt es keinen Graph, den man abfragen koennte.
    pub head_input: Vec<f32>,
}

impl TinyModel {
    /// Baut das Modell deterministisch aus `seed`.
    pub fn new(config: ModelConfig, seed: u64) -> Result<Self> {
        config.validate()?;
        let d = config.d_model;
        let mut rng = InitRng::new(seed);
        // Standardabweichung nach ueblicher Praxis: 1/sqrt(d).
        let s = 1.0 / (d as f32).sqrt();

        let layers = (0..config.n_layers)
            .map(|_| LayerWeights {
                wq: rng.vec(d * d, s),
                wk: rng.vec(d * d, s),
                wv: rng.vec(d * d, s),
                wo: rng.vec(d * d, s),
                w_gate: rng.vec(config.d_ff * d, s),
                w_up: rng.vec(config.d_ff * d, s),
                w_down: rng.vec(d * config.d_ff, s),
                norm_attn: ones(d),
                norm_ffn: ones(d),
            })
            .collect();

        Ok(Self {
            token_embedding: rng.vec(config.vocab_size * d, s),
            w_out: rng.vec(config.vocab_size * d, s),
            b_out: vec![0.0; config.vocab_size],
            layers,
            norm_final: ones(d),
            config,
            seed,
        })
    }

    /// Anzahl der Werte im Checkpoint (Ausgabekopf).
    pub fn checkpoint_len(&self) -> usize {
        self.w_out.len() + self.b_out.len()
    }

    /// Ausgabekopf als flacher Vektor - genau das, was in den Checkpoint geht.
    pub fn head_params(&self) -> Vec<f32> {
        let mut v = self.w_out.clone();
        v.extend_from_slice(&self.b_out);
        v
    }

    /// Setzt den Ausgabekopf aus einem flachen Vektor (Checkpoint laden).
    pub fn set_head_params(&mut self, params: &[f32]) -> Result<()> {
        if params.len() != self.checkpoint_len() {
            bail!(
                "Checkpoint hat {} Parameter, erwartet {}",
                params.len(),
                self.checkpoint_len()
            );
        }
        let split = self.w_out.len();
        self.w_out.copy_from_slice(&params[..split]);
        self.b_out.copy_from_slice(&params[split..]);
        Ok(())
    }

    /// Forward-Pass ueber einen Batch.
    ///
    /// `want_head_input` muss `true` sein, wenn anschliessend ein
    /// Gradientenschritt folgt; dann wird der normierte Reststrom mitgegeben.
    pub fn forward(&self, batch: &Batch, want_head_input: bool) -> ForwardOut {
        let d = self.config.d_model;
        let heads = self.config.n_heads;
        let hd = self.config.head_dim();
        let ffn = self.config.d_ff;
        let vocab = self.config.vocab_size;
        let (bs, seq) = (batch.batch_size, batch.seq_len);
        let eps = self.config.rms_norm_eps;

        let mut logits = vec![0.0f32; bs * seq * vocab];
        let mut head_input = if want_head_input {
            vec![0.0f32; bs * seq * d]
        } else {
            Vec::new()
        };

        // Puffer einmal allokieren, dann je Batch-Zeile wiederverwenden.
        let mut x = vec![0.0f32; d];
        let mut normed = vec![0.0f32; d];
        let mut proj = vec![0.0f32; d];
        let mut joined = vec![0.0f32; d];
        let mut q = vec![0.0f32; heads * seq * hd];
        let mut k = vec![0.0f32; heads * seq * hd];
        let mut v = vec![0.0f32; heads * seq * hd];
        let mut attn = vec![0.0f32; heads * seq * hd];
        let mut gate = vec![0.0f32; ffn];
        let mut up = vec![0.0f32; ffn];
        let mut down = vec![0.0f32; d];
        let mut logit_row = vec![0.0f32; vocab];

        for b in 0..bs {
            // Byte-Embedding ist der Ausgangspunkt des Reststroms.
            for t in 0..seq {
                let tok = batch.input_at(b, t) as usize;
                x.copy_from_slice(&self.token_embedding[tok * d..(tok + 1) * d]);
            }
            // `x` ist der Reststrom: er wird einmal aus dem Embedding gesetzt und
            // danach von jeder Schicht gelesen UND fortgeschrieben. Die Aufteilung
            // in Eingabe-, Attention- und Feed-Forward-Schleife dient nur der
            // Lesbarkeit; x wird zwischen ihnen nicht zurueckgesetzt.

            for layer in &self.layers {
                // --- Attention: q, k, v je Position und Kopf ---
                for t in 0..seq {
                    ops::rms_norm(&x, &layer.norm_attn, eps, &mut normed);
                    ops::matvec(&normed, &layer.wq, d, d, &mut proj);
                    for h in 0..heads {
                        let dst = (h * seq + t) * hd;
                        q[dst..dst + hd].copy_from_slice(&proj[h * hd..(h + 1) * hd]);
                    }
                    ops::matvec(&normed, &layer.wk, d, d, &mut proj);
                    for h in 0..heads {
                        let dst = (h * seq + t) * hd;
                        k[dst..dst + hd].copy_from_slice(&proj[h * hd..(h + 1) * hd]);
                    }
                    ops::matvec(&normed, &layer.wv, d, d, &mut proj);
                    for h in 0..heads {
                        let dst = (h * seq + t) * hd;
                        v[dst..dst + hd].copy_from_slice(&proj[h * hd..(h + 1) * hd]);
                    }
                }
                // RoPE positionsabhaengig auf q und k.
                for h in 0..heads {
                    for t in 0..seq {
                        let off = (h * seq + t) * hd;
                        ops::apply_rope_inplace(&mut q[off..off + hd], t, self.config.rope_theta);
                        ops::apply_rope_inplace(&mut k[off..off + hd], t, self.config.rope_theta);
                    }
                }
                ops::causal_attention(&q, &k, &v, heads, seq, hd, &mut attn);

                // Koepfe zusammenfuegen und ausgeben, dann Restverbindung.
                for t in 0..seq {
                    for h in 0..heads {
                        let src = (h * seq + t) * hd;
                        joined[h * hd..(h + 1) * hd].copy_from_slice(&attn[src..src + hd]);
                    }
                    ops::matvec(&joined, &layer.wo, d, d, &mut proj);
                    for i in 0..d {
                        x[i] += proj[i];
                    }

                    // --- Feed-Forward (SwiGLU) ---
                    ops::rms_norm(&x, &layer.norm_ffn, eps, &mut normed);
                    ops::matvec(&normed, &layer.w_gate, ffn, d, &mut gate);
                    ops::matvec(&normed, &layer.w_up, ffn, d, &mut up);
                    for i in 0..ffn {
                        gate[i] = ops::silu(gate[i]) * up[i];
                    }
                    ops::matvec(&gate, &layer.w_down, d, ffn, &mut down);
                    for i in 0..d {
                        x[i] += down[i];
                    }
                }
            }

            // --- Ausgabekopf ---
            for t in 0..seq {
                ops::rms_norm(&x, &self.norm_final, eps, &mut normed);
                if want_head_input {
                    let off = (b * seq + t) * d;
                    head_input[off..off + d].copy_from_slice(&normed);
                }
                ops::matvec(&normed, &self.w_out, vocab, d, &mut logit_row);
                let loff = (b * seq + t) * vocab;
                for ((dst, l), bias) in logits[loff..loff + vocab]
                    .iter_mut()
                    .zip(&logit_row)
                    .zip(&self.b_out)
                {
                    *dst = l + bias;
                }
            }
        }

        ForwardOut { logits, head_input }
    }

    /// Kreuzentropie ueber alle echten Zielpositionen eines Batches.
    pub fn loss(&self, logits: &[f32], batch: &Batch) -> f32 {
        ops::cross_entropy(
            logits,
            &batch.target,
            batch.batch_size,
            batch.seq_len,
            self.config.vocab_size,
            &batch.target_positions(),
        )
    }

    /// Sammelt `dL/dlogits = softmax(logits) - onehot(target)` fuer `positions`.
    ///
    /// Rueckgabe: `(loss_summe, gradient_je_klasse)` - der Aufrufer mittelt.
    pub fn grad_logits(
        &self,
        logits: &[f32],
        batch: &Batch,
        positions: &[(usize, usize)],
    ) -> (f32, Vec<f32>) {
        let vocab = self.config.vocab_size;
        let mut grad = vec![0.0f32; vocab];
        if positions.is_empty() {
            return (f32::NAN, grad);
        }
        let mut loss_sum = 0.0f32;
        let mut probs = vec![0.0f32; vocab];
        for &(b, t) in positions {
            let off = (b * batch.seq_len + t) * vocab;
            let row = &logits[off..off + vocab];
            let max = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let log_sum_exp = max + row.iter().map(|v| (v - max).exp()).sum::<f32>().ln();
            let target = batch.target_at(b, t) as usize;
            let target_logit = *row.get(target).unwrap_or(&max);
            loss_sum += log_sum_exp - target_logit;

            probs.copy_from_slice(row);
            ops::softmax_inplace(&mut probs);
            probs[target] -= 1.0;
            for (g, p) in grad.iter_mut().zip(&probs) {
                *g += p;
            }
        }
        (loss_sum, grad)
    }

    /// Aktualisiert den Ausgabekopf mit bereits gemitteltem Gradienten.
    ///
    /// `grad_logits` ist der Mittelwert von `dL/dlogits` ueber die Positionen,
    /// `head_input` der gespeicherte normierte Reststrom.
    ///
    /// Wichtig: `dL/dW = mean_positions(dL/dlogits * input^T)`. Der Beitrag jeder
    /// einzelnen Position ist deshalb durch die Anzahl der Positionen zu teilen.
    /// Ohne diese Division ist die effektive Lernrate um den Faktor "Positionen pro
    /// Batch" zu gross, und der Loss divergiert (im Smoke-Test beobachtet:
    /// 5.61 -> 388.97 bei lr = 0.5 und 252 Positionen).
    pub fn apply_head_update(
        &mut self,
        batch: &Batch,
        positions: &[(usize, usize)],
        head_input: &[f32],
        grad_logits: &[f32],
        lr: f32,
    ) {
        if positions.is_empty() {
            return;
        }
        let d = self.config.d_model;
        let vocab = self.config.vocab_size;
        let scale = lr / positions.len() as f32;

        for (i, g) in grad_logits.iter().enumerate() {
            self.b_out[i] -= lr * g;
        }
        for &(b, t) in positions {
            let off = (b * batch.seq_len + t) * d;
            let input = &head_input[off..off + d];
            for (o, grad_o) in grad_logits.iter().enumerate().take(vocab) {
                let g = scale * grad_o;
                if g == 0.0 {
                    continue;
                }
                let row = &mut self.w_out[o * d..(o + 1) * d];
                for (w, x) in row.iter_mut().zip(input) {
                    *w -= g * x;
                }
            }
        }
    }

    /// Ein vollstaendiger Trainingsschritt: Forward, Gradient, Update.
    ///
    /// Gibt `(loss_vor_dem_schritt, anzahl_positionen)` zurueck.
    pub fn train_step(&mut self, batch: &Batch, lr: f32) -> (f32, usize) {
        let out = self.forward(batch, true);
        let positions = batch.target_positions();
        let (loss_sum, grad) = self.grad_logits(&out.logits, batch, &positions);
        if positions.is_empty() {
            return (f32::NAN, 0);
        }
        let n = positions.len() as f32;
        let grad_mean: Vec<f32> = grad.iter().map(|g| g / n).collect();
        self.apply_head_update(batch, &positions, &out.head_input, &grad_mean, lr);
        (loss_sum / n, positions.len())
    }

    /// Forward-Pass ohne Trainingsabsicht (Inferenz/Eval).
    pub fn logits(&self, batch: &Batch) -> Vec<f32> {
        self.forward(batch, false).logits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::format;
    use crate::data::generate;
    use crate::data::loader::Dataset;
    use crate::data::manifest::{Manifest, ShardEntry, checksum_hex};

    fn cfg() -> ModelConfig {
        ModelConfig {
            name: "tiny".into(),
            d_model: 16,
            n_layers: 1,
            n_heads: 2,
            d_ff: 32,
            vocab_size: 256,
            max_seq_len: 64,
            rope_theta: 10_000.0,
            rms_norm_eps: 1e-5,
            tie_embeddings: false,
            dropout: 0.0,
        }
    }

    /// Baut einen echten Batch aus generierten Beispieldaten.
    ///
    /// Nutzt `testutil::unique_dir`, weil Tests im selben Prozess parallel laufen
    /// und sich sonst gegenseitig den Ordner wegraeumen.
    fn batch(n: usize, seq_len: usize, bs: usize) -> Batch {
        let d = crate::testutil::unique_dir("tiny");
        let records = generate::generate_records(n, 5).unwrap();
        let sp = d.join("shard-0000.bin");
        let file_size = format::write_shard(&sp, &records, "t").unwrap();
        let raw = std::fs::read(&sp).unwrap();
        let shard = format::read_shard(&sp).unwrap();
        let mut m = Manifest::new(5, "t");
        m.shards.push(ShardEntry {
            id: 0,
            path: "shard-0000.bin".into(),
            records: shard.records.len() as u64,
            payload_bytes: shard.payload_bytes(),
            file_size,
            checksum: checksum_hex(&raw),
        });
        m.save(d.join("manifest.json")).unwrap();
        let ds = Dataset::load(&d, "manifest.json").unwrap();
        let b = ds.batch(None, bs, seq_len, 0);
        let _ = std::fs::remove_dir_all(&d);
        b
    }

    #[test]
    fn init_is_deterministic_and_seed_dependent() {
        let a = TinyModel::new(cfg(), 1).unwrap();
        let b = TinyModel::new(cfg(), 1).unwrap();
        let c = TinyModel::new(cfg(), 2).unwrap();
        assert_eq!(a.w_out, b.w_out);
        assert_eq!(a.token_embedding, b.token_embedding);
        assert_ne!(a.w_out, c.w_out);
        // param_count zaehlt ALLE Tensoren: Einbettung, Schichten, Abschluss-Norm,
        // Ausgabekopf (weil nicht geteilt) und Ausgabe-Bias.
        assert_eq!(
            a.config.param_count(),
            16 * 256 + (4 * 16 * 16 + 3 * 16 * 32 + 2 * 16) + 16 + 16 * 256 + 256
        );
    }

    /// Regressionstest zur `tie_embeddings`-Korrektur (o2): `param_count()` muss genau die
    /// Tensoren zaehlen, die das Modell wirklich allokiert.
    ///
    /// Ohne diesen Test kann die Zahl erneut von der Realitaet abdriften - genau das war
    /// der Fehler (21.341.184 gemeldet, 21.440.128 allokiert).
    ///
    /// Hinweis: gilt fuer `tie_embeddings = false`. Mit `true` waere die gezaehlte Zahl
    /// kleiner als die Allokation, weil das Platzhalter-Modell die Matrix nicht teilt.
    #[test]
    fn param_count_matches_allocated_tensors() {
        let c = cfg();
        let m = TinyModel::new(c.clone(), 1).unwrap();
        let mut sum = m.token_embedding.len() + m.w_out.len() + m.b_out.len() + m.norm_final.len();
        for l in &m.layers {
            sum += l.wq.len() + l.wk.len() + l.wv.len() + l.wo.len();
            sum += l.w_gate.len() + l.w_up.len() + l.w_down.len();
            sum += l.norm_attn.len() + l.norm_ffn.len();
        }
        assert_eq!(
            sum,
            c.param_count(),
            "param_count muss die real allokierten Tensoren zaehlen"
        );
    }

    #[test]
    fn init_rejects_invalid_config() {
        let mut m = cfg();
        m.n_heads = 5; // 16 % 5 != 0
        assert!(TinyModel::new(m, 1).is_err());
    }

    #[test]
    fn forward_produces_finite_logits_of_right_shape() {
        let model = TinyModel::new(cfg(), 1).unwrap();
        let b = batch(4, 16, 2);
        let out = model.forward(&b, true);
        assert_eq!(out.logits.len(), 2 * 16 * 256);
        assert!(out.logits.iter().all(|v| v.is_finite()));
        assert!(out.logits.iter().any(|v| v.abs() > 1e-6));
        assert_eq!(out.head_input.len(), 2 * 16 * 16);

        // Ohne Anforderung wird kein Speicher verschwendet (Inferenzpfad).
        let thin = model.forward(&b, false);
        assert!(thin.head_input.is_empty());
        assert_eq!(thin.logits, out.logits, "Logits muessen identisch sein");
    }

    #[test]
    fn layers_change_the_result() {
        // Zwei Schichten muessen etwas anderes ergeben als eine: sonst waere der
        // Reststrom nicht fortgeschrieben (genau der Fehler, der hier behoben wurde).
        let one = TinyModel::new(cfg(), 9).unwrap();
        let mut two_cfg = cfg();
        two_cfg.n_layers = 2;
        let two = TinyModel::new(two_cfg, 9).unwrap();
        let b = batch(4, 16, 1);
        assert_ne!(one.logits(&b), two.logits(&b));
    }

    #[test]
    fn position_and_token_change_logits() {
        // Die Ausgabe muss vom Byte an der Position abhaengen, nicht nur von der
        // Position - sonst waere das Embedding nicht angebunden.
        let model = TinyModel::new(cfg(), 21).unwrap();
        let mut b = batch(4, 8, 1);
        let before = model.logits(&b);
        let last = b.input.len() - 1;
        b.input[last] = b.input[last].wrapping_add(7);
        let after = model.logits(&b);
        assert_ne!(before, after);
    }

    #[test]
    fn loss_is_finite_and_near_uniform_at_init() {
        let model = TinyModel::new(cfg(), 7).unwrap();
        let b = batch(4, 16, 2);
        let logits = model.logits(&b);
        let loss = model.loss(&logits, &b);
        assert!(loss.is_finite() && loss > 0.0);
        // Bei zufaelligen Gewichten liegt der Loss nahe ln(256) = 5.545.
        assert!(loss < 12.0, "Loss {loss} ist unplausibel hoch");
    }

    #[test]
    fn train_step_reduces_loss() {
        let mut model = TinyModel::new(cfg(), 11).unwrap();
        let b = batch(8, 32, 2);
        let (first, n) = model.train_step(&b, 0.5);
        assert!(first.is_finite());
        assert_eq!(n, b.total_valid());
        assert!(n > 0);

        let mut last = first;
        for _ in 0..30 {
            let (l, _) = model.train_step(&b, 0.5);
            last = l;
        }
        assert!(last < first, "Loss muss sinken: {first} -> {last}");
    }

    /// Regressionstest gegen den Divergenzfehler: der Loss darf bei einer
    /// plausiblen Lernrate ueber viele Schritte nicht wegfliegen.
    #[test]
    fn training_does_not_diverge_over_many_steps() {
        let mut model = TinyModel::new(cfg(), 17).unwrap();
        let b = batch(16, 64, 4);
        let (first, _) = model.train_step(&b, 0.05);
        let mut last = first;
        for _ in 0..200 {
            let (l, _) = model.train_step(&b, 0.05);
            assert!(l.is_finite(), "Loss wurde nicht-endlich: {l}");
            assert!(l < 50.0, "Loss divergiert: {first} -> {l}");
            last = l;
        }
        assert!(
            last < first,
            "Loss muss ueber 200 Schritte sinken: {first} -> {last}"
        );
    }

    #[test]
    fn training_is_reproducible() {
        let b = batch(8, 16, 2);
        let mut a = TinyModel::new(cfg(), 5).unwrap();
        let mut c = TinyModel::new(cfg(), 5).unwrap();
        for _ in 0..3 {
            a.train_step(&b, 0.3);
            c.train_step(&b, 0.3);
        }
        assert_eq!(a.head_params(), c.head_params());
    }

    #[test]
    fn head_params_roundtrip() {
        let model = TinyModel::new(cfg(), 3).unwrap();
        let params = model.head_params();
        assert_eq!(params.len(), 256 * 16 + 256);
        assert_eq!(params.len(), model.checkpoint_len());

        let mut other = TinyModel::new(cfg(), 4).unwrap();
        other.set_head_params(&params).unwrap();
        assert_eq!(other.head_params(), params);
        assert!(other.set_head_params(&[0.0; 3]).is_err());
    }
}
