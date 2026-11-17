//! Korpus "Format-Erkennung aus Bytes" - Familie 2 der Denk-Korpora (**D-021**).
//!
//! **Die Spezifikation ist der Verifier.** Ein Format ist genau dann erkannt, wenn
//! ein **Parser** die Bytes akzeptiert: JSON ueber `serde_json`, TOML ueber `toml`,
//! CSV ueber eine Feldzahl-Regel, XML ueber Klammerbilanz, die Signaturformate
//! ueber Kopf **und** Gueltigkeitsprobe (ein PNG-Kopf ohne `IHDR`/`IEND` ist kein
//! PNG). Damit gibt es keine Heuristik, die man glauben muesste, und keinen
//! Modellaufruf - die Antwort steht fest.
//!
//! ## Fuenf aufsteigende Klassen (D-021)
//!
//! | Klasse | Frage | Beispiel |
//! |---|---|---|
//! | 1 Signatur | Welches Format laut Kopf? | `89 50 4E 47` -> PNG |
//! | 2 Signatur gegen Gueltigkeit | Kopf ja, Format nein? | PNG-Kopf ohne `IHDR`/`IEND` |
//! | 3 **Struktur ohne Signatur** | Was ist es inhaltlich? | JSON, JSONL, CSV, TOML, XML, Klartext |
//! | 4 Widerspruch | Kopf gegen Struktur? | PNG-Kopf, Textkoerper |
//! | 5 Falsche Endung | Passt der Name zum Inhalt? | JSON-Inhalt, `.png` |
//!
//! **Antwortwert "nicht bestimmbar" ist zugelassen** und kein Fehler: fuer
//! hoch-entropische Daten ohne Signatur und ohne Struktur ist Erkennung
//! **prinzipiell unmoeglich** (Roadmap Z. 548 sagt dasselbe fuer Verschluesselung).
//!
//! ## Warum hier **keine** Rohbytes in einen Datensatz wandern
//!
//! Die Spezifikation sperrt die Bytes `0x00` und `0xF9` in Datensaetzen. Eine
//! Byteprobe wird deshalb im Korpus als **Hex-Text** gezeigt (z. B.
//! `89 50 4E 47`), nicht roh - dieser Modulteil arbeitet auf **Bytes** und ist
//! davon unabhaengig; das Rendern ist Sache des Generators (spaeterer Schritt).

/// Laengster Kopf, der fuer die Signaturpruefung gebraucht wird.
pub const KOPF_MAX: usize = 64;

/// Die fuenf Klassen aus D-021 plus den zugelassenen Wert "nicht bestimmbar".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Klasse {
    /// Signatur erkannt.
    Signatur,
    /// Signatur vorhanden, Format aber ungueltig.
    SignaturGueltig,
    /// Ohne Signatur aus der Struktur erkannt.
    Struktur,
    /// Kopf und Struktur widersprechen sich.
    Widerspruch,
    /// Inhalt und Dateiendung widersprechen sich.
    FalscheEndung,
    /// Prinzipiell nicht entscheidbar.
    Unbestimmbar,
}

impl Klasse {
    /// Textform fuer Aufgabentext und Manifest.
    pub fn key(self) -> &'static str {
        match self {
            Klasse::Signatur => "signatur",
            Klasse::SignaturGueltig => "signatur-gueltig",
            Klasse::Struktur => "struktur",
            Klasse::Widerspruch => "widerspruch",
            Klasse::FalscheEndung => "falsche-endung",
            Klasse::Unbestimmbar => "unbestimmbar",
        }
    }
}

/// Ergebnis einer Erkennung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Befund {
    /// Erkanntes Format (`None` = nicht bestimmbar).
    pub label: Option<&'static str>,
    pub klasse: Klasse,
    /// Begruendung - der Denkweg im Kleinen.
    pub begruendung: String,
}

/// Ein Signaturformat mit Gueltigkeitsprobe.
///
/// Oeffentlich, weil [`SIGS`] die geschlossene Liste ist, auf die sich Generator,
/// Verifier und Doku gemeinsam berufen (O-K21).
pub struct Sig {
    pub label: &'static str,
    pub magie: &'static [u8],
    pub endungen: &'static [&'static str],
    /// Prueft die innere Gueltigkeit (Klasse 2).
    pub gueltig: fn(&[u8]) -> bool,
    /// Was die Gueltigkeitsprobe verlangt (fuer die Begruendung).
    pub bedingung: &'static str,
    /// Binaerformat? Nur dort ist ein Textrumpf ein **Widerspruch**; PDF ist
    /// textbasiert und darf nicht als Widerspruch gelten.
    pub binaer: bool,
}

