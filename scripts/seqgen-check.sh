#!/usr/bin/env bash
# Korpuspruefung der drei Generalisierungs-Korpora der Familie Zahlenfolgen:
#   O-Z6 Invarianz - O-Z8 Oberflaechenform Stufen 1 und 2 - O-Z9 Komposition
#
# Geprueft wird gegen die Doppelrolle der Bytes:
#   1. Manifest vorhanden, Datensatzzahl, Nutzbytes, Laenge und Pruefsumme
#      stimmen mit der Dokumentation ueberein
#   2. Shard-Kopf BSHD, Marken, ASCII, Laengen, FNV-1a-64 des Shards
#   3. JSONL traegt **dieselben** Datensaetze wie der Shard (Byte-Vergleich)
#   4. O-Z6: jede Zusatzzeile ist strukturell als irrelevant nachweisbar, und
#      die Ruecknahme der Zusatzzeilen ergibt **byte-genau** den Basisdatensatz
#      (Pruefsumme gegen `basis_hash` aus dem JSONL)
#   5. O-Z8: Etikett, Zeilenreihenfolge und Trennzeichen werden aus den Bytes
#      erkannt; die Normalform ergibt byte-genau den Basisdatensatz
#   6. O-Z9: die Antwort wird mit einer ZWEITEN, unabhaengigen Rechnung (Python)
#      nachgerechnet; die Wechselstelle muss eindeutig und der Wechsel sichtbar sein
#   7. Determinismus: ein zweiter Erzeugungslauf ist byte-identisch
#
# Reines Pruefskript, es aendert den Baum nicht. Ohne Argument werden die
# Korpora in einen Temp-Ordner erzeugt; mit Argument wird ein vorhandener
# Ordner geprueft.
#
# Aufruf: bash scripts/seqgen-check.sh [ausgabeordner]
set -u

cd "$(dirname "$0")/.." || exit 1
FERTIG="${1:-}"
OK=0
FEHLER=0

ok() {
    OK=$((OK + 1))
    echo "ok   $1"
}
fehler() {
    FEHLER=$((FEHLER + 1))
    echo "FEHL $1"
}

ERZEUGT=0
if [ -z "$FERTIG" ]; then
    FERTIG="$(mktemp -d)"
    ZWEITER="$(mktemp -d)"
    ERZEUGT=1
    echo "Erzeuge die drei Korpora nach $FERTIG und $ZWEITER"
    export PATH="/home/changchi/.cargo/bin:$PATH"
    if [ -d /home/changchi/.cache/shiro-target ]; then
        export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/home/changchi/.cache/shiro-target}"
    fi
    for ziel in "$FERTIG" "$ZWEITER"; do
        if SHIRO_SEQGEN_OUT="$ziel" cargo test --release --test seqgen -- \
            --ignored --exact schreibe_korpora_auf_anforderung >/dev/null 2>&1; then
            ok "Erzeugung nach $ziel"
        else
            fehler "Erzeugung nach $ziel fehlgeschlagen (cargo test)"
            echo
            echo "Zusammenfassung: $OK ok, $FEHLER Fehler"
            exit 1
        fi
    done
    # 7. Determinismus: Shard, JSONL und Manifest muessen byteweise gleich sein.
    for k in invarianz oberflaeche komposition; do
        for datei in train/shard-0000.bin train/manifest.json jsonl/train.jsonl; do
            if cmp -s "$FERTIG/$k/$datei" "$ZWEITER/$k/$datei"; then
                ok "zweiter Lauf byteidentisch: $k/$datei"
            else
                fehler "zweiter Lauf weicht ab: $k/$datei"
            fi
        done
    done
else
    ok "Pruefe vorhandenen Ordner $FERTIG"
fi

python3 - "$FERTIG" <<'PY'
import json
import os
import sys

ROOT = sys.argv[1]

