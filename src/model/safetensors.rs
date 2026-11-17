//! `safetensors` - das Auslieferungsformat des Modells.
//!
//! `MODEL_FORMAT.md` legt fest: Der fertige Modellordner enthaelt
//! `model.safetensors` als Inferenzartefakt, daneben `config.json`,
//! `special_bytes.json`, `run.json` und `SHA256SUMS`. Das eigene `.bckpt`
//! (`src/checkpoint.rs`) bleibt das interne Resume-Format mit Magic, Version und
//! FNV-1a-Digest; ausgeliefert wird es nie.
//!
//! Warum safetensors und nicht `.bckpt`:
//!
//! - Candle und Burn - die beiden zur Wahl stehenden Backends - lesen und schreiben
//!   das Format direkt. Genau fuer diesen Wechsel (Mac Studio nach CUDA) wird es
//!   gebraucht (Backend-Entscheidung D-010).
//! - Beim Laden wird kein Code ausgefuehrt (im Gegensatz zu Pickle), der Header ist
//!   JSON und damit lesbar, die Tensoren liegen Little-Endian und damit
//!   plattformneutral.
//! - `mmap`-faehig: Laden ohne Kopie, ohne die Gewichte vorher zu materialisieren.
//!
//! **Diese Datei ist kein Tokenizer-Ersatz.** Shiro ist ein Byte-Modell: das
//! Vokabular sind `vocab_size = 256` Byte-Werte plus die Strukturmarken aus
//! `src/data/bytes.rs`. Eine Tokenizer-Datei gibt es nicht und wird es nicht geben;
//! die Sonderbyte-Tabelle wandert als `special_bytes.json` in den Modellordner.
//! `model.safetensors` traegt nur Gewichte und ist ohne diese Begleitdateien nicht
//! vollstaendig - die Liste der Tensornamen und Shapes steht in `MODEL_FORMAT.md`
//! Abschnitt 4.2, die Architekturmetadaten gehoeren nach `config.json`.
//!
//! Aufbau des Formats (offizielles safetensors):
//!
//! ```text
//! Offset  Groesse  Feld
//!      0        8  Laenge des JSON-Headers (LE u64), zaehlt nur die JSON-Bytes
//!      8        N  JSON-Header, beginnt mit `{`
//!  8 + N        -  Rohdaten der Tensoren, in der Reihenfolge der Tensoreintraege
//! ```
//!
//! Der JSON-Header hat die Form
//! `{"__metadata__":{"schluessel":"wert"},"tensor.name":{"dtype":"F32","shape":[2,3],"data_offsets":[0,24]}}`.
//! `__metadata__` ist optional und steht, wenn vorhanden, zuerst. Jeder weitere
//! Eintrag ist ein Tensor mit genau `dtype`, `shape` und `data_offsets` - **keine
//! Zusatzfelder**, die fremde Reader ablehnen oder anders deuten koennten. Die
//! Laenge des Datenblocks ergibt sich aus dem groessten `data_offsets`-Ende.
//! **`data_offsets` sind relativ zum Beginn des Datenblocks, nicht zum Dateibeginn** -
//! das ist die Eigenschaft, an der sich fremde Leser und Schreiber typischerweise
//! trennen. Zwischen den Tensoren wird nicht ausgerichtet und nicht gepolstert:
//! die Reihenfolge der Nutzdaten ist genau die Reihenfolge der Tensoreintraege.
//!
//! Geschrieben wird - wie ein Checkpoint - immer atomar: erst `<name>.tmp`, dann
//! `rename`. Ein Absturz mitten im Schreiben darf nie eine halbfertige Datei
//! hinterlassen.
//!
//! Gelesen und geschrieben wird hier ausschliesslich `F32` (4 Byte je Parameter,
//! der Master-Stand). Die Umrechnung nach `f16` fuer die Auslieferung ist Aufgabe
//! des Exports, nicht dieser Datei. Ein anderer `dtype` im Header ist deshalb kein
//! stiller Fehler, sondern ein klarer Abbruch.

use std::io::{BufRead, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{Context, Result, bail};

/// Ein gelesener Tensor: Name, Shape, Werte.
pub type TensorData = (String, Vec<usize>, Vec<f32>);

/// Sicht auf einen Tensor zum Schreiben. `data` bleibt ausgeliehen, kopiert wird nichts.
#[derive(Debug, Clone)]
pub struct TensorView<'a> {
    pub name: String,
    pub shape: Vec<usize>,
    pub data: &'a [f32],
}

impl<'a> TensorView<'a> {
    /// Prueft `shape` gegen `data` und lehnt Widersprueche ab.
    ///
    /// Panikfrei: das Produkt der Achsen muss die Zahl der Werte treffen, und die
    /// Multiplikation darf nicht ueberlaufen. Ein falscher Shape ist ein Fehler,
    /// keine Panik - ein Trainingslauf, der hier abstuerzt, hat Stunden gekostet.
    pub fn new(name: impl Into<String>, shape: Vec<usize>, data: &'a [f32]) -> Result<Self> {
        let name = name.into();
        match checked_len(&shape) {
            Some(n) if n == data.len() => Ok(Self { name, shape, data }),
            Some(n) => bail!(
                "Tensor {name}: Shape {shape:?} ergibt {n} Werte, data hat {}",
                data.len()
            ),
            None => bail!("Tensor {name}: Shape {shape:?} laeuft in usize ueber"),
        }
    }
}

/// Ein Tensoreintrag, wie er im Header steht.
#[derive(Debug, Clone)]
struct Entry {
    name: String,
    shape: Vec<usize>,
    /// `[start, end]` relativ zum Beginn des Datenblocks.
    needed: [u64; 2],
}

/// Nutzdaten eines Tensors, in der Reihenfolge der `tensors`-Liste.
struct WritePlan {
    data: Vec<f32>,
}