fn png_gueltig(b: &[u8]) -> bool {
    b.len() > 20 && b[12..16] == *b"IHDR" && b.windows(4).any(|w| w == b"IEND")
}

fn gzip_gueltig(b: &[u8]) -> bool {
    b.len() > 3 && b[2] == 8 && (b[3] & 0xE0) == 0
}

fn pdf_gueltig(b: &[u8]) -> bool {
    b.len() > 8 && b.windows(5).any(|w| w == b"%%EOF")
}

fn elf_gueltig(b: &[u8]) -> bool {
    b.len() >= 16 && (b[4] == 1 || b[4] == 2) && b[6] == 1
}

fn zip_gueltig(b: &[u8]) -> bool {
    b.len() >= 30
}

fn bshard_gueltig(b: &[u8]) -> bool {
    b.len() >= 32
}

/// **Geschlossene, versionierte Formatliste** (O-K21): nur was hier steht, wird
/// erkannt - alles andere ist "nicht bestimmbar" und nicht etwa geraten.
pub const SIGS: &[Sig] = &[
    Sig {
        label: "png",
        magie: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        endungen: &["png"],
        gueltig: png_gueltig,
        bedingung: "IHDR im Kopf und IEND im Rumpf",
        binaer: true,
    },
    Sig {
        label: "gzip",
        magie: &[0x1F, 0x8B],
        endungen: &["gz", "tgz"],
        gueltig: gzip_gueltig,
        bedingung: "Kompressionsverfahren 8 und reservierte Bits null",
        binaer: true,
    },
    Sig {
        label: "elf",
        magie: &[0x7F, 0x45, 0x4C, 0x46],
        endungen: &["elf", "so", "o"],
        gueltig: elf_gueltig,
        bedingung: "Klasse 1 oder 2 und Version 1",
        binaer: true,
    },
    Sig {
        label: "pdf",
        magie: b"%PDF-",
        endungen: &["pdf"],
        gueltig: pdf_gueltig,
        bedingung: "%%EOF im Rumpf",
        binaer: false,
    },
    Sig {
        label: "zip",
        magie: &[0x50, 0x4B, 0x03, 0x04],
        endungen: &["zip", "jar", "docx"],
        gueltig: zip_gueltig,
        bedingung: "mindestens ein vollstaendiger Kopf (30 Byte)",
        binaer: true,
    },
    Sig {
        label: "bshard",
        magie: b"BSHD",
        endungen: &["bshard", "bin"],
        gueltig: bshard_gueltig,
        bedingung: "mindestens der 32-Byte-Kopf",
        binaer: true,
    },
];

/// Ist der ganze Puffer druckbarer Text (ASCII plus Zeilenenden/Tabulator)?
pub fn ist_text(b: &[u8]) -> bool {
    !b.is_empty()
        && b.iter()
            .all(|c| matches!(*c, 0x20..=0x7E | b'\n' | b'\r' | b'\t'))
}

/// Strukturbestimmung **ohne** Signatur (Klasse 3) - jeder Zweig ist ein Parser.
pub fn erkenne_struktur(b: &[u8]) -> Option<&'static str> {
    if !ist_text(b) {
        return None;
    }
    let text = String::from_utf8_lossy(b);
    let t = text.trim();

    // JSON: der Parser entscheidet, nicht ein Muster.
    if (t.starts_with('{') || t.starts_with('['))
        && serde_json::from_str::<serde_json::Value>(t).is_ok()
    {
        return Some("json");
    }

    // JSONL: **jede** Zeile fuer sich gueltiges JSON, mindestens zwei Zeilen.
    let zeilen: Vec<&str> = t.lines().filter(|l| !l.trim().is_empty()).collect();
    if zeilen.len() >= 2
        && zeilen
            .iter()
            .all(|l| serde_json::from_str::<serde_json::Value>(l.trim()).is_ok())
    {
        return Some("jsonl");
    }

    // CSV: mindestens zwei Zeilen mit gleicher Feldzahl (>= 2 Felder).
    if zeilen.len() >= 2 {
        let felder: Vec<usize> = zeilen.iter().map(|l| l.split(',').count()).collect();
        if felder.iter().all(|f| *f >= 2) && felder.iter().all(|f| *f == felder[0]) {
            return Some("csv");
        }
    }

    // TOML: wieder der Parser.
    if t.contains('=') && toml::from_str::<toml::Value>(t).is_ok() {
        return Some("toml");
    }

    // XML: Klammerbilanz und mindestens ein Schluss-Tag.
    if t.starts_with('<') && t.ends_with('>') && t.contains("</") {
        let auf = t.matches('<').count();
        let zu = t.matches('>').count();
        if auf == zu && auf >= 2 {
            return Some("xml");
        }
    }

    // Klartext: druckbar, aber keine der obigen Strukturen.
    if t.chars().count() >= 4 {
        return Some("klartext");
    }
    None
}

