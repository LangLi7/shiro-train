//! Datentypen des Dashboards.
//!
//! Bewusst wenige, flache Typen: sie werden als JSON an den Browser gegeben.
//! Es gibt keine Serialisierungs-Bibliothek - die Ausgabe wird von Hand
//! geschrieben (`jsonout.rs`), damit dieses Werkzeug **keine** Abhaengigkeit hat.

/// Ein einzelner Trainingsschritt, wie er in `logs/*.jsonl` steht.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub step: u64,
    pub loss: f64,
    pub perplexity: f64,
    pub lr: f64,
    pub bytes: u64,
    /// Bytes je Sekunde: steht in der Logzeile als `tokens_per_second`
    /// (Byte-Modell: ein Token ist ein Byte, deshalb ist der Name historisch).
    pub bytes_per_second: f64,
    pub ms: f64,
    /// Pfad eines in genau diesem Schritt geschriebenen Checkpoints.
    pub checkpoint: Option<String>,
    /// Rekonstruierte Zeit (Sekunden seit Epoche). Siehe `state.rs`.
    pub t: f64,
    /// Effektive Rechenleistung in GFLOPS, aus `params`, `bytes` und `ms`
    /// gerechnet (6*N*D/t). `None`, wenn die Parameterzahl unbekannt ist.
    pub gflops: Option<f64>,
}

/// Rahmendaten eines Laufs (`run_start` in der Logdatei).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunInfo {
    pub run: String,
    pub step_target: u64,
    pub seed: u64,
    pub d_model: u64,
    pub n_layers: u64,
    pub n_heads: u64,
    pub params: u64,
    pub batch_size: u64,
    pub seq_len: u64,
    pub weights: String,
    pub autocast: String,
    /// Pfad eines fortgesetzten Stands (**B6**), falls der Lauf fortgesetzt wurde.
    pub resume: Option<String>,
}

/// `data_loaded` - welcher Datensatz gelesen wurde.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataInfo {
    pub dir: String,
    pub shards: u64,
    pub records: u64,
    pub payload_bytes: u64,
}

/// Ein Checkpoint, wie er im Log gemeldet wurde.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckpointEvent {
    pub step: u64,
    pub path: String,
    pub bytes: u64,
    pub t: f64,
}

/// Abschluss eines Laufs (`run_end`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunEnd {
    pub steps: u64,
    pub first_loss: f64,
    pub last_loss: f64,
    pub best_loss: f64,
    pub total_seconds: f64,
    pub total_bytes: u64,
    pub checkpoints: u64,
}

/// Art eines Ereignisses im Feed ("Durchbrueche und Ereignisse").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Start,
    Data,
    /// Neuer persoenlicher Rekord beim Loss - der "Durchbruch".
    Record,
    Checkpoint,
    End,
    /// Loss ist nicht endlich (NaN/inf) - Trainingsabbruch.
    Nan,
    /// Seit vielen Schritten keine Verbesserung mehr.
    Stagnation,
    /// Ein neuer Lauf hat einen frueheren fortgesetzt (**B6**).
    Resume,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Start => "start",
            Kind::Data => "data",
            Kind::Record => "record",
            Kind::Checkpoint => "checkpoint",
            Kind::End => "end",
            Kind::Nan => "nan",
            Kind::Stagnation => "stagnation",
            Kind::Resume => "resume",
        }
    }

    /// Rang fuer die Sortierung im Feed: wichtig zuerst.
    pub fn weight(self) -> u32 {
        match self {
            Kind::Record => 100,
            Kind::Nan => 95,
            Kind::Stagnation => 80,
            Kind::End => 70,
            Kind::Start => 60,
            Kind::Resume => 55,
            Kind::Checkpoint => 40,
            Kind::Data => 30,
        }
    }
}

/// Ein Eintrag im Ereignis-Feed.
#[derive(Debug, Clone, PartialEq)]
pub struct FeedItem {
    pub kind: Kind,
    pub step: u64,
    pub t: f64,
    pub text: String,
    /// Zahlenwert zum Ereignis (Loss beim Rekord, Bytegroesse beim Checkpoint).
    pub value: Option<f64>,
}

/// Ein fertig ausgewerteter Lauf.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// Dateiname der Logdatei - dient als stabile Kennung in der API.
    pub id: String,
    pub started: Option<RunInfo>,
    pub data: Option<DataInfo>,
    pub end: Option<RunEnd>,
    pub steps: Vec<Step>,
    pub checkpoints: Vec<CheckpointEvent>,
    pub feed: Vec<FeedItem>,
    /// Zeitstempel der Logdatei (mtime) - Anker fuer die Zeitleiste.
    pub mtime: f64,
    /// Groesse der Logdatei in Bytes.
    pub file_bytes: u64,
}

