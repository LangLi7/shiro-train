//! Architektur-Mathematik auf flachen `f32`-Slices - backendfrei.
//!
//! Alles hier ist bewusst ohne Framework geschrieben. Die Roadmap laesst die Wahl
//! zwischen Candle und Burn ausdruecklich offen (Phase 0: Backend-Test auf Metal
//! **und** CUDA). Weil diese Datei nur `&[f32]` und `&mut [f32]` kennt, ist der
//! spaetere Wechsel ein lokaler Eingriff statt eines Umbaus.
//!
//! Alle Funktionen sind von `tiny.rs` und den Tests direkt aufrufbar; sie sind die
//! Referenz, gegen die eine Framework-Implementierung spaeter gegengeprueft wird
//! (Roadmap: "ndarray als einfache CPU-Implementierung, um das Modell gegenzupruefen").

/// RMSNorm (Roadmap: "RMSNorm und SwiGLU").
///
/// `y_i = x_i / sqrt(mean(x^2) + eps) * w_i`
pub fn rms_norm(x: &[f32], w: &[f32], eps: f32, out: &mut [f32]) {
    assert_eq!(x.len(), w.len());
    assert_eq!(x.len(), out.len());
    let ms = x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32;
    let inv = 1.0 / (ms + eps).sqrt();
    for i in 0..x.len() {
        out[i] = x[i] * inv * w[i];
    }
}

/// Numerisch stabile Softmax in-place.
pub fn softmax_inplace(x: &mut [f32]) {
    if x.is_empty() {
        return;
    }
    let max = x.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for v in x.iter_mut() {
        *v = (*v - max).exp();
        sum += *v;
    }
    let inv = 1.0 / sum;
    for v in x.iter_mut() {
        *v *= inv;
    }
}

/// GELU (tanh-Naeherung), fuer Modelle, die GELU als Aktivierung verwenden.
pub fn gelu(x: f32) -> f32 {
    const C: f32 = 0.797_884_6; // sqrt(2/pi)
    0.5 * x * (1.0 + (C * (x + 0.044_715 * x * x * x)).tanh())
}

/// SiLU (`x * sigmoid(x)`) - die Gate-Aktivierung des SwiGLU-Blocks.
pub fn silu(x: f32) -> f32 {
    if x >= 0.0 {
        x / (1.0 + (-x).exp())
    } else {
        let exp = x.exp();
        x * exp / (1.0 + exp)
    }
}

/// `out = x @ W^T` mit `W` als `[out_dim, in_dim]` in Zeilenreihenfolge.
pub fn matvec(x: &[f32], w: &[f32], out_dim: usize, in_dim: usize, out: &mut [f32]) {
    assert_eq!(x.len(), in_dim, "Eingabedimension passt nicht");
    assert_eq!(w.len(), out_dim * in_dim, "Gewichtsmatrix passt nicht");
    assert_eq!(out.len(), out_dim);
    for o in 0..out_dim {
        let row = &w[o * in_dim..(o + 1) * in_dim];
        out[o] = row.iter().zip(x).map(|(a, b)| a * b).sum();
    }
}

/// Skalarprodukt, wie es die Attention fuer die Scores braucht.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Sinus-/Kosinus-Paar fuer eine RoPE-Position.
///
/// Roadmap: "Positionscodierung: RoPE". `inv_freq_j = theta^(-2j/head_dim)`.
pub fn rope_theta_factors(head_dim: usize, pos: usize, theta: f32) -> Vec<(f32, f32)> {
    assert!(head_dim.is_multiple_of(2), "head_dim muss gerade sein");
    let half = head_dim / 2;
    (0..half)
        .map(|j| {
            let inv_freq = theta.powf(-2.0 * j as f32 / head_dim as f32);
            let angle = pos as f32 * inv_freq;
            (angle.sin(), angle.cos())
        })
        .collect()
}

/// Wendet RoPE auf einen Kopfvektor an (Rotation paarweise).
pub fn apply_rope_inplace(vec: &mut [f32], pos: usize, theta: f32) {
    let head_dim = vec.len();
    let factors = rope_theta_factors(head_dim, pos, theta);
    for (j, (sin, cos)) in factors.iter().enumerate() {
        let (a, b) = (vec[2 * j], vec[2 * j + 1]);
        vec[2 * j] = a * cos - b * sin;
        vec[2 * j + 1] = a * sin + b * cos;
    }
}