# Die gemessenen Sollwerte - dieselben Zahlen stehen in
# docs/zahlenfolgen-generalisierung.md. Weicht der Korpus ab, ist entweder der
# Generator oder die Dokumentation falsch; beides muss auffallen.
SOLL = {
    "invarianz": {
        "note": "oz6-inv",
        "records": 40,
        "payload": 12267,
        "record_bytes": 12427,
        "maxlen": 455,
        "shard_bytes": 12539,
        "checksum": "83e1b625ddc693df",
    },
    "oberflaeche": {
        "note": "oz8-obe",
        "records": 40,
        "payload": 9998,
        "record_bytes": 10158,
        "maxlen": 367,
        "shard_bytes": 10270,
        "checksum": "0e70f07c2709e62a",
    },
    "komposition": {
        "note": "oz9-kom",
        "records": 40,
        "payload": 13515,
        "record_bytes": 13675,
        "maxlen": 400,
        "shard_bytes": 13787,
        "checksum": "2d39c2d47c43fe2c",
    },
}

DOKU = "docs/zahlenfolgen-generalisierung.md"

# Der geschlossene Regelkatalog der Familie (src/data/seq.rs).
REGELN = [
    "gleichbleibende Differenz",
    "gleichbleibender Faktor",
    "Summe der beiden vorigen Zahlen",
    "gleichmaessig wachsende Differenz",
    "zwei verschraenkte Differenzen",
    "verdoppeln plus gleiche Zahl",
]
# Die Fragewendungen, an denen der Aufgabentyp haengt.
WENDUNGEN = [
    "kommt als Naechstes",
    "Nach welcher Regel",
    "passt nicht",
    "muss geaendert werden",
    "A oder B",
    "Welche Zahl fehlt",
]
# Die Zeilenmarken des Kerns.
MARKER = ["Folge:", "Regel:", "Frage:"]
ETIKETTEN = ["Folge", "Zahlen", "Reihe"]
TRENNER = [", ", ",", " , "]
PRAEFIXE = ["Folge:", "Folge A:", "Folge B:", "Regel:", "Frage:"]

T_DIFF = "gleichbleibende Differenz"
T_FAKTOR = "gleichbleibender Faktor"
T_VERDOPPELN = "verdoppeln plus gleiche Zahl"

ok = 0
fehler = 0


def melde(cond, text):
    global ok, fehler
    if cond:
        ok += 1
        print("ok  ", text)
    else:
        fehler += 1
        print("FEHL", text)


def fnv1a64(data):
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def hex64(data):
    return "%016x" % fnv1a64(data)


def zeilen(text):
    return text.split("\n")


def schluessel(zeile):
    return zeile.split(":", 1)[0].strip() if ":" in zeile else None


def wert(zeile):
    return zeile.split(":", 1)[1].strip()


def ist_relevant(zeile):
    return any(zeile.startswith(p) for p in PRAEFIXE)


def ist_irrelevant(zeile):
    if not zeile.strip():
        return True
    if ist_relevant(zeile):
        return False
    if any(m in zeile for m in MARKER):
        return False
    if "?" in zeile:
        return False
    if any(w in zeile for w in WENDUNGEN):
        return False
    return not any(r in zeile for r in REGELN)


def folge_lesen(text):
    """`1, 2, ?, 6` -> Liste aus int/None. Akzeptiert alle drei Schreibweisen."""
    out = []
    for teil in text.split(","):
        t = teil.strip()
        if t == "?":
            out.append(None)
        elif t.isdigit():
            out.append(int(t))
        else:
            return None
    return out if len(out) >= 3 else None


def folge_rendern(werte, trenner):
    return trenner.join("?" if v is None else str(v) for v in werte)


def trenner_aus(text, werte):
    for t in TRENNER:
        if folge_rendern(werte, t) == text.strip():
            return t
    return None


def rekord(task, rest):
    """Datensatz aus Aufgabentext und Rest (Denkweg + Antwort)."""
    return b"\xfe" + task.encode() + rest


# ---------------------------------------------------------------------------
# O-Z9: zweite, unabhaengige Rechnung - reines Python, kein Rust-Aufruf
# ---------------------------------------------------------------------------

def fortsetzung(regel, werte):
    """Fortsetzung der ersten Regel - None, wenn sie nicht passt."""
    if regel == T_DIFF:
        if len(werte) < 3:
            return None
        d = werte[1] - werte[0]
        if d < 1:
            return None
        if any(b - a != d for a, b in zip(werte, werte[1:])):
            return None
        return werte[-1] + d
    if regel == T_FAKTOR:
        if len(werte) < 3 or werte[0] < 1 or werte[1] % werte[0] != 0:
            return None
        r = werte[1] // werte[0]
        if r < 2:
            return None
        if any(w != werte[0] * r**i for i, w in enumerate(werte)):
            return None
        return werte[-1] * r
    return None