impl Run {
    /// Bester Loss im Lauf und der Schritt, in dem er erreicht wurde.
    pub fn best(&self) -> Option<(f64, u64)> {
        self.steps
            .iter()
            .filter(|s| s.loss.is_finite())
            .fold(None, |acc: Option<(f64, u64)>, s| match acc {
                Some((bl, _)) if bl <= s.loss => acc,
                _ => Some((s.loss, s.step)),
            })
    }

    pub fn last(&self) -> Option<&Step> {
        self.steps.last()
    }

    pub fn first(&self) -> Option<&Step> {
        self.steps.first()
    }

    /// Ist der Lauf abgeschlossen (`run_end` gesehen) oder laeuft er noch?
    pub fn finished(&self) -> bool {
        self.end.is_some()
    }

    /// Anzeigename: Run-Name aus dem Log, sonst der Dateiname.
    pub fn name(&self) -> &str {
        match &self.started {
            Some(i) if !i.run.is_empty() => &i.run,
            _ => &self.id,
        }
    }

    /// Fortschritt in Prozent (0..100) - **dieses** Laufs, nicht des Zaehlers.
    ///
    /// ## Der Fehler, den das behebt
    ///
    /// Der Schrittzaehler wird seit **B6** **global** gefuehrt: ein
    /// fortgesetzter Lauf beginnt nicht bei 1, sondern bei der Nummer des
    /// Elternstands. `letzter Schritt / step_target` ergibt dann ein Vielfaches
    /// von 100, das die Anzeige auf 100 kappt - ein Lauf mit 10 % sah fertig
    /// aus. Gemessen an den echten Daten (08.10.2026): **43 von 127** Laeufen
    /// haben dieses Problem, der deutlichste mit letztem Schritt 22000 gegen
    /// Ziel 4800, angezeigt als 100 %.
    ///
    /// Deshalb hier die Zahl der **Messpunkte** dieses Laufs gegen das Ziel.
    /// Laeuft der Zaehler nicht global (letzter Schritt <= Ziel), bleibt die
    /// direkte Rechnung gueltig und wird bevorzugt: sie vertraegt auch einen
    /// ausgeduennten Kurvenverlauf, bei dem Messpunkte fehlen.
    ///
    /// **Es gibt genau eine Quelle fuer diese Zahl.** Vorher stand die Rechnung
    /// zusaetzlich in `server.rs` und die SMART-Karte benutzte weiter die alte -
    /// dort las man "Ziel 22000 von 4800 Schritten (100.0 %)".
    pub fn progress(&self) -> Option<f64> {
        let target = self.started.as_ref()?.step_target;
        if target == 0 {
            return None;
        }
        let getan = self.done_steps();
        Some((100.0 * getan as f64 / target as f64).min(100.0))
    }

    /// Wie viele Schritte **dieser** Lauf getan hat.
    ///
    /// Der letzte Schritt ist die genaue Angabe, wenn der Zaehler bei 1 beginnt.
    /// Liegt er ueber dem Ziel, wurde global weitergezaehlt (B6) - dann ist die
    /// Zahl der Messpunkte die belastbare Untergrenze.
    pub fn done_steps(&self) -> u64 {
        let target = self.started.as_ref().map(|i| i.step_target).unwrap_or(0);
        let last = self.steps.last().map(|s| s.step).unwrap_or(0);
        if target > 0 && last > target {
            self.steps.len() as u64
        } else {
            last
        }
    }

    /// Mittlere Dauer je Schritt ueber die letzten `window` Schritte (ms).
    ///
    /// Nur die letzten Schritte zaehlen: ein Lauf wird typischerweise
    /// langsamer oder schneller, und die Gesamtmitte verschleiert das.
    pub fn ms_per_step(&self, window: usize) -> Option<f64> {
        let n = self.steps.len();
        if n == 0 {
            return None;
        }
        let from = n.saturating_sub(window.max(1));
        let slice = &self.steps[from..];
        let sum: f64 = slice.iter().map(|s| s.ms).sum();
        Some(sum / slice.len() as f64)
    }

    /// Restzeit in Sekunden, falls Ziel und Tempo bekannt sind.
    ///
    /// Rechnet mit `done_steps()` und nicht mit der rohen letzten
    /// Schrittnummer: bei globalem Zaehler (B6) waere `done >= target` immer
    /// wahr und die Restzeit faelschlich **0**.
    pub fn eta_seconds(&self, window: usize) -> Option<f64> {
        let target = self.started.as_ref()?.step_target;
        let done = self.done_steps();
        if done >= target {
            return Some(0.0);
        }
        let per_step = self.ms_per_step(window)?;
        Some((target - done) as f64 * per_step / 1000.0)
    }
}

/// Ein Checkpoint **auf der Platte** (nicht aus dem Log).
#[derive(Debug, Clone, PartialEq)]
pub struct CheckpointFile {
    pub name: String,
    pub bytes: u64,
    pub modified: f64,
}
