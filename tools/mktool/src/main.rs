use std::env;
use std::fs;
use std::path::PathBuf;

const B_TASK: u8 = 0xFE;
const B_THINK: u8 = 0xFD;
const B_ANSWER: u8 = 0xFC;
const B_END: u8 = 0xFF;
const B_PLAN: u8 = 0xFB;
const COLS: [&str; 10] = ["E", "Z", "H", "T", "K", "L", "M", "N", "O", "P"];

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.below((hi - lo + 1) as u64) as i64)
    }
}

fn pow10(n: u32) -> u64 {
    let mut v = 1u64;
    for _ in 0..n {
        v *= 10;
    }
    v
}

fn add_trace(a: u64, b: u64) -> (String, u64) {
    let mut steps: Vec<String> = Vec::new();
    let mut carry = 0u64;
    let mut i = 0u32;
    while i < 19 {
        let da = (a / pow10(i)) % 10;
        let db = (b / pow10(i)) % 10;
        if i > 0 && a / pow10(i) == 0 && b / pow10(i) == 0 && carry == 0 {
            break;
        }
        let s = da + db + carry;
        let digit = s % 10;
        let cout = s / 10;
        let lhs = if carry == 1 {
            format!("{}+{}+1", da, db)
        } else {
            format!("{}+{}", da, db)
        };
        steps.push(format!("{} {}={} -> {} u{}", COLS[(i as usize) % COLS.len()], lhs, s, digit, cout));
        carry = cout;
        i += 1;
    }
    let sum = a + b;
    let mut t = String::from(" ");
    t.push_str(&steps.join(" ; "));
    t.push_str(&format!(" ; R {} ", sum));
    (t, sum)
}