/// Alles, was zum Schreiben gebraucht wird - vor dem ersten Schreibvorgang fertig.
struct Prepared {
    json: String,
    plans: Vec<WritePlan>,
}

/// Der geparste Header: alle Tensoreintraege, in der Reihenfolge ihrer Offsets.
struct Header {
    entries: Vec<Entry>,
}

/// Schluessel des reservierten Metadatenfelds im Header.
const METADATA_KEY: &str = "__metadata__";

/// Schreibt Tensoren im safetensors-Format - atomar ueber `<name>.tmp` plus `rename`.
///
/// `metadata` landet als optionales `__metadata__`-Objekt (String zu String) zuerst
/// im Header. Ohne Eintraege entfaellt das Feld ganz.
pub fn write_f32(
    path: impl AsRef<Path>,
    tensors: &[TensorView<'_>],
    metadata: &[(&str, &str)],
) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Modellordner nicht anlegbar: {}", parent.display()))?;
    }

    let prepared = prepare(tensors, metadata)?;
    let tmp = tmp_path(path);

    {
        let f = std::fs::File::create(&tmp)
            .with_context(|| format!("temporaere Datei nicht schreibbar: {}", tmp.display()))?;
        let mut w = BufWriter::new(f);
        w.write_all(&(prepared.json.len() as u64).to_le_bytes())?;
        w.write_all(prepared.json.as_bytes())
            .with_context(|| format!("JSON-Header nicht schreibbar: {}", tmp.display()))?;
        for plan in &prepared.plans {
            let mut buf = Vec::with_capacity(plan.data.len().saturating_mul(4));
            for v in &plan.data {
                buf.extend_from_slice(&v.to_le_bytes());
            }
            w.write_all(&buf)
                .with_context(|| format!("Tensordaten nicht schreibbar: {}", tmp.display()))?;
        }
        w.flush()?;
        w.get_ref().sync_all()?;
    }

    std::fs::rename(&tmp, path).with_context(|| {
        format!(
            "Modell konnte nicht umbenannt werden: {} -> {}",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

/// Liest saemtliche Tensoren einer Datei. Nur `F32`; jede andere `dtype` bricht ab.
pub fn read_f32(path: impl AsRef<Path>) -> Result<Vec<TensorData>> {
    let path = path.as_ref();
    let raw = std::fs::read(path)
        .with_context(|| format!("safetensors nicht lesbar: {}", path.display()))?;
    let (header, payload) = read_header(&raw, path)?;

    let mut out = Vec::with_capacity(header.entries.len());
    for entry in &header.entries {
        let len = usize::try_from(entry.needed[1] - entry.needed[0]).map_err(|_| {
            crate::error::anyhow!("Tensor {} ist zu gross fuer diesen Rechner", entry.name)
        })?;
        let start = usize::try_from(entry.needed[0]).map_err(|_| {
            crate::error::anyhow!("Tensor {} hat einen Offset jenseits von usize", entry.name)
        })?;
        let ende = start.checked_add(len).ok_or_else(|| {
            crate::error::anyhow!("Tensor {} laeuft ueber das Dateiende hinaus", entry.name)
        })?;
        let bytes = payload
            .get(start..ende)
            .with_context(|| format!("Tensor {} endet vorzeitig bei {ende}", entry.name))?;
        let data = bytes
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        out.push((entry.name.clone(), entry.shape.clone(), data));
    }
    Ok(out)
}

/// Nur die im Header deklarierte Byte-Laenge des Datenblocks.
///
/// Sie ergibt sich aus den `data_offsets` der Tensoreintraege: das groesste Ende
/// ist die Laenge des Datenblocks. Fuer Tests und Doku; gelesen wird kein Wert.
pub fn declared_len(path: impl AsRef<Path>) -> Result<usize> {
    let path = path.as_ref();
    let raw = std::fs::read(path)
        .with_context(|| format!("safetensors nicht lesbar: {}", path.display()))?;
    let (header, _) = read_header(&raw, path)?;
    let len = header
        .entries
        .iter()
        .map(|e| e.needed[1])
        .max()
        .unwrap_or(0);
    usize::try_from(len)
        .map_err(|_| crate::error::anyhow!("deklarierte Laenge passt nicht in usize"))
}

/// Baut Header-JSON und Schreibplaene. Alle Fehler entstehen hier, vor jedem Schreibvorgang.
fn prepare(tensors: &[TensorView<'_>], metadata: &[(&str, &str)]) -> Result<Prepared> {
    let mut names: Vec<&str> = Vec::with_capacity(tensors.len());
    for t in tensors {
        let expected = checked_len(&t.shape).ok_or_else(|| {
            crate::error::anyhow!(
                "Tensor {}: Shape {:?} laeuft in usize ueber",
                t.name,
                t.shape
            )
        })?;
        if expected != t.data.len() {
            bail!(
                "Tensor {}: Shape {:?} ergibt {expected} Werte, data hat {}",
                t.name,
                t.shape,
                t.data.len()
            );
        }
        if t.name.is_empty() || t.name == METADATA_KEY {
            bail!(
                "Tensor {}: unzulaessiger Name (leer oder reserviert: {METADATA_KEY})",
                t.name
            );
        }
        if names.contains(&t.name.as_str()) {
            bail!("Tensor {} kommt doppelt vor", t.name);
        }
        names.push(&t.name);
    }

    let mut json = String::from("{");
    let mut erster_eintrag = true;
    if !metadata.is_empty() {
        json.push_str("\"__metadata__\":{");
        for (i, (k, v)) in metadata.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            put_string(&mut json, k);
            json.push(':');
            put_string(&mut json, v);
        }
        json.push('}');
        erster_eintrag = false;
    }

    // Nur Tensoreintraege mit dtype/shape/data_offsets - keine Zusatzfelder, die
    // fremde Reader ablehnen koennten. Die Laenge des Datenblocks ergibt sich aus
    // dem letzten `data_offsets`-Ende.
    let mut plans = Vec::with_capacity(tensors.len());
    let mut offset: u64 = 0;
    for t in tensors {
        if erster_eintrag {
            erster_eintrag = false;
        } else {
            json.push(',');
        }
        put_string(&mut json, &t.name);
        json.push_str(":{\"dtype\":\"F32\",\"shape\":[");
        for (i, dim) in t.shape.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&dim.to_string());
        }
        // data_offsets sind relativ zum Beginn des Datenblocks, nicht des Files.
        let start = offset;
        let end = offset + (t.data.len() as u64) * 4;
        json.push_str("],\"data_offsets\":[");
        json.push_str(&start.to_string());
        json.push(',');
        json.push_str(&end.to_string());
        json.push_str("]}");
        offset = end;
        plans.push(WritePlan {
            data: t.data.to_vec(),
        });
    }
    json.push('}');

    if !json.starts_with('{') {
        bail!("interner Fehler: Header-JSON beginnt nicht mit '{{'");
    }
    // `offset` ist nach der Schleife die Gesamtlaenge des Datenblocks, also genau
    // das letzte `data_offsets`-Ende - die Zahl, aus der `declared_len` liest.
    let erwartet: u64 = plans.iter().map(|p| (p.data.len() as u64) * 4).sum();
    if offset != erwartet {
        bail!("interner Fehler: Datenblock {erwartet} Bytes, Offsets bis {offset}");
    }
    Ok(Prepared { json, plans })
}

/// Liest Laenge, JSON-Header und Tensoreintraege und prueft alle Laengenangaben.
///
/// Gibt den Header und die Nutzdaten (ab dem ersten Datenbyte) zurueck.
fn read_header<'a>(raw: &'a [u8], path: &Path) -> Result<(Header, &'a [u8])> {
    if raw.len() < 8 {
        bail!(
            "safetensors {} zu kurz fuer die Kopflaenge ({} Bytes)",
            path.display(),
            raw.len()
        );
    }
    let header_len = u64::from_le_bytes(raw[0..8].try_into().unwrap_or([0u8; 8]));
    let header_size = usize::try_from(header_len).map_err(|_| {
        crate::error::anyhow!(
            "safetensors {}: Kopflaenge {header_len} passt nicht in usize",
            path.display()
        )
    })?;
    if header_size == 0 {
        bail!(
            "safetensors {}: Kopflaenge 0 ist kein Header",
            path.display()
        );
    }
    // Vor jeder Auswertung gegen die Dateigroesse pruefen: eine verfaelschte
    // Kopflaenge darf weder ueberlaufen (header_len = u64::MAX) noch auf
    // Speicher zeigen, den die Datei nicht hat.
    let file_len = raw.len() as u64;
    if header_len.checked_add(8).is_none_or(|ende| ende > file_len) {
        bail!(
            "safetensors {}: Kopflaenge {header_len} passt nicht in die Datei ({file_len} Bytes)",
            path.display()
        );
    }
    let (json, payload) = raw.split_at(8 + header_size);
    let json = &json[8..];
    if json.first() != Some(&b'{') {
        bail!(
            "safetensors {}: Header beginnt nicht mit '{{'",
            path.display()
        );
    }
    let text = std::str::from_utf8(json).with_context(|| {
        format!(
            "safetensors {}: Header ist kein gueltiges UTF-8",
            path.display()
        )
    })?;

    let value: JsonValue = parse_json(text, path)?;
    let JsonValue::Object(members) = value else {
        bail!(
            "safetensors {}: Header ist kein JSON-Objekt",
            path.display()
        );
    };

    // Nur Tensoreintraege mit dtype/shape/data_offsets. `__metadata__` ist ein
    // String-zu-String-Objekt und wird beim Lesen der Tensoren nicht gebraucht.
    let mut entries: Vec<Entry> = Vec::new();
    for (key, value) in &members {
        if key == METADATA_KEY {
            continue;
        }
        if key.starts_with("__") && key.ends_with("__") {
            bail!(
                "safetensors {}: unbekannter reservierter Schluessel {key}",
                path.display()
            );
        }
        let obj = value.as_object().ok_or_else(|| {
            crate::error::anyhow!(
                "safetensors {}: Eintrag {key} ist kein JSON-Objekt",
                path.display()
            )
        })?;
        let mut dtype = None;
        let mut shape: Option<Vec<usize>> = None;
        let mut needed: Option<[u64; 2]> = None;
        for (field, field_value) in obj {
            match field.as_str() {
                "dtype" => {
                    dtype = Some(
                        field_value
                            .as_str()
                            .ok_or_else(|| {
                                crate::error::anyhow!(
                                    "safetensors {}: dtype von {key} ist keine Zeichenkette",
                                    path.display()
                                )
                            })?
                            .to_string(),
                    );
                }
                "shape" => shape = Some(shape_of(field_value, key, path)?),
                "data_offsets" => needed = Some(offsets_of(field_value, key, path)?),
                _ => {}
            }
        }
        let dtype = dtype.ok_or_else(|| {
            crate::error::anyhow!("safetensors {}: dtype fehlt bei {key}", path.display())
        })?;
        if dtype != "F32" {
            bail!(
                "safetensors {}: Tensor {key} hat dtype {dtype}, diese Lesefunktion kann nur F32",
                path.display()
            );
        }
        let shape = shape.ok_or_else(|| {
            crate::error::anyhow!("safetensors {}: shape fehlt bei {key}", path.display())
        })?;
        let needed = needed.ok_or_else(|| {
            crate::error::anyhow!(
                "safetensors {}: data_offsets fehlt bei {key}",
                path.display()
            )
        })?;
        let expected = checked_len(&shape)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| {
                crate::error::anyhow!(
                    "safetensors {}: Shape {shape:?} von {key} laeuft in usize ueber",
                    path.display()
                )
            })?;
        if needed[1] < needed[0] {
            bail!(
                "safetensors {}: Tensor {key} hat data_offsets [{}, {}] in falscher Reihenfolge",
                path.display(),
                needed[0],
                needed[1]
            );
        }
        let span = needed[1] - needed[0];
        if span != expected as u64 {
            bail!(
                "safetensors {}: Tensor {key} hat Span {span} Bytes, Shape {shape:?} in F32 braucht {expected}",
                path.display()
            );
        }
        entries.push(Entry {
            name: key.clone(),
            shape,
            needed,
        });
    }

    // Nutzdaten liegen in der Reihenfolge der Offsets.
    entries.sort_by_key(|e| e.needed[0]);

    Ok((Header { entries }, payload))
}

