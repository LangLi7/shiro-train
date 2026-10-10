//! Minimaler JSON-Leser fuer die Logzeilen.
//!
//! Warum selbst geschrieben und nicht `serde_json`: dieses Werkzeug soll
//! **keine** Abhaengigkeit haben. Es muss nur lesen und ein paar Zahlen aus
//! flachen Objekten holen - dafuer ist ein vollstaendiger JSON-Stack zu viel.
//!
//! Die Logzeilen erzeugt `src/logging.rs` mit `serde_json::to_string` auf einem
//! Enum mit `#[serde(tag = "event")]`. Das Ergebnis ist ein flaches Objekt;
//! `step` ist eine Ausnahme: dort liegt der Schritt als **verschachteltes**
//! Objekt unter dem Schluessel `step`.

use std::collections::BTreeMap;

/// Ein JSON-Wert. Objekte sind nach Schluessel sortiert (`BTreeMap`) - die
/// Reihenfolge spielt fuer das Lesen keine Rolle, und es gibt keine
/// Hash-Iterationsreihenfolge, die Tests wackelig macht.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    Obj(BTreeMap<String, Value>),
}

impl Value {
    /// Feld eines Objekts. `None`, wenn es kein Objekt ist oder das Feld fehlt.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(m) => m.get(key),
            _ => None,
        }
    }

    /// Wie `get`, aber nur wenn der Wert nicht `null` ist.
    ///
    /// Wichtig, weil `logging.rs` optionale Felder mit `skip_serializing_if`
    /// weglaesst - sie fehlen also ganz statt `null` zu sein. Beide Faelle
    /// muessen gleich behandelt werden.
    pub fn field(&self, key: &str) -> Option<&Value> {
        match self.get(key) {
            None | Some(Value::Null) => None,
            Some(v) => Some(v),
        }
    }

    /// Zahl als `f64`. Nimmt auch ganze Zahlen an.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Num(n) if n.is_finite() => Some(*n),
            _ => None,
        }
    }

    /// Zahl als `u64` - nur wenn sie nicht negativ und ganzzahlig ist.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Num(n) if *n >= 0.0 && n.fract() == 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Zahl aus einem Feld; fehlt sie, gilt `default`.
    pub fn num_or(&self, key: &str, default: f64) -> f64 {
        self.field(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    /// Ganzzahl aus einem Feld; fehlt sie, gilt `default`.
    pub fn u64_or(&self, key: &str, default: u64) -> u64 {
        self.field(key).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    /// Text aus einem Feld; fehlt er, gilt `default`.
    ///
    /// Die Rueckgabe leiht entweder den Feldtext **oder** den Vorgabewert -
    /// deshalb haengt sie an `&self` und nicht nur an `default`.
    pub fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.field(key).and_then(|v| v.as_str()).unwrap_or(default)
    }

    /// Text aus einem Feld als `String`, falls vorhanden und nicht leer.
    pub fn string(&self, key: &str) -> Option<String> {
        self.field(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    /// Ist der Wert ein Objekt?
    pub fn is_obj(&self) -> bool {
        matches!(self, Value::Obj(_))
    }
}

/// Liest **ein** JSON-Objekt aus einer Zeile.
///
/// Gibt `None` zurueck, wenn die Zeile leer, kein JSON oder kein Objekt ist.
/// Fehler werden bewusst verschluckt: eine halb geschriebene letzte Zeile darf
/// das Dashboard nicht zum Absturz bringen.
pub fn parse_line(line: &str) -> Option<Value> {
    let mut p = Parser {
        bytes: line.as_bytes(),
        pos: 0,
    };
    let v = p.value()?;
    p.ws();
    // Nach dem Wert darf nur noch Whitespace stehen.
    if p.pos != p.bytes.len() {
        return None;
    }
    match v {
        Value::Obj(_) => Some(v),
        _ => None,
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn ws(&mut self) {
        while matches!(
            self.peek(),
            Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n')
        ) {
            self.pos += 1;
        }
    }

    fn lit(&mut self, word: &str) -> Option<()> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Some(())
        } else {
            None
        }
    }

    fn value(&mut self) -> Option<Value> {
        self.ws();
        match self.peek()? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => self.string().map(Value::Str),
            b't' => self.lit("true").map(|_| Value::Bool(true)),
            b'f' => self.lit("false").map(|_| Value::Bool(false)),
            b'n' => self.lit("null").map(|_| Value::Null),
            _ => self.number(),
        }
    }

    fn object(&mut self) -> Option<Value> {
        self.bump()?; // '{'
        let mut map = BTreeMap::new();
        self.ws();
        if self.peek()? == b'}' {
            self.bump();
            return Some(Value::Obj(map));
        }
        loop {
            self.ws();
            let key = self.string()?;
            self.ws();
            if self.bump()? != b':' {
                return None;
            }
            let val = self.value()?;
            map.insert(key, val);
            self.ws();
            match self.bump()? {
                b',' => continue,
                b'}' => break,
                _ => return None,
            }
        }
        Some(Value::Obj(map))
    }

    fn array(&mut self) -> Option<Value> {
        self.bump()?; // '['
        let mut out = Vec::new();
        self.ws();
        if self.peek()? == b']' {
            self.bump();
            return Some(Value::Arr(out));
        }
        loop {
            out.push(self.value()?);
            self.ws();
            match self.bump()? {
                b',' => continue,
                b']' => break,
                _ => return None,
            }
        }
        Some(Value::Arr(out))
    }

    fn string(&mut self) -> Option<String> {
        if self.bump()? != b'"' {
            return None;
        }
        // Bytes sammeln und **am Ende** als UTF-8 deuten.
        //
        // Warum nicht Zeichen fuer Zeichen: ein Umlaut ist in UTF-8 zwei
        // Bytes. Wer jedes Byte als `char` nimmt, macht aus "ä" die zwei
        // Zeichen "Ã¤" - genau dieser Fehler ist in einem Test aufgefallen.
        // Maskierte Zeichen (\\uXXXX, \\n, ...) werden weiterhin direkt als
        // Zeichen angehaengt; sie sind in UTF-8 ein- oder mehrbytig, aber
        // nie Teil einer mehrbytigen Folge.
        let mut bytes: Vec<u8> = Vec::with_capacity(32);
        loop {
            let b = self.bump()?;
            match b {
                b'"' => break,
                b'\\' => {
                    let esc = self.bump()?;
                    match esc {
                        b'"' => bytes.push(b'"'),
                        b'\\' => bytes.push(b'\\'),
                        b'/' => bytes.push(b'/'),
                        b'b' => bytes.push(0x08),
                        b'f' => bytes.push(0x0c),
                        b'n' => bytes.push(b'\n'),
                        b'r' => bytes.push(b'\r'),
                        b't' => bytes.push(b'\t'),
                        b'u' => {
                            let hex = self.take(4)?;
                            let s = std::str::from_utf8(hex).ok()?;
                            let cp = u32::from_str_radix(s, 16).ok()?;
                            // Surrogatpaare werden nicht zusammengesetzt: die
                            // Logdateien des Projekts sind ASCII-transliteriert
                            // (docs/programmiersprachen/sprachstufe.md). Ein
                            // Ersatzzeichen ist besser als ein Fehler.
                            let c = char::from_u32(cp).unwrap_or('\u{fffd}');
                            let mut buf = [0u8; 4];
                            bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => return None,
                    }
                }
                b if b < 0x20 => bytes.push(b' '),
                b => bytes.push(b),
            }
        }
        // Tolerant: die Quellbytes stammen aus einer Datei, nicht aus dem Netz.
        // Bei ungueltigem UTF-8 wird ersetzt statt die Zeile zu verwerfen.
        Some(match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
        })
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.bytes.get(self.pos..self.pos + n)?;
        self.pos += n;
        Some(s)
    }

    fn number(&mut self) -> Option<Value> {
        let start = self.pos;
        while matches!(
            self.peek(),
            Some(b'-') | Some(b'+') | Some(b'.') | Some(b'e') | Some(b'E') | Some(b'0'..=b'9')
        ) {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos]).ok()?;
        // Waechter gegen nicht-endliche Zahlen: `"1e999"` ist syntaktisch ein
        // JSON-Zahl-Literal, ueberlaeuft aber `f64` und ergaebe `inf`. Eine
        // solche Zeile wird abgelehnt statt als Zahl weitergereicht - sonst
        // kaeme der Wert als `null` im Browser an und wuerde die Verlustkurve
        // still verfaelschen. `NaN` ist in JSON gar kein gueltiges Literal und
        // scheitert schon an `parse`.
        match s.parse::<f64>() {
            Ok(n) if n.is_finite() => Some(Value::Num(n)),
            _ => None,
        }
    }
}