fn gen_add(rng: &mut Rng, wmin: u32, wmax: u32) -> (String, u64) {
    let wa = rng.range(wmin as i64, wmax as i64) as u32;
    let wb = rng.range(wmin as i64, wmax as i64) as u32;
    let a = rng.range(pow10(wa - 1) as i64, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(pow10(wb - 1) as i64, pow10(wb) as i64 - 1) as u64;
    let (trace, sum) = add_trace(a, b);
    (format!(" {}+{} ", a, b), 0).0;
    let _ = sum;
    (format!(" {}+{} ", a, b), sum)
}

fn build_add(rng: &mut Rng, wmin: u32, wmax: u32) -> Vec<u8> {
    let wa = rng.range(wmin as i64, wmax as i64) as u32;
    let wb = rng.range(wmin as i64, wmax as i64) as u32;
    let a = rng.range(pow10(wa - 1) as i64, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(pow10(wb - 1) as i64, pow10(wb) as i64 - 1) as u64;
    let (trace, sum) = add_trace(a, b);
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(trace.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", sum).as_bytes());
    r.push(B_END);
    r
}

fn fmt_scaled(v: i64) -> String {
    if v % 100 == 0 {
        format!("{}", v / 100)
    } else {
        let neg = v < 0;
        let a = v.abs();
        let s = format!("{}.{:02}", a / 100, a % 100);
        let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        if neg { format!("-{s}") } else { s }
    }
}

fn gen_level(level: u32, rng: &mut Rng) -> (String, i64) {
    match level {
        0 => {
            let a = rng.range(-99, 99) * 100;
            let b = rng.range(-99, 99) * 100;
            if rng.below(2) == 0 {
                (format!("{}+{}", fmt_scaled(a), fmt_scaled(b)), a + b)
            } else {
                (format!("{}-{}", fmt_scaled(a), fmt_scaled(b)), a - b)
            }
        }
        1 => {
            let a = rng.range(1, 40) * 100;
            let b = rng.range(1, 40) * 100;
            let c = rng.range(2, 9);
            if rng.below(2) == 0 {
                (format!("({}+{})*{c}", fmt_scaled(a), fmt_scaled(b)), (a + b) * c)
            } else {
                (format!("({}-{})*{c}", fmt_scaled(a), fmt_scaled(b)), (a - b) * c)
            }
        }
        2 => {
            let a = rng.range(-999, 999);
            let b = rng.range(-999, 999);
            if rng.below(2) == 0 {
                (format!("{}+{}", fmt_scaled(a), fmt_scaled(b)), a + b)
            } else {
                (format!("{}-{}", fmt_scaled(a), fmt_scaled(b)), a - b)
            }
        }
        3 => {
            let a = rng.range(1, 99);
            let b = rng.range(1, 99);
            let c = rng.range(2, 9);
            if rng.below(2) == 0 {
                (format!("({}+{})*{c}", fmt_scaled(a), fmt_scaled(b)), (a + b) * c)
            } else {
                (format!("({}-{})*{c}", fmt_scaled(a), fmt_scaled(b)), (a - b) * c)
            }
        }
        _ => {
            let a = rng.range(2, 12);
            let e = rng.range(2, 3);
            let v = if e == 2 { a * a } else { a * a * a };
            (format!("{a}^{e}"), v * 100)
        }
    }
}

fn plain_record(expr: &str, val: i64) -> Vec<u8> {
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {expr} ").as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(
        format!(" 1. Klammern zuerst. 2. Punkt vor Strich. 3. Ergebnis {}.", fmt_scaled(val)).as_bytes(),
    );
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", fmt_scaled(val)).as_bytes());
    r.push(B_END);
    r
}

fn fnv1a64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in data {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn add_steps(a: u64, b: u64) -> Vec<String> {
    let mut steps: Vec<String> = Vec::new();
    let mut carry = 0u64;
    let mut i = 0u32;
    while i < 19 {
        let da = (a / pow10(i)) % 10;
        let db = (b / pow10(i)) % 10;
        if i > 0 && a / pow10(i) == 0 && b / pow10(i) == 0 && carry == 0 {
            break;
        }
        let s = da + db + carry;
        let digit = s % 10;
        let cout = s / 10;
        let lhs = if carry == 1 { format!("{}+{}+1", da, db) } else { format!("{}+{}", da, db) };
        steps.push(format!("{} {}={} -> {} u{}", COLS[(i as usize) % COLS.len()], lhs, s, digit, cout));
        carry = cout;
        i += 1;
    }
    steps
}

fn build_add_first(rng: &mut Rng, wmin: u32, wmax: u32) -> Vec<u8> {
    let wa = rng.range(wmin as i64, wmax as i64) as u32;
    let wb = rng.range(wmin as i64, wmax as i64) as u32;
    let a = rng.range(pow10(wa - 1) as i64, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(pow10(wb - 1) as i64, pow10(wb) as i64 - 1) as u64;
    let steps = add_steps(a, b);
    let first = steps.first().unwrap().clone();
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", first).as_bytes());
    r.push(B_END);
    r
}

fn build_add_step(rng: &mut Rng, wmin: u32, wmax: u32) -> Vec<u8> {
    let wa = rng.range(wmin as i64, wmax as i64) as u32;
    let wb = rng.range(wmin as i64, wmax as i64) as u32;
    let a = rng.range(pow10(wa - 1) as i64, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(pow10(wb - 1) as i64, pow10(wb) as i64 - 1) as u64;
    let steps = add_steps(a, b);
    let last = steps.last().unwrap().clone();
    let head = steps[..steps.len() - 1].join(" ; ");
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(" {} ; ", head).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", last).as_bytes());
    r.push(B_END);
    r
}

fn build_add_path(rng: &mut Rng, wmin: u32, wmax: u32) -> Vec<u8> {
    let wa = rng.range(wmin as i64, wmax as i64) as u32;
    let wb = rng.range(wmin as i64, wmax as i64) as u32;
    let a = rng.range(pow10(wa - 1) as i64, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(pow10(wb - 1) as i64, pow10(wb) as i64 - 1) as u64;
    let (trace, _sum) = add_trace(a, b);
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    r.push(B_ANSWER);
    r.extend_from_slice(trace.as_bytes());
    r.push(B_END);
    r
}

fn write_shard(out: &PathBuf, recs: &[Vec<u8>], seed: u64, kind: &str) {
    let mut payload = Vec::new();
    for r in recs {
        payload.extend_from_slice(&(r.len() as u16).to_le_bytes());
        payload.extend_from_slice(r);
    }
    let mut header = Vec::new();
    header.extend_from_slice(b"BSHD");
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&32u16.to_le_bytes());
    header.extend_from_slice(&(recs.len() as u64).to_le_bytes());
    header.extend_from_slice(&0u64.to_le_bytes());
    header.extend_from_slice(b"mktool\0\0");
    let mut file = header.clone();
    file.extend_from_slice(&payload);
    let dir = out.join("val");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("shard-0000.bin"), &file).unwrap();
    let nutz: usize = recs.iter().map(|r| r.len()).sum();
    let manifest = format!(
        "{{\n \"format_version\": 1,\n \"vocab_size\": 256,\n \"byte_order\": \"little\",\n \"record_encoding\": \"raw-bytes-with-special-tokens\",\n \"seed\": {seed},\n \"note\": \"mktool\",\n \"tasks\": {{\"typ.{kind}\": {n}}},\n \"shards\": [{{\n  \"id\": 0,\n  \"path\": \"shard-0000.bin\",\n  \"records\": {n},\n  \"payload_bytes\": {nutz},\n  \"file_size\": {size},\n  \"checksum\": \"{ck:016x}\"\n }}]\n}}",
        n = recs.len(),
        size = file.len(),
        ck = fnv1a64(&payload)
    );
    fs::write(dir.join("manifest.json"), manifest).unwrap();
}

fn rewrite_para(rec: &[u8]) -> Vec<u8> {
    let cut = rec.iter().position(|b| *b == B_THINK).unwrap_or(rec.len());
    let head = &rec[..cut];
    let tail = &rec[cut..];
    let s = String::from_utf8_lossy(head).replace("Alle ", "Jedes ").replace(" sind ", " ist ");
    let mut out = s.as_bytes().to_vec();
    out.extend_from_slice(tail);
    out
}

fn paraphrase_file(src: &str, out: &PathBuf, seed: u64) {
    let data = fs::read(src).expect("shard");
    let n = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let mut off = 32usize;
    let mut recs: Vec<Vec<u8>> = Vec::new();
    for _ in 0..n {
        let len = u16::from_le_bytes(data[off..off + 2].try_into().unwrap()) as usize;
        off += 2;
        recs.push(rewrite_para(&data[off..off + len]));
        off += len;
    }
    write_shard(out, &recs, seed, "category_chain");
    println!("paraphrase: {} Saetze -> {}", recs.len(), out.display());
}

const WNAMES: [&str; 28] = ["Aldo","Bento","Ciril","Duro","Elmo","Fenn","Gero","Halm","Ilva","Jorik",
    "Kalu","Lemb","Miro","Nolan","Ovid","Parlo","Quilo","Rendo","Savo","Tulio",
    "Ubar","Vasto","Welk","Xeno","Yaro","Zelmo","Anka","Brix"];

fn build_world(rng: &mut Rng, l: usize, force: Option<u64>) -> Vec<u8> {
    let mut names: Vec<&str> = WNAMES.to_vec();
    for i in (1..names.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        names.swap(i, j);
    }
    let mut edges: Vec<usize> = (0..l).collect();
    for i in (1..edges.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        edges.swap(i, j);
    }
    let mut cite = vec![0usize; l];
    for (k, e) in edges.iter().enumerate() {
        cite[*e] = k + 1;
    }
    let kind = force.unwrap_or_else(|| rng.below(3));
    let (frm, to, answer, open) = if kind == 0 {
        let f = rng.range(0, (l - 1) as i64) as usize;
        let t = rng.range((f + 1) as i64, l as i64) as usize;
        (f, t, "ja", false)
    } else {
        let f = rng.range(1, l as i64) as usize;
        let t = rng.range(0, (f - 1) as i64) as usize;
        if kind == 1 { (f, t, "nein", false) } else { (f, t, "nicht ableitbar", true) }
    };
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(b" Gegeben: ");
    for i in 0..l {
        r.extend_from_slice(format!("({}) Alle {} sind {}. ", i + 1, names[i], names[i + 1]).as_bytes());
    }
    if answer != "ja" && !open {
        r.extend_from_slice(b"Es gilt nichts anderes. ");
    }
    r.extend_from_slice(format!("Ist {} ein {}?", names[frm], names[to]).as_bytes());
    r.push(B_THINK);
    let mut t = String::from(" 1. Ich verfolge von ");
    t.push_str(names[frm]);
    t.push_str(" aus nach oben.");
    if answer == "ja" {
        t.push_str(&format!(" {} liegt in der Spur. Also ist {} ein {}.", names[to], names[frm], names[to]));
    } else if answer == "nein" {
        t.push_str(&format!(" {} liegt nicht in der Spur, und die Saetze sind vollstaendig. Also ist {} kein {}.", names[to], names[frm], names[to]));
    } else {
        t.push_str(&format!(" {} liegt nicht in der Spur, aber die Saetze schliessen es nicht aus. Also nicht ableitbar.", names[to]));
    }
    r.extend_from_slice(t.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", answer).as_bytes());
    r.push(B_END);
    let _ = cite;
    r
}
fn fmtf(v: i64) -> String {
    if v % 100 == 0 {
        format!("{}", v / 100)
    } else {
        let neg = v < 0;
        let a = v.abs();
        let s = format!("{}.{:02}", a / 100, a % 100);
        let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        if neg { format!("-{s}") } else { s }
    }
}

fn ladder_record(rng: &mut Rng, level: u32) -> Vec<u8> {
    let (expr, steps): (String, Vec<String>) = match level {
        0 => {
            let a = rng.range(-99, -1);
            let b = rng.range(1, 99);
            let s = a + b;
            (format!("{}+{}", a, b), vec![format!("{}+{}={}", a, b, s)])
        }
        1 => {
            let a = rng.range(1, 40);
            let b = rng.range(1, 40);
            let c = rng.range(2, 9);
            let inner = a + b;
            (format!("({}+{})*{}", a, b, c),
             vec![format!("{}+{}={}", a, b, inner), format!("{}*{}={}", inner, c, inner * c)])
        }
        2 => {
            let a = rng.range(-40, -1);
            let b = rng.range(1, 40);
            let c = rng.range(2, 9);
            let inner = a - b;
            (format!("({}-{})*{}", a, b, c),
             vec![format!("{}-{}={}", a, b, inner), format!("{}*{}={}", inner, c, inner * c)])
        }
        3 => {
            let ai = rng.range(100, 999);
            let bi = rng.range(100, 999);
            let s = ai + bi;
            (format!("{}+{}", fmtf(ai), fmtf(bi)),
             vec![format!("{}*100+{}*100={} -> {}", ai, bi, s, fmtf(s))])
        }
        4 => {
            let a = rng.range(1, 99);
            let c = rng.range(2, 9);
            let v = a * c;
            (format!("{}*{}", fmtf(a), c), vec![format!("{}*{}={} -> {}", a, c, v, fmtf(v))])
        }
        _ => {
            let a = rng.range(1, 99);
            let sh = if rng.below(2) == 0 { 10 } else { 100 };
            let v = a * sh;
            (format!("{}*{}", fmtf(a), sh), vec![format!("Komma {} Stellen nach rechts: {}", if sh == 10 { 1 } else { 2 }, fmtf(v))])
        }
    };
    let mut path = String::from(" ");
    for (i, s) in steps.iter().enumerate() {
        path.push_str(&format!("{}. {}. ", i + 1, s));
    }
    let ergebnis = match level {
        0 => { let a = 0; a }
        _ => 0,
    };
    let _ = ergebnis;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {} ", expr).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(path.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(b" siehe Weg ");
    r.push(B_END);
    r
}
fn ladder2_record(rng: &mut Rng, level: u32) -> Vec<u8> {
    let (expr, steps, result): (String, Vec<String>, i64) = match level {
        0 => {
            let a = rng.range(-99, -1);
            let b = rng.range(1, 99);
            let s = a + b;
            (format!("{}+{}", a, b), vec![format!("{}+{}={}", a, b, s)], s * 100)
        }
        1 => {
            let a = rng.range(1, 40);
            let b = rng.range(1, 40);
            let c = rng.range(2, 9);
            let inner = a + b;
            (format!("({}+{})*{}", a, b, c),
             vec![format!("{}+{}={}", a, b, inner), format!("{}*{}={}", inner, c, inner * c)],
             inner * c * 100)
        }
        2 => {
            let a = rng.range(-40, -1);
            let b = rng.range(1, 40);
            let c = rng.range(2, 9);
            let inner = a - b;
            (format!("({}-{})*{}", a, b, c),
             vec![format!("{}-{}={}", a, b, inner), format!("{}*{}={}", inner, c, inner * c)],
             inner * c * 100)
        }
        3 => {
            let ai = rng.range(100, 999);
            let bi = rng.range(100, 999);
            let s = ai + bi;
            (format!("{}+{}", fmtf(ai), fmtf(bi)),
             vec![format!("{}+{}={} -> {}", ai, bi, s, fmtf(s))], s)
        }
        4 => {
            let a = rng.range(1, 99);
            let c = rng.range(2, 9);
            let v = a * c;
            (format!("{}*{}", fmtf(a), c),
             vec![format!("{}*{}={} -> {}", a, c, v, fmtf(v))], v)
        }
        _ => {
            let a = rng.range(1, 99);
            let sh = if rng.below(2) == 0 { 10 } else { 100 };
            let v = a * sh;
            let stellen = if sh == 10 { "1 Stelle" } else { "2 Stellen" };
            (format!("{}*{}", fmtf(a), sh),
             vec![format!("Komma {} nach rechts: {}", stellen, fmtf(v))], v)
        }
    };
    let mut path = String::from(" ");
    for (i, s) in steps.iter().enumerate() {
        path.push_str(&format!("{}. {}. ", i + 1, s));
    }
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {} ", expr).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(path.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", fmtf(result)).as_bytes());
    r.push(B_END);
    r
}
fn ladder_parts(rng: &mut Rng, level: u32) -> (String, String, i64) {
    let (expr, steps, result): (String, Vec<String>, i64) = match level {
        0 => {
            let a = rng.range(-99, -1);
            let b = rng.range(1, 99);
            let s = a + b;
            (format!("{}+{}", a, b), vec![format!("{}+{}={}", a, b, s)], s * 100)
        }
        1 => {
            let a = rng.range(1, 40);
            let b = rng.range(1, 40);
            let c = rng.range(2, 9);
            let inner = a + b;
            (format!("({}+{})*{}", a, b, c),
             vec![format!("{}+{}={}", a, b, inner), format!("{}*{}={}", inner, c, inner * c)],
             inner * c * 100)
        }
        2 => {
            let a = rng.range(-40, -1);
            let b = rng.range(1, 40);
            let c = rng.range(2, 9);
            let inner = a - b;
            (format!("({}-{})*{}", a, b, c),
             vec![format!("{}-{}={}", a, b, inner), format!("{}*{}={}", inner, c, inner * c)],
             inner * c * 100)
        }
        3 => {
            let ai = rng.range(100, 999);
            let bi = rng.range(100, 999);
            let s = ai + bi;
            (format!("{}+{}", fmtf(ai), fmtf(bi)),
             vec![format!("{}+{}={} -> {}", ai, bi, s, fmtf(s))], s)
        }
        4 => {
            let a = rng.range(1, 99);
            let c = rng.range(2, 9);
            let v = a * c;
            (format!("{}*{}", fmtf(a), c),
             vec![format!("{}*{}={} -> {}", a, c, v, fmtf(v))], v)
        }
        _ => {
            let a = rng.range(1, 99);
            let sh = if rng.below(2) == 0 { 10 } else { 100 };
            let v = a * sh;
            let stellen = if sh == 10 { "1 Stelle" } else { "2 Stellen" };
            (format!("{}*{}", fmtf(a), sh),
             vec![format!("Komma {} nach rechts: {}", stellen, fmtf(v))], v)
        }
    };
    let mut path = String::new();
    for (i, s) in steps.iter().enumerate() {
        path.push_str(&format!("{}. {}. ", i + 1, s));
    }
    (expr, path, result)
}

fn mix_record(rng: &mut Rng) -> Vec<u8> {
    let level = rng.below(6) as u32;
    let as_path = rng.below(2) == 0;
    let (expr, path, result) = ladder_parts(rng, level);
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {} ", expr).as_bytes());
    r.push(B_THINK);
    if as_path {
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", path.trim()).as_bytes());
    } else {
        r.extend_from_slice(format!(" {} ", path.trim()).as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", fmtf(result)).as_bytes());
    }
    r.push(B_END);
    r
}
fn pack_record(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let level = rng.below(6) as u32;
        let as_path = rng.below(2) == 0;
        let (expr, path, result) = ladder_parts(rng, level);
        let mut r = Vec::new();
        r.push(B_TASK);
        r.extend_from_slice(format!(" {} ", expr).as_bytes());
        r.push(B_THINK);
        if as_path {
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", path.trim()).as_bytes());
        } else {
            r.extend_from_slice(format!(" {} ", path.trim()).as_bytes());
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", fmtf(result)).as_bytes());
        }
        r.push(B_END);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn build_add_range(rng: &mut Rng, wmin: u32, wmax: u32, amin: i64, amax: i64, parity: u64) -> Vec<u8> {
    let wa = rng.range(wmin as i64, wmax as i64) as u32;
    let wb = rng.range(wmin as i64, wmax as i64) as u32;
    let lo = if amin > 0 { amin } else { pow10(wa - 1) as i64 };
    let hi = if amax > 0 { amax } else { pow10(wa) as i64 - 1 };
    let mut a = rng.range(lo, hi) as u64;
    if parity == 1 && a % 2 == 1 { a += 1; if a > hi as u64 { a -= 2; } }
    if parity == 2 && a % 2 == 0 { a += 1; if a > hi as u64 { a -= 2; } }
    let b = rng.range(pow10(wb - 1) as i64, pow10(wb) as i64 - 1) as u64;
    let (trace, sum) = add_trace(a, b);
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(trace.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", sum).as_bytes());
    r.push(B_END);
    r
}

fn pack_path_record(rng: &mut Rng, budget: usize, wmin: u32, wmax: u32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = build_add_path(rng, wmin, wmax);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}

fn pack_add_record(rng: &mut Rng, budget: usize, wmin: u32, wmax: u32, amin: i64, amax: i64, parity: u64) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = build_add_range(rng, wmin, wmax, amin, amax, parity);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn carry_count(a: u64, b: u64) -> u32 {
    let mut carry = 0u64;
    let mut n = 0u32;
    let mut i = 0u32;
    while i < 10 {
        let da = (a / pow10(i)) % 10;
        let db = (b / pow10(i)) % 10;
        if i > 0 && a / pow10(i) == 0 && b / pow10(i) == 0 && carry == 0 {
            break;
        }
        let s = da + db + carry;
        carry = s / 10;
        if carry == 1 {
            n += 1;
        }
        i += 1;
    }
    n
}

fn build_carry(rng: &mut Rng, width: u32, want: u32) -> Vec<u8> {
    loop {
        let a = rng.range(pow10(width - 1) as i64, pow10(width) as i64 - 1) as u64;
        let b = rng.range(pow10(width - 1) as i64, pow10(width) as i64 - 1) as u64;
        if carry_count(a, b) != want {
            continue;
        }
        let (trace, sum) = add_trace(a, b);
        let mut r = Vec::new();
        r.push(B_TASK);
        r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
        r.push(B_THINK);
        r.extend_from_slice(trace.as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", sum).as_bytes());
        r.push(B_END);
        return r;
    }
}
fn pack_both_record(rng: &mut Rng, budget: usize, wmin: u32, wmax: u32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = if rng.below(2) == 0 {
            build_add(rng, wmin, wmax)
        } else {
            build_add_path(rng, wmin, wmax)
        };
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn verify_path(path: &str, expect_sum: &str) -> Result<(), String> {
    let cols = ["E", "Z", "H", "T", "K", "L", "M", "N", "O", "P"];
    let mut carry = 0u64;
    let mut digits: Vec<char> = Vec::new();
    let mut idx = 0usize;
    let mut saw_r = false;
    for step in path.split(';') {
        let s = step.trim();
        if s.is_empty() {
            continue;
        }
        if let Some(rest) = s.strip_prefix("R ") {
            let sum = rest.trim();
            let expect: String = digits.iter().rev().collect();
            if sum != expect.trim_start_matches('0') && sum != expect {
                return Err(format!("R {} passt nicht zu den Stellen {}", sum, expect));
            }
            if !expect_sum.trim().is_empty() && sum != expect_sum.trim() {
                return Err(format!("R {} != erwartete Summe {}", sum, expect_sum.trim()));
            }
            saw_r = true;
            continue;
        }
        let mut it = s.split_whitespace();
        let col = it.next().ok_or("leerer Schritt")?;
        if col != cols[idx % cols.len()] {
            return Err(format!("Spalte {} statt {}", col, cols[idx % cols.len()]));
        }
        idx += 1;
        let lhs = it.next().ok_or("keine Rechnung")?;
        let rhs = it.next().ok_or("kein Pfeil")?;
        let out = it.next().ok_or("kein Ergebnis")?;
        let u = it.next().ok_or("kein Übertrag")?;
        if rhs != "->" {
            return Err(format!("Pfeil fehlt in '{}'", s));
        }
        let parts: Vec<&str> = lhs.split('=').collect();
        if parts.len() != 2 {
            return Err(format!("'=' fehlt in '{}'", lhs));
        }
        let operands: Vec<i64> = parts[0].split('+').map(|x| x.trim().parse().unwrap_or(-1)).collect();
        if operands.iter().any(|x| *x < 0) {
            return Err(format!("Operanden unlesbar in '{}'", lhs));
        }
        let got: i64 = operands.iter().sum();
        if got != parts[1].trim().parse::<i64>().unwrap_or(-2) {
            return Err(format!("Summe stimmt nicht in '{}'", lhs));
        }
        if got % 10 != out.parse::<i64>().unwrap_or(-3) {
            return Err(format!("Ergebnis stimmt nicht in '{}'", s));
        }
        let d = got % 10;
        let c = got / 10;
        if !u.trim_start_matches('u').parse::<u64>().map(|x| x == c as u64).unwrap_or(false) {
            return Err(format!("Übertrag stimmt nicht in '{}'", s));
        }
        digits.push(char::from_digit(d as u32, 10).ok_or("Ziffer")?);
        carry = c as u64;
    }
    if carry != 0 {
        return Err("Übertrag am Ende nicht verrechnet".to_string());
    }
    if !saw_r {
        return Err("kein R-Schritt".to_string());
    }
    Ok(())
}
fn verify_logic(task: &str, weg: &str, soll: &str) -> Result<(), String> {
    // Praemissen: (i) Alle X sind Y.
    let mut kette: Vec<String> = Vec::new();
    for teil in task.split("Alle ").skip(1) {
        if let Some(pos) = teil.find(" sind ") {
            let von = teil[..pos].trim().to_string();
            let bis = teil[pos + 6..].split('.').next().unwrap_or("").trim().to_string();
            if kette.is_empty() {
                kette.push(von);
            }
            kette.push(bis);
        }
    }
    if kette.len() < 2 {
        return Err("Praemissen unlesbar".to_string());
    }
    // Frage: Ist X ein Y?
    let frage = task.rsplit("Ist ").next().unwrap_or("");
    let f = frage.split(" ein ").next().unwrap_or("").trim().to_string();
    let t = frage.rsplit(" ein ").next().unwrap_or("").trim().trim_end_matches('?').to_string();
    let start = kette.iter().position(|n| *n == f).ok_or("Startname nicht in der Kette")?;
    let erreichbar: Vec<String> = kette[start..].to_vec();
    // Weg: 1. Erreichbar von X (n Namen): a, b, c.  2. ...
    let schritt1 = weg.split("2.").next().unwrap_or("");
    let liste_teil = schritt1.split(':').nth(1).unwrap_or("").trim();
    let liste: Vec<String> = liste_teil
        .split(',')
        .map(|s| s.trim().trim_end_matches('.').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if liste != erreichbar {
        return Err(format!("Liste {:?} != erreichbar {:?}", liste, erreichbar));
    }
    let drin = erreichbar.iter().any(|n| *n == t);
    let erwartet = if drin { "ja" } else if soll.trim() == "nein" { "nein" } else { "nicht ableitbar" };
    if erwartet != soll.trim() {
        return Err(format!("Antwort {} passt nicht zu drin={}", soll.trim(), drin));
    }
    Ok(())
}
fn verify_puzzle(task: &str, weg: &str, soll: &str) -> Result<(), String> {
    let mut kette: Vec<String> = Vec::new();
    for teil in task.split(" ist groesser als ") {
        let name = teil.split('.').next().unwrap_or("").trim();
        if name.is_empty() {
            continue;
        }
        let name = name.split_whitespace().last().unwrap_or("").to_string();
        if kette.is_empty() {
            kette.push(name);
        } else {
            kette.push(name);
        }
    }
    if kette.len() < 3 {
        return Err("Kette unlesbar".to_string());
    }
    let im_weg = weg.split("Kette: ").nth(1).ok_or("keine Kette im Weg")?;
    let im_weg = im_weg.split('.').next().unwrap_or("").trim();
    let weg_kette: Vec<String> = im_weg.split(" > ").map(|s| s.trim().to_string()).collect();
    if weg_kette != kette {
        return Err(format!("Wegkette {:?} != Praemissen {:?}", weg_kette, kette));
    }
    let erwartet = if task.contains("am kleinsten") {
        kette.last().cloned().unwrap_or_default()
    } else if task.contains("Wer ist groesser,") {
        let rest = task.split("Wer ist groesser,").nth(1).unwrap_or("");
        let a = rest.split(" oder ").next().unwrap_or("").trim().to_string();
        let b = rest.split(" oder ").nth(1).unwrap_or("").split('?').next().unwrap_or("").trim().to_string();
        let ia = kette.iter().position(|n| *n == a).ok_or("A nicht in Kette")?;
        let ib = kette.iter().position(|n| *n == b).ok_or("B nicht in Kette")?;
        kette[ia.min(ib)].clone()
    } else {
        return Err("Frage unlesbar".to_string());
    };
    if erwartet != soll.trim() {
        return Err(format!("Antwort {} != erwartet {}", soll.trim(), erwartet));
    }
    Ok(())
}
fn check_corpus(path: &str) {
    let data = fs::read(path).expect("shard");
    let n = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let mut off = 32usize;
    let mut ok = 0usize;
    let mut bad = 0usize;
    for _ in 0..n {
        let len = u16::from_le_bytes(data[off..off + 2].try_into().unwrap()) as usize;
        off += 2;
        let rec = &data[off..off + len];
        off += len;
        let at = rec.iter().position(|b| *b == B_ANSWER).unwrap_or(0);
        let end = rec.iter().position(|b| *b == B_END).unwrap_or(rec.len());
        let ts = rec.iter().position(|b| *b == B_THINK).unwrap_or(0);
        let zwischen = String::from_utf8_lossy(&rec[ts + 1..at]).to_string();
        let antwort = String::from_utf8_lossy(&rec[at + 1..end]).to_string();
        let aufgabe_ist_logik = zwischen.contains("Erreichbar");
        let (weg, soll) = if zwischen.trim().is_empty() {
            (antwort.trim().to_string(), String::new())
        } else {
            (zwischen.trim().to_string(), antwort.trim().to_string())
        };
        // Textaufgaben: die Zahlen aus dem Satz muessen zu den Spalten passen.
        let mut fehler: Option<String> = None;
        if let Some(zi) = weg.find("Zahlen:") {
            let rest = &weg[zi + 7..];
            let nums: Vec<u64> = rest.split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .take(2)
                .filter_map(|s| s.parse().ok())
                .collect();
            if nums.len() == 2 {
                let letzte = weg.rsplit("E ").next().unwrap_or("");
                let erster_schritt = weg.split("E ").nth(1).unwrap_or("");
                let _ = letzte;
                let teile: Vec<&str> = erster_schritt.split('=').collect();
                if teile.len() == 2 {
                    let ops: Vec<u64> = teile[0]
                        .split('+')
                        .filter_map(|x| x.trim().parse().ok())
                        .collect();
                    if ops.len() >= 2 && (ops[0] != nums[0] % 10 || ops[1] != nums[1] % 10) {
                        fehler = Some(format!(
                            "Textzahlen {} und {} passen nicht zur ersten Spalte",
                            nums[0], nums[1]
                        ));
                    }
                }
            }
        }
        if let Some(e) = fehler {
            bad += 1;
            if bad <= 3 {
                println!("  FEHLER: {} | Weg: {}", e, &weg[..weg.len().min(90)]);
            }
            continue;
        }
        let weg = match weg.find("E ") {
            Some(p) if p > 0 => weg[p..].to_string(),
            _ => weg,
        };
        if weg.contains("Kette: ") {
            match verify_puzzle(&String::from_utf8_lossy(&rec[..ts]), &weg, &soll) {
                Ok(()) => { ok += 1; }
                Err(e) => {
                    bad += 1;
                    if bad <= 3 {
                        println!("  RAETSEL-FEHLER: {} | Weg: {}", e, &weg[..weg.len().min(90)]);
                    }
                }
            }
            continue;
        }
        if aufgabe_ist_logik {
            let task_txt = String::from_utf8_lossy(&rec[..ts]).to_string();
            let mut okk = true;
            if let Err(e) = verify_logic(task_txt.trim(), &weg, &soll) {
                okk = false;
                bad += 1;
                if bad <= 3 {
                    println!("  LOGIK-FEHLER: {} | Weg: {}", e, &weg[..weg.len().min(90)]);
                }
            }
            if okk { ok += 1; }
            continue;
        }
        match verify_path(&weg, &soll) {
            Ok(()) => ok += 1,
            Err(e) => {
                bad += 1;
                if bad <= 3 {
                    println!("  FEHLER: {} | Weg: {}", e, &weg[..weg.len().min(90)]);
                }
            }
        }
    }
    println!("{}: {} geprueft, {} gueltig, {} ungueltig", path, ok + bad, ok, bad);
}
fn build_carry_mix(rng: &mut Rng, wmin: u32, wmax: u32, want: u32) -> Vec<u8> {
    loop {
        let wa = rng.range(wmin as i64, wmax as i64) as u32;
        let wb = rng.range(wmin as i64, wmax as i64) as u32;
        let a = rng.range(pow10(wa - 1) as i64, pow10(wa) as i64 - 1) as u64;
        let b = rng.range(pow10(wb - 1) as i64, pow10(wb) as i64 - 1) as u64;
        if carry_count(a, b) != want {
            continue;
        }
        let (trace, sum) = add_trace(a, b);
        let mut r = Vec::new();
        r.push(B_TASK);
        r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
        r.push(B_THINK);
        r.extend_from_slice(trace.as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", sum).as_bytes());
        r.push(B_END);
        return r;
    }
}
const COLS2: [&str; 10] = ["E", "Z", "H", "T", "K", "L", "M", "N", "O", "P"];

fn edge_record(rng: &mut Rng, kind: u64) -> Vec<u8> {
    let (a, b) = match kind {
        0 => (0u64, rng.range(1, 999) as u64),
        1 => (rng.range(1, 999) as u64, 0u64),
        2 => (0u64, 0u64),
        3 => {
            let w = rng.range(1, 4) as u32;
            let a = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
            (a, 0u64)
        }
        4 => {
            let a = rng.range(1, 9) as u64;
            (a, 10 - a)
        }
        _ => {
            let w = rng.range(1, 4) as u32;
            let a = pow10(w) as u64 - 1;
            (a, 1u64)
        }
    };
    let mut carry = 0u64;
    let mut steps: Vec<String> = Vec::new();
    let mut i = 0u32;
    while i < 10 {
        let da = (a / pow10(i)) % 10;
        let db = (b / pow10(i)) % 10;
        if i > 0 && a / pow10(i) == 0 && b / pow10(i) == 0 && carry == 0 {
            break;
        }
        let s = da + db + carry;
        let cout = s / 10;
        let c = if carry == 1 { format!("{}+{}+1", da, db) } else { format!("{}+{}", da, db) };
        steps.push(format!("{} {}={} -> {} u{}", COLS2[i as usize], c, s, s % 10, cout));
        carry = cout;
        i += 1;
    }
    let sum = a + b;
    let mut t = String::from(" ");
    t.push_str(&steps.join(" ; "));
    t.push_str(&format!(" ; R {} ", sum));
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(t.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", sum).as_bytes());
    r.push(B_END);
    r
}
fn build_add_full(rng: &mut Rng, maxw: u32) -> Vec<u8> {
    let wa = rng.range(1, maxw as i64) as u32;
    let wb = rng.range(1, maxw as i64) as u32;
    let a = rng.range(0, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(0, pow10(wb) as i64 - 1) as u64;
    let (trace, sum) = add_trace(a, b);
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(trace.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", sum).as_bytes());
    r.push(B_END);
    r
}

fn pack_full_record(rng: &mut Rng, budget: usize, maxw: u32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = build_add_full(rng, maxw);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn build_full_both(rng: &mut Rng, maxw: u32) -> Vec<u8> {
    let wa = rng.range(1, maxw as i64) as u32;
    let wb = rng.range(1, maxw as i64) as u32;
    let a = rng.range(0, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(0, pow10(wb) as i64 - 1) as u64;
    let (trace, sum) = add_trace(a, b);
    let as_path = rng.below(2) == 0;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    if as_path {
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", trace.trim()).as_bytes());
    } else {
        r.extend_from_slice(trace.as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", sum).as_bytes());
    }
    r.push(B_END);
    r
}

fn build_full_three(rng: &mut Rng, maxw: u32) -> Vec<u8> {
    let wa = rng.range(1, maxw as i64) as u32;
    let wb = rng.range(1, maxw as i64) as u32;
    let a = rng.range(0, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(0, pow10(wb) as i64 - 1) as u64;
    let (trace, sum) = add_trace(a, b);
    let first = trace.split(';').next().unwrap_or("").trim().to_string();
    let mode = rng.below(3);
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
    r.push(B_THINK);
    match mode {
        0 => {
            r.extend_from_slice(trace.as_bytes());
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", sum).as_bytes());
        }
        1 => {
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", trace.trim()).as_bytes());
        }
        _ => {
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", first).as_bytes());
        }
    }
    r.push(B_END);
    r
}

fn pack_full_three(rng: &mut Rng, budget: usize, maxw: u32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = build_full_three(rng, maxw);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn pack_full_both(rng: &mut Rng, budget: usize, maxw: u32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = build_full_both(rng, maxw);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn marked_record(rng: &mut Rng, maxw: u32, mode: u64) -> Vec<u8> {
    let wa = rng.range(1, maxw as i64) as u32;
    let wb = rng.range(1, maxw as i64) as u32;
    let a = rng.range(0, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(0, pow10(wb) as i64 - 1) as u64;
    let (trace, sum) = add_trace(a, b);
    let first = trace.split(';').next().unwrap_or("").trim().to_string();
    let aufgabe = match mode {
        0 => format!(" {}+{} ", a, b),
        1 => format!(" {}+{} (Weg) ", a, b),
        _ => format!(" {}+{} (erste Spalte) ", a, b),
    };
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(aufgabe.as_bytes());
    r.push(B_THINK);
    match mode {
        0 => {
            r.extend_from_slice(trace.as_bytes());
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", sum).as_bytes());
        }
        1 => {
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", trace.trim()).as_bytes());
        }
        _ => {
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", first).as_bytes());
        }
    }
    r.push(B_END);
    r
}

fn pack_marked(rng: &mut Rng, budget: usize, maxw: u32) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let mode = rng.below(3);
        let r = marked_record(rng, maxw, mode);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
const LNAMES: [&str; 30] = ["Aldo", "Bento", "Ciril", "Duro", "Elmo", "Fenn", "Gero", "Halm", "Ilva", "Jorik",
    "Kalu", "Lemb", "Miro", "Nolan", "Ovid", "Parlo", "Quilo", "Rendo", "Savo", "Tulio",
    "Ubar", "Vasto", "Welk", "Xeno", "Yaro", "Zelmo", "Anka", "Brix", "Cuno", "Dorn"];

fn logic_record(rng: &mut Rng, force: Option<u64>, world_marker: bool, paraphrasiert: bool) -> Vec<u8> {
    let l = rng.range(2, 7) as usize;
    let mut names: Vec<&str> = LNAMES.to_vec();
    for i in (1..names.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        names.swap(i, j);
    }
    let kind = force.unwrap_or_else(|| rng.below(3));
    let (f, t) = if kind == 0 {
        let f = rng.range(0, (l - 1) as i64) as usize;
        let t = rng.range((f + 1) as i64, l as i64) as usize;
        (f, t)
    } else {
        let f = rng.range(1, l as i64) as usize;
        let t = rng.range(0, (f - 1) as i64) as usize;
        (f, t)
    };
    let closed = kind == 1;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(b" Gegeben: ");
    for i in 0..l {
        if paraphrasiert {
            r.extend_from_slice(format!("({}) Jedes {} ist {}. ", i + 1, names[i], names[i + 1]).as_bytes());
        } else {
            r.extend_from_slice(format!("({}) Alle {} sind {}. ", i + 1, names[i], names[i + 1]).as_bytes());
        }
    }
    if closed {
        r.extend_from_slice(b"Es gilt nichts anderes. ");
    }
    if world_marker {
        r.extend_from_slice(if closed { b"[Welt: geschlossen] " } else { b"[Welt: offen] " });
    }
    r.extend_from_slice(format!("Ist {} ein {}?", names[f], names[t]).as_bytes());
    r.push(B_THINK);
    let mut kette = String::new();
    for i in f..=l {
        if i > f {
            kette.push_str(", ");
        }
        kette.push_str(names[i]);
    }
    let mut trace = format!(" 1. Erreichbar von {}: {}. ", names[f], kette);
    let answer = match kind {
        0 => {
            trace.push_str(&format!("2. {} steht in der Liste, also ja. ", names[t]));
            "ja"
        }
        1 => {
            trace.push_str(&format!("2. {} steht nicht in der Liste, und die Saetze sind vollstaendig, also nein. ", names[t]));
            "nein"
        }
        _ => {
            trace.push_str(&format!("2. {} steht nicht in der Liste, aber die Saetze schliessen es nicht aus, also nicht ableitbar. ", names[t]));
            "nicht ableitbar"
        }
    };
    r.extend_from_slice(trace.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", answer).as_bytes());
    r.push(B_END);
    r
}

// Random nouns: 130 syllables, 16900 two-syllable words. A model cannot memorise a word it
// meets once, so it has to learn the transitivity rule instead (round 251: with 30 fixed nouns
// the model scored 97.5 on training chains and 1.7 on new ones).
fn rand_noun(rng: &mut Rng) -> String {
    const KONS: [&str; 13] = ["b", "d", "f", "g", "k", "l", "m", "n", "p", "r", "s", "t", "v"];
    const VOK: [&str; 5] = ["a", "e", "i", "o", "u"];
    let mut s = String::new();
    // 1 to 4 syllables: the nouns must differ in LENGTH too, not only in letters. In round 255
    // every noun had four characters and the same shape, and the model could not tell them apart.
    let silben = 1 + rng.below(4);
    for _ in 0..silben {
        s.push_str(KONS[rng.below(KONS.len() as u64) as usize]);
        s.push_str(VOK[rng.below(VOK.len() as u64) as usize]);
    }
    s
}

fn logic_record_rand(rng: &mut Rng) -> Vec<u8> {
    let l = rng.range(2, 7) as usize;
    let mut names: Vec<String> = Vec::new();
    while names.len() < l + 1 {
        let n = rand_noun(rng);
        if !names.contains(&n) {
            names.push(n);
        }
    }
    let kind = rng.below(3);
    let (f, t) = if kind == 0 {
        let f = rng.range(0, (l - 1) as i64) as usize;
        let t = rng.range((f + 1) as i64, l as i64) as usize;
        (f, t)
    } else {
        let f = rng.range(1, l as i64) as usize;
        let t = rng.range(0, (f - 1) as i64) as usize;
        (f, t)
    };
    let closed = kind == 1;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(b" Gegeben: ");
    for i in 0..l {
        r.extend_from_slice(format!("({}) Alle {} sind {}. ", i + 1, names[i], names[i + 1]).as_bytes());
    }
    if closed {
        r.extend_from_slice(b"Sonst gilt nichts. ");
    }
    let vorwaerts = kind == 0;
    let antwort = if vorwaerts { "ja" } else { "nein" };
    r.extend_from_slice(format!("Ist {} ein {}?", names[f], names[t]).as_bytes());
    r.push(B_THINK);
    if vorwaerts {
        r.extend_from_slice(format!(" 1. Erreichbar von {}: {}. 2. {} steht in der Liste, also ja. ", names[f], names[f..=t].join(", "), names[t]).as_bytes());
    } else {
        r.extend_from_slice(format!(" 1. Erreichbar von {}: {}. 2. {} steht nicht in der Liste. ", names[f], names[f..].join(", "), names[t]).as_bytes());
    }
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", antwort).as_bytes());
    r.push(B_END);
    r
}

fn pack_logic_rand(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = logic_record_rand(rng);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn pack_logic(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = logic_record(rng, None, false, false);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn logic_record_en(rng: &mut Rng, force: Option<u64>) -> Vec<u8> {
    let l = rng.range(2, 7) as usize;
    let mut names: Vec<&str> = LNAMES.to_vec();
    for i in (1..names.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        names.swap(i, j);
    }
    let kind = force.unwrap_or_else(|| rng.below(3));
    let (f, t) = if kind == 0 {
        let f = rng.range(0, (l - 1) as i64) as usize;
        let t = rng.range((f + 1) as i64, l as i64) as usize;
        (f, t)
    } else {
        let f = rng.range(1, l as i64) as usize;
        let t = rng.range(0, (f - 1) as i64) as usize;
        (f, t)
    };
    let closed = kind == 1;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(b" Given: ");
    for i in 0..l {
        r.extend_from_slice(format!("({}) All {} are {}. ", i + 1, names[i], names[i + 1]).as_bytes());
    }
    if closed {
        r.extend_from_slice(b"Nothing else holds. ");
    }
    r.extend_from_slice(format!("Is {} a {}?", names[f], names[t]).as_bytes());
    r.push(B_THINK);
    let mut kette = String::new();
    for i in f..=l {
        if i > f {
            kette.push_str(", ");
        }
        kette.push_str(names[i]);
    }
    let mut trace = format!(" 1. Reachable from {}: {}. ", names[f], kette);
    let answer = match kind {
        0 => {
            trace.push_str(&format!("2. {} is in the list, so yes. ", names[t]));
            "yes"
        }
        1 => {
            trace.push_str(&format!("2. {} is not in the list, and the statements are complete, so no. ", names[t]));
            "no"
        }
        _ => {
            trace.push_str(&format!("2. {} is not in the list, but the statements do not rule it out, so not derivable. ", names[t]));
            "not derivable"
        }
    };
    r.extend_from_slice(trace.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", answer).as_bytes());
    r.push(B_END);
    r
}

fn pack_logic_en(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = logic_record_en(rng, None);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn logic_scaffold_en(rng: &mut Rng, force: Option<u64>) -> Vec<u8> {
    let l = rng.range(2, 7) as usize;
    let mut names: Vec<&str> = LNAMES.to_vec();
    for i in (1..names.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        names.swap(i, j);
    }
    let kind = force.unwrap_or_else(|| rng.below(3));
    let (f, t) = if kind == 0 {
        let f = rng.range(0, (l - 1) as i64) as usize;
        let t = rng.range((f + 1) as i64, l as i64) as usize;
        (f, t)
    } else {
        let f = rng.range(1, l as i64) as usize;
        let t = rng.range(0, (f - 1) as i64) as usize;
        (f, t)
    };
    let closed = kind == 1;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(b" Given: ");
    for i in 0..l {
        r.extend_from_slice(format!("({}) All {} are {}. ", i + 1, names[i], names[i + 1]).as_bytes());
    }
    if closed {
        r.extend_from_slice(b"Nothing else holds. ");
    }
    r.extend_from_slice(format!("Is {} a {}?", names[f], names[t]).as_bytes());

    // Das Modell waehlt seine Zerlegung selbst, abhaengig von der Kettenlaenge.
    let long = l >= 5;
    r.push(B_PLAN);
    if long {
        r.extend_from_slice(b" Plan: long chain, collect all reachable names first, then compare. ");
    } else {
        r.extend_from_slice(b" Plan: short chain, step up one premise at a time and compare. ");
    }

    r.push(B_THINK);
    let answer;
    if long {
        let mut kette = String::new();
        for i in f..=l {
            if i > f {
                kette.push_str(", ");
            }
            kette.push_str(names[i]);
        }
        let head = format!(" 1. Reachable from {}: {}. ", names[f], kette);
        r.extend_from_slice(head.as_bytes());
        match kind {
            0 => {
                r.extend_from_slice(format!("2. {} is in the list, so yes. ", names[t]).as_bytes());
                answer = "yes";
            }
            1 => {
                r.extend_from_slice(format!("2. {} is not in the list, and the statements are complete, so no. ", names[t]).as_bytes());
                answer = "no";
            }
            _ => {
                r.extend_from_slice(format!("2. {} is not in the list, but the statements do not rule it out, so not derivable. ", names[t]).as_bytes());
                answer = "not derivable";
            }
        }
    } else {
        let mut walk = String::new();
        walk.push_str(names[f]);
        for i in f..l {
            walk.push_str(&format!(" -> {}", names[i + 1]));
        }
        let head = format!(" 1. Walk {}: {}. ", names[f], walk);
        r.extend_from_slice(head.as_bytes());
        match kind {
            0 => {
                r.extend_from_slice(format!("2. {} is on that walk, so yes. ", names[t]).as_bytes());
                answer = "yes";
            }
            1 => {
                r.extend_from_slice(format!("2. {} is not on that walk, and the statements are complete, so no. ", names[t]).as_bytes());
                answer = "no";
            }
            _ => {
                r.extend_from_slice(format!("2. {} is not on that walk, but the statements do not rule it out, so not derivable. ", names[t]).as_bytes());
                answer = "not derivable";
            }
        }
    }
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", answer).as_bytes());
    r.push(B_END);
    r
}

fn pack_scaffold_en(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = logic_scaffold_en(rng, None);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn logic_pos_record(rng: &mut Rng, force: Option<u64>) -> Vec<u8> {
    let l = rng.range(2, 7) as usize;
    let mut names: Vec<&str> = LNAMES.to_vec();
    for i in (1..names.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        names.swap(i, j);
    }
    let kind = force.unwrap_or_else(|| rng.below(3));
    let (f, t) = if kind == 0 {
        let f = rng.range(0, (l - 1) as i64) as usize;
        let t = rng.range((f + 1) as i64, l as i64) as usize;
        (f, t)
    } else {
        let f = rng.range(1, l as i64) as usize;
        let t = rng.range(0, (f - 1) as i64) as usize;
        (f, t)
    };
    let closed = kind == 1;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(b" Gegeben: ");
    for i in 0..l {
        r.extend_from_slice(format!("({}) Alle {} sind {}. ", i + 1, names[i], names[i + 1]).as_bytes());
    }
    if closed {
        r.extend_from_slice(b"Es gilt nichts anderes. ");
    }
    r.extend_from_slice(format!("Ist {} ein {}?", names[f], names[t]).as_bytes());
    r.push(B_THINK);
    let kette_n = l - f + 1;
    let mut kette = String::new();
    for i in f..=l {
        if i > f {
            kette.push_str(", ");
        }
        kette.push_str(names[i]);
    }
    let pos = if kind == 0 { t - f + 1 } else { 0 };
    let mut trace = format!(" 1. Erreichbar von {} ({} Namen): {}. ", names[f], kette_n, kette);
    let answer = match kind {
        0 => {
            trace.push_str(&format!("2. {} steht an Stelle {} von {}, also ja. ", names[t], pos, kette_n));
            "ja"
        }
        1 => {
            trace.push_str(&format!("2. {} steht unter den {} Namen nicht, und die Saetze sind vollstaendig, also nein. ", names[t], kette_n));
            "nein"
        }
        _ => {
            trace.push_str(&format!("2. {} steht unter den {} Namen nicht, aber die Saetze schliessen es nicht aus, also nicht ableitbar. ", names[t], kette_n));
            "nicht ableitbar"
        }
    };
    r.extend_from_slice(trace.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", answer).as_bytes());
    r.push(B_END);
    r
}

fn pack_logic_pos(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = logic_pos_record(rng, None);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
const PNAMES: [&str; 16] = ["Anna", "Ben", "Clara", "David", "Emir", "Frida", "Greta", "Hannes",
    "Ida", "Jonas", "Klara", "Linus", "Maja", "Nils", "Olga", "Paul"];
const POBJ: [&str; 8] = ["Aepfel", "Birnen", "Murmeln", "Steine", "Karten", "Buecher", "Stifte", "Bilder"];

fn sach_record(rng: &mut Rng) -> Vec<u8> {
    let wa = rng.range(1, 3) as u32;
    let wb = rng.range(1, 3) as u32;
    let a = rng.range(pow10(wa) as i64 / 10, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(pow10(wb) as i64 / 10, pow10(wb) as i64 - 1) as u64;
    let name = PNAMES[rng.below(PNAMES.len() as u64) as usize];
    let obj = POBJ[rng.below(POBJ.len() as u64) as usize];
    let frage = rng.below(3);
    let satz = match frage {
        0 => format!("{} hat {} {} und bekommt {} dazu. Wie viele {} hat {} jetzt?", name, a, obj, b, obj, name),
        1 => format!("{} sammelt {} {}. Danach findet {} noch {}. Wie viele {} sind es?", name, a, obj, name, b, obj),
        _ => format!("{} hat {} {}. Sie kauft {} {}. Wie viele sind es insgesamt?", name, a, obj, b, obj),
    };
    let (trace, sum) = add_trace(a, b);
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {} ", satz).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(" 1. Zahlen: {} und {}. ", a, b).as_bytes());
    r.extend_from_slice(trace.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", sum).as_bytes());
    r.push(B_END);
    r
}

fn pack_sach(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = sach_record(rng);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn puzzle_record(rng: &mut Rng, force: Option<u64>) -> Vec<u8> {
    let n = rng.range(3, 6) as usize;
    let mut names: Vec<&str> = PNAMES.to_vec();
    for i in (1..names.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        names.swap(i, j);
    }
    // names[0] > names[1] > ... > names[n-1]
    let art = force.unwrap_or_else(|| rng.below(2));
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(b" ");
    for i in 0..n - 1 {
        r.extend_from_slice(format!("{} ist groesser als {}. ", names[i], names[i + 1]).as_bytes());
    }
    let (frage, antwort, pos) = match art {
        0 => {
            let p = rng.range(0, (n - 2) as i64) as usize;
            (format!("Wer ist groesser, {} oder {}?", names[p], names[p + 1]), names[p].to_string(), p)
        }
        _ => (format!("Wer ist am kleinsten?"), names[n - 1].to_string(), n - 1),
    };
    r.extend_from_slice(frage.as_bytes());
    r.push(B_THINK);
    let mut kette = String::new();
    for i in 0..n {
        if i > 0 {
            kette.push_str(" > ");
        }
        kette.push_str(names[i]);
    }
    r.extend_from_slice(format!(" 1. Kette: {}. ", kette).as_bytes());
    r.extend_from_slice(format!("2. {} steht an Stelle {} von {}, also {}. ", antwort, pos + 1, n, antwort).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", antwort).as_bytes());
    r.push(B_END);
    r
}

fn pack_puzzle(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = puzzle_record(rng, None);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn logic_steps_record(rng: &mut Rng, force: Option<u64>) -> Vec<u8> {
    let l = rng.range(2, 7) as usize;
    let mut names: Vec<&str> = LNAMES.to_vec();
    for i in (1..names.len()).rev() {
        let j = rng.below((i + 1) as u64) as usize;
        names.swap(i, j);
    }
    let kind = force.unwrap_or_else(|| rng.below(3));
    let (f, t) = if kind == 0 {
        let f = rng.range(0, (l - 1) as i64) as usize;
        let t = rng.range((f + 1) as i64, l as i64) as usize;
        (f, t)
    } else {
        let f = rng.range(1, l as i64) as usize;
        let t = rng.range(0, (f - 1) as i64) as usize;
        (f, t)
    };
    let closed = kind == 1;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(b" Gegeben: ");
    for i in 0..l {
        r.extend_from_slice(format!("({}) Alle {} sind {}. ", i + 1, names[i], names[i + 1]).as_bytes());
    }
    if closed {
        r.extend_from_slice(b"Es gilt nichts anderes. ");
    }
    r.extend_from_slice(format!("Ist {} ein {}?", names[f], names[t]).as_bytes());
    r.push(B_THINK);
    let n = l - f + 1;
    let mut liste = String::new();
    for i in f..=l {
        if i > f {
            liste.push_str(", ");
        }
        liste.push_str(names[i]);
    }
    // Schritt 1: Menge. Schritt 2: Mitgliedschaft. Schritt 3: Welt und Urteil.
    r.extend_from_slice(format!(" 1. Erreichbar von {} ({} Namen): {}. ", names[f], n, liste).as_bytes());
    if kind == 0 {
        let pos = t - f + 1;
        r.extend_from_slice(format!("2. Gesucht {}: an Stelle {} von {}. 3. Welt egal, also ja. ", names[t], pos, n).as_bytes());
    } else {
        r.extend_from_slice(format!("2. Gesucht {}: nicht in der Liste. ", names[t]).as_bytes());
        if closed {
            r.extend_from_slice(b"3. Welt geschlossen, also nein. ");
        } else {
            r.extend_from_slice(b"3. Welt offen, also nicht ableitbar. ");
        }
    }
    r.push(B_ANSWER);
    let antwort = match kind { 0 => "ja", 1 => "nein", _ => "nicht ableitbar" };
    r.extend_from_slice(format!(" {} ", antwort).as_bytes());
    r.push(B_END);
    r
}

fn pack_logic_steps(rng: &mut Rng, budget: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    loop {
        let r = logic_steps_record(rng, None);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
use std::collections::HashSet;

fn hash64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn seen_path(domain: &str, split: &str) -> PathBuf {
    let dir = PathBuf::from(std::env::var("TEMP").unwrap_or_else(|_| ".".into())).join("pmh").join("seen");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(format!("{}.{}.txt", domain, split))
}

fn seen_load(p: &PathBuf) -> HashSet<u64> {
    let mut s = HashSet::new();
    if let Ok(txt) = std::fs::read_to_string(p) {
        for line in txt.lines() {
            if let Ok(v) = u64::from_str_radix(line.trim(), 16) {
                s.insert(v);
            }
        }
    }
    s
}

fn seen_save(p: &PathBuf, s: &HashSet<u64>) {
    let txt: String = s.iter().map(|v| format!("{:016x}\n", v)).collect();
    let _ = std::fs::write(p, txt);
}
fn draw_pair(rng: &mut Rng, maxw: u32) -> (u64, u64) {
    let minw: u32 = std::env::var("SHIRO_MIN_WIDTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .min(maxw);
    let wa = rng.range(minw as i64, maxw as i64) as u32;
    let wb = rng.range(minw as i64, maxw as i64) as u32;
    let a = rng.range(0, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(0, pow10(wb) as i64 - 1) as u64;
    (a, b)
}

fn pair_hash(a: u64, b: u64) -> u64 {
    hash64(format!("{}+{}", a, b).as_bytes())
}

fn pack_unique(rng: &mut Rng, budget: usize, maxw: u32, seen: &mut HashSet<u64>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut guard = 0;
    loop {
        let (a, b) = draw_pair(rng, maxw);
        let h = pair_hash(a, b);
        guard += 1;
        if seen.contains(&h) || guard > 4000 {
            if guard > 4000 {
                break;
            }
            continue;
        }
        seen.insert(h);
        meta_push("arith", a, b);
        let (trace, sum) = add_trace(a, b);
        let mut r = Vec::new();
        r.push(B_TASK);
        r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
        r.push(B_THINK);
        r.extend_from_slice(trace.as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", sum).as_bytes());
        r.push(B_END);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn pack_generic<F: Fn(&mut Rng) -> Vec<u8>>(
    rng: &mut Rng,
    budget: usize,
    gen: F,
    seen: &mut HashSet<u64>,
) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut guard = 0;
    loop {
        let r = gen(rng);
        guard += 1;
        if guard > 50_000 {
            break;
        }
        let h = hash64(&r);
        if seen.contains(&h) {
            continue;
        }
        seen.insert(h);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}

fn unique_records<F: Fn(&mut Rng) -> Vec<u8>>(
    rng: &mut Rng,
    n: usize,
    gen: F,
    train: &HashSet<u64>,
    own: &mut HashSet<u64>,
) -> Vec<Vec<u8>> {
    let mut recs: Vec<Vec<u8>> = Vec::new();
    let mut guard = 0;
    while recs.len() < n && guard < n * 500 {
        guard += 1;
        let r = gen(rng);
        let h = hash64(&r);
        if train.contains(&h) || own.contains(&h) {
            continue;
        }
        own.insert(h);
        recs.push(r);
    }
    recs
}
fn pack_unique_plain(rng: &mut Rng, budget: usize, maxw: u32, seen: &mut HashSet<u64>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut guard = 0;
    loop {
        let (a, b) = draw_pair(rng, maxw);
        let h = pair_hash(a, b);
        guard += 1;
        if seen.contains(&h) {
            if guard > 4000 {
                break;
            }
            continue;
        }
        seen.insert(h);
        let sum = a + b;
        let mut r = Vec::new();
        r.push(B_TASK);
        r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
        r.push(B_THINK);
        r.extend_from_slice(b" ");
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", sum).as_bytes());
        r.push(B_END);
        if !out.is_empty() && out.len() + r.len() > budget {
            break;
        }
        out.extend_from_slice(&r);
    }
    out
}
fn meta_push(domain: &str, a: u64, b: u64) {
    use std::io::Write;
    let dir = PathBuf::from(std::env::var("TEMP").unwrap_or_else(|_| ".".into())).join("pmh").join("seen");
    let _ = std::fs::create_dir_all(&dir);
    let p = dir.join(format!("{}.meta.txt", domain));
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) {
        let _ = writeln!(f, "{};{};{}", pair_hash(a, b), a, b);
    }
}
fn check_plain(path: &str) -> (usize, usize) {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return (0, 0),
    };
    let mut recs: Vec<&[u8]> = Vec::new();
    let mut i = 0usize;
    if bytes.len() < 32 { return (0, 0); }
    i = 32;
    while i + 2 <= bytes.len() {
        let len = u16::from_le_bytes([bytes[i], bytes[i + 1]]) as usize;
        i += 2;
        if i + len > bytes.len() { break; }
        recs.push(&bytes[i..i + len]);
        i += len;
    }
    let mut ok = 0usize;
    let mut bad = 0usize;
    for rec in recs {
        for seg in rec.split(|b| *b == B_END) {
            let task_start = seg.iter().position(|b| *b == B_TASK);
            let think_start = seg.iter().position(|b| *b == B_THINK);
            let ans_start = seg.iter().position(|b| *b == B_ANSWER);
            let (ts, th, as_) = match (task_start, think_start, ans_start) {
                (Some(t), Some(h), Some(a)) => (t + 1, h, a + 1),
                _ => continue,
            };
            let task = String::from_utf8_lossy(&seg[ts..th]).to_string();
            let answer = String::from_utf8_lossy(&seg[as_..]).to_string();
            let answer = answer.trim();
            let rechnung = task.split('+').collect::<Vec<_>>();
            if rechnung.len() != 2 {
                bad += 1;
                continue;
            }
            let a: u64 = rechnung[0].trim().parse().unwrap_or(u64::MAX);
            let b: u64 = rechnung[1].trim().parse().unwrap_or(u64::MAX);
            let soll: u64 = answer.parse().unwrap_or(u64::MAX - 1);
            if a == u64::MAX || b == u64::MAX || soll == u64::MAX - 1 {
                bad += 1;
                continue;
            }
            if a + b == soll { ok += 1; } else { bad += 1; }
        }
    }
    (ok, bad)
}
fn add_trace_verbos(a: u64, b: u64) -> (String, u64) {
    let mut steps: Vec<String> = Vec::new();
    let mut carry = 0u64;
    let mut i = 0u32;
    while i < 19 {
        let da = (a / pow10(i)) % 10;
        let db = (b / pow10(i)) % 10;
        if i > 0 && a / pow10(i) == 0 && b / pow10(i) == 0 && carry == 0 {
            break;
        }
        let roh = da + db;
        let s = roh + carry;
        let digit = s % 10;
        let cout = s / 10;
        let col = COLS[(i as usize) % COLS.len()];
        if carry == 1 {
            steps.push(format!(
                "{}. {}: {}+{}={}. {} ohne Uebertrag, mit dem gemerkten 1 sind es {}, also Ziffer {} und Uebertrag {}.",
                i + 1, col, da, db, roh, roh, s, digit, cout
            ));
        } else {
            steps.push(format!(
                "{}. {}: {}+{}={}, also Ziffer {} und Uebertrag {}.",
                i + 1, col, da, db, s, digit, cout
            ));
        }
        carry = cout;
        i += 1;
    }
    let sum = a + b;
    let mut t = String::new();
    for s in &steps {
        t.push(' ');
        t.push_str(s);
    }
    t.push_str(&format!(" R {}. ", sum));
    (t, sum)
}

fn pack_verbos(rng: &mut Rng, budget: usize, maxw: u32, seen: &mut HashSet<u64>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut guard = 0;
    loop {
        let (a, b) = draw_pair(rng, maxw);
        let h = pair_hash(a, b);
        guard += 1;
        if seen.contains(&h) {
            if guard > 4000 { break; }
            continue;
        }
        seen.insert(h);
        let (trace, sum) = add_trace_verbos(a, b);
        let mut r = Vec::new();
        r.push(B_TASK);
        r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
        r.push(B_THINK);
        r.extend_from_slice(trace.as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", sum).as_bytes());
        r.push(B_END);
        if !out.is_empty() && out.len() + r.len() > budget { break; }
        out.extend_from_slice(&r);
    }
    out
}
fn stellen_record(rng: &mut Rng) -> Vec<u8> {
    let w = rng.range(2, 6) as u32;
    let n = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
    let s = n.to_string();
    let pos = rng.range(1, s.len() as i64) as usize;
    let ziffer = s.as_bytes()[s.len() - pos] as char;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" Welche Ziffer steht bei {} an Stelle {} von hinten?", n, pos).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(" 1. Zahl {}. 2. Von hinten zaehlen: {}. 3. Ziffer {}. ", s, pos, ziffer).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", ziffer).as_bytes());
    r.push(B_END);
    r
}

fn menge_record(rng: &mut Rng) -> Vec<u8> {
    let w = rng.range(2, 5) as u32;
    let a = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
    let b = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
    let groesser = a > b;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" Ist {} groesser als {}?", a, b).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(" 1. Stellen: {} hat {} Stellen, {} hat {} Stellen. 2. Groesser ist {}. ",
        a, a.to_string().len(), b, b.to_string().len(),
        if groesser { a } else { b }).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", if groesser { "ja" } else { "nein" }).as_bytes());
    r.push(B_END);
    r
}

fn pack_mixed_probe(rng: &mut Rng, budget: usize, seen: &mut HashSet<u64>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut guard = 0;
    loop {
        let r = if rng.below(2) == 0 { stellen_record(rng) } else { menge_record(rng) };
        let h = hash64(&r);
        guard += 1;
        if seen.contains(&h) {
            if guard > 20000 { break; }
            continue;
        }
        seen.insert(h);
        if !out.is_empty() && out.len() + r.len() > budget { break; }
        out.extend_from_slice(&r);
    }
    out
}
const WOERTER: [(&str, char, u32); 6] = [
    ("Raspberry", 'r', 3),
    ("Banane", 'a', 3),
    ("Aepfel", 'e', 2),
    ("Strasse", 's', 3),
    ("Hallo", 'l', 2),
    ("Krokodil", 'o', 2),
];

fn probe_task(rng: &mut Rng) -> Vec<u8> {
    let art = rng.below(6);
    let mut r = Vec::new();
    r.push(B_TASK);
    let antwort: String;
    match art {
        0 => {
            let w = rng.range(2, 6) as u32;
            let n = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
            let s = n.to_string();
            let k = rng.range(1, s.len() as i64) as usize;
            let z = s.as_bytes()[s.len() - k] as char;
            let form = rng.below(2);
            let frage = if form == 0 {
                format!("Welche Ziffer steht bei {} an Stelle {} von hinten?", n, k)
            } else {
                format!("Ziffer an Position {} von rechts in {}?", k, n)
            };
            r.extend_from_slice(format!(" {} ", frage).as_bytes());
            antwort = z.to_string();
        }
        1 => {
            let w = rng.range(3, 6) as u32;
            let n = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
            let s = n.to_string();
            let k = rng.range(1, s.len() as i64) as usize;
            let z = s.as_bytes()[s.len() - k] as char;
            let wert = (z.to_digit(10).unwrap() as u64) * pow10((k - 1) as u32);
            r.extend_from_slice(format!(" Welchen Wert hat die Ziffer {} in {}? ", z, n).as_bytes());
            antwort = wert.to_string();
        }
        2 => {
            let w = rng.range(4, 6) as u32;
            let n = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
            let s = n.to_string();
            let tausender = (s.as_bytes()[s.len() - 4] as char).to_string();
            let form = rng.below(2);
            let frage = if form == 0 {
                format!("Wie viele Tausender enthaelt {}?", n)
            } else {
                format!("Welche Ziffer steht bei {} an der Tausenderstelle?", n)
            };
            r.extend_from_slice(format!(" {} ", frage).as_bytes());
            antwort = tausender;
        }
        3 => {
            let a: u64 = [9999u64, 10000, 10001, 99999, 70001, 70010, 70100, 71000][rng.below(8) as usize];
            let b: u64 = [10000u64, 9999, 9999, 100000, 70010, 70100, 71000, 70001][rng.below(8) as usize];
            let form = rng.below(2);
            let frage = if form == 0 {
                format!("Ist {} groesser als {}?", a, b)
            } else {
                format!("Vergleiche {} und {}: ist die erste groesser?", a, b)
            };
            r.extend_from_slice(format!(" {} ", frage).as_bytes());
            antwort = if a > b { "ja" } else { "nein" }.to_string();
        }
        4 => {
            let w = rng.range(2, 5) as u32;
            let a = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
            let b = rng.range(pow10(w - 1) as i64, pow10(w) as i64 - 1) as u64;
            r.extend_from_slice(format!(" Welche Zahl ist kleiner: {} oder {}? ", a, b).as_bytes());
            antwort = if a < b { a } else { b }.to_string();
        }
        _ => {
            let (wort, buchstabe, anzahl) = WOERTER[rng.below(WOERTER.len() as u64) as usize];
            let form = rng.below(2);
            let frage = if form == 0 {
                format!("Wie viele {} hat {}?", buchstabe.to_uppercase(), wort)
            } else {
                format!("Zaehle die {} in {}.", buchstabe.to_uppercase(), wort)
            };
            r.extend_from_slice(format!(" {} ", frage).as_bytes());
            antwort = anzahl.to_string();
        }
    }
    r.push(B_THINK);
    r.extend_from_slice(b" ");
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", antwort).as_bytes());
    r.push(B_END);
    r
}
fn check_record(rng: &mut Rng) -> Vec<u8> {
    let wa = rng.range(1, 5) as u32;
    let wb = rng.range(1, 5) as u32;
    let a = rng.range(pow10(wa.saturating_sub(1)) as i64, pow10(wa) as i64 - 1) as u64;
    let b = rng.range(pow10(wb.saturating_sub(1)) as i64, pow10(wb) as i64 - 1) as u64;
    let echt = a + b;
    let modus = std::env::var("SHIRO_ERROR_MODE").unwrap_or_else(|_| "mix".into());
    let stimmt = match modus.as_str() {
        "ok" => true,
        "bad" => false,
        _ => rng.below(2) == 0,
    };
    let angegeben: u64 = if stimmt {
        echt
    } else {
        let art = rng.below(4);
        let s = echt.to_string();
        let mut z: Vec<u8> = s.bytes().collect();
        match art {
            0 => {
                let i = (z.len() - 1) as usize;
                z[i] = b'0' + ((z[i] - b'0' + 1 + rng.below(8) as u8) % 10);
                String::from_utf8(z).unwrap().parse().unwrap_or(echt + 1)
            }
            1 => {
                if z.len() > 2 {
                    let i = (rng.below((z.len() - 1) as u64)) as usize;
                    z[i] = b'0' + ((z[i] - b'0' + 1) % 10);
                    String::from_utf8(z).unwrap().parse().unwrap_or(echt + 1)
                } else {
                    echt + 1
                }
            }
            2 => echt + 10,
            _ => {
                if echt > 10 { echt - 1 } else { echt + 1 }
            }
        }
    };
    let form = rng.below(3);
    let frage = match form {
        0 => format!("Stimmt {} + {} = {}?", a, b, angegeben),
        1 => format!("Ist die Rechnung {} + {} = {} richtig?", a, b, angegeben),
        _ => format!("Pruefe: {} + {} = {}. Korrekt?", a, b, angegeben),
    };
    let (trace, sum) = add_trace(a, b);
    let antwort = if sum == angegeben { "ja" } else { "nein" };
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {} ", frage).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(trace.as_bytes());
    r.extend_from_slice(format!(" Vergleich {} mit {}: {} ", sum, angegeben, antwort).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", antwort).as_bytes());
    r.push(B_END);
    r
}
fn pack_budget() -> usize {
    std::env::var("SHIRO_PACK_BUDGET")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(480)
}
fn mult_record(rng: &mut Rng, art: u64) -> Vec<u8> {
    let (a, b) = match art {
        0 => (rng.range(0, 9) as u64, rng.range(0, 9) as u64),
        1 => (rng.range(10, 999) as u64, rng.range(2, 9) as u64),
        2 => (rng.range(100, 999) as u64, rng.range(10, 999) as u64),
        3 => (rng.range(1000, 99999) as u64, rng.range(10, 9999) as u64),
        4 => (rng.range(100, 999) as u64, rng.range(10, 99) as u64),
        5 => (rng.range(10, 99) as u64, rng.range(2, 9) as u64),
        6 => (rng.range(1000, 9999) as u64, rng.range(10, 99) as u64),
        7 => (rng.range(10, 99) as u64, rng.range(100, 999) as u64),
        8 => (rng.range(10, 99) as u64, rng.range(1000, 9999) as u64),
        9 => (rng.range(2, 9) as u64, rng.range(100, 999) as u64),
        10 => (rng.range(1000, 9999) as u64, rng.range(100, 999) as u64),
        // Two-digit times two-digit: short traces, same task family as art 2.
        14 => (rng.range(10, 99) as u64, rng.range(10, 99) as u64),
        // Three nonzero digits, so the column addition really happens (three partial products).
        13 => (
            rng.range(10000, 99999) as u64,
            (rng.range(1, 9) * 100 + rng.range(1, 9) * 10 + rng.range(1, 9)) as u64,
        ),
        12 => (
            rng.range(1000, 9999) as u64,
            (rng.range(1, 9) * 1000 + rng.range(1, 9) * 100 + rng.range(1, 9) * 10 + rng.range(1, 9)) as u64,
        ),
        11 => (
            rng.range(1000, 9999) as u64,
            (rng.range(1, 9) * 100 + rng.range(1, 9) * 10 + rng.range(1, 9)) as u64,
        ),
        _ => (rng.range(10, 99999) as u64, rng.range(2, 99) as u64),
    };
    let produkt = a * b;
    let mut teile: Vec<(u64, u64, u64)> = Vec::new();
    let bs = b.to_string();
    for (i, ch) in bs.chars().rev().enumerate() {
        let d = ch.to_digit(10).unwrap() as u64;
        if d == 0 {
            continue;
        }
        let faktor = d * pow10(i as u32);
        teile.push((faktor, a * faktor, i as u64));
    }
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}*{} ", a, b).as_bytes());
    r.push(B_THINK);
    // Zerlegung des zweiten Faktors in Ziffer mal Stellenwert, wie ein Mensch sie hinschreibt.
    let bs_txt = b.to_string();
    let stellen: Vec<String> = bs_txt
        .chars()
        .enumerate()
        .filter_map(|(pos, ch)| {
            let d = ch.to_digit(10)? as u64;
            if d == 0 {
                return None;
            }
            let platz = pow10((bs_txt.len() - 1 - pos) as u32);
            Some(format!("{}*{}", d, platz))
        })
        .collect();
    // Varied phrasings so the model learns the operation, not one template.
    let mix = std::env::var("SHIRO_MULT_MIX").is_ok();
    let v: u64 = if mix { rng.below(4) } else { 0 };
    let zerlegung = match v {
        0 => format!(" 1. Zerlege {} in {}.", b, stellen.join(" + ")),
        1 => format!(" 1. {} = {}.", b, stellen.join(" + ")),
        2 => format!(" 1. Aufteilen: {} = {}.", b, stellen.join(" + ")),
        _ => format!(" 1. {} ist {}.", b, stellen.join(" + ")),
    };
    if std::env::var("SHIRO_MULT_STELLEN").is_ok() {
        let stellen_a = a.to_string().len();
        let stellen_b = b.to_string().len();
        let min_s = stellen_a + stellen_b - 1;
        let max_s = stellen_a + stellen_b;
        r.extend_from_slice(
            format!(
                " 0. Stellenkontrolle: {} hat {}, {} hat {} Stellen, das Produkt hat {} oder {} Stellen.",
                a, stellen_a, b, stellen_b, min_s, max_s
            )
            .as_bytes(),
        );
    }
    r.extend_from_slice(zerlegung.as_bytes());
    for (k, (faktor, teil, _)) in teile.iter().rev().enumerate() {
        let zeile = match (v + k as u64) % 3 {
            0 => format!(" {}. {}*{}={}.", k + 2, a, faktor, teil),
            1 => format!(" {}. {}*{} ergibt {}.", k + 2, a, faktor, teil),
            _ => format!(" {}. Teil {}*{}={}.", k + 2, a, faktor, teil),
        };
        r.extend_from_slice(zeile.as_bytes());
    }
    let teile_zahlen: Vec<u64> = teile.iter().map(|(_, t, _)| *t).collect();
    if teile_zahlen.len() >= 2 {
        // Sequential accumulator keeps only one running total instead of several
        // numbers that all have to be carried at once.
        let (trace, sum) = if std::env::var("SHIRO_MULT_SEQ").is_ok() {
            add_seq_trace(&teile_zahlen)
        } else {
            add_many_trace(&teile_zahlen)
        };
        r.extend_from_slice(trace.as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", sum).as_bytes());
    } else {
        r.extend_from_slice(format!(" R {}. ", produkt).as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", produkt).as_bytes());
    }
    r.push(B_END);
    r
}

fn check_mult(path: &str) -> (usize, usize) {
    let bytes = match std::fs::read(path) { Ok(b) => b, Err(_) => return (0, 0) };
    if bytes.len() < 32 { return (0, 0); }
    let mut i = 32usize;
    let mut ok = 0usize;
    let mut bad = 0usize;
    while i + 2 <= bytes.len() {
        let len = u16::from_le_bytes([bytes[i], bytes[i + 1]]) as usize;
        i += 2;
        if i + len > bytes.len() { break; }
        let rec = &bytes[i..i + len];
        i += len;
        for seg in rec.split(|b| *b == B_END) {
            let t0 = seg.iter().position(|b| *b == B_TASK);
            let t1 = seg.iter().position(|b| *b == B_THINK);
            let t2 = seg.iter().position(|b| *b == B_ANSWER);
            let (ts, th, as_) = match (t0, t1, t2) {
                (Some(x), Some(y), Some(z)) => (x + 1, y, z + 1),
                _ => continue,
            };
            let task = String::from_utf8_lossy(&seg[ts..th]).to_string();
            let antwort = String::from_utf8_lossy(&seg[as_..]).to_string();
            let antwort = antwort.trim();
            let teile: Vec<&str> = task.trim().split('*').collect();
            if teile.len() != 2 { bad += 1; continue; }
            let a: u64 = teile[0].trim().parse().unwrap_or(u64::MAX);
            let b: u64 = teile[1].trim().parse().unwrap_or(u64::MAX);
            let soll: u64 = antwort.parse().unwrap_or(u64::MAX - 1);
            if a == u64::MAX || b == u64::MAX || soll == u64::MAX - 1 { bad += 1; continue; }
            if a * b == soll { ok += 1; } else { bad += 1; }
        }
    }
    (ok, bad)
}
fn case_record(a: u64, b: u64, mal: bool) -> Vec<u8> {
    let mut r = Vec::new();
    let ergebnis = if mal { a * b } else { a + b };
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}{}{} ", a, if mal { "*" } else { "+" }, b).as_bytes());
    r.push(B_THINK);
    if mal {
        let bs = b.to_string();
        for (i, ch) in bs.chars().rev().enumerate() {
            let d = ch.to_digit(10).unwrap() as u64;
            if d == 0 { continue; }
            let faktor = d * pow10(i as u32);
            r.extend_from_slice(format!(" {}*{}={}. ", a, faktor, a * faktor).as_bytes());
        }
        r.extend_from_slice(format!("Summe: {}. R {}. ", ergebnis, ergebnis).as_bytes());
    } else {
        let (trace, sum) = add_trace(a, b);
        r.extend_from_slice(trace.as_bytes());
        let _ = sum;
    }
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", ergebnis).as_bytes());
    r.push(B_END);
    r
}
fn sem_record(rng: &mut Rng) -> Vec<u8> {
    let a = rng.range(1, 9) as u64;
    let b = rng.range(1, 9) as u64;
    let mut summe = String::new();
    for i in 0..b {
        if i > 0 {
            summe.push('+');
        }
        summe.push_str(&a.to_string());
    }
    let produkt = a * b;
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}*{} ", a, b).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(" 1. {}*{} heisst {} {} mal addieren. 2. {}={}. R {}. ",
        a, b, a, b, summe, produkt, produkt).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", produkt).as_bytes());
    r.push(B_END);
    r
}
fn add_many_trace(parts: &[u64]) -> (String, u64) {
    let summe: u64 = parts.iter().sum();
    let mut steps: Vec<String> = Vec::new();
    let mut carry = 0u64;
    let mut i = 0u32;
    while i < 19 {
        let mut ziffern: Vec<u64> = Vec::new();
        let mut rest = 0u64;
        for p in parts {
            let d = (p / pow10(i)) % 10;
            ziffern.push(d);
            rest += p / pow10(i);
        }
        if i > 0 && rest == 0 && carry == 0 {
            break;
        }
        let s: u64 = ziffern.iter().sum::<u64>() + carry;
        let digit = s % 10;
        let cout = s / 10;
        let lhs = if carry == 1 {
            let mut t: Vec<String> = ziffern.iter().map(|d| d.to_string()).collect();
            t.push("1".to_string());
            t.join("+")
        } else {
            ziffern.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("+")
        };
        // Compact form: no prose, short tokens. Same content, fewer bytes, so the way fits
        // the usable context of a mixed model more easily.
        if std::env::var("SHIRO_MULT_KOMPAKT").is_ok() {
            steps.push(format!("{}:{}+..={}>{}u{};", COLS[(i as usize) % COLS.len()], ziffern.len(), s, digit, cout));
            carry = cout;
            i += 1;
            continue;
        }
        // Default keeps the carry at the end of the column; "uebertrag" names the incoming
        // carry at the start and repeats it as its own sentence, so a carry cannot pass
        // unnoticed from one column to the next.
        if std::env::var("SHIRO_MULT_UEBERTRAG").is_ok() {
            steps.push(format!(
                "{} ue{} {}={} -> {}. Ue{}. ",
                COLS[(i as usize) % COLS.len()], carry, lhs, s, digit, cout
            ));
        } else {
            steps.push(format!("{} {}={} -> {} u{}", COLS[(i as usize) % COLS.len()], lhs, s, digit, cout));
        }
        carry = cout;
        i += 1;
    }
    let mut t = String::from(" ");
    t.push_str(&steps.join(" ; "));
    t.push_str(&format!(" ; R {} ", summe));
    (t, summe)
}
const SACH_WAREN: [(&str, &str); 8] = [
    ("Aepfel", "Stueck"), ("Birnen", "Stueck"), ("Stifte", "Stueck"), ("Hefte", "Stueck"),
    ("Kisten", "Kisten"), ("Flaeschen", "Flaeschen"), ("Pakete", "Pakete"), ("Tische", "Tische"),
];

fn sach2_record(rng: &mut Rng) -> Vec<u8> {
    let (ware, einheit) = SACH_WAREN[rng.below(SACH_WAREN.len() as u64) as usize];
    let anzahl = rng.range(2, 40) as u64;
    let pro = rng.range(2, 25) as u64;
    let op = rng.below(2);
    let (name_a, name_b, name_c) = ("anzahl", "pro", "gesamt");
    let wert;
    let formel;
    let rechnung;
    let aufgabe;
    if op == 0 {
        wert = anzahl * pro;
        formel = format!("{} = {} * {}", name_c, name_a, name_b);
        rechnung = format!("{} * {} = {}", anzahl, pro, wert);
        aufgabe = format!(
            " Es gibt {} {} mit je {} {}. Wie viele {} sind es insgesamt?",
            anzahl, ware, pro, einheit, einheit
        );
    } else {
        wert = anzahl + pro;
        formel = format!("{} = {} + {}", name_c, name_a, name_b);
        rechnung = format!("{} + {} = {}", anzahl, pro, wert);
        aufgabe = format!(
            " Es gibt {} {}. Dazu kommen {} {}. Wie viele {} sind es zusammen?",
            anzahl, ware, pro, ware, einheit
        );
    }
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(aufgabe.as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(
        " GEGEBEN: {} = {}, {} = {}. GESETZT: {}. WEG: {}. ",
        name_a, anzahl, name_b, pro, formel, rechnung
    ).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", wert).as_bytes());
    r.push(B_END);
    r
}
fn calc_section(a: u64, b: u64, mal: bool) -> String {
    if mal {
        let teile: Vec<(u64, u64)> = b
            .to_string()
            .chars()
            .rev()
            .enumerate()
            .filter_map(|(i, ch)| {
                let d = ch.to_digit(10)? as u64;
                if d == 0 { return None; }
                let f = d * pow10(i as u32);
                Some((f, a * f))
            })
            .collect();
        let mut s = String::from(" BERECHNUNG:");
        for (f, t) in &teile {
            s.push_str(&format!(" {}*{}={},", a, f, t));
        }
        if teile.len() >= 2 {
            let summe: u64 = teile.iter().map(|(_, t)| *t).sum();
            let glieder: Vec<String> = teile.iter().map(|(_, t)| t.to_string()).collect();
            s.push_str(&format!(" {}={}.", glieder.join("+"), summe));
        } else {
            let t = teile.first().map(|(_, t)| *t).unwrap_or(0);
            s.push_str(&format!(" Ergebnis {}.", t));
        }
        s
    } else {
        format!(" BERECHNUNG: {}+{}={}.", a, b, a + b)
    }
}

fn sach3_record(rng: &mut Rng) -> Vec<u8> {
    let (ware, einheit) = SACH_WAREN[rng.below(SACH_WAREN.len() as u64) as usize];
    let name = ware.to_lowercase();
    let anzahl = rng.range(2, 40) as u64;
    let pro = rng.range(2, 25) as u64;
    let op = rng.below(2);
    let (aufgabe, formel, wert, mal) = if op == 0 {
        (
            format!(" Es gibt {} {} mit je {} {}. Wie viele {} sind es insgesamt?", anzahl, ware, pro, einheit, einheit),
            format!("gesamt = {} * pro", name),
            anzahl * pro,
            true,
        )
    } else {
        (
            format!(" Es gibt {} {}. Dazu kommen {} {}. Wie viele {} sind es zusammen?", anzahl, ware, pro, ware, einheit),
            format!("gesamt = {} + pro", name),
            anzahl + pro,
            false,
        )
    };
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(aufgabe.as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(
        " GEGEBEN: {} = {}, pro = {}. GESETZT: {}.{}",
        name, anzahl, pro, formel, calc_section(anzahl, pro, mal)
    ).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", wert).as_bytes());
    r.push(B_END);
    r
}
fn formel_record(rng: &mut Rng) -> Vec<u8> {
    let art = rng.below(4);
    // Zwei Schreibweisen: sprechende Namen oder kurze Symbole. Die Bindung
    // (Symbol = Wert) ist der Schritt, der bei Formeln zaehlt.
    let sym = rng.below(2) == 0;
    let mut r = Vec::new();
    r.push(B_TASK);
    let (frage, gegeben, gesetzt, rechnung, wert): (String, String, String, String, u64) = match art {
        0 => {
            let l = rng.range(2, 30) as u64;
            let b = rng.range(2, 30) as u64;
            (format!(" Ein Rechteck ist {} cm lang und {} cm breit. Wie gross ist die Flaeche?", l, b),
             if sym { format!(" l = {}, b = {}", l, b) } else { format!(" laenge = {}, breite = {}", l, b) },
             if sym { " A = l * b".to_string() } else { " A = laenge * breite".to_string() },
             format!(" {} * {} = {}", l, b, l * b), l * b)
        }
        1 => {
            let a = rng.range(2, 25) as u64;
            (format!(" Ein Quadrat hat die Seitenlaenge {} cm. Wie gross ist die Flaeche?", a),
             format!(" seite = {}", a),
             " A = seite * seite".to_string(),
             format!(" {} * {} = {}", a, a, a * a), a * a)
        }
        2 => {
            let l = rng.range(2, 25) as u64;
            let b = rng.range(2, 25) as u64;
            (format!(" Ein Rechteck ist {} cm lang und {} cm breit. Wie lang ist der Umfang?", l, b),
             if sym { format!(" l = {}, b = {}", l, b) } else { format!(" laenge = {}, breite = {}", l, b) },
             if sym { " U = 2*l + 2*b".to_string() } else { " U = 2*laenge + 2*breite".to_string() },
             format!(" 2*{}={} ; 2*{}={} ; {}+{}={}", l, 2 * l, b, 2 * b, 2 * l, 2 * b, 2 * l + 2 * b),
             2 * l + 2 * b)
        }
        _ => {
            let n = rng.range(2, 40) as u64;
            let p = rng.range(2, 15) as u64;
            (format!(" {} Stueck kosten je {} Euro. Was kosten sie zusammen?", n, p),
             format!(" anzahl = {}, preis = {}", n, p),
             " P = anzahl * preis".to_string(),
             format!(" {} * {} = {}", n, p, n * p), n * p)
        }
    };
    r.extend_from_slice(frage.as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(" GEGEBEN:{}. GESETZT:{}. BERECHNUNG:{}.", gegeben, gesetzt, rechnung).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", wert).as_bytes());
    r.push(B_END);
    r
}
fn add_seq_trace(parts: &[u64]) -> (String, u64) {
    let mut summe = parts[0];
    let mut s = format!(" 1. sum = {}.", summe);
    for (k, p) in parts.iter().enumerate().skip(1) {
        let alt = summe;
        summe += *p;
        s.push_str(&format!(" {}. sum = {}+{} = {}.", k + 1, alt, p, summe));
    }
    (s, summe)
}
// Signed arithmetic: normalize the sign first, then work on magnitudes.
fn signed_record(rng: &mut Rng, stufe: u64) -> Vec<u8> {
    let (a, b, op) = match stufe {
        0 => (rng.range(0, 9) as i64, rng.range(0, 9) as i64, '+'),
        1 => (rng.range(0, 20) as i64, rng.range(0, 20) as i64, '-'),
        2 => (rng.range(0, 20) as i64, -rng.range(1, 20) as i64, '-'),
        3 => (-rng.range(1, 20) as i64, rng.range(0, 20) as i64, '-'),
        4 => (-rng.range(1, 20) as i64, -rng.range(1, 20) as i64, '-'),
        5 => (rng.range(0, 99) as i64, -rng.range(1, 99) as i64, '-'),
        6 => (-rng.range(1, 999) as i64, rng.range(0, 999) as i64, '-'),
        7 => (-rng.range(1, 999) as i64, -rng.range(1, 999) as i64, '-'),
        8 => (rng.range(0, 999) as i64, -rng.range(1, 999) as i64, '+'),
        _ => (-rng.range(1, 99) as i64, rng.range(0, 99) as i64, '+'),
    };
    let ergebnis = if op == '+' { a + b } else { a - b };
    // Normalform: Vorzeichen getrennt, Betraege positiv, eine der vier Regeln.
    let (x, y, mag_op, negativ, regel) = match (a >= 0, op, b >= 0) {
        (true, '+', true) => (a as u64, b as u64, '+', false, "+ + bleibt +"),
        (true, '+', false) => {
            let (x, y) = (a as u64, (-b) as u64);
            (x, y, '-', x < y, "+ - wird -")
        }
        (true, '-', true) => (a as u64, b as u64, '-', (a as u64) < (b as u64), "- + wird -"),
        (true, '-', false) => (a as u64, (-b) as u64, '+', false, "- - wird +"),
        (false, '+', true) => {
            let (x, y) = ((-a) as u64, b as u64);
            (x, y, '-', y < x, "- + wird -")
        }
        (false, '+', false) => ((-a) as u64, (-b) as u64, '+', true, "+ + bleibt +"),
        (false, '-', true) => ((-a) as u64, b as u64, '+', true, "- + wird -"),
        (false, '-', false) => {
            let (x, y) = ((-a) as u64, (-b) as u64);
            (x, y, '-', y < x, "- - wird +")
        }
        _ => (a.unsigned_abs(), b.unsigned_abs(), '+', false, "unbekannt"),
    };
    let betrag = if mag_op == '+' { x + y } else if x >= y { x - y } else { y - x };
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {}{}{} ", a, op, b).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(format!(" 1. Vorzeichen: {}.", regel).as_bytes());
    if mag_op == '+' {
        let (trace, sum) = add_trace(x, y);
        r.extend_from_slice(format!(" 2. Betraege: {}+{}.", x, y).as_bytes());
        r.extend_from_slice(trace.as_bytes());
        let _ = sum;
    } else {
        r.extend_from_slice(format!(" 2. Betraege: {} und {}, {}", x, y,
            if x >= y { format!("{}-{}={}", x, y, x - y) } else { format!("{}-{}={}", y, x, y - x) }).as_bytes());
    }
    let vorzeichen = if betrag == 0 { "" } else if negativ { "-" } else { "" };
    r.extend_from_slice(format!(" 3. Ergebnis: {}{}. ", vorzeichen, betrag).as_bytes());
    let _ = ergebnis;
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {}{} ", vorzeichen, betrag).as_bytes());
    r.push(B_END);
    r
}
const HEXZ: [char; 16] = ['0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F'];

fn to_base(mut n: u64, base: u64) -> String {
    if n == 0 { return "0".to_string(); }
    let mut z: Vec<char> = Vec::new();
    while n > 0 {
        let d = (n % base) as usize;
        z.push(if d < 16 { HEXZ[d] } else { (b'A' + (d as u8 - 10)) as char });
        n /= base;
    }
    z.iter().rev().collect()
}

fn base_record(rng: &mut Rng, richtung: u64, groesse: u64) -> Vec<u8> {
    let max: u64 = match groesse { 0 => 255, 1 => 65535, _ => 4294967295 };
    let n = rng.range(0, max as i64) as u64;
    let base: u64 = if richtung % 3 == 1 { 2 } else { 16 };
    let mut r = Vec::new();
    r.push(B_TASK);
    if richtung < 3 {
        // dezimal -> andere Basis
        let ziel = to_base(n, base);
        r.extend_from_slice(format!(" {} in Basis {} ", n, base).as_bytes());
        r.push(B_THINK);
        let mut rest = n;
        let mut schritte: Vec<String> = Vec::new();
        let mut k = 1;
        if rest == 0 { schritte.push(" 1. 0 bleibt 0.".to_string()); }
        while rest > 0 {
            schritte.push(format!(" {}. {} / {} = {} Rest {}.", k, rest, base, rest / base, rest % base));
            rest /= base;
            k += 1;
        }
        for s in &schritte { r.extend_from_slice(s.as_bytes()); }
        r.extend_from_slice(format!(" Ziffern von unten nach oben: {}. ", ziel).as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", ziel).as_bytes());
    } else {
        // andere Basis -> dezimal
        let quelle = to_base(n, base);
        r.extend_from_slice(format!(" {} aus Basis {} nach dezimal ", quelle, base).as_bytes());
        r.push(B_THINK);
        let mut summe = 0u64;
        for (i, ch) in quelle.chars().enumerate() {
            let d = ch.to_digit(16).unwrap_or(0) as u64;
            let vorher = summe;
            summe = summe * base + d;
            r.extend_from_slice(format!(" {}: {}*{} + {} = {}.", i + 1, vorher, base, d, summe).as_bytes());
        }
        r.extend_from_slice(format!(" Wert {}. ", n).as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", n).as_bytes());
    }
    r.push(B_END);
    r
}
// A chain of assignments where each step uses earlier variables.
fn chain_record(rng: &mut Rng, max_steps: u64) -> Vec<u8> {
    let namen = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p"];
    let mut werte: Vec<i64> = Vec::new();
    let mut zeilen: Vec<String> = Vec::new();
    let erste = rng.range(1, 20) as i64;
    werte.push(erste);
    zeilen.push(format!("{} = {}", namen[0], erste));
    let schritte = rng.range(2, max_steps as i64) as usize;
    for i in 1..=schritte {
        // Distinct operands, no multiplication with large factors, no zero results.
        let mut links = rng.below(i as u64) as usize;
        let mut rechts = rng.below(i as u64) as usize;
        let mut versuche = 0;
        while (rechts == links || (werte[links] == 0 || werte[rechts] == 0)) && versuche < 20 {
            links = rng.below(i as u64) as usize;
            rechts = rng.below(i as u64) as usize;
            versuche += 1;
        }
        // First step has only one variable, so it must be a self-operation.
        // Optional restriction: SHIRO_KETTEN_OPS="01" keeps chains additive, so a chain never
        // mixes in multiplication. Useful to separate "chain" from "composition".
        let erlaubte_ops: Vec<u64> = std::env::var("SHIRO_KETTEN_OPS")
            .unwrap_or_else(|_| "012".into())
            .chars()
            .filter_map(|c| c.to_digit(10).map(|d| d as u64))
            .collect();
        let ops = if erlaubte_ops.is_empty() { vec![0, 1, 2] } else { erlaubte_ops };
        let mut op = if i == 1 {
            0
        } else {
            ops[rng.below(ops.len() as u64) as usize]
        };
        if op == 2 && (werte[links].abs() > 99 || werte[rechts].abs() > 99) {
            op = rng.below(2);
        }
        // A zero result carries no information and collapses the rest of the chain.
        if op == 1 && werte[links] == werte[rechts] {
            op = 0;
        }
        let (name, wert, rechnung) = match op {
            0 => (
                namen[i],
                werte[links] + werte[rechts],
                format!("{} + {} = {}", werte[links], werte[rechts], werte[links] + werte[rechts]),
            ),
            1 => {
                // Label the subtraction in the order that is actually computed.
                let (gross, klein, gn, kn) = if werte[links] >= werte[rechts] {
                    (werte[links], werte[rechts], namen[links], namen[rechts])
                } else {
                    (werte[rechts], werte[links], namen[rechts], namen[links])
                };
                (
                    namen[i],
                    gross - klein,
                    format!("{} - {}", gn, kn),
                )
            }
            _ => (
                namen[i],
                werte[links] * werte[rechts],
                format!("{} * {} = {}", werte[links], werte[rechts], werte[links] * werte[rechts]),
            ),
        };
        let _ = rechnung;
        // Optional: keep every intermediate below 100. The model fails three-digit arithmetic
        // (686-84 -> 646, 105+98 -> 1198), so a chain reaching three digits measures the
        // place-value gap, not chaining. SHIRO_KETTEN_KLEIN=1 keeps the family two-digit.
        if std::env::var("SHIRO_KETTEN_KLEIN").is_ok() && wert.abs() > 99 {
            return Vec::new();
        }
        werte.push(wert);
        let linksname = match op {
            1 => if werte[links] >= werte[rechts] { namen[links] } else { namen[rechts] },
            _ => namen[links],
        };
        let rechtsname = match op {
            1 => if werte[links] >= werte[rechts] { namen[rechts] } else { namen[links] },
            _ => namen[rechts],
        };
        zeilen.push(format!("{}={}{}{}={}", name, linksname,
            ["+", "-", "*"][op as usize], rechtsname, wert));
    }
    // The task must carry the chain's operands, otherwise the answer is not determined.
    // Line 0 is the start value; every later line drops its computed result.
    let mut aufgabe: Vec<String> = Vec::new();
    aufgabe.push(format!("{}={}", namen[0], erste));
    for z in zeilen.iter().skip(1) {
        let ohne = match z.rfind('=') {
            Some(p) => &z[..p],
            None => z.as_str(),
        };
        aufgabe.push(ohne.to_string());
    }
    let mut r = Vec::new();
    r.push(B_TASK);
    // Optional length marker: the model tends to continue past the asked variable, so the task
    // can state how many variables it has. Useful to test whether the length is read at all.
    let laenge = std::env::var("SHIRO_KETTE_LAENGE").is_ok();
    let kopf = if laenge {
        format!(" Kette({}): ", schritte + 1)
    } else {
        " Kette: ".to_string()
    };
    r.extend_from_slice(
        format!("{}{};{}? ", kopf, aufgabe.join(";"), namen[schritte]).as_bytes(),
    );
    r.push(B_THINK);
    // "namen" writes every variable; "wert" writes only the running value, so a long
    // chain asks the model to hold one number instead of thirteen names.
    let weg_form = std::env::var("SHIRO_KETTEN_WEG").unwrap_or_else(|_| "namen".into());
    if weg_form == "wert" {
        for w in werte.iter().take(schritte + 1) {
            r.extend_from_slice(format!("{};", w).as_bytes());
        }
        r.extend_from_slice(format!("={}. ", werte[schritte]).as_bytes());
        r.push(B_ANSWER);
        r.extend_from_slice(format!(" {} ", werte[schritte]).as_bytes());
        r.push(B_END);
        return r;
    }
    for z in &zeilen {
        r.extend_from_slice(format!("{};", z).as_bytes());
    }
    r.extend_from_slice(format!("{}={}. ", namen[schritte], werte[schritte]).as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", werte[schritte]).as_bytes());
    r.push(B_END);
    r
}
const DETEK_NAMEN: [&str; 12] = ["Ovid","Parlo","Jorik","Duro","Lemb","Fenn","Tarq","Silo","Vurn","Kelm","Rask","Hob"];

fn detektiv_record(rng: &mut Rng) -> Vec<u8> {
    // Same three-valued logic, but every logical word is invented.
    let mut namen = DETEK_NAMEN.to_vec();
    for i in (1..namen.len()).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        namen.swap(i, j);
    }
    let a = namen[0];
    let b = namen[1];
    let c = namen[2];
    let fremd = namen[3];
    let art = rng.below(3);
    // Vocabulary switches by run: different words, same structure.
    let vok = std::env::var("SHIRO_DETEK_VOK").unwrap_or_else(|_| "gor/nik".into());
    let (w_alle, w_keiner) = match vok.as_str() {
        "alle/keiner" => ("alle", "keiner"),
        "zor/vim" => ("zor", "vim"),
        "tul/mak" => ("tul", "mak"),
        _ => ("gor", "nik"),
    };
    let (praemissen, subjekt, ziel, antwort, weg) = match art {
        0 => (
            format!("{} {} {}. {} {} {}.", w_alle, a, b, w_alle, b, c),
            a,
            c,
            "ja",
            format!(" 1. Kette: {} in {}, {} in {}. 2. Also {} in {}. ", a, b, b, c, a, c),
        ),
        1 => (
            format!("{} {} {}. {} {} {}.", w_alle, a, b, w_keiner, b, c),
            a,
            c,
            "nein",
            format!(" 1. {} in {}, nichts in {}. 2. Also {} nicht in {}. ", a, b, c, a, c),
        ),
        _ => (
            format!("{} {} {}.", w_alle, a, b),
            fremd,
            b,
            "unbekannt",
            format!(" 1. Bekannt: {} in {}. 2. Ueber {} steht nichts. ", a, b, fremd),
        ),
    };
    // Same structure, several question shapes: the surface must not become the signal.
    let frage = match rng.below(3) {
        0 => format!("Ist {} in {}?", subjekt, ziel),
        1 => format!("Gilt {} in {}?", subjekt, ziel),
        _ => format!("Folgt {} in {}?", subjekt, ziel),
    };
    let legende = match rng.below(2) {
        0 => format!("Legende: {} = alle, {} = keiner.", w_alle, w_keiner),
        _ => format!("({} = alle, {} = keiner)", w_alle, w_keiner),
    };
    let mut r = Vec::new();
    r.push(B_TASK);
    r.extend_from_slice(format!(" {} {} {} ", legende, praemissen, frage).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(weg.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", antwort).as_bytes());
    r.push(B_END);
    r
}
// Longer chains with mixed quantifiers. The answer follows from where the first
// exclusion sits, not from a guess.
fn chain2_record(rng: &mut Rng) -> Vec<u8> {
    let mut namen = DETEK_NAMEN.to_vec();
    for i in (1..namen.len()).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        namen.swap(i, j);
    }
    let n = rng.range(2, 4) as usize; // number of links
    // Category controls where the exclusion sits, so a score can be attributed.
    let kat = std::env::var("SHIRO_KETTEN_KAT").unwrap_or_else(|_| "zufall".into());
    let vok = std::env::var("SHIRO_DETEK_VOK").unwrap_or_else(|_| "tul/mak".into());
    let (w_alle, w_keiner) = match vok.as_str() {
        "alle/keiner" => ("alle", "keiner"),
        "zor/vim" => ("zor", "vim"),
        "gor/nik" => ("gor", "nik"),
        _ => ("tul", "mak"),
    };
    let k = rng.range(2, n as i64) as usize;
    let mak_pos: Vec<usize> = match kat.as_str() {
        "start" => vec![0],
        "vorziel" => vec![k - 1],
        "mitte" if k >= 4 => vec![rng.range(1, (k - 2) as i64) as usize],
        "mehrere" => {
            let a = rng.range(1, (k - 1) as i64) as usize;
            let b = if a + 1 < k { a + 1 } else { a };
            vec![a, b]
        }
        "positiv" => vec![],
        _ => {
            // random: at most one exclusion, never at the first link unless drawn so
            let mut v = Vec::new();
            for i in 0..n {
                if rng.below(100) < 30 && i > 0 {
                    v.push(i);
                    break;
                }
            }
            v
        }
    };
    let mut links: Vec<(String, bool)> = Vec::new();
    for i in 0..n {
        let keiner = mak_pos.contains(&i);
        links.push((format!("{} {} {}", if keiner { w_keiner } else { w_alle }, namen[i], namen[i + 1]), keiner));
    }
    // Work out the answer from the chain. A break gets its own line, so the exclusion
    // is a step in the trace instead of a word inside the conclusion.
    let (antwort, weg) = if links[0].1 {
        ("unbekannt", format!(" 1. BRUCH bei {}: Kette beginnt mit {}. 2. Ueber {} in {} folgt nichts. ", namen[0], w_keiner, namen[0], namen[k]))
    } else {
        let mut ergebnis = "ja";
        let mut stelle = 0usize;
        for j in 1..k {
            if links[j].1 {
                stelle = j + 1;
                ergebnis = if j + 1 == k { "nein" } else { "unbekannt" };
                break;
            }
        }
        let mut w = String::from(" 1. ");
        for i in 0..k {
            w.push_str(&format!("{} in {}. ", namen[i], namen[i + 1]));
        }
        if stelle == 0 {
            w.push_str(&format!("2. Kein BRUCH. Also {} in {}. ", namen[0], namen[k]));
        } else if ergebnis == "nein" {
            w.push_str(&format!("2. BRUCH bei {}: also {} nicht in {}. ", namen[stelle], namen[0], namen[k]));
        } else {
            w.push_str(&format!("2. BRUCH bei {}: ueber {} folgt nichts. ", namen[stelle], namen[k]));
        }
        (ergebnis, w)
    };
    let mut r = Vec::new();
    r.push(B_TASK);
    let praemissen: Vec<String> = links.iter().map(|(t, _)| t.clone()).collect();
    r.extend_from_slice(format!(" Legende: {} = alle, {} = keiner. {}. Folgt {} in {}? ",
        w_alle, w_keiner, praemissen.join(". "), namen[0], namen[k]).as_bytes());
    r.push(B_THINK);
    r.extend_from_slice(weg.as_bytes());
    r.push(B_ANSWER);
    r.extend_from_slice(format!(" {} ", antwort).as_bytes());
    r.push(B_END);
    r
}
fn dump(path: &str, k: usize) {
    let data = fs::read(path).expect("shard");
    let n = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let mut off = 32usize;
    let mut shown = 0usize;
    let step = (n as usize / k.max(1)).max(1);
    for i in 0..n as usize {
        let len = u16::from_le_bytes(data[off..off + 2].try_into().unwrap()) as usize;
        off += 2;
        let rec = &data[off..off + len];
        off += len;
        if i % step == 0 && shown < k {
            let s: String = rec.iter().map(|b| match b { 0xFE => '|', 0xFD => '#', 0xFC => '=', 0xFF => '$', 0xFB => '^', 0x20..=0x7E => *b as char, _ => '.' }).collect();
            println!("[{i}] {}", s.trim());
            shown += 1;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 3 && args[1] == "--dump" {
        dump(&args[2], args[3].parse().unwrap());
        return;
    }
    if args.len() > 4 && args[1] == "--world" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args[4].parse().unwrap();
        let mut rng = Rng(seed);
        let force: Option<u64> = args.get(5).and_then(|s| s.parse().ok());
        let recs: Vec<Vec<u8>> = (0..n).map(|_| build_world(&mut rng, 7, force)).collect();
        write_shard(&out, &recs, seed, "category_chain");
        println!("world: {} -> {}", n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--para" {
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args[4].parse().unwrap_or(1);
        paraphrase_file(&args[2], &out, seed);
        return;
    }
    if args.len() > 2 && args[1] == "--check" {
        check_corpus(&args[2]);
        return;
    }
    if args.len() > 5 && args[1] == "--packboth" {
        let wmin: u32 = args[2].parse().unwrap();
        let wmax: u32 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_both_record(&mut rng, pack_budget(), wmin, wmax)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechnen");
        println!("packboth w{}-{}: {} Saetze, {} B -> {}", wmin, wmax, n, avg, out.display());
        return;
    }
    if args.len() > 2 && args[1] == "--checkplain" {
        let (ok, bad) = check_plain(&args[2]);
        println!("{}: {} gueltig, {} ungueltig", args[2], ok, bad);
        return;
    }
    if args.len() > 4 && args[1] == "--kettenpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                let mut r = chain2_record(&mut rng);
                // Optional task label: it separates two otherwise identical task shapes, so the
                // model can tell them apart by the marker instead of guessing from the pattern.
                if let Ok(lab) = std::env::var("SHIRO_KETTEN_LABEL") {
                    if let Some(pos) = r.iter().position(|&b| b == B_TASK) {
                        let mut neu = Vec::with_capacity(r.len() + lab.len() + 2);
                        neu.extend_from_slice(&r[..=pos]);
                        neu.extend_from_slice(format!(" {} ", lab).as_bytes());
                        neu.extend_from_slice(&r[pos + 1..]);
                        r = neu;
                    }
                }
                if !rec.is_empty() && rec.len() + r.len() > budget { break; }
                rec.extend_from_slice(&r);
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "ketten");
        println!("kettenpack: {} Saetze -> {}", n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--detektivpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        // Reserve the test split first, otherwise a small task space leaves nothing held out.
        let split = std::env::var("SHIRO_DETEK_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("detektiv", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("detektiv", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("detektiv", "test")) {
            seen.insert(h);
        }
        let vorher = seen.len();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            let mut versuche = 0usize;
            loop {
                versuche += 1;
                if versuche > 5000 {
                    break;
                }
                let r = detektiv_record(&mut rng);
                let h = hash64(&r);
                if seen.contains(&h) {
                    continue;
                }
                seen.insert(h);
                if !rec.is_empty() && rec.len() + r.len() > budget {
                    break;
                }
                rec.extend_from_slice(&r);
            }
            if rec.is_empty() {
                break;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        let neu = seen.len() - vorher;
        write_shard(&out, &recs, seed, "detektiv");
        println!("detektivpack: {} Saetze, {} neue Aufgaben -> {}", recs.len(), neu, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--sprachpack" {
        // Sentence roles from case and word order. Closed vocabulary, no vocabulary learning:
        // the model must decide WHO acts, from the article (der/den) or, when case is
        // ambiguous, from the position of the noun relative to the verb.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_SPRACH_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("sprach", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("sprach", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("sprach", "test")) {
            seen.insert(h);
        }
        // (noun, genus): m = masculine, f = feminine, n = neuter
        // Foreign vocabulary: nouns that never appear in training. If the model answers these,
        // it learned the case rule; if not, it memorised the training words.
        let fremd = std::env::var("SHIRO_SPRACH_FREMD").is_ok();
        // Third vocabulary: never trained, used only to test whether a model that has SEEN 36
        // nouns can handle a 37th. This is the generalisation test that the 24-noun family
        // failed at 0 percent.
        let dritt = std::env::var("SHIRO_SPRACH_DRITT").is_ok();
        // Misch: train on the 24 known plus the 12 foreign nouns (36 total).
        let misch = std::env::var("SHIRO_SPRACH_MISCH").is_ok();
        let nomen_dritt: [(&str, char); 12] = [
            ("Vater", 'm'),
            ("Bruder", 'm'),
            ("Hahn", 'm'),
            ("Hirsch", 'm'),
            ("Mutter", 'f'),
            ("Schwester", 'f'),
            ("Rose", 'f'),
            ("Taube", 'f'),
            ("Tier", 'n'),
            ("Haus", 'n'),
            ("Bild", 'n'),
            ("Rad", 'n'),
        ];
        let nomen_fremd: [(&str, char); 12] = [
            ("Opa", 'm'),
            ("Onkel", 'm'),
            ("Tiger", 'm'),
            ("Loewe", 'm'),
            ("Oma", 'f'),
            ("Tante", 'f'),
            ("Blume", 'f'),
            ("Wolke", 'f'),
            ("Baby", 'n'),
            ("Auto", 'n'),
            ("Fenster", 'n'),
            ("Buch", 'n'),
        ];
        let nomen: [(&str, char); 24] = [
            ("Hund", 'm'),
            ("Mann", 'm'),
            ("Junge", 'm'),
            ("Rabe", 'm'),
            ("Lehrer", 'm'),
            ("Arzt", 'm'),
            ("Wolf", 'm'),
            ("Fuchs", 'm'),
            ("Katze", 'f'),
            ("Frau", 'f'),
            ("Maus", 'f'),
            ("Ente", 'f'),
            ("Eule", 'f'),
            ("Gans", 'f'),
            ("Kuh", 'f'),
            ("Ziege", 'f'),
            ("Kind", 'n'),
            ("Pferd", 'n'),
            ("Schaf", 'n'),
            ("Lamm", 'n'),
            ("Kalb", 'n'),
            ("Huhn", 'n'),
            ("Schwein", 'n'),
            ("Reh", 'n'),
        ];
        let verben = ["packt", "sieht", "kennt", "sucht", "ruft", "findet"];
        // Definite articles: nominative and accusative. Masculine is the only genus where they
        // differ - that is exactly the cue the task is about.
        let nom_art = |g: char| match g {
            'm' => "der",
            'f' => "die",
            _ => "das",
        };
        let akk_art = |g: char| match g {
            'm' => "den",
            'f' => "die",
            _ => "das",
        };
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 200 {
            versuche += 1;
            // Lexicon mode: every noun comes with its grammatical entry in the prompt, the way
            // a dictionary lookup would. That is the vocabulary hypothesis: structure is given,
            // the model only has to use it - on words it has never seen.
            let lexikon = std::env::var("SHIRO_SPRACH_LEXIKON").is_ok();
            let (si, oi, subj, sg, obj, og) = if dritt {
                let a = rng.range(0, (nomen_dritt.len() - 1) as i64) as usize;
                let mut b = rng.range(0, (nomen_dritt.len() - 1) as i64) as usize;
                if b == a {
                    b = (a + 1) % nomen_dritt.len();
                }
                (a, b, nomen_dritt[a].0, nomen_dritt[a].1, nomen_dritt[b].0, nomen_dritt[b].1)
            } else if misch {
                // 24 known + 12 foreign = 36 nouns
                let pool: Vec<(&str, char)> = nomen.iter().cloned()
                    .chain(nomen_fremd.iter().cloned()).collect();
                let a = rng.range(0, (pool.len() - 1) as i64) as usize;
                let mut b = rng.range(0, (pool.len() - 1) as i64) as usize;
                if b == a {
                    b = (a + 1) % pool.len();
                }
                (a, b, pool[a].0, pool[a].1, pool[b].0, pool[b].1)
            } else if fremd {
                let a = rng.range(0, (nomen_fremd.len() - 1) as i64) as usize;
                let mut b = rng.range(0, (nomen_fremd.len() - 1) as i64) as usize;
                if b == a {
                    b = (a + 1) % nomen_fremd.len();
                }
                (a, b, nomen_fremd[a].0, nomen_fremd[a].1, nomen_fremd[b].0, nomen_fremd[b].1)
            } else {
                let a = rng.range(0, (nomen.len() - 1) as i64) as usize;
                let mut b = rng.range(0, (nomen.len() - 1) as i64) as usize;
                if b == a {
                    b = (a + 1) % nomen.len();
                }
                (a, b, nomen[a].0, nomen[a].1, nomen[b].0, nomen[b].1)
            };
            let _ = (si, oi);
            let verb = verben[rng.range(0, (verben.len() - 1) as i64) as usize];
            // Without a masculine noun nothing marks the case ("die Katze" is nominative and
            // accusative), so an inverted sentence would be genuinely ambiguous - both readings
            // are grammatical. Only the masculine article forces an unambiguous reading; where
            // it is absent the German base order (subject first) is the only defensible form.
            let eindeutig = sg == 'm' || og == 'm';
            let hinten = if eindeutig { rng.range(0, 1) == 1 } else { false };
            let (satz, grund) = if !hinten {
                (
                    format!("{} {} {} {} {}", nom_art(sg), subj, verb, akk_art(og), obj),
                    if sg == 'm' || og == 'm' {
                        format!(
                            "Artikel \"{}\" ist Nominativ, \"{}\" ist Akkusativ. Das Subjekt steht im Nominativ",
                            nom_art(sg),
                            akk_art(og)
                        )
                    } else {
                        format!(
                            "\"{}\" ist Nominativ und Akkusativ zugleich. Kein Artikel entscheidet, also gilt die Grundstellung mit dem Subjekt vorn",
                            nom_art(sg)
                        )
                    },
                )
            } else {
                (
                    format!("{} {} {} {} {}", akk_art(og), obj, verb, nom_art(sg), subj),
                    if sg == 'm' || og == 'm' {
                        format!(
                            "Artikel \"{}\" ist Akkusativ, \"{}\" ist Nominativ. Das Subjekt steht im Nominativ",
                            akk_art(og),
                            nom_art(sg)
                        )
                    } else {
                        format!(
                            "\"{}\" ist Nominativ und Akkusativ zugleich. Die Stellung entscheidet: das Subjekt steht hinten",
                            nom_art(sg)
                        )
                    },
                )
            };
            let h = hash64(format!("{}|{}", satz, subj).as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            if std::env::var("SHIRO_SPRACH_LEXIKON").is_ok() {
                // Dictionary entry in the prompt: structure is given, the model has to use it.
                let eintrag = |wort: &str, genus: char| -> String {
                    let nom = match genus { 'm' => "der", 'f' => "die", _ => "das" };
                    let akk = match genus { 'm' => "den", 'f' => "die", _ => "das" };
                    format!("{} = Substantiv, {}; Nominativ \"{} {}\", Akkusativ \"{} {}\". ",
                        wort,
                        match genus { 'm' => "maskulin", 'f' => "feminin", _ => "neutrum" },
                        nom, wort, akk, wort)
                };
                rec.extend_from_slice(format!(" Lexikon: {}{} ", eintrag(subj, sg), eintrag(obj, og)).as_bytes());
            }
            rec.extend_from_slice(format!(" Satz: {}. Wer {}? ", satz, verb).as_bytes());
            // Answer first: the decision is the centre, the reasoning is wrapped around it.
            // With the answer before the trace the model cannot "compute its way" to it.
            if std::env::var("SHIRO_SPRACH_ANTWORT_ZUERST").is_ok() {
                rec.push(B_ANSWER);
                rec.extend_from_slice(format!(" {} ", subj).as_bytes());
                rec.push(B_THINK);
                rec.extend_from_slice(format!("{}. Das Subjekt ist {}. ", grund, subj).as_bytes());
            } else {
                rec.push(B_THINK);
                rec.extend_from_slice(format!("{}. Das Subjekt ist {}. ", grund, subj).as_bytes());
                rec.push(B_ANSWER);
                rec.extend_from_slice(format!(" {} ", subj).as_bytes());
            }
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "sprach");
        println!("sprachpack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }    if args.len() > 4 && args[1] == "--progpack" {
        // Program execution: assignments in sequence, then the value of one variable.
        // The task line carries the INSTRUCTIONS symbolically; the trace shows the same steps
        // with numbers. Both are built from one list, so they cannot drift apart.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_PROG_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("prog", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("prog", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("prog", "test")) {
            seen.insert(h);
        }
        let namen = ["x", "y", "z"];
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 400 {
            versuche += 1;
            let start: [i64; 3] = [
                rng.range(1, 9) as i64,
                rng.range(1, 9) as i64,
                rng.range(1, 9) as i64,
            ];
            let mut werte = start;
            let schritte = rng.range(2, 4) as usize;
            // (ziel, quelle a, quelle b, op) - one list feeds both the task and the trace.
            let mut plan: Vec<(usize, usize, usize, u8)> = Vec::new();
            let mut ziel = rng.range(0, 2) as usize;
            let mut abbruch = false;
            for _ in 0..schritte {
                let op = match rng.range(0, 2) {
                    0 => b'+',
                    1 => b'-',
                    _ => b'*',
                };
                let a = rng.range(0, 2) as usize;
                // Two different sources, so `z - z` cannot make the step a no-op.
                let mut b = rng.range(0, 2) as usize;
                if b == a {
                    b = (a + 1) % 3;
                }
                let (q1, q2) = (werte[a], werte[b]);
                let neu = match op {
                    b'+' => q1 + q2,
                    b'-' => q1 - q2,
                    _ => q1 * q2,
                };
                if neu.abs() > 999 {
                    abbruch = true;
                    break;
                }
                werte[ziel] = neu;
                plan.push((ziel, a, b, op));
                ziel = (ziel + 1) % 3;
            }
            if abbruch || plan.is_empty() {
                continue;
            }
            // The queried variable must be one the program actually assigns, and its value must
            // differ from the start - otherwise the answer is a copy from the task line.
            let ziele: Vec<usize> = plan.iter().map(|s| s.0).collect();
            let abfrage = ziele[rng.range(0, (ziele.len() - 1) as i64) as usize];
            if werte[abfrage] == start[abfrage] {
                continue;
            }
            let startzeilen: Vec<String> = namen
                .iter()
                .zip(start.iter())
                .map(|(nm, w)| format!("{} = {}", nm, w))
                .collect();
            let programm: Vec<String> = plan
                .iter()
                .map(|(z, a, b, op)| {
                    format!("{} = {} {} {}", namen[*z], namen[*a], *op as char, namen[*b])
                })
                .collect();
            // The trace runs the same plan with the numbers substituted at each point.
            let mut laufend = start;
            let mut spur: Vec<String> = startzeilen.clone();
            for (z, a, b, op) in &plan {
                let (q1, q2) = (laufend[*a], laufend[*b]);
                let ergebnis = match op {
                    b'+' => q1 + q2,
                    b'-' => q1 - q2,
                    _ => q1 * q2,
                };
                spur.push(format!(
                    "{} = {} {} {} = {}",
                    namen[*z], q1, *op as char, q2, ergebnis
                ));
                laufend[*z] = ergebnis;
            }
            debug_assert_eq!(laufend[abfrage], werte[abfrage]);
            let h = hash64(format!("{:?}{:?}", startzeilen, programm).as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            rec.extend_from_slice(
                format!(
                    " Programm: {}; {}. {}? ",
                    startzeilen.join("; "),
                    programm.join("; "),
                    namen[abfrage]
                )
                .as_bytes(),
            );
            rec.push(B_THINK);
            rec.extend_from_slice(format!("{}. ", spur.join("; ")).as_bytes());
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", werte[abfrage]).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "prog");
        println!("progpack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }    if args.len() > 4 && args[1] == "--bruchpack" {
        // Fractions with a common denominator: every line names its operands, so the
        // reference stays visible (unlike a bare number list).
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_BRUCH_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("bruch", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("bruch", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("bruch", "test")) {
            seen.insert(h);
        }
        fn ggt(mut a: i64, mut b: i64) -> i64 {
            while b != 0 {
                let r = a % b;
                a = b;
                b = r;
            }
            a.abs()
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 400 {
            versuche += 1;
            // Same denominator on request: no extension step at all, and a wider space so the
            // family has enough examples.
            let gleich = std::env::var("SHIRO_BRUCH_GLEICH").is_ok();
            let b_d = if gleich { rng.range(2, 19) as i64 } else { rng.range(2, 9) as i64 };
            let d_d = if gleich { b_d } else { rng.range(2, 9) as i64 };
            let a_z = rng.range(1, (b_d - 1) as i64) as i64;
            let c_z = rng.range(1, (d_d - 1) as i64) as i64;
            let nenner = b_d * d_d / ggt(b_d, d_d);
            let lz = a_z * (nenner / b_d);
            let rz = c_z * (nenner / d_d);
            let num = lz + rz;
            let den = nenner;
            if num == den || num > 99 || den > 99 {
                continue;
            }
            let h = hash64(format!("{}/{}+{}/{}", a_z, b_d, c_z, d_d).as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            rec.extend_from_slice(format!(" {}/{} + {}/{} = ? ", a_z, b_d, c_z, d_d).as_bytes());
            rec.push(B_THINK);
            rec.extend_from_slice(
                format!(
                    "Nenner {}; {}/{} + {}/{} = {}/{}. ",
                    nenner, lz, nenner, rz, nenner, num, den
                )
                .as_bytes(),
            );
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {}/{} ", num, den).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "bruch");
        println!("bruchpack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--fibpack" {
        // A sequence built only from additions: fully decomposable, unlike division.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_FIB_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("fib", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("fib", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("fib", "test")) {
            seen.insert(h);
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 200 {
            versuche += 1;
            let a = rng.range(1, 15) as i64;
            let b = rng.range(1, 15) as i64;
            let k = rng.range(5, 14) as usize;
            let mut werte = vec![a, b];
            for i in 2..k {
                let naechste = werte[i - 1] + werte[i - 2];
                werte.push(naechste);
            }
            let h = hash64(format!("{};{};{}", a, b, k).as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let liste: Vec<String> = werte.iter().map(|w| w.to_string()).collect();
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            rec.extend_from_slice(
                format!(" Fibo ab {},{}: das {}. Glied? ", a, b, k).as_bytes(),
            );
            rec.push(B_THINK);
            rec.extend_from_slice(format!("{}. ", liste.join(",")).as_bytes());
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", werte[k - 1]).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "fib");
        println!("fibpack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--restpack" {
        // Integer division with remainder: the building block Euclid needs and no other
        // family trains.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 200 {
            versuche += 1;
            let b = rng.range(3, 9) as i64;
            let q = rng.range(2, 9) as i64;
            let r = rng.range(0, (b - 1) as i64) as i64;
            let a = q * b + r;
            if a > 999 {
                continue;
            }
            let suche = std::env::var("SHIRO_REST_SUCHE").is_ok();
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            rec.extend_from_slice(format!(" {} : {} = ? Rest ? ", a, b).as_bytes());
            rec.push(B_THINK);
            if suche {
                // The quotient is not guessed: every candidate is written out, the first one
                // that overshoots stops the search. Each step is a multiplication and a compare.
                // The decision is the check, not the enumeration: the product below fits, the
                // next one overshoots. Two comparisons instead of up to nine.
                rec.extend_from_slice(
                    format!(
                        "{}*{}={} <= {}, {}*{}={} > {} -> q = {}; {} - {} = {}. Rest {}. ",
                        b,
                        q,
                        b * q,
                        a,
                        b,
                        q + 1,
                        b * (q + 1),
                        a,
                        q,
                        a,
                        b * q,
                        r,
                        r
                    )
                    .as_bytes(),
                );
            } else {
                rec.extend_from_slice(
                    format!(
                        "{}*{} = {}; {} - {} = {}. q = {}, r = {}. ",
                        q, b, q * b, a, q * b, r, q, r
                    )
                    .as_bytes(),
                );
            }
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} Rest {} ", q, r).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "rest");
        println!("restpack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--ggtpack" {
        // Euclid's algorithm: a real procedure with a checkable path, not a lookup.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_GGT_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("ggt", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("ggt", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("ggt", "test")) {
            seen.insert(h);
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 400 {
            versuche += 1;
            let a = rng.range(12, 99) as i64;
            let b = rng.range(10, 98) as i64;
            if a == b {
                continue;
            }
            let (mut x, mut y) = if a >= b { (a, b) } else { (b, a) };
            let mut zeilen: Vec<String> = Vec::new();
            let mut schritte = 0;
            while y != 0 && schritte < 12 {
                let q = x / y;
                let r = x % y;
                zeilen.push(format!("{} = {}*{} + {}", x, q, y, r));
                x = y;
                y = r;
                schritte += 1;
            }
            let ggt = x;
            // A trivial answer (1) or a one-step path carries little; keep the interesting ones.
            if schritte < 2 || ggt == 1 {
                continue;
            }
            let h = hash64(format!("{};{}", a, b).as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            rec.extend_from_slice(format!(" ggT({},{}) = ? ", a, b).as_bytes());
            rec.push(B_THINK);
            rec.extend_from_slice(format!("{}. ggT = {}. ", zeilen.join("; "), ggt).as_bytes());
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", ggt).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "ggt");
        println!("ggtpack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }
    if args.len() > 3 && args[1] == "--wegextract" {
        let src = PathBuf::from(&args[2]);
        let out = PathBuf::from(&args[3]);
        let n: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(10);
        std::fs::create_dir_all(&out).ok();
        let bytes = std::fs::read(&src).expect("shard lesbar");
        // The shard is a header followed by records; find record frames by the marker bytes.
        let mut idx = 0usize;
        let mut geschrieben = 0usize;
        let mut i = 0usize;
        while i < bytes.len() && geschrieben < n {
            if bytes[i] == B_TASK {
                // A record runs from B_TASK to the next B_TASK (or the end).
                let mut ende = i + 1;
                while ende < bytes.len() && bytes[ende] != B_TASK {
                    ende += 1;
                }
                let rec = &bytes[i..ende];
                if let (Some(t), Some(a)) = (
                    rec.iter().position(|&b| b == B_THINK),
                    rec.iter().position(|&b| b == B_ANSWER),
                ) {
                    // Order-independent: the answer may precede the trace (Middle-Out variant).
                    let erster = t.min(a);
                    let prompt = &rec[..=erster];
                    let end_pos = rec.iter().position(|&b| b == B_END).unwrap_or(rec.len());
                    let teil = |von: usize, andere: usize| -> &[u8] {
                        let mut e = end_pos;
                        if andere > von && andere < e { e = andere; }
                        &rec[von + 1..e]
                    };
                    let weg = teil(t, a);
                    let ant = teil(a, t);
                    std::fs::write(out.join(format!("p{idx}.txt")), prompt).ok();
                    std::fs::write(out.join(format!("w{idx}.txt")), weg).ok();
                    std::fs::write(out.join(format!("a{idx}.txt")), ant).ok();
                    idx += 1;
                    geschrieben += 1;
                }
                i = ende;
                continue;
            }
            i += 1;
        }
        println!("wegextract: {} Aufgaben -> {}", geschrieben, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--combopack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let buget = pack_budget();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                // Four shapes that combine two families: arithmetic inside an equation,
                // a chain with a coefficient, a two-stage equation, and a word problem.
                // Shapes can be restricted, so a combination can be held out of training.
                let erlaubt: Vec<u64> = std::env::var("SHIRO_KOMBO_FORMEN")
                    .unwrap_or_else(|_| "0,1,2,3,4".into())
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                if erlaubt.is_empty() {
                    break;
                }
                let shape = erlaubt[rng.below(erlaubt.len() as u64) as usize];
                let (aufgabe, weg, ergebnis) = match shape {
                    0 => {
                        let b = rng.range(2, 9) as u64;
                        let c = rng.range(2, 9) as u64;
                        let a = rng.range(2, 9) as u64;
                        let bc = b * c;
                        if bc % a != 0 {
                            continue;
                        }
                        let x = bc / a;
                        (
                            format!("{}x = {}*{}", a, b, c),
                            format!(" 1. {}*{} = {}. 2. {}x = {}. 3. x = {} / {} = {}. ", b, c, bc, a, bc, bc, a, x),
                            x,
                        )
                    }
                    1 => {
                        let v = rng.range(2, 20) as u64;
                        let k = rng.range(2, 6) as u64;
                        let d = rng.range(1, 15) as u64;
                        (
                            format!("a={}; b={}a+{}; b?", v, k, d),
                            format!(" 1. {}a = {}. 2. {}+{} = {}. ", k, k * v, k * v, d, k * v + d),
                            k * v + d,
                        )
                    }
                    2 => {
                        let x = rng.range(2, 12) as u64;
                        let a = rng.range(2, 9) as u64;
                        let d = rng.range(1, 20) as u64;
                        let m = rng.range(2, 5) as u64;
                        (
                            format!("{}x + {} = {}; {}x?", a, d, a * x + d, m),
                            format!(
                                " 1. {}x = {} - {} = {}. 2. x = {} / {} = {}. 3. {}x = {}. ",
                                a, a * x + d, d, a * x, a * x, a, x, m, m * x
                            ),
                            m * x,
                        )
                    }
                    3 => {
                        let b = rng.range(10, 40) as u64;
                        let k = rng.range(2, 12) as u64;
                        (
                            format!("Anna ist {} Jahre aelter als Berta. Berta ist {}. Wie alt Anna?", k, b),
                            format!(" 1. Berta ist {}. 2. +{} = {}. ", b, k, b + k),
                            b + k,
                        )
                    }
                    4 => {
                        let b = rng.range(2, 9) as u64;
                        let c = rng.range(2, 9) as u64;
                        let k = rng.range(2, 12) as u64;
                        (
                            format!(
                                "Anna ist {} Jahre aelter als Berta. Berta ist {}*{}. Wie alt Anna?",
                                k, b, c
                            ),
                            format!(" 1. {}*{} = {}. 2. {}+{} = {}. ", b, c, b * c, b * c, k, b * c + k),
                            b * c + k,
                        )
                    }
                    5 => {
                        let v = rng.range(2, 20) as u64;
                        let k = rng.range(2, 6) as u64;
                        let d = rng.range(1, 10) as u64;
                        (
                            format!("a={}; b={}a-{}; c=b+a; c?", v, k, d),
                            format!(" 1. {}a = {}. 2. -{} = {}. 3. +{} = {}. ", k, k * v, d, k * v - d, v, k * v - d + v),
                            k * v - d + v,
                        )
                    }
                    6 => {
                        let x = rng.range(2, 12) as u64;
                        let a = rng.range(2, 9) as u64;
                        let q = rng.range(2, 9) as u64;
                        let lhs = a * x * q;
                        (
                            format!("{}x = {}/{}", a, lhs, q),
                            format!(" 1. {}/{} = {}. 2. {}x = {}. 3. x = {} / {} = {}. ", lhs, q, lhs / q, a, lhs / q, lhs / q, a, x),
                            x,
                        )
                    }
                    7 => {
                        let x = rng.range(2, 12) as u64;
                        let b = rng.range(1, 9) as u64;
                        let k = rng.range(2, 5) as u64;
                        (
                            format!("(x+{})*{} = {}", b, k, (x + b) * k),
                            format!(" 1. x+{} = {} / {} = {}. 2. x = {} - {} = {}. ", b, (x + b) * k, k, x + b, x + b, b, x),
                            x,
                        )
                    }
                    8 => {
                        let half = rng.range(6, 30) as u64;
                        (
                            format!("Anna ist halb so alt wie Berta. Berta ist {}. Wie alt Anna?", half * 2),
                            format!(" 1. Berta ist {}. 2. /2 = {}. ", half * 2, half),
                            half,
                        )
                    }
                    9 => {
                        let y = rng.range(2, 9) as u64;
                        let x = rng.range(2, 12) as u64;
                        let a = rng.range(2, 6) as u64;
                        let c = rng.range(2, 6) as u64;
                        (
                            format!("{}x + {}y = {}; y={}; x?", a, c, a * x + c * y, y),
                            format!(" 1. {}y = {}. 2. {}x = {} - {} = {}. 3. x = {} / {} = {}. ", c, c * y, a, a * x + c * y, c * y, a * x, a * x, a, x),
                            x,
                        )
                    }
                    // Unseen by design: square, double, subtract — three dependent steps.
                    _ => {
                        let v = rng.range(2, 12) as u64;
                        let b = v * v;
                        let grenze = (2 * b - 1).min(20) as i64;
                        if grenze < 2 {
                            continue;
                        }
                        let d = rng.range(2, grenze) as u64;
                        let c = 2 * b - d;
                        (
                            format!("a={}; b=a*a; c=2b-{}; c?", v, d),
                            format!(" 1. a*a = {}. 2. 2*{} = {}. 3. -{} = {}. ", b, b, 2 * b, d, c),
                            c,
                        )
                    }
                };
                let mut r: Vec<u8> = Vec::new();
                r.push(B_TASK);
                r.extend_from_slice(format!(" {} ", aufgabe).as_bytes());
                r.push(B_THINK);
                r.extend_from_slice(weg.as_bytes());
                r.push(B_ANSWER);
                r.extend_from_slice(format!(" {} ", ergebnis).as_bytes());
                r.push(B_END);
                if !rec.is_empty() && rec.len() + r.len() > buget {
                    break;
                }
                rec.extend_from_slice(&r);
            }
            if rec.is_empty() {
                break;
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "kombo");
        println!("combopack: {} Saetze -> {}", recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--regelpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let buget = pack_budget();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                // Two shapes nobody was trained on, with the rule stated in the task.
                let b = rng.range(2, 9) as u64;
                let x = rng.range(2, 12) as u64;
                let (aufgabe, regel, ergebnis) = if rng.below(2) == 0 {
                    let a = b * x;
                    (format!("{}/x = {}", a, b), "a/x=b -> x=a/b", format!("{}", a / b))
                } else {
                    let a = x / b.max(1);
                    let _ = a;
                    (format!("x/{} = {}", b, b * x / b), "x/a=b -> x=a*b", format!("{}", b * x / b * b))
                };
                let mut r: Vec<u8> = Vec::new();
                r.push(B_TASK);
                r.extend_from_slice(format!(" Regel: {}. Aufgabe: {}. ", regel, aufgabe).as_bytes());
                r.push(B_THINK);
                r.extend_from_slice(
                    format!(" 1. Regel {}. 2. Also {}. ", regel, ergebnis).as_bytes(),
                );
                r.push(B_ANSWER);
                r.extend_from_slice(format!(" {} ", ergebnis).as_bytes());
                r.push(B_END);
                if !rec.is_empty() && rec.len() + r.len() > buget {
                    break;
                }
                rec.extend_from_slice(&r);
            }
            if rec.is_empty() {
                break;
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "regel");
        println!("regelpack: {} Saetze -> {}", recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--merkpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let fuell_max: usize = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(200);
        let mut rng = Rng(seed);
        let buget = pack_budget();
        let woerter = ["blau", "gruen", "rot", "gelb", "leise", "schnell", "oben", "unten", "hell", "dunkel"];
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                let v = rng.range(1000, 9999) as u64;
                // Filler length grows with the record so the model never sees a fixed distance.
                let ziel = rng.below(fuell_max as u64) as usize;
                let mut filler = String::new();
                while filler.len() < ziel {
                    filler.push_str(woerter[rng.below(woerter.len() as u64) as usize]);
                    filler.push(' ');
                }
                let mut r: Vec<u8> = Vec::new();
                r.push(B_TASK);
                r.extend_from_slice(format!(" Merkzahl {}. {}Frage: Merkzahl? ", v, filler).as_bytes());
                r.push(B_THINK);
                r.extend_from_slice(format!(" 1. Merkzahl {} steht am Anfang. 2. Also {}. ", v, v).as_bytes());
                r.push(B_ANSWER);
                r.extend_from_slice(format!(" {} ", v).as_bytes());
                r.push(B_END);
                if !rec.is_empty() && rec.len() + r.len() > buget {
                    break;
                }
                rec.extend_from_slice(&r);
            }
            if rec.is_empty() {
                break;
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "merken");
        println!("merkpack: {} Saetze, Fueller bis {} Bytes -> {}", recs.len(), fuell_max, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--gleichpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let buget = pack_budget();
        let raum = std::env::var("SHIRO_GLEICH_RAUM").unwrap_or_else(|_| "gleich".into());
        let p = seen_path(&raum, "train");
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path(&raum, "test")) {
            seen.insert(h);
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            let mut versuche = 0usize;
            loop {
                versuche += 1;
                if versuche > 8000 {
                    break;
                }
                // Five shapes of a linear equation, each solved for an integer x.
                let variante = std::env::var("SHIRO_GLEICH_VARIANTE")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(rng.below(5));
                let x = rng.range(1, 12) as u64;
                let a = rng.range(2, 9) as u64;
                let (aufgabe, weg) = match variante {
                    // a*x + b = c
                    0 => {
                        let b = rng.range(1, 30) as u64;
                        let c = a * x + b;
                        (
                            format!(" {}x + {} = {} ", a, b, c),
                            format!(
                                " 1. {}x + {} = {}. 2. {}x = {} - {} = {}. 3. x = {} / {} = {}. ",
                                a, b, c, a, c, b, a * x, a * x, a, x
                            ),
                        )
                    }
                    // a*x - b = c
                    1 => {
                        let c = rng.range(1, 40) as u64;
                        let b = a * x - c;
                        if b == 0 || a * x < c {
                            continue;
                        }
                        (
                            format!(" {}x - {} = {} ", a, b, c),
                            format!(
                                " 1. {}x - {} = {}. 2. {}x = {} + {} = {}. 3. x = {} / {} = {}. ",
                                a, b, c, a, c, b, a * x, a * x, a, x
                            ),
                        )
                    }
                    // b + a*x = c
                    2 => {
                        let b = rng.range(1, 30) as u64;
                        let c = a * x + b;
                        (
                            format!(" {} + {}x = {} ", b, a, c),
                            format!(
                                " 1. {} + {}x = {}. 2. {}x = {} - {} = {}. 3. x = {} / {} = {}. ",
                                b, a, c, a, c, b, a * x, a * x, a, x
                            ),
                        )
                    }
                    // a*(x + b) = c
                    3 => {
                        let b = rng.range(1, 9) as u64;
                        let c = a * (x + b);
                        (
                            format!(" {}(x + {}) = {} ", a, b, c),
                            format!(
                                " 1. {}(x + {}) = {}. 2. x + {} = {} / {} = {}. 3. x = {} - {} = {}. ",
                                a,
                                b,
                                c,
                                b,
                                c,
                                a,
                                x + b,
                                x + b,
                                b,
                                x
                            ),
                        )
                    }
                    // a*x + b = c*x + d
                    _ => {
                        // k must stay below a, otherwise a - k underflows on u64.
                        if a < 3 {
                            continue;
                        }
                        let k = rng.range(1, (a - 2) as i64) as u64;
                        let cc = a - k;
                        if cc < 2 {
                            continue;
                        }
                        let b = rng.range(0, 20) as u64;
                        // (a - cc) * x = d - b, so d = b + (a - cc) * x
                        let d = b + (a - cc) * x;
                        (
                            format!(" {}x + {} = {}x + {} ", a, b, cc, d),
                            format!(
                                " 1. {}x + {} = {}x + {}. 2. {}x - {}x = {} - {} = {}. 3. {}x = {}. 4. x = {} / {} = {}. ",
                                a, b, cc, d, a, cc, d, b, d - b, a - cc, d - b, d - b, a - cc, x
                            ),
                        )
                    }
                };
                let mut r: Vec<u8> = Vec::new();
                r.push(B_TASK);
                r.extend_from_slice(aufgabe.as_bytes());
                r.push(B_THINK);
                r.extend_from_slice(weg.as_bytes());
                r.push(B_ANSWER);
                r.extend_from_slice(format!(" {} ", x).as_bytes());
                r.push(B_END);
                let h = hash64(&r);
                if seen.contains(&h) {
                    continue;
                }
                seen.insert(h);
                if !rec.is_empty() && rec.len() + r.len() > buget {
                    break;
                }
                rec.extend_from_slice(&r);
            }
            if rec.is_empty() {
                break;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "gleichung");
        println!("gleichpack: {} Saetze -> {}", recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--chainpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let max_steps: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(6);
        let mut rng = Rng(seed);
        let buget = pack_budget();
        // Registry keeps train and test disjoint; reserve the test split first.
        let split = std::env::var("SHIRO_KETTEN_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("kette", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("kette", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("kette", "test")) {
            seen.insert(h);
        }
        let vorher = seen.len();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            let mut versuche = 0usize;
            loop {
                versuche += 1;
                if versuche > 8000 {
                    break;
                }
                let r = chain_record(&mut rng, max_steps);
                let h = hash64(&r);
                if seen.contains(&h) {
                    continue;
                }
                seen.insert(h);
                if !rec.is_empty() && rec.len() + r.len() > buget {
                    break;
                }
                rec.extend_from_slice(&r);
            }
            if rec.is_empty() {
                break;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        let neu = seen.len() - vorher;
        write_shard(&out, &recs, seed, "kette");
        println!("chainpack: {} Saetze, {} neue Ketten, bis {} Schritte -> {}", recs.len(), neu, max_steps, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--basepack" {
        let richtung: u64 = args[2].parse().unwrap();
        let groesse: u64 = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let n = 40usize;
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                let r = base_record(&mut rng, richtung, groesse);
                if !rec.is_empty() && rec.len() + r.len() > budget { break; }
                rec.extend_from_slice(&r);
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "basis");
        println!("basepack r{} g{}: {} Saetze -> {}", richtung, groesse, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--signedpack" {
        let stufe: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                let r = signed_record(&mut rng, stufe);
                if !rec.is_empty() && rec.len() + r.len() > budget { break; }
                rec.extend_from_slice(&r);
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "vorzeichen");
        println!("signedpack stufe{}: {} Saetze -> {}", stufe, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--lexpack" {
        // Same closed-form tasks, but the formula is IN THE PROMPT. The model only has to read
        // it, substitute the values and compute. This isolates "can it use a lookup" from "does
        // it know the formula" - the closed-form family failed at the latter, not the former.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_LEX_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("lex", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("lex", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("lex", "test")) {
            seen.insert(h);
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 200 {
            versuche += 1;
            // Binet needs sqrt(5) and phi - irrational, the model has no operation for it (see 180).
            let art = rng.range(0, 2);
            let (lex, frage, weg, wert): (String, String, String, i64) = match art {
                0 => {
                    // Small numbers only: every intermediate stays two-digit, so the task tests
                    // the formula and not three-digit arithmetic (which the model fails, see 3x3).
                    let klein = std::env::var("SHIRO_LEX_KLEIN").is_ok();
                    let a = if klein { rng.range(1, 20) } else { rng.range(1, 60) };
                    let b = if klein { a + rng.range(2, 20) } else { a + rng.range(5, 180) };
                    let anzahl = b - a + 1;
                    let w = (a + b) * anzahl / 2;
                    (
                        {
                            // Three spellings of the same formula: the model must READ the
                            // entry, not recognise one fixed string.
                            let s = match rng.range(0, 2) {
                                0 => " Reihe(a,b) = (a+b)*(b-a+1)/2 ",
                                1 => " Reihe(a,b) = (a+b)*n/2, n = b-a+1 ",
                                _ => " Reihe(a,b) = (a+b)*anzahl/2 ",
                            };
                            s.to_string()
                        },
                        format!("Summe von {} bis {}. ", a, b),
                        format!(
                            "(a+b)*(b-a+1)/2 mit a={}, b={}: {}+{}={}, anzahl={}, *{}={}, /2={}",
                            a,
                            b,
                            a,
                            b,
                            a + b,
                            anzahl,
                            anzahl,
                            (a + b) * anzahl,
                            w
                        ),
                        w,
                    )
                }
                1 => {
                    let k = if std::env::var("SHIRO_LEX_KLEIN").is_ok() { rng.range(3, 9) } else { rng.range(5, 20) };
                    let w = k * (k + 1) * (2 * k + 1) / 6;
                    (
                        {
                            let s = match rng.range(0, 1) {
                                0 => " Quadrat(n) = n*(n+1)*(2n+1)/6 ",
                                _ => " Quadrat(n) = n*(n+1)*(2*n+1)/6 ",
                            };
                            s.to_string()
                        },
                        format!("Summe der Quadrate 1 bis {}. ", k),
                        format!(
                            "n*(n+1)*(2n+1)/6 mit n={}: {}+1={}, 2n+1={}, *{}*{}={}, /6={}",
                            k,
                            k,
                            k + 1,
                            2 * k + 1,
                            k,
                            k + 1,
                            k * (k + 1) * (2 * k + 1),
                            w
                        ),
                        w,
                    )
                }
                2 => {
                    let k = if std::env::var("SHIRO_LEX_KLEIN").is_ok() { rng.range(3, 10) } else { rng.range(5, 20) };
                    let w = (1i64 << k) - 1;
                    (
                        {
                            let s = match rng.range(0, 1) {
                                0 => " Geometrisch(k) = 2^k - 1 ",
                                _ => " Geometrisch(k) = 2**k - 1 ",
                            };
                            s.to_string()
                        },
                        format!("Summe 1+2+4+...+2^{}. ", k - 1),
                        format!("2^k - 1 mit k={}: 2^{}={}, -1={}", k, k, 1i64 << k, w),
                        w,
                    )
                }
                _ => {
                    let k = rng.range(5, 15);
                    let mut a: i64 = 0;
                    let mut b: i64 = 1;
                    for _ in 0..k {
                        let c = a + b;
                        a = b;
                        b = c;
                    }
                    (
                        " Fib(n) = Binet, F(n) = (phi^n - psi^n)/sqrt(5) ".to_string(),
                        format!("Fibonacci F({}). ", k),
                        format!("Binet mit n={}: F={}", k, a),
                        a,
                    )
                }
            };
            let h = hash64(format!("{}|{}|{}", lex, frage, wert).as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            rec.extend_from_slice(format!(" Merke: {}. {} ", lex.trim(), frage.trim()).as_bytes());
            rec.push(B_THINK);
            rec.extend_from_slice(format!("{}. ", weg).as_bytes());
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", wert).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "lex");
        println!("lexpack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }    if args.len() > 4 && args[1] == "--disjunkt" {
        // Remove test records whose task already appears in a training corpus. The only proof
        // of a clean split is a set comparison of the task texts - the generator seed cannot
        // guarantee it (round 246: 38 percent of the addition test tasks were in the training).
        let test_shard = PathBuf::from(&args[2]);
        let train_dir = PathBuf::from(&args[3]);
        let out = PathBuf::from(&args[4]);
        let mut aufgaben: std::collections::HashSet<u64> = std::collections::HashSet::new();
        let mut shards = 0usize;
        for eintrag in fs::read_dir(&train_dir).expect("Trainingsordner lesbar") {
            let p = eintrag.expect("Eintrag").path();
            if p.extension().and_then(|s| s.to_str()) != Some("bin") {
                continue;
            }
            let bytes = match fs::read(&p) {
                Ok(b) => b,
                Err(_) => continue,
            };
            shards += 1;
            let mut i = 32usize;
            while i + 2 <= bytes.len() {
                let len = u16::from_le_bytes([bytes[i], bytes[i + 1]]) as usize;
                let start = i + 2;
                if start + len > bytes.len() {
                    break;
                }
                let rec = &bytes[start..start + len];
                // rec begins with B_TASK; the task text runs to the NEXT marker.
                let erster = rec
                    .iter()
                    .skip(1)
                    .position(|b| *b == B_TASK || *b == B_THINK || *b == B_ANSWER || *b == B_END)
                    .map(|p| p + 1)
                    .unwrap_or(rec.len());
                aufgaben.insert(hash64(&rec[1..erster]));
                i = start + len;
            }
        }
        let bytes = fs::read(&test_shard).expect("Test-Shard lesbar");
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut verworfen = 0usize;
        let mut i = 32usize;
        while i + 2 <= bytes.len() {
            let len = u16::from_le_bytes([bytes[i], bytes[i + 1]]) as usize;
            let start = i + 2;
            if start + len > bytes.len() {
                break;
            }
            let rec = &bytes[start..start + len];
            let erster = rec
                .iter()
                .skip(1)
                .position(|b| *b == B_TASK || *b == B_THINK || *b == B_ANSWER || *b == B_END)
                .map(|p| p + 1)
                .unwrap_or(rec.len());
            if aufgaben.contains(&hash64(&rec[1..erster])) {
                verworfen += 1;
            } else {
                recs.push(rec.to_vec());
            }
            i = start + len;
        }
        let behalten = recs.len();
        write_shard(&out, &recs, 4242, "disjunkt");
        println!(
            "disjunkt: {} Trainings-Shards, {} Aufgaben; {} Tests behalten, {} verworfen -> {}",
            shards, aufgaben.len(), behalten, verworfen, out.display()
        );
        return;
    }    if args.len() > 4 && args[1] == "--teilsatz" {
        // Take only the first N records of a shard: tests whether variety alone is the
        // limiting factor, without adding any new task family.
        let quelle = PathBuf::from(&args[2]);
        let out = PathBuf::from(&args[3]);
        let n: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(300);
        let bytes = fs::read(&quelle).expect("Shard lesbar");
        let mut recs: Vec<Vec<u8>> = Vec::new();
        // Shard layout: 32-byte header (BSHD, version, header size, records, 0, pad),
        // then record frames of u16 length + bytes.
        let mut i = 32usize;
        while i + 2 <= bytes.len() && recs.len() < n {
            let len = u16::from_le_bytes([bytes[i], bytes[i + 1]]) as usize;
            let start = i + 2;
            if start + len > bytes.len() {
                break;
            }
            recs.push(bytes[start..start + len].to_vec());
            i = start + len;
        }
        let anzahl = recs.len();
        let groesse: usize = recs.iter().map(|r| r.len()).sum();
        write_shard(&out, &recs, 4242, "teil");
        println!("teilsatz: {} Records, {} Bytes (Durchschnitt {}) -> {}", anzahl, groesse, groesse / anzahl.max(1), out.display());
        return;
    }    if args.len() > 4 && args[1] == "--kopiepack" {
        // Copy only: can the model read digits off the page at all? If this fails, every
        // arithmetic family is measuring the same defect and no label can fix it.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_KOPIE_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("kopie", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("kopie", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("kopie", "test")) {
            seen.insert(h);
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 100 {
            versuche += 1;
            let laenge = 1 + (rng.range(0, 5) as usize);
            let mut ziffern = String::new();
            for _ in 0..laenge {
                ziffern.push(char::from(b'0' + rng.range(1, 9) as u8));
            }
            let h = hash64(ziffern.as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            // label, so this cannot collide with the arithmetic families
            rec.extend_from_slice(format!(" Kopie: {} ", ziffern).as_bytes());
            rec.push(B_THINK);
            rec.extend_from_slice(format!(" Die Zahl hat {} Stellen: {}. ", laenge, ziffern).as_bytes());
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", ziffern).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        // Chain records: short tasks must not become their own tiny batches. Twelve copies
        // make one record of ~670 bytes, matching the reference corpus (695 bytes/record).
        // Without this the batches collapse to a few hundred bytes and training goes unstable
        // (round 225: loss jumping between 0.06 and 5.08).
        let pro: usize = std::env::var("SHIRO_VERKETTEN")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let recs = if pro > 1 {
            let mut kett: Vec<Vec<u8>> = Vec::new();
            for stueck in recs.chunks(pro) {
                let mut r: Vec<u8> = Vec::new();
                for c in stueck {
                    r.extend_from_slice(c);
                }
                kett.push(r);
            }
            kett
        } else {
            recs
        };
        let anzahl = recs.len();
        write_shard(&out, &recs, seed, "kopie");
        println!("kopiepack: {} Aufgaben in {} Records (verkettet x{}) -> {}", anzahl, anzahl, pro, out.display());
        return;
    }    if args.len() > 4 && args[1] == "--urteilpack" {
        // The three answer classes the brief requires: proven / refuted / not derivable.
        // A model that only ever sees solvable tasks learns to invent an answer - this family
        // teaches it to say "not derivable" when the equation has no solution.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_URTEIL_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("urteil", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("urteil", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("urteil", "test")) {
            seen.insert(h);
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 200 {
            versuche += 1;
            let klasse = rng.range(0, 2);
            let a = rng.range(1, 9) as i64;
            let b = rng.range(1, 40) as i64;
            let c = rng.range(1, 9) as i64;
            let d = rng.range(1, 40) as i64;
            let (aufgabe, weg, antwort, vergleich) = match klasse {
                0 => {
                    // proven: ax + b = cx + d has exactly one solution. Pick x first, then
                    // set d = b + (a-c)*x so that this x solves it.
                    let x = rng.range(-9, 9) as i64;
                    let aa = 1 + (a % 5);
                    let cc = 1 + ((c + 2) % 5);
                    if aa == cc {
                        continue;
                    }
                    let bb = rng.range(-20, 20) as i64;
                    let dd = bb + (aa - cc) * x;
                    let links = aa * x + bb;
                    (
                        format!("{}x {} {} = {}x {} {}", aa, if bb < 0 { "-" } else { "+" }, bb.abs(), cc, if dd < 0 { "-" } else { "+" }, dd.abs()),
                        format!("{}x - {}x = {} - ({}), also {}x = {}, x = {}. Probe: {} = {}.", aa, cc, dd, bb, aa - cc, dd - bb, x, links, cc * x + dd),
                        "bewiesen".to_string(),
                        format!("Links minus rechts ergibt {} = {}.", links, cc * x + dd),
                    )
                }
                1 => {
                    // refuted: x cancels out and the remaining statement is false
                    let b2 = b + rng.range(1, 20) as i64;
                    (
                        format!("{}x + {} = {}x + {}", a, b, a, b2),
                        format!("{}x - {}x = {} - {}, also 0 = {}. Falsch fuer jedes x.", a, a, b2, b, b2 - b),
                        "widerlegt".to_string(),
                        format!("Links minus rechts ergibt 0 = {}.", b2 - b),
                    )
                }
                _ => {
                    // not derivable: x cancels out and the statement is true for all x
                    (
                        format!("{}x + {} = {}x + {}", a, b, a, b),
                        format!("{}x - {}x = {} - {}, also 0 = 0. Jedes x erfuellt die Gleichung.", a, a, b, b),
                        "nicht ableitbar".to_string(),
                        "Links minus rechts ergibt 0 = 0.".to_string(),
                    )
                }
            };
            let h = hash64(aufgabe.as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            // The label is what separates this family from the plain equation family: without
            // it the model sees the same equation and does not know whether a number or a
            // verdict is expected (round 219: 2.6 percent destroyed the model).
            // SHIRO_URTEIL_DIFF=1 also hands the comparison to the model instead of asking it
            // to read two digits and subtract them (round 240: that step is the plateau).
            if std::env::var("SHIRO_URTEIL_DIFF").is_ok() {
                // Hand the comparison over: the model gets "0 = 6" instead of having to read
                // two numbers and subtract them.
                rec.extend_from_slice(format!(" Urteil: {} {}", aufgabe, vergleich).as_bytes());
            } else {
                rec.extend_from_slice(format!(" Urteil: {} ", aufgabe).as_bytes());
            }
            rec.push(B_THINK);
            rec.extend_from_slice(format!(" {} ", weg).as_bytes());
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", antwort).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "urteil");
        println!("urteilpack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }    if args.len() > 3 && args[1] == "--defpack" {
        // Dictionary knowledge as a quiz: "What is X?" -> definition, and the inverse
        // "Which word means ...?" -> X. This is the vocabulary idea as training data
        // instead of as a prompt hint.
        let quelle = PathBuf::from(&args[2]);
        let out = PathBuf::from(&args[3]);
        let n: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(20000);
        let split = std::env::var("SHIRO_DEF_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("def", &split);
        let mut seen = seen_load(&p);
        let budget = pack_budget();
        let inhalt = fs::read(&quelle).expect("Definitionsdatei lesbar");
        let text = String::from_utf8_lossy(&inhalt);
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut genutzt = 0usize;
        let mut gesehen = 0usize;
        let mut zu_kurz = 0usize;
        let mut kein_trenner = 0usize;
        let mut def_laenge = 0usize;
        for zeile in text.split('\n') {
            let z = zeile.trim();
            if z.is_empty() {
                continue;
            }
            gesehen += 1;
            if z.len() < 60 {
                zu_kurz += 1;
                continue;
            }
            let (wort, def) = match z.split_once(": ") {
                Some((a, b)) => (a.trim(), b.trim()),
                None => {
                    kein_trenner += 1;
                    continue;
                }
            };
            if wort.is_empty() || def.len() < 40 || def.len() > 360 {
                def_laenge += 1;
                continue;
            }
            if wort.is_empty() || def.len() < 40 || def.len() > 360 {
                continue;
            }
            let h = hash64(z.as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            genutzt += 1;
            // forward: question -> definition
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            rec.extend_from_slice(format!(" Was ist {}? ", wort).as_bytes());
            rec.push(B_THINK);
            rec.extend_from_slice(b" Ich schlage im Wissen nach. ");
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", def).as_bytes());
            rec.push(B_END);
            if rec.len() <= budget {
                recs.push(rec);
            }
            // inverse: definition -> word
            let mut rec2: Vec<u8> = Vec::new();
            rec2.push(B_TASK);
            rec2.extend_from_slice(format!(" Welches Wort ist gemeint: {} ", def).as_bytes());
            rec2.push(B_THINK);
            rec2.extend_from_slice(b" Ich schlage im Wissen nach. ");
            rec2.push(B_ANSWER);
            rec2.extend_from_slice(format!(" {} ", wort).as_bytes());
            rec2.push(B_END);
            if rec2.len() <= budget {
                recs.push(rec2);
            }
            if genutzt >= n {
                break;
            }
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, 4242, "def");
        println!("defpack: {} Woerter, {} Aufgaben -> {} (Zeilen {}, zu kurz {}, ohne Trenner {}, Def-Laenge {})", genutzt, recs.len(), out.display(), gesehen, zu_kurz, kein_trenner, def_laenge);
        return;
    }    if args.len() > 4 && args[1] == "--add3pack" {
        // Three-digit addition with an explicit place-by-place trace. The model fails 23+178;
        // this family tests whether that is a training gap or a capacity limit.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_ADD3_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("add3", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("add3", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("add3", "test")) {
            seen.insert(h);
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 200 {
            versuche += 1;
            // Without a carry every column is independent: the task then tests reading and
            // single-column addition, not carry propagation. If this works while the carry
            // version fails, the problem is the carry - not the reading of the digits.
            let ohne_uebertrag = std::env::var("SHIRO_ADD3_OHNE_UEBERTRAG").is_ok();
            // Exactly one carry column: the prediction of the rule "one carry is fine, two are
            // not" is that this variant scores high while the general one scores low.
            let genau_ein_uebertrag = std::env::var("SHIRO_ADD3_EIN_UEBERTRAG").is_ok();
            let a = rng.range(100, 999) as i64;
            let b = if ohne_uebertrag {
                // pick b so that each of the three columns sums to at most 9
                let da = [(a / 100) % 10, (a / 10) % 10, a % 10];
                let mut bb = 0i64;
                for (i, d) in da.iter().enumerate() {
                    let frei = 9 - d;
                    let z = if frei <= 0 { 0 } else { rng.range(0, frei) };
                    bb += z * 10i64.pow((2 - i) as u32);
                }
                if bb < 100 {
                    continue;
                }
                bb
            } else {
                rng.range(100, 999) as i64
            };
            if genau_ein_uebertrag {
                let trage = (0..3)
                    .filter(|i| {
                        let w = 10i64.pow(*i as u32);
                        (a / w) % 10 + (b / w) % 10 > 9
                    })
                    .count();
                if trage != 1 {
                    continue;
                }
            }
            let h = hash64(format!("{}+{}", a, b).as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            // Place by place from the right, with carries, exactly as the two-digit family does.
            let mut spur = String::new();
            let mut uebertrag = 0i64;
            let stellen = [("E", 1i64), ("Z", 10), ("H", 100), ("T", 1000)];
            let summe = a + b;
            for (name, wert) in stellen {
                let da = (a / wert) % 10;
                let db = (b / wert) % 10;
                if wert == 1000 && da == 0 && db == 0 && uebertrag == 0 {
                    break;
                }
                let s = da + db + uebertrag;
                let ziffer = s % 10;
                let neu_uebertrag = s / 10;
                if uebertrag > 0 {
                    spur.push_str(&format!(
                        " ; {} {}+{}+{}={} -> {} u{}",
                        name, da, db, uebertrag, s, ziffer, neu_uebertrag
                    ));
                } else {
                    spur.push_str(&format!(
                        " ; {} {}+{}={} -> {} u{}",
                        name, da, db, s, ziffer, neu_uebertrag
                    ));
                }
                uebertrag = neu_uebertrag;
            }
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            // The digit count has to be labelled: three addition families share the same trace
            // shape, and without a marker the model overwrites one with another (round 195:
            // two-digit addition fell from 98 to 18 when a three-digit family was added).
            rec.extend_from_slice(format!(" 3-stellig: {}+{} ", a, b).as_bytes());
            rec.push(B_THINK);
            rec.extend_from_slice(format!("{}{} ; R {} = {}", "", spur.trim_start_matches(" ; "), summe, summe).as_bytes());
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", summe).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "add3");
        println!("add3pack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }    if args.len() > 3 && args[1] == "--textpack" {
        // Plain text into training shards: the GPT objective needs text, not task traces.
        // Paragraphs become records; the trainer cuts 1024-byte windows across them.
        let quelle = PathBuf::from(&args[2]);
        let out = PathBuf::from(&args[3]);
        let max_rec: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(900);
        let middle_out = std::env::var("SHIRO_MIDDLE_OUT").is_ok();
        let inhalt = fs::read(&quelle).expect("Textdatei lesbar");
        let text = String::from_utf8_lossy(&inhalt);
        let mut recs: Vec<Vec<u8>> = Vec::new();
                // Fixed-size chunks instead of paragraphs: a record length is u16, and Wikipedia
        // paragraphs are up to 12 KB. Cut on newline boundaries where possible.
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let bytes = text.as_bytes();
        let mut pos = 0usize;
        while pos < bytes.len() {
            let mut ende = (pos + max_rec).min(bytes.len());
            // First move to a UTF-8 boundary, then look for a newline before it: slicing a
            // string at a byte index inside a multi-byte character panics.
            while ende > pos && ende < bytes.len() && (bytes[ende] & 0xC0) == 0x80 {
                ende -= 1;
            }
            if ende < bytes.len() {
                if let Some(nl) = text[pos..ende].rfind('\n') {
                    if nl > max_rec / 2 {
                        ende = pos + nl;
                    }
                }
            }
            let stueck = &bytes[pos..ende];
            if !stueck.is_empty() {
            if !stueck.is_empty() {
                // Middle-out: start at the centre, alternate left/right outwards. The chain
                // rule holds for any permutation, so this only re-orders the same bytes.
                if std::env::var("SHIRO_MIDDLE_OUT").is_ok() {
                    let n = stueck.len();
                    let c = n / 2;
                    let mut perm: Vec<u8> = Vec::with_capacity(n);
                    perm.push(stueck[c]);
                    let mut k = 1usize;
                    while perm.len() < n {
                        if c >= k {
                            perm.push(stueck[c - k]);
                        }
                        if c + k < n {
                            perm.push(stueck[c + k]);
                        }
                        k += 1;
                    }
                    recs.push(perm);
                } else {
                    recs.push(stueck.to_vec());
                }
            }
            }
            pos = ende;
        }let seed: u64 = 12345;
        write_shard(&out, &recs, seed, "text");
        let bytes: usize = recs.iter().map(|r| r.len()).sum();
        println!("textpack: {} Absaetze, {} Bytes -> {}", recs.len(), bytes, out.display());
        return;
    }    if args.len() > 4 && args[1] == "--closepack" {
        // Closed form instead of a loop. The task is not "compute this" but "know the closed
        // form, then plug the one value in". Each trace names the formula first.
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_CLOSE_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("close", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("close", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("close", "test")) {
            seen.insert(h);
        }
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut versuche = 0usize;
        while recs.len() < n && versuche < n * 200 {
            versuche += 1;
            // Binet needs sqrt(5) and phi - irrational, the model has no operation for it (see 180).
            let art = rng.range(0, 2);
            let (frage, weg, wert): (String, String, i64) = match art {
                // Arithmetic series from a to b: (a+b)*(b-a+1)/2. Two free numbers instead of
                // one, so the task space is large enough for training and a reserved test set.
                0 => {
                    let a = rng.range(1, 60);
                    let b = a + rng.range(5, 180);
                    let anzahl = b - a + 1;
                    let w = (a + b) * anzahl / 2;
                    (
                        format!(" Reihe: Summe von {} bis {} in geschlossener Form. ", a, b),
                        format!(
                            "Arithmetische Reihe: (a+b)*anzahl/2. a = {}, b = {}, anzahl = {}: {}+{} = {}, *{} = {}, /2 = {}",
                            a,
                            b,
                            anzahl,
                            a,
                            b,
                            a + b,
                            anzahl,
                            (a + b) * anzahl,
                            w
                        ),
                        w,
                    )
                }
                // Sum of squares: 1+4+...+n^2 = n(n+1)(2n+1)/6
                1 => {
                    let k = if std::env::var("SHIRO_LEX_KLEIN").is_ok() { rng.range(3, 9) } else { rng.range(5, 20) };
                    let w = k * (k + 1) * (2 * k + 1) / 6;
                    (
                        format!(" Quadrat: Summe der Quadrate 1 bis {} in geschlossener Form. ", k),
                        format!(
                            "Formel n*(n+1)*(2n+1)/6. n = {}: {}*{}*{} = {}, /6 = {}",
                            k,
                            k,
                            k + 1,
                            2 * k + 1,
                            k * (k + 1) * (2 * k + 1),
                            w
                        ),
                        w,
                    )
                }
                // Geometric series: 1+2+4+...+2^(k-1) = 2^k - 1
                2 => {
                    let k = if std::env::var("SHIRO_LEX_KLEIN").is_ok() { rng.range(3, 10) } else { rng.range(5, 20) };
                    let w = (1i64 << k) - 1;
                    (
                        format!(" Geometrisch: Summe 1+2+4+...+2^{} in geschlossener Form. ", k - 1),
                        format!(
                            "Geometrische Reihe: 2^k - 1. k = {}: 2^{} = {}, -1 = {}",
                            k,
                            k,
                            1i64 << k,
                            w
                        ),
                        w,
                    )
                }
                // Fibonacci via Binet: only small n, so the value is exact after rounding
                _ => {
                    let k = rng.range(5, 15);
                    let mut a: i64 = 0;
                    let mut b: i64 = 1;
                    for _ in 0..k {
                        let c = a + b;
                        a = b;
                        b = c;
                    }
                    (
                        format!(" Binet: Fibonacci F({}) mit der geschlossenen Form. ", k),
                        format!(
                            "Binet: F(n) = (phi^n - psi^n)/sqrt(5), phi = (1+sqrt5)/2. n = {}: F = {}",
                            k, a
                        ),
                        a,
                    )
                }
            };
            let h = hash64(format!("{}|{}", frage, wert).as_bytes());
            if seen.contains(&h) {
                continue;
            }
            seen.insert(h);
            let mut rec: Vec<u8> = Vec::new();
            rec.push(B_TASK);
            rec.extend_from_slice(frage.as_bytes());
            rec.push(B_THINK);
            rec.extend_from_slice(format!("{}. ", weg).as_bytes());
            rec.push(B_ANSWER);
            rec.extend_from_slice(format!(" {} ", wert).as_bytes());
            rec.push(B_END);
            if rec.len() > budget {
                continue;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "close");
        println!("closepack: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }    if args.len() > 4 && args[1] == "--formelpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                let r = formel_record(&mut rng);
                if !rec.is_empty() && rec.len() + r.len() > budget { break; }
                rec.extend_from_slice(&r);
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "formel");
        println!("formelpack: {} Saetze -> {}", n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--sach3pack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_SACH3_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("sach3", &split);
        let mut seen = seen_load(&p);
        for h in seen_load(&seen_path("sach3", "train")) { seen.insert(h); }
        for h in seen_load(&seen_path("sach3", "test")) { seen.insert(h); }
        let vorher = seen.len();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            let mut versuche = 0usize;
            loop {
                versuche += 1;
                if versuche > 4000 { break; }
                let r = sach3_record(&mut rng);
                let h = hash64(&r);
                if seen.contains(&h) { continue; }
                seen.insert(h);
                if !rec.is_empty() && rec.len() + r.len() > budget { break; }
                rec.extend_from_slice(&r);
            }
            if rec.is_empty() { break; }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        let neu = seen.len() - vorher;
        write_shard(&out, &recs, seed, "sach3");
        println!("sach3pack: {} Saetze, {} neue Aufgaben -> {}", recs.len(), neu, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--sach2pack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let split = std::env::var("SHIRO_SACH2_SPLIT").unwrap_or_else(|_| "train".into());
        let p = seen_path("sach2", &split);
        let mut seen = seen_load(&p);
        // Beide Seiten sperren: der Aufgabenraum ist klein, Zufall allein kollidiert.
        for h in seen_load(&seen_path("sach2", "train")) {
            seen.insert(h);
        }
        for h in seen_load(&seen_path("sach2", "test")) {
            seen.insert(h);
        }
        let vorher = seen.len();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            let mut versuche = 0usize;
            loop {
                versuche += 1;
                if versuche > 4000 {
                    break;
                }
                let r = sach2_record(&mut rng);
                let h = hash64(&r);
                if seen.contains(&h) { continue; }
                seen.insert(h);
                if !rec.is_empty() && rec.len() + r.len() > budget { break; }
                rec.extend_from_slice(&r);
            }
            if rec.is_empty() {
                break;
            }
            recs.push(rec);
        }
        seen_save(&p, &seen);
        let neu = seen.len() - vorher;
        write_shard(&out, &recs, seed, "sach2");
        println!("sach2pack: {} Saetze, {} neue Aufgaben -> {}", n, neu, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--sempack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                let r = sem_record(&mut rng);
                if !rec.is_empty() && rec.len() + r.len() > budget { break; }
                rec.extend_from_slice(&r);
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "bedeutung");
        println!("sempack: {} Saetze -> {}", n, out.display());
        return;
    }
    if args.len() > 2 && args[1] == "--case" {
        let out = PathBuf::from(&args[2]);
        let mut recs: Vec<Vec<u8>> = Vec::new();
        recs.push(case_record(325472435, 53287087, false));
        recs.push(case_record(214, 125, true));
        recs.push(case_record(2345, 43, true));
        recs.push(case_record(214, 5, true));
        recs.push(case_record(234567, 89123, true));
        write_shard(&out, &recs, 1, "fall");
        println!("case: {} Aufgaben -> {}", recs.len(), out.display());
        return;
    }
    if args.len() > 2 && args[1] == "--checkmult" {
        let (ok, bad) = check_mult(&args[2]);
        println!("{}: {} gueltig, {} ungueltig", args[2], ok, bad);
        return;
    }
    if args.len() > 4 && args[1] == "--multpack" {
        let art: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let budget = pack_budget();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        for _ in 0..n {
            let mut rec: Vec<u8> = Vec::new();
            loop {
                let r = mult_record(&mut rng, art);
                if !rec.is_empty() && rec.len() + r.len() > budget { break; }
                rec.extend_from_slice(&r);
            }
            recs.push(rec);
        }
        write_shard(&out, &recs, seed, "malnehmen");
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        println!("multpack art{}: {} Saetze, {} B/Satz -> {}", art, n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--errortest" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("error", "test");
        let mut seen = seen_load(&p);
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut guard = 0;
        while recs.len() < n && guard < n * 200 {
            guard += 1;
            let r = check_record(&mut rng);
            let h = hash64(&r);
            if seen.contains(&h) { continue; }
            seen.insert(h);
            recs.push(r);
        }
        let gesamt = seen.len();
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "pruefen");
        println!("errortest: {} Aufgaben (Registry gesamt {}) -> {}", recs.len(), gesamt, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--probe2" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("probe2", "test");
        let mut seen = seen_load(&p);
        let vorher = seen.len();
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut guard = 0;
        while recs.len() < n && guard < n * 100 {
            guard += 1;
            let r = probe_task(&mut rng);
            let h = hash64(&r);
            if seen.contains(&h) { continue; }
            seen.insert(h);
            recs.push(r);
        }
        let neu = seen.len() - vorher;
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "probe");
        println!("probe2: {} Aufgaben, {} neue -> {}", neu, recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--probepack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("probe", "train");
        let mut seen = seen_load(&p);
        let vorher = seen.len();
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_mixed_probe(&mut rng, pack_budget(), &mut seen)).collect();
        let neu = seen.len() - vorher;
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "probe");
        println!("probepack: {} Saetze, {} neue Aufgaben -> {}", n, neu, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--verbopack" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("arith_verbos", "train");
        let mut seen = seen_load(&p);
        let vorher = seen.len();
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_verbos(&mut rng, pack_budget(), maxw, &mut seen)).collect();
        let neu = seen.len() - vorher;
        seen_save(&p, &seen);
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechnen");
        println!("verbopack w1-{}: {} Saetze, {} neue Paare, {} B/Satz -> {}", maxw, n, neu, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--plainpack" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("arith_plain", "train");
        let mut seen = seen_load(&p);
        let vorher = seen.len();
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_unique_plain(&mut rng, pack_budget(), maxw, &mut seen)).collect();
        let neu = seen.len() - vorher;
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "rechnen");
        println!("plainpack w1-{}: {} Saetze, {} neue Paare -> {}", maxw, n, neu, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--plaintest" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let train = seen_load(&seen_path("arith_plain", "train"));
        let p = seen_path("arith_plain", "test");
        let mut own = seen_load(&p);
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut guard = 0;
        while recs.len() < n && guard < n * 200 {
            guard += 1;
            let (a, b) = draw_pair(&mut rng, maxw);
            let h = pair_hash(a, b);
            if train.contains(&h) || own.contains(&h) {
                continue;
            }
            own.insert(h);
            let sum = a + b;
            let mut r = Vec::new();
            r.push(B_TASK);
            r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
            r.push(B_THINK);
            r.extend_from_slice(b" ");
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", sum).as_bytes());
            r.push(B_END);
            recs.push(r);
        }
        seen_save(&p, &own);
        write_shard(&out, &recs, seed, "rechnen");
        println!("plaintest w1-{}: {} Aufgaben, garantiert nicht im Training -> {}", maxw, recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--uniqpack" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("arith", "train");
        let mut seen = seen_load(&p);
        // Auch die Pruefpaare sperren: sonst koennen spaetere Trainingslaeufe
        // Paare erzeugen, die in einer frueheren Pruefmenge stehen.
        for h in seen_load(&seen_path("arith", "test")) {
            seen.insert(h);
        }
        let vorher = seen.len();
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_unique(&mut rng, pack_budget(), maxw, &mut seen)).collect();
        let neu = seen.len() - vorher;
        seen_save(&p, &seen);
        write_shard(&out, &recs, seed, "rechnen");
        println!("uniqpack w1-{}: {} Saetze, {} neue Paare -> {}", maxw, n, neu, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--uniqtest" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("arith", "train");
        let train = seen_load(&p);
        let pt = seen_path("arith", "test");
        let mut test = seen_load(&pt);
        let mut recs: Vec<Vec<u8>> = Vec::new();
        let mut guard = 0;
        while recs.len() < n && guard < n * 200 {
            guard += 1;
            let (a, b) = draw_pair(&mut rng, maxw);
            let h = pair_hash(a, b);
            if train.contains(&h) || test.contains(&h) {
                continue;
            }
            test.insert(h);
            meta_push("arith_test", a, b);
            let (trace, sum) = add_trace(a, b);
            let mut r = Vec::new();
            r.push(B_TASK);
            r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
            r.push(B_THINK);
            r.extend_from_slice(trace.as_bytes());
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", sum).as_bytes());
            r.push(B_END);
            recs.push(r);
        }
        seen_save(&pt, &test);
        write_shard(&out, &recs, seed, "rechnen");
        println!("uniqtest w1-{}: {} Aufgaben, garantiert nicht im Training -> {}", maxw, recs.len(), out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--steptest" {
        let kind: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let train = seen_load(&seen_path("logic", "train"));
        let p = seen_path("logic", "test");
        let mut own = seen_load(&p);
        let recs = unique_records(&mut rng, n, |r| logic_steps_record(r, Some(kind)), &train, &mut own);
        seen_save(&p, &own);
        println!("  ({} Aufgaben, keine im Training)", recs.len());
        write_shard(&out, &recs, seed, "logik");
        println!("steptest k{}: {} -> {}", kind, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--steppack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("logic", "train");
        let mut seen = seen_load(&p);
        let vorher = seen.len();
        let recs: Vec<Vec<u8>> = (0..n)
            .map(|_| pack_generic(&mut rng, pack_budget(), |r| logic_steps_record(r, None), &mut seen))
            .collect();
        let neu = seen.len() - vorher;
        seen_save(&p, &seen);
        println!("  ({} neue eindeutige Aufgaben)", neu);
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "logik");
        println!("steppack: {} Saetze, {} B -> {}", n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--puzzletest" {
        let art: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let train = seen_load(&seen_path("puzzle", "train"));
        let p = seen_path("puzzle", "test");
        let mut own = seen_load(&p);
        let recs = unique_records(&mut rng, n, |r| puzzle_record(r, Some(art)), &train, &mut own);
        seen_save(&p, &own);
        println!("  ({} Aufgaben, keine im Training)", recs.len());
        write_shard(&out, &recs, seed, "raetsel");
        println!("puzzletest a{}: {} -> {}", art, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--puzzlepack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("puzzle", "train");
        let mut seen = seen_load(&p);
        let vorher = seen.len();
        let recs: Vec<Vec<u8>> = (0..n)
            .map(|_| pack_generic(&mut rng, pack_budget(), |r| puzzle_record(r, None), &mut seen))
            .collect();
        let neu = seen.len() - vorher;
        seen_save(&p, &seen);
        println!("  ({} neue eindeutige Aufgaben)", neu);
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "raetsel");
        println!("puzzlepack: {} Saetze, {} B -> {}", n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--sachtest" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let train = seen_load(&seen_path("sach", "train"));
        let p = seen_path("sach", "test");
        let mut own = seen_load(&p);
        let recs = unique_records(&mut rng, n, sach_record, &train, &mut own);
        seen_save(&p, &own);
        println!("  ({} Aufgaben, keine im Training)", recs.len());
        write_shard(&out, &recs, seed, "sachaufgabe");
        println!("sachtest: {} -> {}", n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--sachpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let p = seen_path("sach", "train");
        let mut seen = seen_load(&p);
        let vorher = seen.len();
        let recs: Vec<Vec<u8>> = (0..n)
            .map(|_| pack_generic(&mut rng, pack_budget(), |r| sach_record(r), &mut seen))
            .collect();
        let neu = seen.len() - vorher;
        seen_save(&p, &seen);
        println!("  ({} neue eindeutige Aufgaben)", neu);
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "sachaufgabe");
        println!("sachpack: {} Saetze, {} B -> {}", n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--postest" {
        let kind: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| logic_pos_record(&mut rng, Some(kind))).collect();
        write_shard(&out, &recs, seed, "logik");
        println!("postest k{}: {} -> {}", kind, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--pospack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_logic_pos(&mut rng, 480)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "logik");
        println!("pospack: {} Saetze, {} B -> {}", n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--scafftest-en" {
        let kind: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| logic_scaffold_en(&mut rng, Some(kind))).collect();
        write_shard(&out, &recs, seed, "logic-en");
        println!("scafftest-en k{}: {} -> {}", kind, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--scaffpack-en" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_scaffold_en(&mut rng, 480)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "logic-en");
        println!("scaffpack-en: {} Saetze, {} B -> {}", n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--logictest-en" {
        let kind: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| logic_record_en(&mut rng, Some(kind))).collect();
        write_shard(&out, &recs, seed, "logic-en");
        println!("logictest-en k{}: {} -> {}", kind, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--logicpack-en" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_logic_en(&mut rng, 480)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "logic-en");
        println!("logicpack-en: {} Saetze, {} B -> {}", n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--logicpara" {
        let kind: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| logic_record(&mut rng, Some(kind), true, true)).collect();
        write_shard(&out, &recs, seed, "logik");
        println!("logicpara k{}: {} -> {}", kind, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--logictest" {
        let kind: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| logic_record(&mut rng, Some(kind), false, false)).collect();
        write_shard(&out, &recs, seed, "logik");
        println!("logictest k{}: {} -> {}", kind, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--logicrand" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_logic_rand(&mut rng, 480)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "logik-rand");
        println!("logicrand: {} Saetze, {} B -> {}", n, avg, out.display());
        return;
    }    if args.len() > 4 && args[1] == "--logicpack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_logic(&mut rng, 480)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "logik");
        println!("logicpack: {} Saetze, {} B -> {}", n, avg, out.display());
        return;
    }
    if args.len() > 5 && args[1] == "--marktest" {
        let maxw: u32 = args[2].parse().unwrap();
        let mode: u64 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| marked_record(&mut rng, maxw, mode)).collect();
        write_shard(&out, &recs, seed, "rechnen");
        println!("marktest w{} m{}: {} -> {}", maxw, mode, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--packmark" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_marked(&mut rng, pack_budget(), maxw)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechnen");
        println!("packmark w1-{}: {} Saetze, {} B -> {}", maxw, n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--packthree" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_full_three(&mut rng, pack_budget(), maxw)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechnen");
        println!("packthree w1-{}: {} Saetze, {} B -> {}", maxw, n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--fullpath" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| {
            let wa = rng.range(1, maxw as i64) as u32;
            let wb = rng.range(1, maxw as i64) as u32;
            let a = rng.range(0, pow10(wa) as i64 - 1) as u64;
            let b = rng.range(0, pow10(wb) as i64 - 1) as u64;
            let (trace, _sum) = add_trace(a, b);
            let mut r = Vec::new();
            r.push(B_TASK);
            r.extend_from_slice(format!(" {}+{} ", a, b).as_bytes());
            r.push(B_THINK);
            r.push(B_ANSWER);
            r.extend_from_slice(format!(" {} ", trace.trim()).as_bytes());
            r.push(B_END);
            r
        }).collect();
        write_shard(&out, &recs, seed, "rechenweg");
        println!("fullpath w1-{}: {} -> {}", maxw, n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--packfullboth" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_full_both(&mut rng, pack_budget(), maxw)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechnen");
        println!("packfullboth w1-{}: {} Saetze, {} B -> {}", maxw, n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--packfull" {
        let maxw: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_full_record(&mut rng, pack_budget(), maxw)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechnen");
        println!("packfull w1-{}: {} Saetze, {} B -> {}", maxw, n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--edge" {
        let kind: u64 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| edge_record(&mut rng, kind)).collect();
        write_shard(&out, &recs, seed, "rechnen");
        println!("edge {}: {} -> {}", kind, n, out.display());
        return;
    }
    if args.len() > 6 && args[1] == "--carrymix" {
        let wmin: u32 = args[2].parse().unwrap();
        let wmax: u32 = args[3].parse().unwrap();
        let want: u32 = args[4].parse().unwrap();
        let n: usize = args[5].parse().unwrap();
        let out = PathBuf::from(&args[6]);
        let seed: u64 = args.get(7).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| build_carry_mix(&mut rng, wmin, wmax, want)).collect();
        write_shard(&out, &recs, seed, "rechnen");
        println!("carrymix w{}-{} k{}: {} -> {}", wmin, wmax, want, n, out.display());
        return;
    }
    if args.len() > 5 && args[1] == "--carry" {
        let width: u32 = args[2].parse().unwrap();
        let want: u32 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| build_carry(&mut rng, width, want)).collect();
        write_shard(&out, &recs, seed, "rechnen");
        println!("carry w{} k{}: {} -> {}", width, want, n, out.display());
        return;
    }
    if args.len() > 5 && args[1] == "--packpath" {
        let wmin: u32 = args[2].parse().unwrap();
        let wmax: u32 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_path_record(&mut rng, pack_budget(), wmin, wmax)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechenweg");
        println!("packpath w{}-{}: {} Saetze, {} B -> {}", wmin, wmax, n, avg, out.display());
        return;
    }
    if args.len() > 5 && args[1] == "--packadd" {
        let wmin: u32 = args[2].parse().unwrap();
        let wmax: u32 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let amin: i64 = args.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
        let amax: i64 = args.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);
let parity: u64 = args.get(9).and_then(|s| s.parse().ok()).unwrap_or(0);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_add_record(&mut rng, pack_budget(), wmin, wmax, amin, amax, parity)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechnen");
        println!("packadd w{}-{}: {} Saetze, {} B -> {}", wmin, wmax, n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--pack" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| pack_record(&mut rng, 480)).collect();
        let avg: usize = recs.iter().map(|r| r.len()).sum::<usize>() / n.max(1);
        write_shard(&out, &recs, seed, "rechnen");
        println!("pack: {} Saetze, mittlere Laenge {} B -> {}", n, avg, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--mix" {
        let n: usize = args[2].parse().unwrap();
        let out = PathBuf::from(&args[3]);
        let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| mix_record(&mut rng)).collect();
        write_shard(&out, &recs, seed, "rechnen");
        println!("mix: {} -> {}", n, out.display());
        return;
    }
    if args.len() > 4 && args[1] == "--ladder" {
        let level: u32 = args[2].parse().unwrap();
        let n: usize = args[3].parse().unwrap();
        let out = PathBuf::from(&args[4]);
        let seed: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| ladder2_record(&mut rng, level)).collect();
        write_shard(&out, &recs, seed, "rechnen");
        println!("ladder {}: {} -> {}", level, n, out.display());
        return;
    }
    if args.len() > 6 && args[1] == "--addfirst" {
        let wmin: u32 = args[2].parse().unwrap();
        let wmax: u32 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args[6].parse().unwrap();
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| build_add_first(&mut rng, wmin, wmax)).collect();
        write_shard(&out, &recs, seed, "rechenschritt");
        println!("addfirst w{}-{}: {} -> {}", wmin, wmax, n, out.display());
        return;
    }
    if args.len() > 6 && args[1] == "--addstep" {
        let wmin: u32 = args[2].parse().unwrap();
        let wmax: u32 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args[6].parse().unwrap();
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| build_add_step(&mut rng, wmin, wmax)).collect();
        write_shard(&out, &recs, seed, "rechenschritt");
        println!("addstep w{}-{}: {} -> {}", wmin, wmax, n, out.display());
        return;
    }
    if args.len() > 6 && args[1] == "--addpath" {
        let wmin: u32 = args[2].parse().unwrap();
        let wmax: u32 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args[6].parse().unwrap();
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| build_add_path(&mut rng, wmin, wmax)).collect();
        write_shard(&out, &recs, seed, "rechenweg");
        println!("addpath w{}-{}: {} -> {}", wmin, wmax, n, out.display());
        return;
    }
    if args.len() > 6 && args[1] == "--add" {
        let wmin: u32 = args[2].parse().unwrap();
        let wmax: u32 = args[3].parse().unwrap();
        let n: usize = args[4].parse().unwrap();
        let out = PathBuf::from(&args[5]);
        let seed: u64 = args[6].parse().unwrap();
        let mut rng = Rng(seed);
        let recs: Vec<Vec<u8>> = (0..n).map(|_| build_add(&mut rng, wmin, wmax)).collect();
        write_shard(&out, &recs, seed, "rechnen");
        println!("add w{}-{}: {} records -> {}", wmin, wmax, n, out.display());
        return;
    }
    let mut level = 0u32;
    let mut n = 40usize;
    let mut seed = 1u64;
    let mut out = PathBuf::from(".");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--level" => { level = args[i + 1].parse().unwrap(); i += 2; }
            "--n" => { n = args[i + 1].parse().unwrap(); i += 2; }
            "--seed" => { seed = args[i + 1].parse().unwrap(); i += 2; }
            "--out" => { out = PathBuf::from(&args[i + 1]); i += 2; }
            _ => i += 1,
        }
    }
    let mut rng = Rng(seed);
    let recs: Vec<Vec<u8>> = (0..n)
        .map(|_| {
            let (e, v) = gen_level(level, &mut rng);
            plain_record(&e, v)
        })
        .collect();
    write_shard(&out, &recs, seed, "rechnen");
    println!("level {level}: {n} -> {}", out.display());
}

























































































































































































