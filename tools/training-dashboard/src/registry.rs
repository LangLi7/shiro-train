//! Die **inhaltliche** Seite des Trainings: was wurde je Runde trainiert,
//! welche Aufgabentypen wurden gemessen, und wie weit ist die Runde von der
//! Projektmarke **D-047** entfernt?
//!
//! ## Warum es dieses Modul gibt
//!
//! Die Trainingslogs (`logs/*.jsonl`) tragen **keine** Inhaltsdaten: gemessen
//! ueber alle Logdateien gibt es nur `step`, `checkpoint_written`, `run_start`,
//! `data_loaded` und `run_end` - **0 Treffer** fuer `per_category`,
//! `exact_match` oder `kategorie`. Eine Anzeige "welche Faehigkeit, welche
//! Aufgabentypen" ist aus den Logs deshalb **nicht** baubar.
//!
//! Die Zahlen stehen in `registry/experiments.jsonl`: eine JSON-Zeile je Runde.
//! Dort liegen `capability`, `experiment_id`, `pack`, `urteil`, die Kopfzahl
//! `exact_match` - und in einem Teil der Runden `panels` mit den per-Typ-Zahlen
//! unter `panels[i].nachher.pro_typ` als `{typ: [gestellt, richtig]}`.
//!
//! ## Zwei Regeln, die dieses Modul durchsetzt
//!
//! 1. **Kein stilles Zusammenlegen.** Eine Runde traegt in der Regel **zwei**
//!    Panels (`A` und `B`), die auf **disjunkten** Fragesaetzen messen. Die
//!    Anzeige zeigt je Panel eine eigene Tabelle und rechnet je Panel; was
//!    gepoolt wird, wird als gepoolt **gekennzeichnet** (das Urteil D-047).
//! 2. **Nicht-endliche Zahlen fliessen nirgends als Zahl.** `NaN` und
//!    `Infinity` sind in JSON nicht darstellbar; sie kaemen im Browser als
//!    `null` an und wuerden die Verlustkurve still verfaelschen. Deshalb gilt:
//!    was nicht endlich ist, wird **Text** und **keine** Zahl - hier und in
//!    `jsonout::Out::num`.
//!
//! ## Woher die Marke kommt
//!
//! Die Marke steht **nicht** in diesem Werkzeug und auch nicht in der
//! Konfigurationsdatei der Faehigkeit - dort steht nur `exact_match_min = 0.60`,
//! und diese Schwelle ist mit 84,2 % erreicht und taugt trotzdem nicht als
//! Nachweis. Sie steht als Entscheidung im Projektregister: siehe
//! [`D_047_QUELLE`].

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::jsonread::{self, Value};

/// Kurzbeschreibung der Quelle, aus der die beiden Marken unten stammen.
///
/// Der Pfad steht hier **korrekt** (Verzeichnis `docs/`, Dateiname
/// `abschluss-vorbereitungsphase.md`): eine falsche Schreibweise waere in einem
/// Beleg schlimmer als gar keine, weil sie wie eine Quelle aussieht.
pub const D_047_QUELLE: &str =
    "D-047, docs/abschluss-vorbereitungsphase.md (Entscheidungsregister)";

/// Kopfzahl der Marke: Held-out Exact Match **mindestens** 90 %.
///
/// Wortlaut der Registerzeile: "der Held-out Exact Match >= 90 % erreicht".
/// 90 % von 120 Fragen lassen 12 Fehler zu und liegen damit ueber jeder
/// gemessenen Shortcut-Baseline der Familie (hoechste 49,3 % bei `vergleich`).
pub const D_047_EXACT_MATCH_MIN: f64 = 0.90;

/// Typmarke: **kein** inhaltlicher Aufgabentyp unter 75 %.
///
/// Wortlaut der Registerzeile: "kein inhaltlicher Aufgabentyp unter 75 % liegt".
/// Begruendung dort: 75 % je Typ schliesst aus, dass ein Typ strukturell
/// ungeloest bleibt; bei der kleinsten Stichprobe (n = 12) verlangt es
/// mindestens 9 Treffer.
pub const D_047_TYP_MIN: f64 = 0.75;

/// Erster Teil des Kriterientextes - wortgleich mit der Registerzeile.
pub const D_047_BEDINGUNG_1: &str = "Held-out Exact Match >= 90 %";

/// Zweiter Teil des Kriterientextes - wortgleich mit der Registerzeile.
pub const D_047_BEDINGUNG_2: &str = "kein inhaltlicher Aufgabentyp unter 75 %";

/// Vorgabe fuer `?limit=` am Endpunkt `/api/registry`.
///
/// 500 Zeilen sind bei den gemessenen 2,4 KB je Zeile rund 1,2 MB - und der
/// Browser bekommt damit jede Runde, die es je gab, ohne dass die Antwort
/// waechst. Der Endpunkt liest **nur** die letzten N Zeilen, nie die ganze Datei.
pub const STANDARD_LIMIT: usize = 500;

/// Obergrenze fuer `?limit=`: darueber ist es kein "letzte N Zeilen" mehr,
/// sondern ein Vollscan - und genau das soll dieser Endpunkt nicht sein.
pub const MAX_LIMIT: usize = 5000;

/// Eine Zelle der Typentabelle: `[gestellt, richtig]` aus `pro_typ`.
///
/// Beide Werte sind `Option`, weil eine Zahl nicht endlich oder gar nicht
/// vorhanden sein kann. `None` heisst **nicht** 0, sondern "nicht gemessen" -
/// eine 0 waere eine Behauptung, die die Datei nicht traegt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zelle {
    pub gestellt: Option<u64>,
    pub richtig: Option<u64>,
}

impl Zelle {
    /// Der gemessene Anteil, `None` wenn nicht messbar.
    ///
    /// `None` bei fehlenden Werten **und** bei `gestellt == 0` (Division durch
    /// null) sowie bei einem Verhaeltnis, das nicht endlich ist.
    pub fn quote(&self) -> Option<f64> {
        let g = self.gestellt? as f64;
        let r = self.richtig? as f64;
        if g <= 0.0 {
            return None;
        }
        let q = r / g;
        if q.is_finite() {
            Some(q)
        } else {
            None
        }
    }
}