def komp_wert(regel, param, j, anker):
    if regel == T_DIFF:
        w = anker + param * j
    elif regel == T_FAKTOR:
        w = anker * param**j
    elif regel == T_VERDOPPELN:
        two = 2**j
        w = two * anker + param * (two - 1)
    else:
        return None
    return w if 1 <= w <= 20000 else None


def params_aus(regel, v1, v2):
    if regel == T_DIFF:
        d = v2 - v1
        return d if d >= 1 else None
    if regel == T_FAKTOR:
        if v1 < 1 or v2 % v1 != 0:
            return None
        r = v2 // v1
        return r if r >= 2 else None
    if regel == T_VERDOPPELN:
        c = v2 - 2 * v1
        return c if c >= 1 else None
    return None


def komp_loese(erste, zweite, s, typ):
    """Gibt (antwort, grund) zurueck; antwort ist None, wenn nicht eindeutig."""
    n = len(s)
    stellen = []
    for k in range(n):
        if k + 1 < 3 or any(v is None for v in s[: k + 1]):
            continue
        cont = fortsetzung(erste, s[: k + 1])
        if cont is None:
            continue
        bekannt = [(i, v) for i, v in enumerate(s[k + 1 :], k + 1) if v is not None]
        luecken = [i for i, v in enumerate(s[k + 1 :], k + 1) if v is None]
        if len(luecken) > 1 or (luecken and luecken[0] != k + 1):
            continue
        if len(bekannt) < 2:
            continue
        (i1, v1), (i2, v2) = bekannt[0], bekannt[1]
        if i2 != i1 + 1:
            continue
        param = params_aus(zweite, v1, v2)
        if param is None:
            continue
        anker = s[k]
        if any(komp_wert(zweite, param, i - k, anker) != v for i, v in bekannt):
            continue
        hat_luecke = bool(luecken)
        if hat_luecke and komp_wert(zweite, param, 1, anker) is None:
            continue
        stellen.append((k, param, hat_luecke, cont))
    if len(stellen) != 1:
        return None, "wechselstelle nicht eindeutig (%d Kandidaten)" % len(stellen)
    k, param, hat_luecke, cont = stellen[0]
    anker = s[k]
    if cont == komp_wert(zweite, param, 1, anker):
        return None, "unsichtbarer Regelwechsel"
    if typ == "wechsel":
        if hat_luecke:
            return None, "wechsel mit luecke"
        return "Stelle %d" % (k + 2), "ok"
    if typ == "naechste":
        if hat_luecke:
            return None, "naechste mit luecke"
        w = komp_wert(zweite, param, n - k, anker)
        return (None, "naechstes nicht berechenbar") if w is None else (str(w), "ok")
    if typ == "luecke":
        if not hat_luecke:
            return None, "luecke ohne luecke"
        w = komp_wert(zweite, param, 1, anker)
        return (None, "luecke nicht berechenbar") if w is None else (str(w), "ok")
    return None, "unbekannter Aufgabentyp"


# ---------------------------------------------------------------------------
# Pruefung je Korpus
# ---------------------------------------------------------------------------

def rekord_bytes(text):
    if any(ord(c) > 255 for c in text):
        return None
    return bytes(ord(c) for c in text)