/// Achsen einer `shape` als `usize`; negative Werte sind kein gueltiger Shape.
fn shape_of(value: &JsonValue, key: &str, path: &Path) -> Result<Vec<usize>> {
    let items = value.as_array().ok_or_else(|| {
        crate::error::anyhow!(
            "safetensors {}: shape von {key} ist kein JSON-Array",
            path.display()
        )
    })?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match item.as_u64().and_then(|n| usize::try_from(n).ok()) {
            Some(n) => out.push(n),
            None => bail!(
                "safetensors {}: shape von {key} enthaelt keinen nicht-negativen Integer",
                path.display()
            ),
        }
    }
    Ok(out)
}

/// `data_offsets` als zwei Byte-Zahlen.
fn offsets_of(value: &JsonValue, key: &str, path: &Path) -> Result<[u64; 2]> {
    let items = value.as_array().ok_or_else(|| {
        crate::error::anyhow!(
            "safetensors {}: data_offsets von {key} ist kein JSON-Array",
            path.display()
        )
    })?;
    if items.len() != 2 {
        bail!(
            "safetensors {}: data_offsets von {key} hat {} statt 2 Werte",
            path.display(),
            items.len()
        );
    }
    let start = items[0].as_u64().ok_or_else(|| {
        crate::error::anyhow!(
            "safetensors {}: data_offsets von {key} sind keine Integer",
            path.display()
        )
    })?;
    let end = items[1].as_u64().ok_or_else(|| {
        crate::error::anyhow!(
            "safetensors {}: data_offsets von {key} sind keine Integer",
            path.display()
        )
    })?;
    Ok([start, end])
}