/// Ein Messband einer Runde. Panel A und Panel B messen auf **disjunkten**
/// Fragesaetzen und werden deshalb **nie** stillschweigend zusammengelegt.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Panel {
    /// `A` oder `B` (Feld `panels[i].name`); leer, wenn die Datei keinen nennt.
    pub name: String,
    /// Versatz der Auswahlregel (Feld `panels[i].offset`) - 0 oder 5.
    pub offset: Option<u64>,
    /// Typ -> Zelle, **vor** der Runde. Sortiert (`BTreeMap`), damit die
    /// Anzeige eine feste Reihenfolge hat.
    pub vorher: BTreeMap<String, Zelle>,
    /// Typ -> Zelle, **nach** der Runde.
    pub nachher: BTreeMap<String, Zelle>,
    /// Kopfzahl des Panels vor der Runde (`vorher.exact_match`).
    pub em_vorher: Option<f64>,
    /// Kopfzahl des Panels nach der Runde (`nachher.exact_match`) - sie speist
    /// die Kopf-Bedingung der Marke.
    pub em_nachher: Option<f64>,
    /// Inhaltlicher Anteil nach der Runde (`nachher.exact_match_content`).
    pub inhalt_nachher: Option<f64>,
    /// Anzahl gestellter inhaltlicher Fragen (`nachher.content_asked`).
    pub inhalt_gestellt: Option<u64>,
    /// Anzahl richtiger inhaltlicher Antworten (`nachher.content_correct`).
    pub inhalt_richtig: Option<u64>,
    /// Schwaechster Typ laut Datei (`nachher.schwaechster_typ`).
    pub schwaechster_typ: Option<String>,
    /// Dessen Quote laut Datei (`nachher.schwaechster_quote`).
    pub schwaechster_quote: Option<f64>,
    /// Anzahl Messpunkte des Panels (`nachher.asked`), falls angegeben.
    pub gefragt: Option<u64>,
    /// Gesamtzahl der im Panel gemessenen Fragen ueber alle Typen.
    pub typ_summe: u64,
    /// Wortlaut zur Herkunft: Panels sind disjunkt, nicht additiv.
    pub hinweis: String,
}
/// Eine Runde aus `registry/experiments.jsonl`.
///
/// Alle Felder sind optional, weil die Datei ueber die Zeit gewachsen ist: die
/// ersten Zeilen tragen kein `urteil` und kein `deterministisch`, spaetere
/// zusaetzlich `replay_share`, `pack` und `leiterstufe`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Runde {
    pub experiment_id: String,
    pub capability: String,
    pub parent_checkpoint: Option<String>,
    pub dataset_version: Option<String>,
    pub generator_version: Option<String>,
    pub verifier_version: Option<String>,
    pub seed: Option<u64>,
    /// `training_config` ist Freitext - oft ein ganzer Satz, keine Zahl.
    pub training_config: Option<String>,
    pub evaluation_config: Option<String>,
    pub steps: Option<u64>,
    pub learning_rate: Option<f64>,
    /// Nur in den spaeteren Zeilen vorhanden.
    pub replay_share: Option<f64>,
    pub pack: Option<String>,
    pub leiterstufe: Option<f64>,
    pub loss: Option<f64>,
    pub perplexity: Option<f64>,
    pub exact_match: Option<f64>,
    pub structural_accuracy: Option<f64>,
    pub regression_results: Option<String>,
    pub failure_distribution: BTreeMap<String, u64>,
    pub selected_next_action: Option<String>,
    pub reason_for_action: Option<String>,
    pub checkpoint: Option<String>,
    pub accepted: Option<bool>,
    pub score: Option<f64>,
    pub score_before: Option<f64>,
    pub urteil: Option<String>,
    pub deterministisch: Option<bool>,
    /// Leer, wenn die Zeile keinen Schluessel `panels` traegt **oder** wenn er
    /// ein leeres Array ist. Ein Panel mit leerem `pro_typ` wird dagegen
    /// aufgenommen: es ist ein Panel ohne Zahlen, nicht kein Panel.
    pub panels: Vec<Panel>,
}

impl Runde {
    /// Ein Kurzname fuer die Auswahlliste.
    ///
    /// `experiment_id` **wiederholt sich** in der Datei (gemessen: `i1`, `i2`,
    /// `i3` sind die Iterationen eines Aufrufs, und 11 der 38 Zeilen heissen
    /// `sequence_reasoning-i1`). Ein Dropdown ueber `experiment_id` allein
    /// haette also elf gleich benannte Eintraege; Schritt und Checkpoint-Name
    /// machen sie unterscheidbar.
    pub fn kurzname(&self) -> String {
        let seit = match self.steps {
            Some(s) => format!("{s} Schritte"),
            None => "Schritte n/a".to_string(),
        };
        let ort = match (&self.parent_checkpoint, &self.checkpoint) {
            (Some(a), Some(b)) => format!("{} -> {}", kurz_datei(a), kurz_datei(b)),
            (Some(a), None) => kurz_datei(a),
            (None, Some(b)) => kurz_datei(b),
            (None, None) => "Checkpoint n/a".to_string(),
        };
        format!("{} - {} - {}", self.experiment_id, seit, ort)
    }
}

/// Letztes Pfadstueck eines Checkpoint-Pfades.
fn kurz_datei(p: &str) -> String {
    p.rsplit(['/', '\\']).next().unwrap_or(p).to_string()
}

// ---------------------------------------------------------------------------
// Lesen
// ---------------------------------------------------------------------------

/// Was beim Lesen passiert ist - Zahlen, die die Anzeige nennen muss.
///
/// Ohne diese Angaben sieht eine leere Antwort wie "es gab keine Runden aus",
/// obwohl der Grund ein fehlender Pfad oder eine kaputte Datei sein kann.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Lesebericht {
    pub runden: Vec<Runde>,
    /// Uebersprungene Zeilen (kein JSON, kein Objekt, kein `experiment_id`).
    pub uebersprungen: u64,
    /// Gelesene Zeilen insgesamt - die guten **und** die kaputten.
    pub zeilen: u64,
    /// `true`, wenn die Datei von ihrem Anfang gelesen wurde (Vollscan).
    /// `false` heisst: es wurde **nur** der Schwanz gelesen, die Zeilenzahl
    /// bezieht sich also auf den gelesenen Teil.
    pub vollstaendig: bool,
    /// Pfad, aus dem gelesen wurde.
    pub pfad: PathBuf,
    /// Hinweis im Klartext, wenn die Datei fehlt oder nicht lesbar ist.
    pub hinweis: Option<String>,
}

/// Hoechstlaenge einer Zeile, die verarbeitet wird.
///
/// Gemessen ist die laengste echte Zeile 4116 B (`panels` mit zwei Baendern).
/// Eine deutlich laengere Zeile ist kein Datensatz mehr, sondern ein Fehler -
/// sie wird gezaehlt und uebersprungen, statt Speicher zu belegen.
const MAX_ZEILE_BYTES: usize = 1 << 16;