def pruefe(korpus):
    soll = SOLL[korpus]
    manifest_pfad = os.path.join(ROOT, korpus, "train", "manifest.json")
    shard_pfad = os.path.join(ROOT, korpus, "train", "shard-0000.bin")
    jsonl_pfad = os.path.join(ROOT, korpus, "jsonl", "train.jsonl")
    melde(os.path.isfile(manifest_pfad), "%s: Manifest vorhanden" % korpus)
    if not os.path.isfile(manifest_pfad):
        return []
    with open(manifest_pfad, encoding="utf-8") as f:
        m = json.load(f)
    melde(
        m["format_version"] == 1 and m["vocab_size"] == 256,
        "%s: Manifest-Kopf (Version 1, 256 Klassen)" % korpus,
    )
    melde(m["seed"] == 1337, "%s: Seed 1337 im Manifest" % korpus)
    eintrag = m["shards"][0]
    melde(eintrag["path"] == "shard-0000.bin", "%s: Shard-Pfad im Manifest" % korpus)
    melde(
        eintrag["records"] == soll["records"],
        "%s: %d Datensaetze (erwartet %d)" % (korpus, eintrag["records"], soll["records"]),
    )
    melde(
        eintrag["payload_bytes"] == soll["payload"],
        "%s: %d B Nutzlast (erwartet %d)" % (korpus, eintrag["payload_bytes"], soll["payload"]),
    )
    melde(
        eintrag["checksum"] == soll["checksum"],
        "%s: Manifest-Pruefsumme %s (erwartet %s)" % (korpus, eintrag["checksum"], soll["checksum"]),
    )

    with open(shard_pfad, "rb") as f:
        roh = f.read()
    melde(roh[:4] == b"BSHD", "%s: Shard-Kopf BSHD" % korpus)
    melde(
        len(roh) == soll["shard_bytes"],
        "%s: Shard %d B (erwartet %d)" % (korpus, len(roh), soll["shard_bytes"]),
    )
    melde(hex64(roh) == soll["checksum"], "%s: FNV-1a-64 des Shards stimmt" % korpus)
    kopf_note = roh[24:32].split(b"\x00")[0].decode()
    melde(kopf_note == soll["note"], "%s: Shard-Notiz '%s'" % (korpus, kopf_note))

    records = []
    pos = 32
    while pos < len(roh):
        ln = roh[pos] | (roh[pos + 1] << 8)
        pos += 2
        records.append(roh[pos : pos + ln])
        pos += ln
    melde(len(records) == soll["records"], "%s: %d Datensaetze im Shard gelesen" % (korpus, len(records)))
    nutz = sum(len(r) for r in records)
    melde(
        nutz == soll["record_bytes"],
        "%s: %d B Datensatzlaengen nachgezaehlt (erwartet %d)" % (korpus, nutz, soll["record_bytes"]),
    )
    maxlen = max(len(r) for r in records)
    melde(
        maxlen == soll["maxlen"],
        "%s: laengster Datensatz %d B (erwartet %d)" % (korpus, maxlen, soll["maxlen"]),
    )
    melde(all(len(r) <= 3500 for r in records), "%s: alle Datensaetze <= RECORD_MAX 3500 B" % korpus)
    melde(all(len(r) < 4096 for r in records), "%s: alle Datensaetze < MAX_RECORD_LEN 4096 B" % korpus)
    melde(
        32 + 2 * len(records) + nutz == len(roh),
        "%s: Dateigroesse = 32 + 2 x n + Datensatzlaengen (%d)" % (korpus, len(roh)),
    )
    melde(
        nutz - sum(4 for _ in records) == soll["payload"],
        "%s: Nutzbytes = Datensatzlaengen minus vier Strukturmarken je Datensatz" % korpus,
    )

    def marken_ok(r):
        return (
            r[0] == 0xFE
            and r[-1] == 0xFF
            and 0xFD in r
            and 0xFC in r
            and all(b < 128 or b >= 0xFA for b in r)
        )

    melde(
        all(marken_ok(r) for r in records),
        "%s: Marken, ASCII und Struktur in jedem Datensatz" % korpus,
    )

    with open(jsonl_pfad, encoding="utf-8") as f:
        jz = [json.loads(z) for z in f if z.strip()]
    melde(len(jz) == len(records), "%s: JSONL-Zeilenzahl = Shard-Datensatzzahl" % korpus)
    gleich = all(
        rekord_bytes(z["text"]) == r and z["id"] == "%s-%03d" % (korpus, i)
        for i, (z, r) in enumerate(zip(jz, records))
    )
    melde(gleich, "%s: JSONL und Shard tragen byteweise dieselben Datensaetze" % korpus)
    melde(all(z["korpus"] == "oz%s.%s" % ({"invarianz": "6", "oberflaeche": "8", "komposition": "9"}[korpus], korpus) for z in jz),
          "%s: JSONL nennt den Korpus" % korpus)
    melde(all(z["antwort"].strip() != "" for z in jz), "%s: jede Antwort ist nicht leer" % korpus)
    melde(all(z["basis_hash"] == "" for z in jz) if korpus == "komposition" else all(len(z["basis_hash"]) == 16 for z in jz),
          "%s: basis_hash wie erwartet gesetzt" % korpus)
    return list(zip(jz, records))