/// Kausale Multi-Head-Self-Attention.
///
/// `q`, `k`, `v`: `[n_heads * seq_len * head_dim]`, zeilenweise je Position.
/// `out` muss dieselbe Form haben.
///
/// Vollstaendige Sequenz-Attention, also ohne KV-Cache: im Training ist das
/// korrekt und schneller (die GPU-Analyse rechnet den KV-Cache nur fuer Inferenz).
pub fn causal_attention(
    q: &[f32],
    k: &[f32],
    v: &[f32],
    n_heads: usize,
    seq_len: usize,
    head_dim: usize,
    out: &mut [f32],
) {
    let total = n_heads * seq_len * head_dim;
    assert_eq!(q.len(), total);
    assert_eq!(k.len(), total);
    assert_eq!(v.len(), total);
    assert_eq!(out.len(), total);

    let scale = 1.0 / (head_dim as f32).sqrt();
    let mut scores = vec![0.0f32; seq_len];
    for h in 0..n_heads {
        for i in 0..seq_len {
            let q_off = (h * seq_len + i) * head_dim;
            let qs = &q[q_off..q_off + head_dim];
            // Score gegen alle Positionen <= i (kausal).
            for (j, score) in scores.iter_mut().enumerate().take(i + 1) {
                let k_off = (h * seq_len + j) * head_dim;
                *score = dot(qs, &k[k_off..k_off + head_dim]) * scale;
            }
            softmax_inplace(&mut scores[..=i]);
            let o_off = (h * seq_len + i) * head_dim;
            for d in 0..head_dim {
                let mut acc = 0.0f32;
                for (j, s) in scores.iter().enumerate().take(i + 1) {
                    acc += s * v[(h * seq_len + j) * head_dim + d];
                }
                out[o_off + d] = acc;
            }
        }
    }
}

/// Mittelwert der Kreuzentropie ueber die angegebenen Positionen.
///
/// `logits` hat die Form `[batch * seq * vocab]`, `positions` enthaelt `(b, t)`.
pub fn cross_entropy(
    logits: &[f32],
    targets: &[u8],
    batch_size: usize,
    seq_len: usize,
    vocab: usize,
    positions: &[(usize, usize)],
) -> f32 {
    assert_eq!(logits.len(), batch_size * seq_len * vocab);
    assert_eq!(targets.len(), batch_size * seq_len);
    if positions.is_empty() {
        return f32::NAN;
    }
    let mut sum = 0.0f32;
    for &(b, t) in positions {
        let off = (b * seq_len + t) * vocab;
        let row = &logits[off..off + vocab];
        let max = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let log_sum_exp = max + row.iter().map(|v| (v - max).exp()).sum::<f32>().ln();
        let target = targets[b * seq_len + t] as usize;
        // Direkter Zugriff auf die Zielklasse: hier ist der Index der Inhalt, nicht
        // eine Iterationsvariable.
        let target_logit = *row.get(target).unwrap_or(&max);
        sum += log_sum_exp - target_logit;
    }
    sum / positions.len() as f32
}