/// Produkt der Achsen, `None` bei Ueberlauf. Ein leeres Shape ist ein Skalar (1 Wert).
fn checked_len(shape: &[usize]) -> Option<usize> {
    let mut n: usize = 1;
    for dim in shape {
        n = n.checked_mul(*dim)?;
    }
    Some(n)
}

/// Haengt eine JSON-Zeichenkette an, inklusive Anfuehrungszeichen und Escapes.
fn put_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Endung mit `.tmp` davor - dieselbe Regel wie in `src/checkpoint.rs`.
fn tmp_path(path: &Path) -> PathBuf {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => path.with_extension(format!("{ext}.tmp")),
        None => path.with_extension("tmp"),
    }
}

/// Der geparste JSON-Wert, so weit er fuer einen Header gebraucht wird.
#[derive(Debug, Clone)]
enum JsonValue {
    /// `true`, `false` und `null` dienen nur der Vollstaendigkeit des Parsers;
    /// ein Header besteht aus `__metadata__` und Tensoreintraegen, deren Werte
    /// selbst nie gebraucht werden.
    #[allow(dead_code)]
    Bool(bool),
    Number(u64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    fn as_object(&self) -> Option<&Vec<(String, JsonValue)>> {
        match self {
            JsonValue::Object(m) => Some(m),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }
}

/// Minimaler JSON-Parser - der Header ist klein, deshalb ohne eigene Abhaengigkeit.
fn parse_json(text: &str, path: &Path) -> Result<JsonValue> {
    let mut cursor = std::io::Cursor::new(text.as_bytes());
    let value = parse_value(&mut cursor, path)?;
    skip_ws(&mut cursor);
    if peek_byte(&mut cursor).is_some() {
        bail!(
            "safetensors {}: nach dem Header-JSON steht noch Text",
            path.display()
        );
    }
    Ok(value)
}

fn parse_value(reader: &mut std::io::Cursor<&[u8]>, path: &Path) -> Result<JsonValue> {
    match peek_byte(reader) {
        None => bail!(
            "safetensors {}: Header-JSON endet mitten im Wert",
            path.display()
        ),
        Some(b't') => {
            expect_keyword(reader, b"true", path)?;
            Ok(JsonValue::Bool(true))
        }
        Some(b'f') => {
            expect_keyword(reader, b"false", path)?;
            Ok(JsonValue::Bool(false))
        }
        Some(b'n') => {
            expect_keyword(reader, b"null", path)?;
            Ok(JsonValue::Bool(false))
        }
        Some(b'"') => Ok(JsonValue::String(parse_string(reader, path)?)),
        Some(b'[') => parse_array(reader, path),
        Some(b'{') => parse_object(reader, path),
        Some(b'-') => bail!(
            "safetensors {}: negative Zahl in einem Header, der nur nicht-negative Zahlen kennt",
            path.display()
        ),
        Some(c) if c.is_ascii_digit() => Ok(JsonValue::Number(parse_number(reader))),
        Some(c) => bail!(
            "safetensors {}: unerwartetes Zeichen '{}' im Header-JSON",
            path.display(),
            c as char
        ),
    }
}

fn parse_array(reader: &mut std::io::Cursor<&[u8]>, path: &Path) -> Result<JsonValue> {
    let mut items = Vec::new();
    read_byte(reader);
    loop {
        skip_ws(reader);
        if peek_byte(reader) == Some(b']') {
            read_byte(reader);
            return Ok(JsonValue::Array(items));
        }
        items.push(parse_value(reader, path)?);
        skip_ws(reader);
        match peek_byte(reader) {
            Some(b',') => {
                read_byte(reader);
            }
            Some(b']') => {}
            _ => bail!(
                "safetensors {}: Array im Header-JSON ist unvollstaendig",
                path.display()
            ),
        }
    }
}

fn parse_object(reader: &mut std::io::Cursor<&[u8]>, path: &Path) -> Result<JsonValue> {
    let mut members = Vec::new();
    read_byte(reader);
    loop {
        skip_ws(reader);
        if peek_byte(reader) == Some(b'}') {
            read_byte(reader);
            return Ok(JsonValue::Object(members));
        }
        let key = parse_string(reader, path)?;
        skip_ws(reader);
        if read_byte(reader) != Some(b':') {
            bail!(
                "safetensors {}: Schluessel {key} hat keinen Doppelpunkt",
                path.display()
            );
        }
        skip_ws(reader);
        let value = parse_value(reader, path)?;
        members.push((key, value));
        skip_ws(reader);
        match peek_byte(reader) {
            Some(b',') => {
                read_byte(reader);
            }
            Some(b'}') => {}
            _ => bail!(
                "safetensors {}: Objekt im Header-JSON ist unvollstaendig",
                path.display()
            ),
        }
    }
}

/// Liest eine Zeichenkette samt Escapes inklusive Unicode-Escapes.
fn parse_string(reader: &mut std::io::Cursor<&[u8]>, path: &Path) -> Result<String> {
    if read_byte(reader) != Some(b'"') {
        bail!(
            "safetensors {}: Zeichenkette im Header beginnt nicht mit '\"'",
            path.display()
        );
    }
    let mut bytes: Vec<u8> = Vec::new();
    loop {
        match peek_byte(reader) {
            None => bail!(
                "safetensors {}: Zeichenkette im Header ist nicht geschlossen",
                path.display()
            ),
            Some(b'"') => {
                read_byte(reader);
                break;
            }
            Some(b'\\') => {
                read_byte(reader);
                let esc = read_byte(reader).ok_or_else(|| {
                    crate::error::anyhow!(
                        "safetensors {}: Escape im Header endet vorzeitig",
                        path.display()
                    )
                })?;
                match esc {
                    b'u' => {
                        let mut hex = [0u8; 4];
                        for slot in &mut hex {
                            *slot = read_byte(reader).unwrap_or(b'0');
                        }
                        let code =
                            u32::from_str_radix(std::str::from_utf8(&hex).unwrap_or("0"), 16)
                                .unwrap_or(0);
                        let c = char::from_u32(code).unwrap_or('?');
                        let mut buf = [0u8; 4];
                        bytes.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                    }
                    b'"' => bytes.push(b'"'),
                    b'\\' => bytes.push(b'\\'),
                    b'n' => bytes.push(b'\n'),
                    b't' => bytes.push(b'\t'),
                    b'r' => bytes.push(b'\r'),
                    other => bytes.push(other),
                }
            }
            Some(_) => bytes.push(read_byte(reader).unwrap_or(0)),
        }
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Nicht-negative Ganzzahl; der Header kennt keine Nachkommastellen.
fn parse_number(reader: &mut std::io::Cursor<&[u8]>) -> u64 {
    let mut value: u64 = 0;
    while let Some(c) = peek_byte(reader) {
        if !c.is_ascii_digit() {
            break;
        }
        read_byte(reader);
        value = value.saturating_mul(10).saturating_add(u64::from(c - b'0'));
    }
    value
}

fn expect_keyword(reader: &mut std::io::Cursor<&[u8]>, word: &[u8], path: &Path) -> Result<()> {
    for expected in word {
        match read_byte(reader) {
            Some(c) if c == *expected => {}
            _ => bail!(
                "safetensors {}: Schluesselwort im Header-JSON ist unvollstaendig",
                path.display()
            ),
        }
    }
    Ok(())
}

fn skip_ws(reader: &mut std::io::Cursor<&[u8]>) {
    while let Some(c) = peek_byte(reader) {
        if c == b' ' || c == b'\n' || c == b'\r' || c == b'\t' {
            read_byte(reader);
        } else {
            break;
        }
    }
}

fn peek_byte(reader: &mut std::io::Cursor<&[u8]>) -> Option<u8> {
    match reader.fill_buf() {
        Ok(buf) => buf.first().copied(),
        Err(_) => None,
    }
}

fn read_byte(reader: &mut std::io::Cursor<&[u8]>) -> Option<u8> {
    let mut b = [0u8; 1];
    match reader.read_exact(&mut b) {
        Ok(()) => Some(b[0]),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Frischer Ordner unter dem Temp-Verzeichnis; die Prozess-ID trennt parallele Laeufe.
    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "shiro-safetensors-{}-{tag}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("Temp-Ordner anlegen");
        dir
    }

    fn drei_tensoren() -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let a: Vec<f32> = (0..6).map(|i| (i as f32) - 3.0).collect();
        let b = vec![1.5f32];
        let c: Vec<f32> = (0..16).map(|i| (i as f32) * 0.25).collect();
        (a, b, c)
    }

    fn sechzehn() -> Vec<f32> {
        (0..16).map(|i| 1.0 + (i as f32) / 16.0).collect()
    }

    /// Header-Laenge, geparster JSON-Header und Nutzdaten einer Datei.
    fn zerlege(raw: &[u8]) -> (usize, serde_json::Value, Vec<u8>) {
        let len = u64::from_le_bytes(raw[0..8].try_into().unwrap()) as usize;
        let value: serde_json::Value = serde_json::from_slice(&raw[8..8 + len]).unwrap();
        (len, value, raw[8 + len..].to_vec())
    }

    /// Baut eine Datei mit veraendertem Header - fuer dtypes und Offsets, die der
    /// eigene Schreiber gar nicht erzeugt. Die Nutzdaten bleiben unveraendert.
    fn datei_mit_header(raw: &[u8], f: impl Fn(&serde_json::Value) -> String) -> Vec<u8> {
        let (_, value, nutzdaten) = zerlege(raw);
        let json = f(&value);
        let mut neu = Vec::new();
        neu.extend_from_slice(&(json.len() as u64).to_le_bytes());
        neu.extend_from_slice(json.as_bytes());
        neu.extend_from_slice(&nutzdaten);
        neu
    }

    /// Schaltet den Tensor-`dtype` auf einen anderen um. Der Footer-Eintrag bleibt
    /// F32: sonst traefe die Aenderung genau den als Footer erkannten Eintrag, und
    /// die Meldung kaeme vom falschen Tensor.
    fn mit_fremdem_dtype(raw: &[u8], neu: &str) -> Vec<u8> {
        let (_, _, nutzdaten) = zerlege(raw);
        let text = std::str::from_utf8(&raw[8..raw.len() - nutzdaten.len()])
            .unwrap()
            .to_string();
        let alt = "\"dtype\":\"F32\"";
        let pos = text.rfind(alt).expect("dtype im Header");
        let ersetzt = format!(
            "{}{}{}",
            &text[..pos],
            format_args!("\"dtype\":\"{neu}\""),
            &text[pos + alt.len()..]
        );
        serde_json::from_str::<serde_json::Value>(&ersetzt).expect("geaenderter Header ist JSON");
        let head = ersetzt.into_bytes();
        let mut out = Vec::new();
        out.extend_from_slice(&(head.len() as u64).to_le_bytes());
        out.extend_from_slice(&head);
        out.extend_from_slice(&nutzdaten);
        out
    }

    #[test]
    fn roundtrip_ist_bitgenau_mit_metadaten() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("model.safetensors");
        let (a, b, c) = drei_tensoren();

        let tensor_a = TensorView::new("model.embed_tokens.weight", vec![2, 3], &a).unwrap();
        let tensor_b = TensorView::new("lm_head.bias", vec![1], &b).unwrap();
        let tensor_c =
            TensorView::new("model.layers.0.self_attn.q_proj.weight", vec![4, 4], &c).unwrap();
        let tensors = vec![tensor_a, tensor_b, tensor_c];
        let meta = [
            ("format", "safetensors"),
            ("modell", "shiro-s"),
            ("dtype", "F32"),
        ];
        write_f32(&path, &tensors, &meta).unwrap();

        let back = read_f32(&path).unwrap();
        assert_eq!(back.len(), 3);
        assert_eq!(back[0].0, "model.embed_tokens.weight");
        assert_eq!(back[0].1, vec![2, 3]);
        assert_eq!(back[0].2, a, "Werte muessen bitgleich sein");
        assert_eq!(back[1].0, "lm_head.bias");
        assert_eq!(back[1].1, vec![1]);
        assert_eq!(back[1].2, b);
        assert_eq!(back[2].0, "model.layers.0.self_attn.q_proj.weight");
        assert_eq!(back[2].1, vec![4, 4]);
        assert_eq!(back[2].2, c);

        // Dateigroesse = 8 Byte Laenge + Header + Nutzdaten.
        assert_eq!(declared_len(&path).unwrap(), (6 + 1 + 16) * 4);
        let raw = std::fs::read(&path).unwrap();
        let (len, value, nutzdaten) = zerlege(&raw);
        assert_eq!(value["__metadata__"]["modell"], "shiro-s");
        assert_eq!(raw.len(), 8 + len + declared_len(&path).unwrap());
        assert_eq!(nutzdaten.len(), (6 + 1 + 16) * 4);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn header_enthaelt_nur_tensoreintraege_und_metadaten() {
        let dir = temp_dir("header-form");
        let path = dir.join("model.safetensors");
        let (a, b, c) = drei_tensoren();

        let tensors = vec![
            TensorView::new("a", vec![2, 3], &a).unwrap(),
            TensorView::new("b", vec![1], &b).unwrap(),
            TensorView::new("c", vec![4, 4], &c).unwrap(),
        ];
        let meta = [("k", "v")];
        write_f32(&path, &tensors, &meta).unwrap();

        let raw = std::fs::read(&path).unwrap();
        let (_, value, nutzdaten) = zerlege(&raw);
        let text = std::str::from_utf8(&raw[8..raw.len() - nutzdaten.len()]).unwrap();
        assert!(
            text.starts_with("{\"__metadata__\":{\"k\":\"v\"},\"a\":"),
            "Reihenfolge im Header: {text}"
        );
        // Genau __metadata__ und die Tensoren - keine Zusatzfelder.
        let obj = value.as_object().expect("Header ist ein Objekt");
        let schluessel: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(schluessel, vec!["__metadata__", "a", "b", "c"], "{text}");
        for name in ["a", "b", "c"] {
            // `serde_json` sortiert Objektschluessel, deshalb als Menge geprueft.
            let felder: Vec<&str> = value[name]
                .as_object()
                .expect("Tensor ist ein Objekt")
                .keys()
                .map(String::as_str)
                .collect();
            let mut sortiert = felder.clone();
            sortiert.sort_unstable();
            assert_eq!(
                sortiert,
                vec!["data_offsets", "dtype", "shape"],
                "genau dtype/shape/data_offsets, nichts weiter: {text}"
            );
            assert_eq!(value[name]["dtype"], "F32");
        }

        // Die Laenge des Datenblocks folgt aus dem letzten data_offsets-Ende.
        assert_eq!(value["a"]["data_offsets"][0], 0);
        assert_eq!(value["a"]["data_offsets"][1], 24);
        assert_eq!(value["b"]["data_offsets"][0], 24);
        assert_eq!(value["b"]["data_offsets"][1], 28);
        assert_eq!(value["c"]["data_offsets"][0], 28);
        assert_eq!(value["c"]["data_offsets"][1], 92);
        assert_eq!(declared_len(&path).unwrap(), 92);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn headerlaenge_ist_korrektes_json() {
        let dir = temp_dir("header");
        let path = dir.join("model.safetensors");
        let werte = drei_tensoren().0;
        let tensors = vec![TensorView::new("a", vec![2, 3], &werte).unwrap()];
        write_f32(&path, &tensors, &[]).unwrap();

        let raw = std::fs::read(&path).unwrap();
        let len = u64::from_le_bytes(raw[0..8].try_into().unwrap()) as usize;
        let header = &raw[8..8 + len];
        assert!(
            header.first() == Some(&b'{'),
            "Header muss mit '{{' beginnen"
        );
        // Die Laenge zaehlt genau die JSON-Bytes.
        assert_eq!(len, header.len());
        let value: serde_json::Value = serde_json::from_slice(header).expect("Header ist JSON");
        assert!(value.is_object());
        assert_eq!(value["a"]["dtype"], "F32");
        assert_eq!(value["a"]["shape"][0], 2);
        assert_eq!(value["a"]["shape"][1], 3);
        // Offsets sind relativ zum Datenblock, nicht zum File.
        assert_eq!(value["a"]["data_offsets"][0], 0);
        assert_eq!(value["a"]["data_offsets"][1], 24);
        assert!(
            value.get("__metadata__").is_none(),
            "leere Metadaten entfallen"
        );
        // Keine Zusatzfelder: der Header kennt nur diesen einen Tensor.
        let obj = value.as_object().expect("Header ist ein Objekt");
        let schluessel: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(schluessel, vec!["a"], "{schluessel:?}");
        // Die Nutzdaten sind die ersten 24 Bytes nach dem Header.
        let (_, _, nutzdaten) = zerlege(&raw);
        assert_eq!(nutzdaten.len(), 24);
        let erwartet: Vec<u8> = werte.iter().flat_map(|v| v.to_le_bytes()).collect();
        assert_eq!(nutzdaten, erwartet, "Little-Endian in Schreibreihenfolge");
        assert_eq!(raw.len(), 8 + len + 24);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadaten_stehen_zuerst_im_header() {
        let dir = temp_dir("meta");
        let path = dir.join("model.safetensors");
        let werte = drei_tensoren().0;
        let tensors = vec![TensorView::new("lang.gewicht", vec![2, 3], &werte).unwrap()];
        let meta = [("pfad", "mit \"Zitat\" und \\ Backslash"), ("leer", "")];
        write_f32(&path, &tensors, &meta).unwrap();

        let raw = std::fs::read(&path).unwrap();
        let (len, value, _) = zerlege(&raw);
        let text = std::str::from_utf8(&raw[8..8 + len]).unwrap();
        assert!(
            text.starts_with("{\"__metadata__\":{"),
            "__metadata__ muss zuerst stehen: {text}"
        );
        assert_eq!(
            value["__metadata__"]["pfad"],
            "mit \"Zitat\" und \\ Backslash"
        );
        assert_eq!(value["__metadata__"]["leer"], "");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn shape_produkt_muss_zur_datenlaenge_passen() {
        let data = [0.0f32; 6];
        // Kein Fehler, sondern ein Result: keine Panik bei falschem Shape.
        let fehler = TensorView::new("kaputt", vec![2, 4], &data)
            .unwrap_err()
            .to_string();
        assert!(fehler.contains("8 Werte"), "{fehler}");
        assert!(TensorView::new("leer", vec![0], &[1.0]).is_err());
        assert!(TensorView::new("skalar", vec![], &[1.0]).is_ok());
        assert!(TensorView::new("gut", vec![2, 3], &data).is_ok());

        // Und der Schreibpfad prueft ebenfalls, statt zu schreiben.
        let dir = temp_dir("shape");
        let path = dir.join("model.safetensors");
        let tensors = vec![TensorView::new("gut", vec![2, 3], &data).unwrap()];
        assert!(write_f32(&path, &tensors, &[]).is_ok());

        let data2 = [0.0f32; 6];
        let mut gefaelscht = TensorView::new("gut", vec![2, 3], &data2).unwrap();
        gefaelscht.shape = vec![7];
        assert!(write_f32(&path, &[gefaelscht], &[]).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn leere_tensorliste_erzeugt_valide_datei() {
        let dir = temp_dir("leer");
        let path = dir.join("model.safetensors");
        write_f32(&path, &[], &[]).unwrap();

        let raw = std::fs::read(&path).unwrap();
        let len = u64::from_le_bytes(raw[0..8].try_into().unwrap()) as usize;
        assert!(raw[8] == b'{');
        let value: serde_json::Value = serde_json::from_slice(&raw[8..8 + len]).unwrap();
        assert!(value.is_object());
        assert!(value.get("__metadata__").is_none());

        assert_eq!(declared_len(&path).unwrap(), 0);
        assert!(read_f32(&path).unwrap().is_empty());
        assert_eq!(raw.len(), 8 + len);
        assert!(path.is_file());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn leerer_tensor_mit_offsets_null_ist_lesbar() {
        let dir = temp_dir("leertensor");
        let path = dir.join("model.safetensors");
        let tensors = vec![TensorView::new("leer", vec![0], &[]).unwrap()];
        write_f32(&path, &tensors, &[]).unwrap();

        let back = read_f32(&path).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].1, vec![0]);
        assert!(back[0].2.is_empty());
        assert_eq!(declared_len(&path).unwrap(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn kein_tmp_rest_nach_dem_schreiben() {
        let dir = temp_dir("tmp");
        let path = dir.join("model.safetensors");
        let werte = sechzehn();
        let tensors = vec![TensorView::new("t", vec![4, 4], &werte).unwrap()];
        let meta = [("k", "v")];
        write_f32(&path, &tensors, &meta).unwrap();

        assert!(!tmp_path(&path).exists(), "tmp-Datei blieb liegen");
        let namen: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(namen, vec!["model.safetensors".to_string()], "{namen:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ersetzt_vorhandene_datei_anstandslos() {
        let dir = temp_dir("ersetzt");
        let path = dir.join("model.safetensors");
        let eins = [1.0f32, 2.0];
        let zwei = [3.0f32, 4.0];
        let t1 = TensorView::new("t", vec![2], &eins).unwrap();
        write_f32(&path, &[t1], &[]).unwrap();
        let t2 = TensorView::new("t", vec![2], &zwei).unwrap();
        write_f32(&path, &[t2], &[]).unwrap();

        let back = read_f32(&path).unwrap();
        assert_eq!(back[0].2, zwei.to_vec());
        assert!(!tmp_path(&path).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fremder_dtype_ist_ein_klarer_fehler() {
        let dir = temp_dir("dtype");
        let path = dir.join("modell.safetensors");
        let werte = [1.0f32, 2.0];
        let tensors = vec![TensorView::new("t", vec![2], &werte).unwrap()];
        write_f32(&path, &tensors, &[]).unwrap();

        // F16 statt F32: gueltiges safetensors, das dieser Leser nicht deuten darf.
        let gut = std::fs::read(&path).unwrap();
        let raw = mit_fremdem_dtype(&gut, "F16");
        std::fs::write(&path, &raw).unwrap();

        let fehler = read_f32(&path).unwrap_err().to_string();
        assert!(fehler.contains("nur F32"), "{fehler}");
        assert!(fehler.contains("dtype F16"), "{fehler}");
        // Der Header ist lesbar, nur der dtype passt nicht. `declared_len` liest
        // denselben Header und bricht deshalb ebenfalls ab - die Nutzdaten der
        // Datei bleiben aber vollstaendig.
        let (_, value, nutzdaten) = zerlege(&raw);
        assert_eq!(value["t"]["data_offsets"][1], 8);
        assert_eq!(nutzdaten.len(), 8);
        assert!(declared_len(&path).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn kaputte_kopflaenge_und_leerer_header_werden_erkannt() {
        let dir = temp_dir("kaputt");
        let path = dir.join("modell.safetensors");
        let werte = drei_tensoren().0;
        let tensors = vec![TensorView::new("t", vec![2, 3], &werte).unwrap()];
        write_f32(&path, &tensors, &[]).unwrap();
        let gut = std::fs::read(&path).unwrap();

        // Laenge zeigt weit ueber das Dateiende hinaus - Fehler, kein capacity overflow.
        let mut raw = gut.clone();
        raw[0..8].copy_from_slice(&u64::MAX.to_le_bytes());
        std::fs::write(&path, &raw).unwrap();
        let fehler = read_f32(&path).unwrap_err().to_string();
        assert!(fehler.contains("passt nicht in die Datei"), "{fehler}");

        // Plausible, aber zu grosse Kopflaenge.
        let mut raw = gut.clone();
        raw[0..8].copy_from_slice(&1_000_000u64.to_le_bytes());
        std::fs::write(&path, &raw).unwrap();
        assert!(read_f32(&path).is_err());

        // Header beginnt nicht mit '{'.
        let mut raw = gut.clone();
        raw[8] = b'[';
        std::fs::write(&path, &raw).unwrap();
        assert!(read_f32(&path).is_err());

        // Header-JSON ist abgeschnitten.
        let mut raw = gut.clone();
        let cut = raw.len() / 2;
        raw.truncate(cut);
        std::fs::write(&path, &raw).unwrap();
        assert!(read_f32(&path).is_err());

        // Datei ist kleiner als die acht Laengenbytes.
        std::fs::write(&path, b"kurz").unwrap();
        assert!(read_f32(&path).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn offsets_werden_gegen_die_shape_geprueft() {
        let dir = temp_dir("offsets");
        let path = dir.join("modell.safetensors");
        let werte: Vec<f32> = (0..3).map(|i| i as f32).collect();
        let tensors = vec![TensorView::new("t", vec![3], &werte).unwrap()];
        write_f32(&path, &tensors, &[]).unwrap();
        let gut = std::fs::read(&path).unwrap();
        let (_, _, nutzdaten) = zerlege(&gut);
        let text = std::str::from_utf8(&gut[8..gut.len() - nutzdaten.len()]).unwrap();
        assert!(text.contains("[0,12]"), "{text}");

        // Shape [3] in F32 braucht 12 Bytes, die Offsets versprechen 8.
        let raw = datei_mit_header(&gut, |v| v.to_string().replace("[0,12]", "[0,8]"));
        std::fs::write(&path, &raw).unwrap();
        let fehler = read_f32(&path).unwrap_err().to_string();
        assert!(fehler.contains("Span 8 Bytes"), "{fehler}");

        // Offsets in falscher Reihenfolge.
        let raw = datei_mit_header(&gut, |v| v.to_string().replace("[0,12]", "[8,0]"));
        std::fs::write(&path, &raw).unwrap();
        assert!(read_f32(&path).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn metadaten_mit_sonderzeichen_bleiben_json() {
        let dir = temp_dir("escapes");
        let path = dir.join("model.safetensors");
        let werte = [1.0f32, 2.0, 3.0, 4.0];
        let tensors = vec![TensorView::new("t", vec![2, 2], &werte).unwrap()];
        let meta = [("text", "Zeile\nTab\tUmlaut ae")];
        write_f32(&path, &tensors, &meta).unwrap();

        let raw = std::fs::read(&path).unwrap();
        let (_len, value, _) = zerlege(&raw);
        assert_eq!(value["__metadata__"]["text"], "Zeile\nTab\tUmlaut ae");
        assert_eq!(read_f32(&path).unwrap()[0].2, werte.to_vec());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