/// Vorgabe fuer die Obergrenze eines Rueckwaerts-Laufs. Schuetzt davor, eine
/// Datei mit lauter kaputten Zeilen komplett zu lesen, nur um `limit` zu fuellen.
pub const MAX_BYTES: u64 = 16 << 20;
/// Liest **nur** die letzten `limit` gueltigen Zeilen der Registry.
///
/// ## Warum rueckwaerts gelesen wird
///
/// Der Endpunkt braucht die **neuesten** Runden. Ein zeilenweiser Vorwaertslauf
/// muesste jede Zeile parsen und die letzten N behalten - die Datei waechst mit
/// dem Projekt, die Antwort aber nicht. Gelesen wird deshalb in Bloecken von
/// hinten; im Speicher steht immer nur der ungelesene Anfang des aktuellen
/// Fensters, nie die ganze Datei.
///
/// Fehlt die Datei, ist das **kein Fehler**: zurueck kommt ein leerer Bericht
/// mit `hinweis`.
pub fn lies_letzte(pfad: &Path, limit: usize, max_bytes: u64) -> Lesebericht {
    let limit = limit.min(MAX_LIMIT);
    let mut bericht = Lesebericht {
        pfad: pfad.to_path_buf(),
        vollstaendig: false,
        ..Default::default()
    };
    if limit == 0 {
        bericht.hinweis = Some("limit 0 - es wurden keine Zeilen angefordert".to_string());
        return bericht;
    }

    let mut file = match File::open(pfad) {
        Ok(f) => f,
        Err(e) => {
            bericht.hinweis = Some(hinweis_fehlt(pfad, &e));
            return bericht;
        }
    };
    let ende = match file.seek(SeekFrom::End(0)) {
        Ok(n) => n,
        Err(e) => {
            bericht.hinweis = Some(format!("{} ist nicht lesbar: {e}", pfad.display()));
            return bericht;
        }
    };
    if ende == 0 {
        bericht.vollstaendig = true;
        bericht.hinweis = Some(format!("{} ist leer (0 Byte)", pfad.display()));
        return bericht;
    }

    const BLOCK: u64 = 64 * 1024;
    let mut buf: Vec<u8> = Vec::new();
    let mut pos = ende;
    let mut gelesen: u64 = 0;
    let mut grenze_erreicht = false;
    let mut genug = false;

    'aussen: while pos > 0 {
        let n = BLOCK.min(pos);
        pos -= n;
        let mut block = vec![0u8; n as usize];
        if let Err(e) = file
            .seek(SeekFrom::Start(pos))
            .and_then(|_| file.read_exact(&mut block))
        {
            bericht.hinweis = Some(format!("{} ist nicht lesbar: {e}", pfad.display()));
            return bericht;
        }
        let mut neu = block;
        neu.extend_from_slice(&buf);
        buf = neu;
        gelesen += n;

        // Zeilen von hinten abarbeiten. Der vorderste Eintrag kann abgeschnitten
        // sein - er bleibt liegen, bis der naechste Block davor liegt.
        //
        // Die Zeile wird als KOPIE der Nutzbytes gebildet (ein Bereich reicht,
        // kopiert wird nur, was zur Zeile gehoert). Fuer einen 64-KB-Block mit
        // 2,4-KB-Zeilen sind das 27 Bereiche - kein quadratischer Aufwand.
        let mut bis = buf.len();
        while let Some(rest) = buf[..bis].iter().rposition(|b| *b == b'\n') {
            // Zeile = (rest, bis), ohne den abschliessenden Zeilenumbruch.
            let roh = &buf[rest + 1..bis];
            let zeile: &[u8] = match roh.last() {
                Some(&b'\r') => &roh[..roh.len() - 1],
                _ => roh,
            };
            // Leere Zeilen (z. B. eine Datei, die mit zwei Umbruechen endet)
            // sind keine kaputten Zeilen und werden gar nicht gezaehlt.
            if zeile.iter().all(|b| *b == b' ' || *b == b'\t') {
                bis = rest;
                continue;
            }
            if zeile.len() > MAX_ZEILE_BYTES {
                bericht.uebersprungen += 1;
                bericht.zeilen += 1;
            } else {
                bericht.zeilen += 1;
                let text = jsonread::to_text(zeile);
                if let Some(r) = parse_runde(&text) {
                    // Rueckwaerts gelesen: jede neu gefundene Runde ist AELTER
                    // als alles bisher Gefundene. `insert(0, ..)` stellt damit
                    // die Dateireihenfolge her (aelteste zuerst, neueste
                    // zuletzt) - ohne Nachsortieren.
                    bericht.runden.insert(0, r);
                    if bericht.runden.len() >= limit {
                        genug = true;
                        break 'aussen;
                    }
                } else if !text.trim().is_empty() {
                    bericht.uebersprungen += 1;
                }
            }
            bis = rest;
        }
        // `buf` behaelt nur den vordersten, moeglicherweise abgeschnittenen
        // Rest; die verarbeiteten Bytes fallen weg.
        buf.truncate(bis);

        if gelesen > max_bytes {
            grenze_erreicht = true;
            break;
        }
    }

    // Rest am Dateianfang: die erste Zeile hat kein '\n' davor. Sie wird nur
    // verarbeitet, wenn der Lauf wirklich bei 0 angekommen ist - sonst ist sie
    // vorne abgeschnitten und wuerde als kaputte Zeile gezaehlt.
    if pos == 0 && !grenze_erreicht && !genug && bericht.runden.len() < limit && !buf.is_empty() {
        bericht.zeilen += 1;
        if buf.len() > MAX_ZEILE_BYTES {
            bericht.uebersprungen += 1;
        } else {
            let text = jsonread::to_text(&buf);
            if let Some(r) = parse_runde(&text) {
                bericht.runden.insert(0, r);
            } else if !text.trim().is_empty() {
                bericht.uebersprungen += 1;
            }
        }
    }
    // Aus der Rueckwaerts-Sammlung die Dateireihenfolge machen und auf die
    // letzten `limit` kappen: die NEUESTEN Runden stehen am Dateiende und
    // damit nach dem `reverse()` am Listenende.
    // Die Sammlung steht bereits in Dateireihenfolge (siehe `insert(0, ..)`
    // oben); hier wird nur noch auf die letzten `limit` gekappt. Weil
    // rueckwaerts gelesen wird, sind die zuletzt gefundenen und damit vorne
    // einsortierten Eintraege die NEUESTEN - gekappt wird deshalb hinten.
    bericht.runden.truncate(limit);
    // `vollstaendig` heisst: die Datei wurde von ihrem Anfang an gelesen.
    // Wurde wegen `limit` abgebrochen, ist nur ein Schwanz gelesen - die
    // Zeilenzahl bezieht sich dann auf diesen Teil, und die Anzeige sagt das.
    bericht.vollstaendig = pos == 0 && !genug && !grenze_erreicht;
    bericht
}

/// Liest die **ganze** Datei - fuer den Startbericht, der die Gesamtzahl der
/// Runden nennt. Der Endpunkt benutzt das **nicht**.
pub fn lies_alle(pfad: &Path) -> Lesebericht {
    let mut bericht = Lesebericht {
        pfad: pfad.to_path_buf(),
        vollstaendig: true,
        ..Default::default()
    };
    let text = match std::fs::read(pfad) {
        Ok(b) => jsonread::to_text(&b),
        Err(e) => {
            bericht.hinweis = Some(hinweis_fehlt(pfad, &e));
            return bericht;
        }
    };
    for zeile in text.lines() {
        bericht.zeilen += 1;
        if zeile.len() > MAX_ZEILE_BYTES || zeile.trim().is_empty() {
            continue;
        }
        if let Some(r) = parse_runde(zeile) {
            bericht.runden.push(r);
        } else {
            bericht.uebersprungen += 1;
        }
    }
    bericht
}

/// Der Hinweis, wenn die Datei nicht da ist. Ein fehlender Pfad ist kein Fehler.
fn hinweis_fehlt(pfad: &Path, e: &std::io::Error) -> String {
    if e.kind() == ErrorKind::NotFound {
        format!(
            "{} nicht gefunden - es wird kein Fehler gemeldet, die Liste bleibt leer \
             (Pfad mit --registry setzen)",
            pfad.display()
        )
    } else {
        format!("{} ist nicht lesbar: {e}", pfad.display())
    }
}

// ---------------------------------------------------------------------------
// Aus einer JSON-Zeile eine Runde bauen
// ---------------------------------------------------------------------------

/// Baut eine Runde aus einer JSON-Zeile.
///
/// `None`, wenn die Zeile kein Objekt ist oder kein `experiment_id` traegt:
/// dann ist sie keine Runde, sondern eine kaputte Zeile.
pub fn parse_runde(line: &str) -> Option<Runde> {
    let v = jsonread::parse_line(line)?;
    let experiment_id = v.string("experiment_id")?;
    let panels = parse_panels(v.field("panels"));
    Some(Runde {
        experiment_id,
        capability: v.string("capability").unwrap_or_default(),
        parent_checkpoint: v.string("parent_checkpoint"),
        dataset_version: v.string("dataset_version"),
        generator_version: v.string("generator_version"),
        verifier_version: v.string("verifier_version"),
        seed: v.field("seed").and_then(|x| x.as_u64()),
        training_config: v.string("training_config"),
        evaluation_config: v.string("evaluation_config"),
        steps: v.field("steps").and_then(|x| x.as_u64()),
        learning_rate: v.field("learning_rate").and_then(|x| x.as_f64()),
        replay_share: v.field("replay_share").and_then(|x| x.as_f64()),
        pack: v.string("pack"),
        leiterstufe: v.field("leiterstufe").and_then(|x| x.as_f64()),
        loss: v.field("loss").and_then(|x| x.as_f64()),
        perplexity: v.field("perplexity").and_then(|x| x.as_f64()),
        exact_match: v.field("exact_match").and_then(|x| x.as_f64()),
        structural_accuracy: v.field("structural_accuracy").and_then(|x| x.as_f64()),
        regression_results: v.string("regression_results"),
        failure_distribution: zahl_karte(v.field("failure_distribution")),
        selected_next_action: v.string("selected_next_action"),
        reason_for_action: v.string("reason_for_action"),
        checkpoint: v.string("checkpoint"),
        accepted: v.field("accepted").and_then(|x| match x {
            Value::Bool(b) => Some(*b),
            _ => None,
        }),
        score: v.field("score").and_then(|x| x.as_f64()),
        score_before: v.field("score_before").and_then(|x| x.as_f64()),
        urteil: v.string("urteil"),
        deterministisch: v.field("deterministisch").and_then(|x| match x {
            Value::Bool(b) => Some(*b),
            _ => None,
        }),
        panels,
    })
}

/// `panels` ist ein Array von Baendern. Kein Array (oder leer) bedeutet: diese
/// Runde traegt keine Panel-Zahlen. Das ist ein gueltiger Zustand - gemessen
/// traegt genau eine der 18 Zeilen mit `panels` ein **leeres** Array.
fn parse_panels(v: Option<&Value>) -> Vec<Panel> {
    let Some(Value::Arr(items)) = v else {
        return Vec::new();
    };
    items
        .iter()
        .filter(|x| x.is_obj())
        .map(parse_panel)
        .collect()
}