/// UTF-8 tolerant lesen: ungueltige Folgen werden ersetzt, nicht abgelehnt.
///
/// Eine Logdatei enthaelt ausschliesslich ASCII. Sollte dennoch ein kaputtes
/// Byte auftauchen, ist eine ersetzte Zeile besser als ein leerer Bildschirm.
pub fn to_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_step_line() {
        let line = r#"{"event":"step","step":200,"loss":1.0836744,"perplexity":2.9555194,"lr":1.8507244e-8,"bytes":5904,"tokens_per_second":1750.75,"ms":3372.26}"#;
        let v = parse_line(line).expect("Zeile muss lesbar sein");
        assert_eq!(v.str_or("event", ""), "step");
        assert_eq!(v.u64_or("step", 0), 200);
        assert!((v.num_or("loss", 0.0) - 1.0836744).abs() < 1e-9);
        // Exponentenschreibweise
        assert!((v.num_or("lr", 0.0) - 1.8507244e-8).abs() < 1e-15);
        assert_eq!(v.u64_or("bytes", 0), 5904);
    }

    #[test]
    fn parses_nested_object_used_by_checkpoint_field() {
        let line =
            r#"{"event":"step","step":1,"loss":2.0,"checkpoint":"checkpoints/x.safetensors"}"#;
        let v = parse_line(line).unwrap();
        assert_eq!(
            v.string("checkpoint").as_deref(),
            Some("checkpoints/x.safetensors")
        );
    }

    #[test]
    fn missing_optional_field_is_none_not_error() {
        // `logging.rs` laesst `checkpoint` ganz weg, wenn None.
        let line = r#"{"event":"step","step":1,"loss":2.0}"#;
        let v = parse_line(line).unwrap();
        assert_eq!(v.field("checkpoint"), None);
        assert_eq!(v.string("checkpoint"), None);
        assert_eq!(v.num_or("loss", -1.0), 2.0);
    }

    #[test]
    fn tolerates_garbage_and_partial_lines() {
        assert!(parse_line("").is_none());
        assert!(parse_line("   ").is_none());
        assert!(parse_line("kein json").is_none());
        assert!(parse_line(r#"{"event":"step""#).is_none(), "abgeschnitten");
        assert!(
            parse_line(r#"{"event":"step"} extra"#).is_none(),
            "Muell danach"
        );
        assert!(parse_line("[1,2,3]").is_none(), "kein Objekt");
        assert!(parse_line("123").is_none(), "kein Objekt");
    }

    #[test]
    fn handles_escapes_and_unicode_escape() {
        let v = parse_line(r#"{"a":"x\"y","b":"\u0041","c":"tab\tok"}"#).unwrap();
        assert_eq!(v.str_or("a", ""), "x\"y");
        assert_eq!(v.str_or("b", ""), "A");
        assert_eq!(v.str_or("c", ""), "tab\tok");
    }

    #[test]
    fn handles_nested_arrays_and_negative_numbers() {
        let v = parse_line(r#"{"a":[1,-2.5,{"b":true}],"n":-7}"#).unwrap();
        assert_eq!(v.num_or("n", 0.0), -7.0);
        match v.get("a") {
            Some(Value::Arr(items)) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[1].as_f64(), Some(-2.5));
                assert!(items[2].is_obj());
            }
            other => panic!("Array erwartet, war {other:?}"),
        }
    }

    #[test]
    fn null_field_counts_as_missing() {
        let v = parse_line(r#"{"checkpoint":null,"step":5}"#).unwrap();
        assert_eq!(v.field("checkpoint"), None);
        assert_eq!(v.u64_or("step", 0), 5);
    }

    #[test]
    fn as_u64_rejects_fractional_and_negative() {
        let v = parse_line(r#"{"a":1.5,"b":-3,"c":7}"#).unwrap();
        assert_eq!(v.get("a").unwrap().as_u64(), None);
        assert_eq!(v.get("b").unwrap().as_u64(), None);
        assert_eq!(v.get("c").unwrap().as_u64(), Some(7));
    }

    /// Eine Zahl, die `f64` ueberlaeuft, ist keine Zahl.
    ///
    /// `1e999` ist syntaktisch ein gueltiges JSON-Zahl-Literal, ergibt aber
    /// `inf`. Wuerde sie durchgereicht, stuende in der Antwort `null` (JSON
    /// kennt kein `Infinity`) und die Verlustkurve bekaeme einen stillen
    /// falschen Wert. Deshalb lehnt der Leser sie ab.
    #[test]
    fn rejects_numbers_that_overflow_f64() {
        assert!(
            parse_line(r#"{"loss":1e999}"#).is_none(),
            "1e999 ist nicht endlich und muss abgelehnt werden"
        );
        assert!(parse_line(r#"{"loss":-1e999}"#).is_none());
        assert!(parse_line("{\"loss\":NaN}").is_none(), "kein JSON-Literal");
        assert!(
            parse_line("{\"loss\":Infinity}").is_none(),
            "kein JSON-Literal"
        );
        // Gegenprobe: grosse, aber endliche Zahlen bleiben lesbar.
        let v = parse_line(r#"{"a":1e308,"b":-1.5e-8}"#).unwrap();
        assert_eq!(v.num_or("a", 0.0), 1e308);
        assert!((v.num_or("b", 0.0) + 1.5e-8).abs() < 1e-20);
    }

    #[test]
    fn reads_multibyte_utf8_as_one_character() {
        // Regressionstest: ein frueherer Fassung las Byte fuer Byte und machte
        // aus "ä" die zwei Zeichen "Ã¤". Ein Umlaut ist in UTF-8 ZWEI Bytes.
        let v = parse_line(r#"{"text":"Durchbruch äöü 日本語"}"#).unwrap();
        assert_eq!(v.str_or("text", ""), "Durchbruch äöü 日本語");
        // 18 Zeichen: "Durchbruch"(10) + Leerzeichen + "äöü"(3) + Leerzeichen
        // + "日本語"(3). Waeren die Umlaute als Einzelbytes gelesen, waeren es
        // 21 - genau der Fehler, den dieser Test festhaelt.
        assert_eq!(v.str_or("text", "").chars().count(), 18);
    }

    #[test]
    fn lossy_text_keeps_going_on_bad_bytes() {
        let bytes = b"{\"a\":\"ok\"}\n\xff\xfe";
        let text = to_text(bytes);
        assert!(text.contains("ok"));
        assert!(text.contains('\u{fffd}') || text.len() >= 13);
    }
}