def rest_von(r):
    """Denkweg+Antwort-Teil (ab der DENKWEG-Marke)."""
    return r[r.index(0xFD) :]


def pruefe_invarianz(paare):
    with_zusatz = 0
    nachweisbar = 0
    passt = 0
    for z, r in paare:
        task = r[1 : r.index(0xFD)].decode()
        kern = [x for x in zeilen(task) if ist_relevant(x)]
        zusatz = [x for x in zeilen(task) if not ist_relevant(x)]
        if zusatz:
            with_zusatz += 1
        if zusatz and all(ist_irrelevant(x) for x in zusatz):
            nachweisbar += 1
        if zusatz and hex64(rekord("\n".join(kern), rest_von(r))) == z["basis_hash"]:
            passt += 1
    melde(
        with_zusatz == len(paare),
        "invarianz: jeder Datensatz traegt mindestens eine Zusatzzeile (%d/%d)" % (with_zusatz, len(paare)),
    )
    melde(
        nachweisbar == len(paare),
        "invarianz: jede Zusatzzeile ist strukturell als irrelevant nachweisbar (%d/%d)"
        % (nachweisbar, len(paare)),
    )
    melde(
        passt == len(paare),
        "invarianz: Ruecknahme der Zusatzzeilen ergibt byte-genau den Basisdatensatz (%d/%d)"
        % (passt, len(paare)),
    )
    doppelt = sum(1 for z, _ in paare if z["variante"] == "hinweis-paar")
    melde(doppelt > 0, "invarianz: auch Varianten mit zwei Zusatzzeilen vorhanden (%d)" % doppelt)


def pruefe_oberflaeche(paare):
    etiketten = set()
    trenner = set()
    reihenfolge = set()
    stufen = set()
    passt = 0
    for z, r in paare:
        task = r[1 : r.index(0xFD)].decode()
        folgen = []
        regel = None
        frage = None
        ok_form = True
        for i, x in enumerate(zeilen(task)):
            k = schluessel(x)
            if k is None:
                ok_form = False
                break
            teile = k.split(" ")
            if teile[0] in ETIKETTEN and (len(teile) == 1 or teile[1] in ("A", "B")):
                folgen.append((teile[0], teile[1] if len(teile) > 1 else "", wert(x), i))
            elif k == "Regel":
                regel = x
            elif k == "Frage":
                frage = (x, i)
            else:
                ok_form = False
                break
        if not ok_form or not folgen or frage is None:
            continue
        if len({f[0] for f in folgen}) != 1:
            continue
        t = None
        gut = True
        for _, _, text, _ in folgen:
            werte = folge_lesen(text)
            if werte is None:
                gut = False
                break
            tt = trenner_aus(text, werte)
            if tt is None or (t is not None and tt != t):
                gut = False
                break
            t = tt
        if not gut:
            continue
        stufe = 1 if t == ", " else 2
        if z["stufe"] != stufe:
            continue
        getauscht = frage[1] < min(f[3] for f in folgen)
        etiketten.add(folgen[0][0])
        trenner.add(t)
        reihenfolge.add(getauscht)
        stufen.add(stufe)
        neu = []
        for e, marke, text, _ in folgen:
            werte = folge_lesen(text)
            neu.append("Folge%s: %s" % ((" " + marke) if marke else "", folge_rendern(werte, ", ")))
        if regel:
            neu.append(regel)
        neu.append(frage[0])
        if hex64(rekord("\n".join(neu), rest_von(r))) == z["basis_hash"]:
            passt += 1
    melde(etiketten == set(ETIKETTEN), "oberflaeche: alle drei Etiketten kommen vor %s" % sorted(etiketten))
    melde(trenner == set(TRENNER), "oberflaeche: alle drei Schreibweisen kommen vor %s" % sorted(trenner))
    melde(reihenfolge == {False, True}, "oberflaeche: beide Zeilenreihenfolgen kommen vor")
    melde(stufen == {1, 2}, "oberflaeche: Stufe 1 und Stufe 2 kommen vor")
    melde(
        passt == len(paare),
        "oberflaeche: Normalform ergibt byte-genau den Basisdatensatz (%d/%d)" % (passt, len(paare)),
    )