/// Ein einzelnes Band: Name, Versatz, `vorher` und `nachher`.
fn parse_panel(v: &Value) -> Panel {
    let vorher = v.get("vorher");
    let nachher = v.get("nachher");
    let p_vorher = vorher.map(parse_pro_typ).unwrap_or_default();
    let p_nachher = nachher.map(parse_pro_typ).unwrap_or_default();
    let typ_summe = p_nachher
        .values()
        .filter_map(|z| z.gestellt)
        .fold(0u64, u64::saturating_add);
    Panel {
        name: v.string("name").unwrap_or_default(),
        offset: v.field("offset").and_then(|x| x.as_u64()),
        vorher: p_vorher,
        nachher: p_nachher,
        em_vorher: wert_num(vorher, "exact_match"),
        em_nachher: wert_num(nachher, "exact_match"),
        inhalt_nachher: wert_num(nachher, "exact_match_content"),
        inhalt_gestellt: wert_int(nachher, "content_asked"),
        inhalt_richtig: wert_int(nachher, "content_correct"),
        schwaechster_typ: nachher.and_then(|x| x.string("schwaechster_typ")),
        schwaechster_quote: wert_num(nachher, "schwaechster_quote"),
        gefragt: wert_int(nachher, "asked"),
        typ_summe,
        hinweis: "eigenes Band auf einem eigenen, disjunkten Fragesatz - Panels werden hier \
                  nicht zusammengelegt"
            .to_string(),
    }
}
/// `{typ: [gestellt, richtig]}` - nur Zellen, die wirklich zwei Zahlen tragen.
///
/// Ein Typ mit einem anderen Wert (nur eine Zahl, Text, `null`) wird **nicht**
/// aufgenommen: eine halbe Messung als Zelle auszugeben, waere eine erfundene
/// Zahl.
fn parse_pro_typ(v: &Value) -> BTreeMap<String, Zelle> {
    let mut out = BTreeMap::new();
    let Some(Value::Obj(m)) = v.get("pro_typ") else {
        return out;
    };
    for (typ, wert) in m {
        if let Some(z) = parse_zelle(wert) {
            out.insert(typ.clone(), z);
        }
    }
    out
}

fn parse_zelle(v: &Value) -> Option<Zelle> {
    let Value::Arr(items) = v else {
        return None;
    };
    if items.len() != 2 {
        return None;
    }
    Some(Zelle {
        gestellt: items[0].as_u64(),
        richtig: items[1].as_u64(),
    })
}

/// Zahl aus einem Feld eines verschachtelten Objekts - nur wenn endlich.
fn wert_num(v: Option<&Value>, key: &str) -> Option<f64> {
    v.and_then(|x| x.field(key)).and_then(|x| x.as_f64())
}

fn wert_int(v: Option<&Value>, key: &str) -> Option<u64> {
    v.and_then(|x| x.field(key)).and_then(|x| x.as_u64())
}