/// Erkennt ein Format. `endung` ist die behauptete Dateiendung (ohne Punkt).
///
/// Reihenfolge: Signatur -> Gueltigkeit -> Widerspruch -> Struktur -> Endung.
/// Jeder Zweig legt eine **Begruendung** vor, denn die ist der Denkweg.
pub fn erkenne(b: &[u8], endung: Option<&str>) -> Befund {
    let mut sig_treffer: Option<&Sig> = None;
    for s in SIGS {
        if b.len() >= s.magie.len() && &b[..s.magie.len()] == s.magie {
            sig_treffer = Some(s);
            break;
        }
    }

    if let Some(s) = sig_treffer {
        // Klasse 2: Kopf ja, Format nein.
        // Der Rumpf nach der Signatur entscheidet ueber den Widerspruch, und nur
        // bei **Binaerformaten** (ein PDF ist textbasiert). "Widerspruch" ist die
        // staerkere Aussage als "ungueltig" und wird deshalb vorgezogen: ein
        // PNG-Kopf mit JSON-Rumpf ist beides.
        let rumpf_struktur = if s.binaer {
            erkenne_struktur(&b[s.magie.len().min(b.len())..])
        } else {
            None
        };
        if rumpf_struktur.is_none() && !(s.gueltig)(b) {
            return Befund {
                label: Some(s.label),
                klasse: Klasse::SignaturGueltig,
                begruendung: format!(
                    "Signatur {} erkannt, aber die Gueltigkeitsprobe scheitert: {} fehlt.",
                    s.label, s.bedingung
                ),
            };
        }
        // Klasse 4: Kopf gegen Struktur. Geprueft wird der **Rumpf nach der
        // Signatur** - nicht der Gesamtpuffer, denn der beginnt ja mit der
        // Binaersignatur und ist damit nie "Text". Und nur bei **Binaerformaten**:
        // ein PDF ist textbasiert und darf kein Widerspruch sein.
        let rumpf = &b[s.magie.len().min(b.len())..];
        if s.binaer {
            if let Some(struktur) = erkenne_struktur(rumpf) {
                return Befund {
                    label: Some(struktur),
                    klasse: Klasse::Widerspruch,
                    begruendung: format!(
                        "Signatur {} sagt Binaerformat, der Rumpf ist aber {} - der Kopf passt nicht zum Inhalt.",
                        s.label, struktur
                    ),
                };
            }
        }
        // Klasse 5: Endung gegen Inhalt.
        if let Some(e) = endung {
            if !s.endungen.contains(&e) {
                return Befund {
                    label: Some(s.label),
                    klasse: Klasse::FalscheEndung,
                    begruendung: format!(
                        "Inhalt ist {}, die Endung .{} gehoert aber zu {:?}.",
                        s.label, e, s.endungen
                    ),
                };
            }
        }
        return Befund {
            label: Some(s.label),
            klasse: Klasse::Signatur,
            begruendung: format!(
                "Signatur {} im Kopf, Gueltigkeitsprobe bestanden ({}).",
                s.label, s.bedingung
            ),
        };
    }

    // Klasse 3: ohne Signatur aus der Struktur.
    if let Some(label) = erkenne_struktur(b) {
        if let Some(e) = endung {
            let erwartet: &[&str] = match label {
                "json" => &["json"],
                "jsonl" => &["jsonl", "ndjson", "json"],
                "csv" => &["csv", "tsv"],
                "toml" => &["toml"],
                "xml" => &["xml", "svg", "html"],
                _ => &["txt", "md", "log", "text"],
            };
            if !erwartet.contains(&e) {
                return Befund {
                    label: Some(label),
                    klasse: Klasse::FalscheEndung,
                    begruendung: format!("Inhalt ist {label}, die Endung .{e} passt nicht dazu."),
                };
            }
        }
        return Befund {
            label: Some(label),
            klasse: Klasse::Struktur,
            begruendung: format!(
                "Keine Signatur; der {label}-Parser akzeptiert den Inhalt, also ist es {label}."
            ),
        };
    }

    Befund {
        label: None,
        klasse: Klasse::Unbestimmbar,
        begruendung:
            "Keine Signatur und keine lesbare Struktur: hoch-entropische Daten sehen einander \
             gleich, die Erkennung ist hier prinzipiell nicht moeglich."
                .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_ok() -> Vec<u8> {
        let mut v = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        v.extend_from_slice(&[0, 0, 0, 13]);
        v.extend_from_slice(b"IHDR");
        v.extend_from_slice(&[0u8; 16]);
        v.extend_from_slice(b"IEND");
        v
    }

    fn zufall(n: usize) -> Vec<u8> {
        (0..n)
            .map(|i| ((i as u8).wrapping_mul(37).wrapping_add(11)) | 0x80)
            .collect()
    }

    #[test]
    fn klasse_1_signatur() {
        let b = png_ok();
        let r = erkenne(&b, Some("png"));
        assert_eq!(r.label, Some("png"));
        assert_eq!(r.klasse, Klasse::Signatur);
        assert!(r.begruendung.contains("IHDR") || r.begruendung.contains("Gueltigkeitsprobe"));
    }

    #[test]
    fn klasse_2_signatur_ohne_gueltigkeit() {
        let mut b = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        b.extend_from_slice(&[0u8; 40]);
        let r = erkenne(&b, Some("png"));
        assert_eq!(r.klasse, Klasse::SignaturGueltig);
        assert!(r.begruendung.contains("IHDR"));
    }

    #[test]
    fn klasse_3_struktur_ohne_signatur() {
        assert_eq!(erkenne(br#"{"a": 1}"#, Some("json")).label, Some("json"));
        assert_eq!(erkenne(b"a,b\n1,2\n3,4\n", Some("csv")).label, Some("csv"));
        assert_eq!(erkenne(b"x = 1\ny = 2\n", Some("toml")).label, Some("toml"));
        assert_eq!(erkenne(b"<a><b>1</b></a>", Some("xml")).label, Some("xml"));
        assert_eq!(
            erkenne(b"{\"a\":1}\n{\"a\":2}\n", Some("jsonl")).label,
            Some("jsonl")
        );
        assert_eq!(
            erkenne(b"nur ein satz ohne struktur", Some("txt")).label,
            Some("klartext")
        );
    }

    #[test]
    fn klasse_4_widerspruch_kopf_gegen_struktur() {
        let mut b = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        b.extend_from_slice(b"{\"a\": 1}");
        let r = erkenne(&b, None);
        assert_eq!(r.klasse, Klasse::Widerspruch);
        assert!(r.begruendung.contains("passt nicht zum Inhalt"));
    }

    #[test]
    fn klasse_5_falsche_endung() {
        let r = erkenne(b"{\"a\": 1}", Some("png"));
        assert_eq!(r.klasse, Klasse::FalscheEndung);
        assert_eq!(r.label, Some("json"));
        let r2 = erkenne(&png_ok(), Some("zip"));
        assert_eq!(r2.klasse, Klasse::FalscheEndung);
    }

    #[test]
    fn unbestimmbar_ist_ein_zugelassener_wert() {
        assert_eq!(erkenne(&zufall(64), None).klasse, Klasse::Unbestimmbar);
        assert_eq!(erkenne(b"", None).klasse, Klasse::Unbestimmbar);
        assert_eq!(erkenne(b"ab", None).klasse, Klasse::Unbestimmbar);
    }

    #[test]
    fn weitere_signaturen_werden_erkannt() {
        assert_eq!(
            erkenne(b"%PDF-1.7\n%%EOF\n", Some("pdf")).label,
            Some("pdf")
        );
        assert_eq!(
            erkenne(&[0x1F, 0x8B, 8, 0, 0, 0, 0, 0], Some("gz")).label,
            Some("gzip")
        );
        assert_eq!(
            erkenne(
                &[0x7F, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                Some("elf")
            )
            .label,
            Some("elf")
        );
    }

    #[test]
    fn untere_grenze_der_liste_ist_geschlossen() {
        // Ein Format ausserhalb der Liste ist "nicht bestimmbar", nicht geraten.
        let bmp = [0x42u8, 0x4D, 0, 0, 0, 0, 0, 0];
        assert_eq!(erkenne(&bmp, Some("bmp")).klasse, Klasse::Unbestimmbar);
    }
}