/// Perplexitaet aus einem Loss - die anschaulichere Zahl beim Bericht.
pub fn perplexity(loss: f32) -> f32 {
    loss.exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rms_norm_normalizes_and_scales() {
        let x = [3.0f32, 4.0];
        let w = [1.0f32, 1.0];
        let mut out = [0.0f32; 2];
        rms_norm(&x, &w, 0.0, &mut out);
        // rms = sqrt((9+16)/2) = sqrt(12.5)
        let rms = 12.5f32.sqrt();
        assert!((out[0] - 3.0 / rms).abs() < 1e-6);
        assert!((out[1] - 4.0 / rms).abs() < 1e-6);

        // Gewicht skaliert.
        let w2 = [2.0f32, 1.0];
        rms_norm(&x, &w2, 0.0, &mut out);
        assert!((out[0] - 2.0 * 3.0 / rms).abs() < 1e-6);

        // eps verhindert Division durch Null bei Nullvektor.
        let z = [0.0f32; 2];
        rms_norm(&z, &w, 1e-5, &mut out);
        assert!(out.iter().all(|v| v.abs() < 1e-9));
    }

    #[test]
    fn softmax_is_a_distribution_and_shift_invariant() {
        let mut a = [1.0f32, 2.0, 3.0];
        softmax_inplace(&mut a);
        assert!((a.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!(a.iter().all(|v| *v > 0.0));
        assert!(a[0] < a[1] && a[1] < a[2]);

        // Numerische Stabilitaet: grosse Werte duerfen nicht in NaN laufen.
        let mut b = [1000.0f32, 1001.0, 1002.0];
        softmax_inplace(&mut b);
        assert!(b.iter().all(|v| v.is_finite()));
        assert!((b.iter().sum::<f32>() - 1.0).abs() < 1e-6);

        let mut c = [1.0f32, 2.0, 3.0];
        let mut d = [11.0f32, 12.0, 13.0];
        softmax_inplace(&mut c);
        softmax_inplace(&mut d);
        for i in 0..3 {
            assert!(
                (c[i] - d[i]).abs() < 1e-6,
                "Verschiebung darf nichts aendern"
            );
        }
        let mut empty: [f32; 0] = [];
        softmax_inplace(&mut empty);
    }

    #[test]
    fn gelu_matches_reference_points() {
        assert!(gelu(0.0).abs() < 1e-6);
        assert!((gelu(1.0) - 0.841_192).abs() < 1e-3);
        assert!((gelu(-1.0) + 0.158_808).abs() < 1e-3);
        assert!(gelu(6.0) > 5.99);
        assert!(gelu(-6.0).abs() < 1e-3);
    }

    #[test]
    fn matvec_computes_rows() {
        // W = [[1,2,3],[4,5,6]], x = [1,1,1] => [6, 15]
        let w = [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let x = [1.0f32, 1.0, 1.0];
        let mut out = [0.0f32; 2];
        matvec(&x, &w, 2, 3, &mut out);
        assert_eq!(out, [6.0, 15.0]);
    }

    #[test]
    fn rope_preserves_norm_and_varies_with_position() {
        let mut v0 = vec![1.0f32, 0.0, 0.5, -0.5];
        let v1 = v0.clone();
        let n0: f32 = v0.iter().map(|x| x * x).sum::<f32>().sqrt();
        apply_rope_inplace(&mut v0, 3, 10_000.0);
        let n1: f32 = v0.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (n0 - n1).abs() < 1e-5,
            "Rotation darf die Norm nicht aendern"
        );
        assert_ne!(v0, v1, "Position 3 muss etwas veraendern");

        // Position 0 ist die Identitaet (Winkel 0).
        let mut v2 = v1.clone();
        apply_rope_inplace(&mut v2, 0, 10_000.0);
        for i in 0..v1.len() {
            assert!((v2[i] - v1[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn rope_factors_have_expected_shape() {
        let f = rope_theta_factors(4, 1, 10_000.0);
        assert_eq!(f.len(), 2);
        // j=0: inv_freq = 1 => sin(1), cos(1)
        assert!((f[0].0 - 1.0f32.sin()).abs() < 1e-6);
        // j=1: inv_freq = 10000^(-0.5) = 0.01
        assert!((f[1].0 - 0.01f32.sin()).abs() < 1e-6);
    }

    #[test]
    fn attention_is_causal_and_averages_over_prefix() {
        let (seq, heads, hd) = (3usize, 1usize, 2usize);
        let q = vec![1.0f32; heads * seq * hd];
        let k = vec![1.0f32; heads * seq * hd];
        let v = vec![1.0f32, 0.0, 2.0, 0.0, 4.0, 0.0];
        let mut out = vec![0.0f32; heads * seq * hd];
        causal_attention(&q, &k, &v, heads, seq, hd, &mut out);

        // Position 0 sieht nur sich selbst: v[0..2]
        assert!((out[0] - 1.0).abs() < 1e-6);
        // Position 1 sieht 0 und 1, beide gleich gewichtet: (1+2)/2
        assert!((out[2] - 1.5).abs() < 1e-5);
        // Position 2 sieht alle: (1+2+4)/3
        assert!((out[4] - 7.0 / 3.0).abs() < 1e-5);

        // Zukunft darf keinen Einfluss haben: aendere v an Position 2.
        let mut v2 = v.clone();
        v2[4] = 100.0;
        let mut out2 = vec![0.0f32; heads * seq * hd];
        causal_attention(&q, &k, &v2, heads, seq, hd, &mut out2);
        assert_eq!(
            out[0], out2[0],
            "Position 0 darf nicht von Position 2 abhaengen"
        );
        assert_eq!(
            out[2], out2[2],
            "Position 1 darf nicht von Position 2 abhaengen"
        );
    }

    #[test]
    fn cross_entropy_is_zero_for_certain_prediction() {
        let (b, s, v) = (1usize, 2usize, 3usize);
        // Zwei Positionen, jeweils klar auf die richtige Klasse.
        let logits = vec![
            0.0f32, 20.0, 0.0, // Ziel 1
            20.0, 0.0, 0.0, // Ziel 0
        ];
        let targets = vec![1u8, 0];
        let loss = cross_entropy(&logits, &targets, b, s, v, &[(0, 0), (0, 1)]);
        assert!(loss < 1e-4, "Loss muss fast 0 sein, war {loss}");
        assert!((perplexity(0.0) - 1.0).abs() < 1e-6);

        // Gleiche Wahrscheinlichkeiten => ln(3)
        let flat = vec![1.0f32; b * s * v];
        let loss = cross_entropy(&flat, &targets, b, s, v, &[(0, 0), (0, 1)]);
        assert!((loss - 3.0f32.ln()).abs() < 1e-5);
    }

    #[test]
    fn cross_entropy_rejects_empty_positions() {
        let logits = vec![0.0f32; 3];
        let targets = vec![0u8];
        assert!(cross_entropy(&logits, &targets, 1, 1, 3, &[]).is_nan());
    }
}
