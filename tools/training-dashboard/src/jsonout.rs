//! JSON-Ausgabe von Hand.
//!
//! Gegenstueck zu `jsonread.rs`: hier wird geschrieben. Ein kleiner Builder
//! haelt die Erzeugung lesbar und stellt sicher, dass **jede** Zeichenkette
//! korrekt maskiert wird - ein unescapetes Anfuehrungszeichen im Pfad eines
//! Checkpoints wuerde sonst die ganze Antwort zerstoeren.

/// Baut JSON-Text. Alle Werte werden ueber die Methoden dieses Typs gesetzt,
/// damit die Maskierung nicht vergessen werden kann.
#[derive(Debug, Default)]
pub struct Out {
    buf: String,
    /// Liegt gerade ein Wert an, der ein Komma braucht?
    needs_comma: bool,
}

impl Out {
    pub fn new() -> Self {
        Self::default()
    }

    /// Beginnt ein Objekt.
    pub fn obj(&mut self) -> &mut Self {
        self.comma();
        self.buf.push('{');
        self.needs_comma = false;
        self
    }

    /// Beendet ein Objekt.
    pub fn end_obj(&mut self) -> &mut Self {
        self.buf.push('}');
        self.needs_comma = true;
        self
    }

    /// Beginnt ein Array.
    pub fn arr(&mut self) -> &mut Self {
        self.comma();
        self.buf.push('[');
        self.needs_comma = false;
        self
    }

    /// Beendet ein Array.
    pub fn end_arr(&mut self) -> &mut Self {
        self.buf.push(']');
        self.needs_comma = true;
        self
    }

    /// Schluessel eines Objekts.
    pub fn key(&mut self, k: &str) -> &mut Self {
        self.comma();
        escape_into(k, &mut self.buf);
        self.buf.push(':');
        self.needs_comma = false;
        self
    }

    /// Zeichenkette als Wert.
    pub fn str(&mut self, s: &str) -> &mut Self {
        self.comma();
        escape_into(s, &mut self.buf);
        self.needs_comma = true;
        self
    }

    /// Zahl als Wert. Nicht-endliche Zahlen werden zu `null` - JSON kennt
    /// weder `NaN` noch `Infinity`, und ein nacktes `NaN` wuerde im Browser
    /// den ganzen Aufruf scheitern lassen.
    pub fn num(&mut self, n: f64) -> &mut Self {
        self.comma();
        if n.is_finite() {
            // Kompakte, aber verlustfreie Darstellung.
            self.buf.push_str(&format!("{n}"));
        } else {
            self.buf.push_str("null");
        }
        self.needs_comma = true;
        self
    }

    /// Ganzzahl als Wert.
    pub fn int(&mut self, n: u64) -> &mut Self {
        self.comma();
        self.buf.push_str(&n.to_string());
        self.needs_comma = true;
        self
    }

    /// Optionaler Wert: `null`, wenn nicht vorhanden.
    pub fn opt_num(&mut self, n: Option<f64>) -> &mut Self {
        match n {
            Some(v) => self.num(v),
            None => self.null(),
        }
    }

    /// Optionaler Text: `null`, wenn nicht vorhanden.
    pub fn opt_str(&mut self, s: Option<&str>) -> &mut Self {
        match s {
            Some(v) => self.str(v),
            None => self.null(),
        }
    }

    pub fn bool(&mut self, b: bool) -> &mut Self {
        self.comma();
        self.buf.push_str(if b { "true" } else { "false" });
        self.needs_comma = true;
        self
    }

    pub fn null(&mut self) -> &mut Self {
        self.comma();
        self.buf.push_str("null");
        self.needs_comma = true;
        self
    }

    /// Fertiger Text.
    pub fn finish(self) -> String {
        self.buf
    }

    fn comma(&mut self) {
        if self.needs_comma {
            self.buf.push(',');
        }
        self.needs_comma = false;
    }
}

/// Maskiert eine Zeichenkette nach RFC 8259 und schreibt sie in `out`.
///
/// Alles unter 0x20 muss maskiert werden; `/` und Nicht-ASCII duerfen roh
/// bleiben (UTF-8 ist in JSON erlaubt). Steuerzeichen ohne Kurzform werden
/// als `\u00XX` geschrieben.
pub fn escape_into(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Maskiert eine Zeichenkette und gibt sie zurueck.
pub fn escape(s: &str) -> String {
    let mut s_out = String::with_capacity(s.len() + 2);
    escape_into(s, &mut s_out);
    s_out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_simple_object() {
        let mut o = Out::new();
        o.obj().key("a").int(1).key("b").str("x").end_obj();
        assert_eq!(o.finish(), r#"{"a":1,"b":"x"}"#);
    }

    #[test]
    fn builds_arrays_and_nested_objects() {
        let mut o = Out::new();
        o.arr()
            .obj()
            .key("n")
            .num(1.5)
            .end_obj()
            .obj()
            .key("n")
            .null()
            .end_obj()
            .end_arr();
        assert_eq!(o.finish(), r#"[{"n":1.5},{"n":null}]"#);
    }

    #[test]
    fn escapes_quotes_backslashes_and_newlines() {
        let mut o = Out::new();
        o.obj().key("p").str("C:\\pfad\\\"x\"\nzeile").end_obj();
        let text = o.finish();
        assert_eq!(text, r#"{"p":"C:\\pfad\\\"x\"\nzeile"}"#);
        // Gegenprobe: der eigene Leser muss das Ergebnis zuruecklesen.
        let back = crate::jsonread::parse_line(&text).expect("lesbar");
        assert_eq!(back.str_or("p", ""), "C:\\pfad\\\"x\"\nzeile");
    }

    #[test]
    fn escapes_control_characters() {
        let mut o = Out::new();
        o.obj().key("k").str("a\u{1}b").end_obj();
        assert_eq!(o.finish(), r#"{"k":"a\u0001b"}"#);
    }

    #[test]
    fn non_finite_numbers_become_null() {
        let mut o = Out::new();
        o.obj()
            .key("nan")
            .num(f64::NAN)
            .key("inf")
            .num(f64::INFINITY)
            .key("ok")
            .num(2.5)
            .end_obj();
        let text = o.finish();
        assert_eq!(text, r#"{"nan":null,"inf":null,"ok":2.5}"#);
        // Muss vom eigenen Leser lesbar sein.
        assert!(crate::jsonread::parse_line(&text).is_some());
    }

    #[test]
    fn optional_values_render_as_null() {
        let mut o = Out::new();
        o.obj()
            .key("a")
            .opt_num(None)
            .key("b")
            .opt_num(Some(1.0))
            .key("c")
            .opt_str(None)
            .key("d")
            .opt_str(Some("x"))
            .end_obj();
        assert_eq!(o.finish(), r#"{"a":null,"b":1,"c":null,"d":"x"}"#);
    }

    #[test]
    fn escaping_keeps_unicode_intact() {
        let mut o = Out::new();
        o.obj().key("u").str("Durchbruch: äöü").end_obj();
        let text = o.finish();
        assert!(text.contains("äöü"), "war: {text}");
        assert_eq!(
            crate::jsonread::parse_line(&text).unwrap().str_or("u", ""),
            "Durchbruch: äöü"
        );
    }
}