def pruefe_komposition(paare):
    gut = 0
    typen = set()
    katalog = set()
    wechselstellen = []
    for z, r in paare:
        task = r[1 : r.index(0xFD)].decode()
        f = {}
        for x in zeilen(task):
            f[schluessel(x)] = wert(x)
        regel = f.get("Regel", "")
        if ", dann " not in regel:
            continue
        erste, zweite = regel.split(", dann ", 1)
        if erste == zweite:
            continue
        s = folge_lesen(f.get("Folge", ""))
        if s is None:
            continue
        antwort, grund = komp_loese(erste, zweite, s, z["typ"])
        if antwort is None or antwort != z["antwort"]:
            continue
        gut += 1
        typen.add(z["typ"])
        katalog.add((erste, zweite))
        if z["typ"] == "wechsel":
            wechselstellen.append(int(antwort.split(" ")[1]))
    melde(
        gut == len(paare),
        "komposition: Antwort mit der zweiten Rechnung (Python) nachgerechnet (%d/%d)" % (gut, len(paare)),
    )
    melde(typen == {"naechste", "luecke", "wechsel"}, "komposition: alle drei Aufgabentypen kommen vor")
    melde(len(katalog) == 4, "komposition: alle vier Regelkombinationen kommen vor (%d)" % len(katalog))
    melde(
        all(w >= 4 for w in wechselstellen),
        "komposition: jede Wechselstelle liegt mindestens bei Stelle 4 (%s)" % sorted(set(wechselstellen)),
    )


def pruefe_doku():
    if not os.path.isfile(DOKU):
        melde(False, "Dokumentation fehlt: %s" % DOKU)
        return
    with open(DOKU, encoding="utf-8") as f:
        text = f.read()
    melde(True, "Dokumentation vorhanden: %s" % DOKU)
    for korpus, soll in SOLL.items():
        melde(str(soll["payload"]) in text, "Doku nennt die Nutzbytes von %s (%d)" % (korpus, soll["payload"]))
        melde(soll["checksum"] in text, "Doku nennt die Pruefsumme von %s (%s)" % (korpus, soll["checksum"]))
        melde(str(soll["maxlen"]) in text, "Doku nennt den laengsten Datensatz von %s (%d)" % (korpus, soll["maxlen"]))
        melde(
            str(soll["record_bytes"]) in text,
            "Doku nennt die Summe der Datensatzlaengen von %s (%d)" % (korpus, soll["record_bytes"]),
        )
    melde("Stufe 3" in text, "Doku benennt O-Z8 Stufe 3")


paare_je_korpus = {}
for _korpus in ("invarianz", "oberflaeche", "komposition"):
    paare_je_korpus[_korpus] = pruefe(_korpus)

pruefe_invarianz(paare_je_korpus["invarianz"])
pruefe_oberflaeche(paare_je_korpus["oberflaeche"])
pruefe_komposition(paare_je_korpus["komposition"])
pruefe_doku()

print()
print("Python-Summe: %d ok, %d Fehler" % (ok, fehler))
sys.exit(1 if fehler else 0)
PY
STATUS=$?
if [ "$STATUS" -ne 0 ]; then
    FEHLER=$((FEHLER + 1))
fi

if [ "$ERZEUGT" -eq 1 ]; then
    rm -rf "$FERTIG" "$ZWEITER"
fi

echo
echo "Zusammenfassung: $OK Shell-Pruefungen, Python-Status $STATUS"
if [ "$FEHLER" -eq 0 ] && [ "$STATUS" -eq 0 ]; then
    echo "Ergebnis: alle Pruefungen bestanden"
    exit 0
fi
echo "Ergebnis: $FEHLER Fehler"
exit 1