/// `{name: ganze Zahl}` - z. B. die Fehlerverteilung einer Runde.
fn zahl_karte(v: Option<&Value>) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    let Some(Value::Obj(m)) = v else {
        return out;
    };
    for (k, val) in m {
        if let Some(n) = val.as_u64() {
            out.insert(k.clone(), n);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Das Urteil zur Marke D-047
// ---------------------------------------------------------------------------

/// Ein Aufgabentyp, der die Typmarke reisst.
#[derive(Debug, Clone, PartialEq)]
pub struct TypReisst {
    pub typ: String,
    pub gestellt: u64,
    pub richtig: u64,
    pub quote: f64,
    /// Um wie viele Prozentpunkte die Marke verfehlt wird (`typ_min - quote`).
    pub fehlt_pp: f64,
}

/// Ein Typ, der in den Panels steht, aber nicht messbar ist.
#[derive(Debug, Clone, PartialEq)]
pub struct TypOhneZahl {
    pub typ: String,
    /// Anzahl der Panels, die diesen Typ nennen.
    pub panels: u64,
}

/// Das Urteil zur Projektmarke, gerechnet **aus den Panels dieser einen Runde**.
///
/// ## Welche Felder die Marke speisen (im Code nachpruefbar)
///
/// * **Kopfzahl:** `panels[i].nachher.exact_match`, gemittelt ueber die Panels
///   mit einem Zahlenwert. Verglichen mit [`D_047_EXACT_MATCH_MIN`]. Gemessen
///   ist diese Zahl bei den Runden mit 120 Fragen je Panel identisch mit dem
///   Top-Level-Feld `exact_match`; die Runden mit 480 Fragen je Panel laufen
///   gegen ein **anderes** Evaluations-Substrat und liegen rund einen Punkt
///   darunter. Weil die Registerzeile ein "fixiertes Panel" nennt, wird die
///   **Panel-Zahl** verwendet und das Top-Level-Feld nur daneben angezeigt.
/// * **Typmarke:** `panels[i].nachher.pro_typ[typ] = [gestellt, richtig]`.
///   Gepoolt wird ueber die Panels (`Summe richtig / Summe gestellt`), und
///   dieses Zusammenlegen wird im Feld [`Urteil::pool_hinweis`] ausdruecklich
///   benannt - es ist die **einzige** Stelle, an der hier gepoolt wird.
/// * Ein Typ mit `gestellt == 0` oder ohne Zahlen reisst die Marke **nicht**:
///   fehlende Messung ist kein Messergebnis, sondern eine Luecke; er steht
///   dann in [`Urteil::typen_ohne_zahl`].
#[derive(Debug, Clone, PartialEq)]
pub struct Urteil {
    /// `true` nur, wenn **beide** Bedingungen halten. Ohne Panel ist es `false`.
    pub erreicht: bool,
    /// `false`, wenn die Runde keine Panel-Zahlen traegt. Dann ist keine
    /// Aussage moeglich - und `erreicht == false` darf nicht als "nicht
    /// bestanden" gelesen werden.
    pub aussage_moeglich: bool,
    /// Kopfzahl: Mittel der `nachher.exact_match` ueber die Panels, `None` wenn
    /// kein Panel eine liefert.
    pub kopf_quote: Option<f64>,
    /// Die Panels, aus denen die Kopfzahl gebildet wurde (Namen).
    pub kopf_aus: Vec<String>,
    /// Haelt die Kopfzahl die Marke?
    pub kopf_erfuellt: bool,
    /// Die Typen unter der Marke, groesster Abstand zuerst.
    pub typen_unter_marke: Vec<TypReisst>,
    /// Typen, die genannt, aber nicht mit Zahlen hinterlegt sind.
    pub typen_ohne_zahl: Vec<TypOhneZahl>,
    /// Anzahl der Typen, die wirklich gemessen wurden.
    pub gemessene_typen: u64,
    /// Wortlaut der beiden Bedingungen.
    pub bedingung_1: String,
    pub bedingung_2: String,
    /// Woher die Marke kommt.
    pub quelle: String,
    /// Was gepoolt wurde - und was nicht.
    pub pool_hinweis: String,
    /// Ein Satz Klartext zum Stand dieser Runde.
    pub begruendung: String,
}
impl Urteil {
    /// Das Urteil, wenn die Runde keine Panel-Zahlen traegt.
    fn ohne_panels() -> Self {
        Self {
            erreicht: false,
            aussage_moeglich: false,
            kopf_quote: None,
            kopf_aus: Vec::new(),
            kopf_erfuellt: false,
            typen_unter_marke: Vec::new(),
            typen_ohne_zahl: Vec::new(),
            gemessene_typen: 0,
            bedingung_1: format!("{D_047_BEDINGUNG_1} ({D_047_EXACT_MATCH_MIN:.2})"),
            bedingung_2: format!("{D_047_BEDINGUNG_2} ({D_047_TYP_MIN:.2})"),
            quelle: D_047_QUELLE.to_string(),
            pool_hinweis: "kein Panel in dieser Runde - es wurde nichts gepoolt".to_string(),
            begruendung: "keine Panel-Zahlen in dieser Runde: das Urteil ist nicht messbar, \
                          es ist NICHT 'nicht bestanden'"
                .to_string(),
        }
    }
}

/// Faellt das Urteil zur Projektmarke **D-047** fuer eine einzelne Runde.
///
/// Das Urteil gilt fuer **diesen einen Eintrag**, nicht fuer den Projektstand.
pub fn beurteile(runde: &Runde) -> Urteil {
    if runde.panels.is_empty() {
        return Urteil::ohne_panels();
    }

    // 1. Kopfzahl: Mittel der Panel-Kopfzahlen (nur Panels mit Wert).
    let mut summe = 0.0f64;
    let mut anzahl = 0u64;
    let mut aus: Vec<String> = Vec::new();
    for p in &runde.panels {
        if let Some(q) = p.em_nachher {
            if q.is_finite() {
                summe += q;
                anzahl += 1;
                aus.push(pname(p));
            }
        }
    }
    let kopf_quote = if anzahl > 0 {
        let m = summe / anzahl as f64;
        if m.is_finite() {
            Some(m)
        } else {
            None
        }
    } else {
        None
    };
    let kopf_erfuellt = kopf_quote
        .map(|q| q >= D_047_EXACT_MATCH_MIN)
        .unwrap_or(false);

    // 2. Typmarke: je Typ ueber die Panels gepoolt. Das ist die EINZIGE
    //    Poolung in diesem Modul; sie wird unten benannt.
    let mut gesamt: BTreeMap<&str, (u64, u64)> = BTreeMap::new();
    let mut genannt: BTreeMap<&str, u64> = BTreeMap::new();
    for p in &runde.panels {
        for (typ, zelle) in &p.nachher {
            *genannt.entry(typ.as_str()).or_insert(0) += 1;
            let e = gesamt.entry(typ.as_str()).or_insert((0, 0));
            if let Some(g) = zelle.gestellt {
                e.0 = e.0.saturating_add(g);
            }
            if let Some(r) = zelle.richtig {
                e.1 = e.1.saturating_add(r);
            }
        }
    }

    let mut unter: Vec<TypReisst> = Vec::new();
    let mut ohne: Vec<TypOhneZahl> = Vec::new();
    let mut gemessen = 0u64;
    for (typ, (g, r)) in &gesamt {
        let n_panels = genannt.get(typ).copied().unwrap_or(0);
        let luecke = TypOhneZahl {
            typ: (*typ).to_string(),
            panels: n_panels,
        };
        if *g == 0 {
            ohne.push(luecke);
            continue;
        }
        let q = *r as f64 / *g as f64;
        if !q.is_finite() {
            ohne.push(luecke);
            continue;
        }
        gemessen += 1;
        if q < D_047_TYP_MIN {
            unter.push(TypReisst {
                typ: (*typ).to_string(),
                gestellt: *g,
                richtig: *r,
                quote: q,
                fehlt_pp: (D_047_TYP_MIN - q) * 100.0,
            });
        }
    }
    // Der groesste Abstand zuerst - das ist der Typ, der den Abschluss aufhaelt.
    unter.sort_by(|a, b| {
        b.fehlt_pp
            .partial_cmp(&a.fehlt_pp)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let erreicht = kopf_erfuellt && unter.is_empty() && gemessen > 0;
    let begruendung = if erreicht {
        format!(
            "beide Bedingungen halten: Kopfzahl {:.1} % (Marke {:.0} %), {} gemessene Typen, \
             keiner unter {:.0} %",
            kopf_quote.unwrap_or(0.0) * 100.0,
            D_047_EXACT_MATCH_MIN * 100.0,
            gemessen,
            D_047_TYP_MIN * 100.0
        )
    } else {
        let mut teile: Vec<String> = Vec::new();
        match kopf_quote {
            Some(q) if kopf_erfuellt => teile.push(format!(
                "Kopfzahl {:.1} % haelt die Marke {:.0} %",
                q * 100.0,
                D_047_EXACT_MATCH_MIN * 100.0
            )),
            Some(q) => teile.push(format!(
                "Kopfzahl {:.1} % verfehlt die Marke {:.0} % um {:.2} Pp",
                q * 100.0,
                D_047_EXACT_MATCH_MIN * 100.0,
                (D_047_EXACT_MATCH_MIN - q) * 100.0
            )),
            None => teile.push("keine Kopfzahl in den Panels".to_string()),
        }
        if unter.is_empty() {
            teile.push(format!(
                "kein gemessener Typ unter {:.0} %",
                D_047_TYP_MIN * 100.0
            ));
        } else {
            teile.push(format!(
                "{} Typ(en) unter {:.0} %: {}",
                unter.len(),
                D_047_TYP_MIN * 100.0,
                unter
                    .iter()
                    .map(|t| format!("{} {:.1} %", t.typ, t.quote * 100.0))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if gemessen == 0 {
            teile.push("kein Typ mit Zahlen - es ist nichts messbar gerechnet".to_string());
        }
        teile.join("; ")
    };

    Urteil {
        erreicht,
        aussage_moeglich: gemessen > 0 || kopf_quote.is_some(),
        kopf_quote,
        kopf_aus: aus,
        kopf_erfuellt,
        typen_unter_marke: unter,
        typen_ohne_zahl: ohne,
        gemessene_typen: gemessen,
        bedingung_1: format!("{D_047_BEDINGUNG_1} ({D_047_EXACT_MATCH_MIN:.2})"),
        bedingung_2: format!("{D_047_BEDINGUNG_2} ({D_047_TYP_MIN:.2})"),
        quelle: D_047_QUELLE.to_string(),
        pool_hinweis: format!(
            "fuer die Typmarke werden die {} Panels dieser Runde je Typ gepoolt \
             (Summe richtig / Summe gestellt); die Panels selbst bleiben getrennt \
             dargestellt",
            runde.panels.len()
        ),
        begruendung,
    }
}

/// Name eines Panels fuer die Anzeige; leerer Name wird zu "Panel ohne Namen".
pub fn pname(p: &Panel) -> String {
    if p.name.trim().is_empty() {
        "Panel ohne Namen".to_string()
    } else {
        format!("Panel {}", p.name)
    }
}

/// Die Union aller Typen ueber alle Panels einer Runde - fuer eine Tabelle, die
/// je Panel eine Spalte zeigt und nichts verschweigt.
pub fn typ_union(runde: &Runde) -> Vec<String> {
    let mut set: BTreeMap<String, ()> = BTreeMap::new();
    for p in &runde.panels {
        for typ in p.nachher.keys() {
            set.insert(typ.clone(), ());
        }
    }
    set.into_keys().collect()
}

/// Beschreibt eine Runde in **einer** Zeile - fuer den Kopf der Anzeige.
///
/// Enthaelt nur, was die Registrierzeile wirklich traegt: Faehigkeit, Runde,
/// Phase (`pack`) und Schritt. Fehlt ein Feld, steht dort ausdruecklich "n/a"
/// und nicht eine erfundene Angabe.
pub fn kontext(r: &Runde) -> String {
    let mut teile: Vec<String> = vec![format!("Faehigkeit {}", anzeige(&r.capability))];
    let phase = match (r.pack.as_deref(), r.leiterstufe) {
        (Some(p), Some(l)) => format!("{p} (Leiterstufe {l})"),
        (Some(p), None) => p.to_string(),
        (None, Some(l)) => format!("n/a (Leiterstufe {l})"),
        (None, None) => "n/a".to_string(),
    };
    teile.push(format!("Phase/Paket {phase}"));
    teile.push(format!("Runde {}", r.experiment_id));
    let schritt = match (&r.parent_checkpoint, &r.checkpoint) {
        (Some(a), Some(b)) => format!("{} -> {}", kurz_datei(a), kurz_datei(b)),
        (Some(a), None) => kurz_datei(a),
        (None, Some(b)) => kurz_datei(b),
        (None, None) => "n/a".to_string(),
    };
    teile.push(format!("Schritt {schritt}"));
    let urteil = match (&r.urteil, r.accepted) {
        (Some(u), Some(true)) => format!("{u} (angenommen)"),
        (Some(u), Some(false)) => format!("{u} (verworfen)"),
        (Some(u), None) => u.clone(),
        (None, None) => "n/a".to_string(),
        (None, Some(a)) => {
            if a {
                "angenommen".to_string()
            } else {
                "verworfen".to_string()
            }
        }
    };
    teile.push(format!("Urteil {urteil}"));
    teile.join("   ·   ")
}

/// Leerer Text wird als "n/a" angezeigt - nie als leere Stelle.
fn anzeige(s: &str) -> String {
    if s.trim().is_empty() {
        "n/a".to_string()
    } else {
        s.to_string()
    }
}

/// Kurzzeile fuer den Startbericht des Servers.
pub fn startzeile(bericht: &Lesebericht) -> String {
    let mit_panels = bericht
        .runden
        .iter()
        .filter(|r| !r.panels.is_empty())
        .count();
    match &bericht.hinweis {
        Some(h) => format!(
            "{} ({} Runden gelesen, davon {} mit panels, {} Zeile(n) uebersprungen) - {}",
            bericht.pfad.display(),
            bericht.runden.len(),
            mit_panels,
            bericht.uebersprungen,
            h
        ),
        None => format!(
            "{} ({} Runden, davon {} mit panels, {} Zeile(n) uebersprungen)",
            bericht.pfad.display(),
            bericht.runden.len(),
            mit_panels,
            bericht.uebersprungen
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    static ZAEHLER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    /// Temp-Ordner mit einer Registry-Datei. Der Zaehler haelt zwei gleichzeitig
    /// laufende Tests auseinander - im selben Prozess liefert eine Uhr in
    /// derselben Mikrosekunde denselben Wert.
    struct Fixture {
        dir: PathBuf,
        datei: PathBuf,
    }

    impl Fixture {
        fn new(tag: &str) -> Self {
            let n = ZAEHLER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let dir =
                std::env::temp_dir().join(format!("sdash-reg-{}-{tag}-{n}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let datei = dir.join("experiments.jsonl");
            Self { dir, datei }
        }

        fn schreibe(&self, text: &str) {
            std::fs::write(&self.datei, text).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    /// Eine synthetische Zeile.
    ///
    /// `prozent` ist die Panel-Kopfzahl `nachher.exact_match` je Panel,
    /// `typen` die Typenliste (Name, gestellt, richtig) je Panel.
    fn zeile(id: &str, prozent: &[f64], typen: &[Vec<(&str, u64, u64)>]) -> String {
        let panels: Vec<String> = typen
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let pro: Vec<String> = t
                    .iter()
                    .map(|(n, g, r)| format!("\"{n}\":[{g},{r}]"))
                    .collect();
                format!(
                    "{{\"name\":\"{}\",\"offset\":{},\"nachher\":{{\"exact_match\":{},\
                     \"exact_match_content\":0.5,\"content_asked\":10,\"content_correct\":5,\
                     \"pro_typ\":{{{}}}}}}}",
                    (b'A' + i as u8) as char,
                    i * 5,
                    prozent[i],
                    pro.join(",")
                )
            })
            .collect();
        format!(
            "{{\"experiment_id\":\"{id}\",\"capability\":\"sequence_reasoning\",\
             \"parent_checkpoint\":\"checkpoints/eltern-{id}.safetensors\",\
             \"steps\":100,\"learning_rate\":0.0003,\"loss\":0.5,\"perplexity\":1.6,\
             \"exact_match\":{},\"urteil\":\"ANGENOMMEN\",\"accepted\":true,\
             \"score\":0.5,\"score_before\":0.4,\"failure_distribution\":{{\"offen\":3}},\
             \"panels\":[{}]}}",
            prozent.first().copied().unwrap_or(0.0),
            panels.join(",")
        )
    }
    // (a) Marke erfuellt ----------------------------------------------------
    #[test]
    fn marke_ist_erfuellt_wenn_kopfzahl_und_alle_typen_halten() {
        let text = zeile(
            "exp-a",
            &[0.95, 0.92],
            &[
                vec![("luecke", 20, 19), ("reparatur", 20, 16)],
                vec![("luecke", 20, 18), ("reparatur", 20, 17)],
            ],
        );
        let r = parse_runde(&text).expect("Zeile muss lesbar sein");
        assert_eq!(r.experiment_id, "exp-a");
        assert_eq!(r.capability, "sequence_reasoning");
        assert_eq!(r.panels.len(), 2);
        assert_eq!(r.panels[0].name, "A");
        assert_eq!(r.panels[1].name, "B");
        assert_eq!(r.panels[1].offset, Some(5));
        assert_eq!(r.failure_distribution.get("offen"), Some(&3));
        // Kopfzahl (0.95 + 0.92) / 2 = 0.935
        let u = beurteile(&r);
        let kopf = u.kopf_quote.expect("Kopfzahl");
        assert!((kopf - 0.935).abs() < 1e-9, "Kopfzahl war {kopf}");
        assert!(u.kopf_erfuellt);
        assert!(u.typen_unter_marke.is_empty(), "{:?}", u.typen_unter_marke);
        assert_eq!(u.gemessene_typen, 2);
        assert!(u.erreicht, "Urteil: {u:?}");
        assert!(u.aussage_moeglich);
        assert_eq!(u.kopf_aus.len(), 2);
        // Der Wortlaut beider Bedingungen muss im Urteil stehen.
        assert!(u.bedingung_1.contains("90 %"), "{}", u.bedingung_1);
        assert!(u.bedingung_2.contains("75 %"), "{}", u.bedingung_2);
        assert!(u.quelle.contains("D-047"));
        assert!(u.quelle.contains("docs/abschluss-vorbereitungsphase.md"));
    }

    // (b) Ein Typ unter 75 % -> nicht erfuellt ------------------------------
    #[test]
    fn ein_typ_unter_der_marke_kippt_das_urteil() {
        let text = zeile(
            "exp-b",
            &[0.95, 0.93],
            &[
                vec![("luecke", 20, 19), ("reparatur", 20, 13)],
                vec![("luecke", 20, 20), ("reparatur", 20, 12)],
            ],
        );
        let r = parse_runde(&text).unwrap();
        let u = beurteile(&r);
        assert!(u.kopf_erfuellt, "die Kopfzahl allein haelt");
        assert!(!u.erreicht, "ein Typ unter 75 % muss das Urteil kippen");
        assert_eq!(u.typen_unter_marke.len(), 1, "{:?}", u.typen_unter_marke);
        let t = &u.typen_unter_marke[0];
        assert_eq!(t.typ, "reparatur");
        // gepoolt: 25 von 40 = 62,5 %
        assert_eq!(t.gestellt, 40);
        assert_eq!(t.richtig, 25);
        assert!((t.quote - 0.625).abs() < 1e-9, "Quote war {}", t.quote);
        assert!(
            (t.fehlt_pp - 12.5).abs() < 1e-9,
            "Abstand war {}",
            t.fehlt_pp
        );
        assert!(u.begruendung.contains("reparatur"), "{}", u.begruendung);
        assert!(
            u.pool_hinweis.contains("getrennt dargestellt"),
            "{}",
            u.pool_hinweis
        );
        assert!(u.typen_ohne_zahl.is_empty());
        // Der Typ muss auch in der Typentabelle auftauchen, die der Server baut.
        assert!(typ_union(&r).contains(&"reparatur".to_string()));
    }

    /// Genau auf der Typmarke: 75 % ist **nicht** unter 75 %.
    #[test]
    fn genau_auf_der_typmarke_haelt() {
        let text = zeile("exp-grenze", &[0.9], &[vec![("luecke", 20, 15)]]);
        let r = parse_runde(&text).unwrap();
        let u = beurteile(&r);
        assert!(u.typen_unter_marke.is_empty(), "{:?}", u.typen_unter_marke);
        assert!(u.erreicht, "{u:?}");
    }

    /// Genau auf der Kopfmarke: 90 % haelt (">= 90 %").
    #[test]
    fn genau_auf_der_kopfmarke_haelt() {
        let text = zeile("exp-kopf", &[0.9], &[vec![("luecke", 20, 20)]]);
        let r = parse_runde(&text).unwrap();
        let u = beurteile(&r);
        assert!(u.kopf_erfuellt);
        assert!(u.erreicht, "{u:?}");
    }

    // (c) Fehlende panels -> kein Panel + kein Absturz ----------------------
    #[test]
    fn runde_ohne_panels_hat_kein_panel_und_stuerzt_nicht_ab() {
        // (1) Schluessel fehlt ganz.
        let ohne = "{\"experiment_id\":\"exp-c\",\"capability\":\"sequence_reasoning\",\
                    \"steps\":600,\"urteil\":\"KEINE_RUNDE\",\"accepted\":false}";
        let r = parse_runde(ohne).expect("auch ohne panels eine Runde");
        assert!(r.panels.is_empty());
        let u = beurteile(&r);
        assert!(!u.erreicht);
        assert!(
            !u.aussage_moeglich,
            "ohne Panel ist keine Aussage moeglich - 'nicht erreicht' waere irrefuehrend"
        );
        assert!(u.begruendung.contains("nicht messbar"), "{}", u.begruendung);
        assert!(u.pool_hinweis.contains("kein Panel"), "{}", u.pool_hinweis);
        assert!(typ_union(&r).is_empty());

        // (2) Schluessel ist ein LEERES Array (gemessener Fall der echten Datei).
        let leer = "{\"experiment_id\":\"exp-c2\",\"capability\":\"sequence_reasoning\",\
                    \"panels\":[]}";
        let r2 = parse_runde(leer).unwrap();
        assert!(r2.panels.is_empty());
        assert!(!beurteile(&r2).erreicht);

        // (3) panels ist kein Array, sondern Muell -> ebenso leer, kein Absturz.
        for muell in ["5", "\"zwei\"", "null", "{}"] {
            let t = format!("{{\"experiment_id\":\"exp-c3\",\"panels\":{muell}}}");
            let r3 = parse_runde(&t).expect("Runde bleibt lesbar");
            assert!(r3.panels.is_empty(), "panels={muell} wurde zu einem Panel");
        }
    }
    /// Ein Panel mit leerem `pro_typ` ist ein Panel OHNE Zahlen - nicht kein
    /// Panel. Das ist der Unterschied, den die Anzeige sichtbar machen muss.
    #[test]
    fn panel_ohne_zahlen_bleibt_ein_panel() {
        let text = "{\"experiment_id\":\"exp-d\",\"panels\":[{\"name\":\"A\",\
                    \"nachher\":{\"exact_match\":0.5,\"pro_typ\":{}}}]}";
        let r = parse_runde(text).unwrap();
        assert_eq!(r.panels.len(), 1, "das Panel selbst fehlt nicht");
        let u = beurteile(&r);
        assert_eq!(u.gemessene_typen, 0);
        assert!(!u.erreicht);
        assert!(!u.kopf_erfuellt, "0.5 liegt unter 0.90");
        assert!(
            u.begruendung.contains("kein Typ mit Zahlen")
                || u.begruendung.contains("kein gemessener Typ"),
            "{}",
            u.begruendung
        );
    }

    // (d) Kaputte Zeile -> uebersprungen ------------------------------------
    #[test]
    fn kaputte_zeile_wird_gezaehlt_und_uebersprungen() {
        let f = Fixture::new("kaputt");
        let gute = zeile("exp-e", &[0.9], &[vec![("luecke", 10, 9)]]);
        // Die kaputte Zeile liegt bewusst ZWISCHEN zwei gueltigen.
        f.schreibe(&format!(
            "{gute}\nkein json\n{}\n",
            zeile("exp-f", &[0.8], &[vec![("luecke", 10, 8)]])
        ));

        let b = lies_letzte(&f.datei, 10, MAX_BYTES);
        assert_eq!(
            b.runden.len(),
            2,
            "die kaputte Zeile darf keine Runde werden"
        );
        assert_eq!(b.uebersprungen, 1);
        assert_eq!(b.zeilen, 3);
        assert!(b.vollstaendig);
        assert!(b.hinweis.is_none(), "{:?}", b.hinweis);
        // Die Reihenfolge bleibt die der Datei.
        assert_eq!(b.runden[0].experiment_id, "exp-e");
        assert_eq!(b.runden[1].experiment_id, "exp-f");

        // Und eine Zeile ohne `experiment_id` ist ebenfalls keine Runde.
        f.schreibe("{\"capability\":\"x\",\"loss\":1.0}\n");
        let b2 = lies_letzte(&f.datei, 10, MAX_BYTES);
        assert!(b2.runden.is_empty());
        assert_eq!(b2.uebersprungen, 1);
    }

    /// Nur die letzten N Zeilen, und zwar die NEUESTEN.
    #[test]
    fn es_werden_nur_die_letzten_n_zeilen_gelesen() {
        let f = Fixture::new("limit");
        let mut text = String::new();
        for i in 0..40 {
            text.push_str(&zeile(
                &format!("exp-{i:02}"),
                &[0.5],
                &[vec![("luecke", 10, 5)]],
            ));
            text.push('\n');
        }
        f.schreibe(&text);

        let b = lies_letzte(&f.datei, 3, MAX_BYTES);
        assert_eq!(b.runden.len(), 3, "limit wirkt nicht");
        assert_eq!(b.runden[0].experiment_id, "exp-37");
        assert_eq!(b.runden[2].experiment_id, "exp-39");
        assert_eq!(b.zeilen, 3, "nur drei Zeilen wurden ueberhaupt angesehen");
        assert!(!b.vollstaendig, "nur der Schwanz wurde gelesen");

        // Ohne Grenze kommen alle 40 - dann ist der Lauf vollstaendig.
        let alle = lies_letzte(&f.datei, MAX_LIMIT, MAX_BYTES);
        assert_eq!(alle.runden.len(), 40);
        assert!(alle.vollstaendig);

        // Und der Vollscan liefert dieselbe Zahl.
        assert_eq!(lies_alle(&f.datei).runden.len(), 40);
    }

    /// Eine fehlende Datei ist kein Fehler.
    #[test]
    fn fehlende_datei_gibt_leere_liste_mit_hinweis() {
        let f = Fixture::new("fehlt");
        let b = lies_letzte(&f.datei, 10, MAX_BYTES);
        assert!(b.runden.is_empty());
        assert_eq!(b.uebersprungen, 0);
        let h = b.hinweis.expect("Hinweis fehlt");
        assert!(h.contains("nicht gefunden"), "{h}");
        assert!(
            h.contains("--registry"),
            "der Hinweis muss den Schalter nennen: {h}"
        );

        let a = lies_alle(&f.datei);
        assert!(a.runden.is_empty());
        assert!(a.hinweis.is_some());
    }

    /// Eine leere Datei nennt den Grund, statt still leer zu bleiben.
    #[test]
    fn leere_datei_wird_als_leer_gemeldet() {
        let f = Fixture::new("leer");
        f.schreibe("");
        let b = lies_letzte(&f.datei, 10, MAX_BYTES);
        assert!(b.runden.is_empty());
        assert!(b.hinweis.unwrap().contains("leer"));
    }

    /// Eine Zeile ohne abschliessenden Zeilenumbruch darf nicht verloren gehen
    /// (die letzte Zeile einer wachsenden Datei).
    #[test]
    fn letzte_zeile_ohne_umbruch_zaehlt() {
        let f = Fixture::new("ohne-umbruch");
        f.schreibe(&zeile("exp-h", &[0.9], &[vec![("luecke", 10, 9)]]));
        let b = lies_letzte(&f.datei, 10, MAX_BYTES);
        assert_eq!(b.runden.len(), 1);
        assert_eq!(b.runden[0].experiment_id, "exp-h");
    }

    /// Die Rueckwaerts-Lesung darf keine Zeile verlieren, auch nicht ueber
    /// mehrere 64-KB-Bloecke hinweg.
    #[test]
    fn rueckwaertslesung_verliert_keine_zeile() {
        let f = Fixture::new("bloecke");
        let mut text = String::new();
        // 300 Zeilen a ~55 B = ~16 KB -> mehrere Bloecke bei kleiner Grenze.
        for i in 0..300 {
            text.push_str(&format!(
                "{{\"experiment_id\":\"exp-{i:03}\",\"capability\":\"c\",\"panels\":[]}}\n"
            ));
        }
        f.schreibe(&text);
        let b = lies_letzte(&f.datei, 300, MAX_BYTES);
        assert_eq!(b.runden.len(), 300, "Zeilen gingen verloren");
        assert_eq!(b.runden[0].experiment_id, "exp-000");
        assert_eq!(b.runden[299].experiment_id, "exp-299");
        assert!(b.vollstaendig);
        assert_eq!(b.uebersprungen, 0);

        // Gegenprobe mit einem sehr kleinen Byte-Budget: dann bricht die
        // Rueckwaertslesung ab, statt die ganze Datei zu lesen - und sagt das.
        let knapp = lies_letzte(&f.datei, 300, 4096);
        assert!(knapp.runden.len() < 300, "die Grenze wirkte nicht");
        assert!(!knapp.vollstaendig);
    }
    // (e) Nicht-endliche Zahl wird nicht als Zahl weitergereicht --------------
    #[test]
    fn nicht_endliche_zahl_wird_keine_zahl() {
        // JSON kennt die Literale NaN und Infinity nicht, `parse_line` lehnt sie
        // deshalb ab - die Zeile ist damit kaputt und wird gezaehlt, nicht
        // stillschweigend mit einer erfundenen Zahl gelesen.
        let f = Fixture::new("nan");
        f.schreibe("{\"experiment_id\":\"exp-nan\",\"loss\":NaN,\"panels\":[]}\n");
        let b = lies_letzte(&f.datei, 10, MAX_BYTES);
        assert!(
            b.runden.is_empty(),
            "eine NaN-Zeile ist keine gueltige Runde"
        );
        assert_eq!(b.uebersprungen, 1);

        // Eine Zelle ohne Quote ist keine Quote, sondern eine Luecke.
        let z0 = Zelle {
            gestellt: Some(0),
            richtig: Some(0),
        };
        assert_eq!(z0.quote(), None);
        let z1 = Zelle {
            gestellt: None,
            richtig: Some(3),
        };
        assert_eq!(z1.quote(), None);
        let z2 = Zelle {
            gestellt: Some(4),
            richtig: Some(2),
        };
        assert_eq!(z2.quote(), Some(0.5));

        // Eine halbe Zelle (nur eine Zahl) wird gar nicht erst zur Zelle.
        let t = "{\"experiment_id\":\"exp-halb\",\"panels\":[{\"name\":\"A\",\"nachher\":\
                 {\"pro_typ\":{\"luecke\":[10],\"offen\":\"zwei\",\"regel\":null,\
                 \"stelle\":[4,4]}}}]}";
        let r = parse_runde(t).unwrap();
        let p = &r.panels[0];
        assert_eq!(
            p.nachher.len(),
            1,
            "nur die vollstaendige Zelle zaehlt: {:?}",
            p.nachher
        );
        assert!(p.nachher.contains_key("stelle"));
        assert_eq!(p.typ_summe, 4);
        // Und der Typ ohne vollstaendige Zelle reisst die Marke NICHT - er ist
        // eine Luecke, keine Null.
        let u = beurteile(&r);
        assert_eq!(u.gemessene_typen, 1);
        assert!(u.typen_unter_marke.is_empty(), "{:?}", u.typen_unter_marke);
    }

    /// Eine Zelle mit gestellt = 0 landet in `typen_ohne_zahl`, nicht in der
    /// Liste der gerissenen Typen.
    #[test]
    fn typ_mit_null_gestellt_ist_eine_luecke_kein_durchfall() {
        let text = zeile(
            "exp-i",
            &[0.9],
            &[vec![("luecke", 10, 10), ("vergleich", 0, 0)]],
        );
        let r = parse_runde(&text).unwrap();
        let u = beurteile(&r);
        assert_eq!(u.gemessene_typen, 1);
        assert!(u.typen_unter_marke.is_empty(), "{:?}", u.typen_unter_marke);
        assert_eq!(u.typen_ohne_zahl.len(), 1);
        assert_eq!(u.typen_ohne_zahl[0].typ, "vergleich");
        assert_eq!(u.typen_ohne_zahl[0].panels, 1);
    }

    /// Die Panels bleiben getrennt: die Union der Typen kennt beide Baender.
    #[test]
    fn typ_union_ueber_alle_panels() {
        let text = zeile(
            "exp-j",
            &[0.9, 0.9],
            &[vec![("luecke", 5, 5)], vec![("stelle", 5, 5)]],
        );
        let r = parse_runde(&text).unwrap();
        assert_eq!(
            typ_union(&r),
            vec!["luecke".to_string(), "stelle".to_string()]
        );
        // Jedes Panel kennt nur seine eigenen Typen - nichts wird gemischt.
        assert_eq!(r.panels[0].nachher.len(), 1);
        assert_eq!(r.panels[1].nachher.len(), 1);
        // Und die Typ-Summe je Panel ist die Zahl der Fragen DIESES Bandes.
        assert_eq!(r.panels[0].typ_summe, 5);
        assert_eq!(r.panels[1].typ_summe, 5);
    }

    /// Der Kurzname ist bei gleichem `experiment_id` unterscheidbar.
    #[test]
    fn kurzname_unterscheidet_gleiche_experiment_ids() {
        let a = parse_runde(&zeile(
            "sequence_reasoning-i1",
            &[0.9],
            &[vec![("luecke", 5, 5)]],
        ))
        .unwrap();
        let b = parse_runde(
            &zeile("sequence_reasoning-i1", &[0.9], &[vec![("luecke", 5, 5)]])
                .replace("eltern-sequence_reasoning-i1", "eltern-andere-runde"),
        )
        .unwrap();
        assert_ne!(a.kurzname(), b.kurzname());
        assert!(a.kurzname().contains("100 Schritte"), "{}", a.kurzname());
    }

    /// Die Uebersichtszahlen fuer den Startbericht.
    #[test]
    fn startzeile_nennt_runden_und_panels() {
        let f = Fixture::new("start");
        f.schreibe(&format!(
            "{}\n{}\n{}\n",
            zeile("exp-k", &[0.9], &[vec![("luecke", 5, 5)]]),
            zeile("exp-l", &[0.9], &[vec![("luecke", 5, 5)]]),
            "{\"experiment_id\":\"exp-m\",\"panels\":[]}"
        ));
        let b = lies_alle(&f.datei);
        let s = startzeile(&b);
        assert!(s.contains("3 Runden"), "{s}");
        assert!(s.contains("2 mit panels"), "{s}");
    }

    /// Der Grenzfall, den die echte Datei traegt: `leiterstufe` ist `null`.
    /// Das ist "nicht gesetzt" und darf keine 0 werden.
    ///
    /// Gemessen an `registry/experiments.jsonl` (38 Zeilen): der Schluessel
    /// fehlt in 30 Zeilen ganz, steht in 8 Zeilen auf `null` und traegt in
    /// **keiner** Zeile eine Zahl. Beide Faelle muessen `None` ergeben.
    #[test]
    fn null_werte_werden_zu_none_nicht_zu_null_zahl() {
        let t = "{\"experiment_id\":\"exp-n\",\"leiterstufe\":null,\"replay_share\":0.35,\
                 \"pack\":\"-\",\"compositional_accuracy\":null,\"deterministisch\":true,\
                 \"panels\":[]}";
        let r = parse_runde(t).unwrap();
        assert_eq!(r.leiterstufe, None);
        assert_eq!(r.replay_share, Some(0.35));
        assert_eq!(r.pack.as_deref(), Some("-"));
        assert_eq!(r.deterministisch, Some(true));
        assert_eq!(r.failure_distribution.len(), 0);
    }
}
