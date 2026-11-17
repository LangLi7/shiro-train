#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Voll-Benchmark "Logisches und Analytisches Denken" - Auswertung.

Nur Standardbibliothek. Deterministisch: kein Zeitstempel, keine Zufallszahl,
keine Mengen-Reihenfolge in der Ausgabe.

Aufruf (immer ueber scripts/benchmark-logik.sh):

    benchmark-logik.py plan                 # nur lesen: Abdeckung, Panels, Jobs
    benchmark-logik.py stage  --work DIR    # Panels/Unterkorpora bereitstellen
    benchmark-logik.py baseline --work DIR  # Kurzstrategie-Baselines an den BYTES
    benchmark-logik.py evaluate --work DIR  # Logs auswerten, Bericht schreiben
    benchmark-logik.py selftest             # Parser- und Schwellenprobe

Warum Portionen STATT einer Auswahl im Messlauf: die Binaries waehlen den
Datensatz ueber `stride = len / limit` und einen Versatz (src/agent.rs
`reference_set_ab`). Ein bereitgestellter Korpus mit GENAU der Panel-Zahl
ergibt `stride = 1` und damit genau die Panel-Datensaetze - die Auswahlregel
des Projekts wird also nicht nachgebaut, sondern benutzt.

Projektregeln, die hier kodiert sind (jede mit Quelle):

* Panels: zwei DISJUNKTE Panels zu je 480 Fragen, jedes Mal der `stride`.te
  Datensatz, Panel A Versatz 0 und Panel B Versatz `stride // 2`
  (docs/training-loop.md Abschnitt 10.7, exp-015; src/agent.rs:2743).
  Bei 5000 Held-out-Datensaetzen ist `stride = 10`, der Versatz also 5.
* Determinismus-Probe: Panel A wird ZWEIMAL gemessen; das Ergebnis muss
  byte-identisch sein, sonst ist die Messung UNGUELTIG (exp-013/015).
* Pflichtlinie 90 % (D-047): gepoolte KOPFZAHL >= 90 % UND kein inhaltlicher
  Typ unter 75 %. Ziellinie 98 %.
* Kurzantwort-Typen kommen aus der Capability-Datei
  (configs/capabilities/sequence_reasoning.toml, Schluessel
  `kurzantwort_typen`) - NICHT aus dem Code. Das Werkzeug prueft die dortige
  Annahme am Korpus nach (Antwortraum) und meldet, wenn sie nicht traegt.
* Antwortformen: der Schalter ist `--pack reparatur-delta` bei
  `agent capability run` (D-049). Die Formachse haengt am BYTE-Kriterium
  `Folge:` und trifft nur die Familie Zahlenfolgen (docs/kombi-korpus.md
  Abschnitt 7) - andere Familien haben KEINE Formachse.
"""

import argparse
import collections
import hashlib
import json
import os
import re
import shutil
import struct
import sys
import tempfile

# --- Spezialbytes (src/data/bytes.rs) -------------------------------------
B_TASK = 0xFE
B_THINK = 0xFD
B_ANSWER = 0xFC
B_END = 0xFF

# --- Shard-Format (src/data/format.rs) ------------------------------------
MAGIC = b"BSHD"
HEADER_SIZE = 32
NOTE_SIZE = 8
MAX_RECORD_LEN = 4096

# --- Messvorschrift ------------------------------------------------------
PANEL_N = 480
MIN_KLASSE = 8

# --- Schwellen -----------------------------------------------------------
KOPF_MIN = 0.90
TYP_MIN = 0.75
ZIEL_MIN = 0.98

# --- Kontextgrenze (configs/model.micro.toml, max_seq_len) ---------------
MAX_SEQ_LEN = 512

# Die Herkunft jeder gemessenen Zeile. Reihenfolge = Ausgabereihenfolge.
STANDS = collections.OrderedDict([
    ("absolut", {
        "pfad": "checkpoints/kombi-step19200.safetensors",
        "sha256": "cd125776acaad53c49cc6f5be4077a2dd27ea76a570003abcb0b2cf0f98d7ed4",
        "korpus": "data/kombi/train",
        "rolle": "Traeger der ABSOLUTEN Linie (Runde 28, D-052)",
    }),
    ("delta", {
        "pfad": "checkpoints/agent-sequence_reasoning-from7600-i1-step12400.safetensors",
        "sha256": "06a5c738647bd8dd91195f0643569d4c58f81caa72da5aa75b756c7a2c3f6406",
        "korpus": "data/seq/train (Delta-Konvention, D-049)",
        "rolle": "Traeger der DELTA-Form (angenommener Stand exp-015/exp-024)",
    }),
    ("mathe", {
        "pfad": "checkpoints/kombi-mathe-step23200.safetensors",
        "sha256": "e6428567b06c0b443c93f7a4a0c562cc431b330b98db0309b84eca11137f8eaf",
        "korpus": "data/kombi-mathe/train (seq+quant+logic-large+algebra)",
        "rolle": "Zusatzkandidat: Algebra OHNE formel (Runde 36, D-060)",
    }),
    ("mathe2", {
        "pfad": "checkpoints/kombi-mathe2-step23200.safetensors",
        "sha256": "d5386d9b9a18728b02f5fe36231514b8f7046c6685e7e91fe7799aca7ac3161c",
        "korpus": "data/kombi-mathe2/train (5 Familien, mit formel)",
        "rolle": "Zusatzkandidat: Algebra UND Formel (Runde 40, D-062)",
    }),
    ("sem200", {
        "pfad": "checkpoints/semantics-step200.safetensors",
        "sha256": "",
        "korpus": "data/semantics/train (Einzelfamilie)",
        "rolle": "Zusatzkandidat: einziger Stand, der auf semantics trainiert ist",
    }),
    ("fmt200", {
        "pfad": "checkpoints/format-step200.safetensors",
        "sha256": "",
        "korpus": "data/format/train (Einzelfamilie)",
        "rolle": "Zusatzkandidat: format-Stand. ACHTUNG O-T3: step205/300/400 entstanden mit Lernrate 0 und sind wertlos; step200 ist der tragbare Stand",
    }),
])
# -------------------------------------------------------------------------
# 1 Abdeckung: die gebauten Familien samt Korpus und Typquelle
# -------------------------------------------------------------------------
#
# `typquelle` sagt, WOHER der Typname eines Datensatzes kommt:
#   `jsonl` - aus der JSONL-Begleitung des Korpus (Feld `typ`). Das ist die
#             exakte Zuordnung des Generators, nicht geraten.
#   `bytes` - aus einer Regel ueber die Datensatz-BYTES, weil fuer diese
#             Familie keine JSONL-Begleitung existiert (logic, algebra,
#             formel). Jede Regel wird beim Start GEGEN DIE MANIFEST-ZAHLUNG
#             geprueft; stimmt sie nicht, bricht das Werkzeug ab.
FAMILIEN = [
    {
        "id": "logic",
        "titel": "Logik und analytisches Denken, Stufe 1",
        "korpus": "data/logic-large/val",
        "jsonl": "",
        "typ_praefix": "logic.",
        "typquelle": "bytes",
        "typregel": "logic",
        "laengenachse": "je_typ",
        "achsen_text": "Kettenlaenge bzw. Stellen-/Variablenzahl, je Typ 3 Klassen",
        "hinweis": "Schnitt logic-large (5000 Held-out). data/logic/val hat 51 Datensaetze - fuer ein 480er-Panel zu klein. data/logic-long/val (2000) ist ein eigener Schnitt mit anderem Seed (4242).",
    },
    {
        "id": "semantics",
        "titel": "Programm-Semantik und Fehlerort (Mini-VM v1)",
        "korpus": "data/semantics/val",
        "jsonl": "data/semantics/jsonl/val.jsonl",
        "typ_praefix": "sem.",
        "typquelle": "jsonl",
        "typregel": "",
        "laengenachse": "familie",
        "achsen_text": "Programmzeilen (3 bis 13), 4 Klassen",
        "hinweis": "5 Typen (sem.*). In KEINEM kombinierten Korpus enthalten (kombi = seq+quant+logic-large, kombi-mathe +algebra, kombi-mathe2 +formel) - die Zeilen auf den eingefrorenen Phasenstaenden messen daher Transfer, nicht trainierte Faehigkeit.",
    },
    {
        "id": "format",
        "titel": "Format-Erkennung aus Bytes",
        "korpus": "data/format/val",
        "jsonl": "data/format/jsonl/val.jsonl",
        "typ_praefix": "fmt.",
        "typquelle": "jsonl",
        "typregel": "",
        "laengenachse": "familie",
        "achsen_text": "Groesse der Probe in Byte, 3 bis 4 Klassen",
        "hinweis": "6 Aufgabentypen (fmt.*). Die fuenf KLASSEN aus D-021 (Signatur, Signatur gegen Gueltigkeit, Struktur ohne Signatur, Widerspruch, falsche Endung) sind eine ANDERE Achse als die sechs Typnamen. In keinem kombinierten Korpus enthalten.",
    },
    {
        "id": "quant",
        "titel": "Quantoren, Syllogistik und Mengen",
        "korpus": "data/quant/val",
        "jsonl": "data/quant/jsonl/val.jsonl",
        "typ_praefix": "quant.",
        "typquelle": "jsonl",
        "typregel": "",
        "laengenachse": "keine",
        "achsen_text": "keine Laengenachse",
        "hinweis": "8 Typen (quant.*) - die hoechste Typenzahl im Projekt. Der Held-out-Schnitt enthaelt NUR die Kombinationen (3 Objekte, 2 Praedikate) und (4 Objekte, 3 Praedikate): Objekt- und Praedikatzahl sind im Korpus perfekt korreliert, es gibt also nur ZWEI Stufen.",
    },
    {
        "id": "seq",
        "titel": "Zahlenfolgen und Regelausfuehrung",
        "korpus": "data/seq/val",
        "jsonl": "data/seq/jsonl/val.jsonl",
        "typ_praefix": "typ.",
        "typquelle": "jsonl",
        "typregel": "",
        "laengenachse": "familie",
        "achsen_text": "Folgenlaenge (5 / 6 / >=7), nicht fuer den Typ vergleich",
        "hinweis": "8 Typnamen, davon 7 inhaltliche (luecke, naechste, offen, regel, reparatur, reparatur1, stelle) und der Kurzantwort-Typ vergleich (Antwortraum 2). Der Typ vergleich traegt keine Folge-Zeile und hat daher keine Laengenachse. EINZIGE Familie mit einer Antwortform-Achse.",
    },
    {
        "id": "algebra",
        "titel": "Algebra A1 - Terme mit Variablenbelegung",
        "korpus": "data/algebra/val",
        "jsonl": "",
        "typ_praefix": "typ.",
        "typquelle": "bytes",
        "typregel": "algebra",
        "laengenachse": "keine",
        "achsen_text": "keine Laengenachse",
        "hinweis": "4 Typen (typ.alg-*). Die Schrittzahl des Denkwegs ist konstant 5 (alg-vergleich hat eine andere, ebenfalls feste Struktur), es gibt also keine Laengenachse.",
    },
    {
        "id": "formel",
        "titel": "Formel-Verstaendnis und Funktion",
        "korpus": "data/formel/val",
        "jsonl": "",
        "typ_praefix": "typ.",
        "typquelle": "bytes",
        "typregel": "formel",
        "laengenachse": "keine",
        "achsen_text": "keine Laengenachse",
        "hinweis": "4 Typen (typ.fkt-*). Die Eingabenzahl hat nur ZWEI Stufen (2 und 3), die Schrittzahl ist je Typ konstant.",
    },
]

# Familie 4 (Sprachlogik Deutsch, O-L4) ist NICHT GEBAUT. Sie wird als
# `nicht gebaut` gefuehrt und NICHT als 0 % gerechnet - eine fehlende Familie
# ist keine Quote.
NICHT_GEBAUT = [
    {
        "id": "sprachlogik",
        "titel": "Sprachlogik Deutsch (Familie 4)",
        "punkt": "O-L4",
        "grund": "Kein Korpus, kein Generator, kein Verifier. docs/programmiersprachen/sprachlogik-und-analytik.md beschreibt den Bauplan; gebaut ist er nicht.",
    },
]

# -------------------------------------------------------------------------
# 2 Typregeln ueber die BYTES (nur fuer Familien ohne JSONL-Begleitung)
# -------------------------------------------------------------------------
# Jede Regel ist eine reine Funktion ueber den Text der AUFGABEN-Sektion
# (zwischen 0xFE und 0xFD). Sie wird beim Start gegen die `typ.*`-Zahlung des
# Manifests geprueft.


def typregel_logic(t):
    t = t.strip()
    if t.startswith("Rechne:"):
        return "carry_arithmetic"
    if t.startswith("Gegeben: P ="):
        return "propositional"
    if re.match(r"^Gegeben: [\d?]", t):
        return "number_sequence"
    if "Wenn" in t:
        return "conditional_rules"
    if t.startswith("Gegeben: (1)"):
        return "category_chain"
    return "(unbekannt)"


def typregel_algebra(t):
    for z in t.splitlines():
        if z.startswith("Frage:"):
            if "Welchen Wert hat der Term" in z:
                return "alg-wert"
            if "In welchem Schritt steckt der Fehler" in z:
                return "alg-stelle"
            if "groesseren Wert" in z:
                return "alg-vergleich"
            if "richtige Wert des falschen Schritts" in z:
                return "alg-reparatur"
    return "(unbekannt)"


def typregel_formel(t):
    for z in t.splitlines():
        if z.startswith("Frage:"):
            if "Werte hat f(x)" in z:
                return "fkt-tabelle"
            if "Geraden" in z:
                return "fkt-liegt"
            if "Mittelpunkt" in z:
                return "fkt-mittel"
            if "Steigung" in z:
                return "fkt-steig"
    return "(unbekannt)"


TYPREGELN = {
    "logic": typregel_logic,
    "algebra": typregel_algebra,
    "formel": typregel_formel,
}
# -------------------------------------------------------------------------
# 3 Laengenachsen
# -------------------------------------------------------------------------
# Rueckgabe: (Achsenname, Klassenname) oder None, wenn dieser Datensatz keine
# Laengenachse hat. Die Grenzen sind FEST und stehen hier - nicht aus der
# Verteilung gerechnet -, damit der Bericht reproduzierbar bleibt.

LOGIC_ACHSEN = {
    "carry_arithmetic": ("Stellen", lambda t: len(re.findall(r"\d", t.split("=")[0])),
                         [(4, "bis 4"), (5, "5"), (10 ** 9, "ab 6")]),
    "category_chain": ("Praemissen", lambda t: len(re.findall(r"\(\d+\)", t)),
                       [(3, "bis 3"), (6, "4 bis 6"), (10 ** 9, "ab 7")]),
    "conditional_rules": ("Praemissen", lambda t: len(re.findall(r"\(\d+\)", t)),
                          [(4, "bis 4"), (6, "5 bis 6"), (10 ** 9, "ab 7")]),
    "number_sequence": ("Folgenlaenge", lambda t: len(re.findall(r"[\d?]+", t.split(".")[0])),
                        [(5, "bis 5"), (6, "6"), (10 ** 9, "ab 7")]),
    "propositional": ("Variablen", lambda t: len(set(re.findall(r"([A-Z]) = ", t))),
                      [(2, "2"), (3, "3"), (10 ** 9, "ab 4")]),
}


def _klasse(messwert, stufen):
    for grenze, name in stufen:
        if messwert <= grenze:
            return name
    return stufen[-1][1]


def laengen_klasse(familie, typ, rec):
    """Laengenachse eines Datensatzes: (Achse, Klasse) oder None.

    Gemessen wird an den BYTES. Bei format steht die Probengroesse im DENKWEG
    (`Probe: N Byte`), deshalb wird dort der ganze Datensatz gelesen; die
    uebrigen Familien messen ihre Achse in der Aufgaben-Sektion.
    """
    fam = familie["id"]
    t = aufgaben_text(rec)
    if fam == "logic":
        eintrag = LOGIC_ACHSEN.get(typ)
        if not eintrag:
            return None
        achse, messer, stufen = eintrag
        return (achse, _klasse(messer(t), stufen))
    if fam == "seq":
        m = re.search(r"Folge: ([^\n]+)", t)
        if not m:
            return None
        n = len([y for y in m.group(1).split(",") if y.strip()])
        return ("Folgenlaenge", _klasse(n, [(5, "5"), (6, "6"), (10 ** 9, "ab 7")]))
    if fam == "semantics":
        i = t.find("PROG")
        j = t.find("ENDE-PROG")
        if i < 0 or j < 0:
            return None
        n = len([z for z in t[i:j].splitlines()[1:] if z.strip()])
        return ("Programmzeilen", _klasse(n, [(4, "bis 4"), (6, "5 bis 6"), (8, "7 bis 8"), (10 ** 9, "ab 9")]))
    if fam == "format":
        m = re.search(r"Probe: (\d+) Byte", rec.decode("utf-8", "replace"))
        if not m:
            return None
        return ("Probengroesse", _klasse(int(m.group(1)), [(15, "bis 15"), (31, "16 bis 31"), (63, "32 bis 63"), (10 ** 9, "ab 64")]))
    return None


STUFEN_JE_FAMILIE = {
    "seq": ["5", "6", "ab 7"],
    "semantics": ["bis 4", "5 bis 6", "7 bis 8", "ab 9"],
    "format": ["bis 15", "16 bis 31", "32 bis 63", "ab 64"],
}


def stufen_namen(familie, typ):
    """Die Klassen einer Laengenachse in STUFENFOLGE (nicht alphabetisch)."""
    fam = familie["id"]
    if fam == "logic":
        e = LOGIC_ACHSEN.get(typ)
        return [n for _, n in e[2]] if e else []
    return list(STUFEN_JE_FAMILIE.get(fam, []))


# -------------------------------------------------------------------------
# 4 Shard- und Manifest-Ein-/Ausgabe
# -------------------------------------------------------------------------


def fnv1a64(data):
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def lies_shard(pfad):
    """Liest die Datensaetze eines Shards. Nur die Bytes, kein Verifier."""
    with open(pfad, "rb") as fh:
        b = fh.read()
    if b[:4] != MAGIC:
        raise SystemExit("kein Shard (Magic fehlt): %s" % pfad)
    (anzahl,) = struct.unpack_from("<Q", b, 8)
    recs = []
    off = HEADER_SIZE
    while off + 2 <= len(b):
        (n,) = struct.unpack_from("<H", b, off)
        off += 2
        if off + n > len(b):
            break
        recs.append(b[off:off + n])
        off += n
    if len(recs) != anzahl:
        raise SystemExit("Shard-Kopf meldet %d Datensaetze, gelesen %d: %s" % (anzahl, len(recs), pfad))
    return recs


def lies_manifest(pfad):
    with open(pfad, "r", encoding="utf-8") as fh:
        return json.load(fh)


def korpus_datensaetze(repo, familie):
    d = os.path.join(repo, familie["korpus"])
    man = lies_manifest(os.path.join(d, "manifest.json"))
    recs = []
    for s in man["shards"]:
        recs.extend(lies_shard(os.path.join(d, s["path"])))
    return recs, man


def typen_aus_jsonl(repo, familie, anzahl):
    pfad = os.path.join(repo, familie["jsonl"])
    typen = []
    with open(pfad, "r", encoding="utf-8") as fh:
        for zeile in fh:
            zeile = zeile.strip()
            if zeile:
                typen.append(json.loads(zeile)["typ"])
    if len(typen) != anzahl:
        raise SystemExit("JSONL hat %d Zeilen, Shard hat %d Datensaetze: %s" % (len(typen), anzahl, pfad))
    return typen


def schreibe_shard(pfad, records, note):
    if len(note) > NOTE_SIZE or not note.isascii():
        raise SystemExit("Shard-Notiz muss ASCII und <= %d Zeichen sein: %r" % (NOTE_SIZE, note))
    kopf = bytearray(HEADER_SIZE)
    kopf[0:4] = MAGIC
    struct.pack_into("<H", kopf, 4, 1)
    struct.pack_into("<H", kopf, 6, HEADER_SIZE)
    struct.pack_into("<Q", kopf, 8, len(records))
    struct.pack_into("<Q", kopf, 16, 0)
    roh = note.encode("ascii")
    kopf[24:24 + len(roh)] = roh
    nutz = 0
    with open(pfad, "wb") as fh:
        fh.write(bytes(kopf))
        for r in records:
            if not r or len(r) > MAX_RECORD_LEN:
                raise SystemExit("Datensatz unbrauchbar (%d Bytes)" % len(r))
            fh.write(struct.pack("<H", len(r)))
            fh.write(r)
            nutz += len(r)
    return nutz, os.path.getsize(pfad)


def lege_korpus_an(ziel, records, seed, note, tasks):
    """Schreibt einen selbsttragenden Korpus: shard-0000.bin + manifest.json."""
    os.makedirs(ziel, exist_ok=True)
    shard = os.path.join(ziel, "shard-0000.bin")
    nutz, groesse = schreibe_shard(shard, records, note)
    with open(shard, "rb") as fh:
        pruef = "%016x" % fnv1a64(fh.read())
    man = collections.OrderedDict()
    man["format_version"] = 1
    man["vocab_size"] = 256
    man["byte_order"] = "little"
    man["record_encoding"] = "raw-bytes-with-special-tokens"
    man["seed"] = seed
    man["note"] = note
    man["tasks"] = collections.OrderedDict(sorted(tasks.items()))
    man["shards"] = [collections.OrderedDict([
        ("id", 0),
        ("path", "shard-0000.bin"),
        ("records", len(records)),
        ("payload_bytes", nutz),
        ("file_size", groesse),
        ("checksum", pruef),
    ])]
    with open(os.path.join(ziel, "manifest.json"), "w", encoding="utf-8", newline="\n") as fh:
        json.dump(man, fh, ensure_ascii=True, indent=1, sort_keys=False)
        fh.write("\n")
    return pruef, nutz


# -------------------------------------------------------------------------
# 5 Panel-Auswahl - DIESELBE Regel wie src/agent.rs reference_set_ab
# -------------------------------------------------------------------------


def panel_indizes(anzahl, panel_n, versatz):
    stride = max(1, anzahl // max(1, panel_n))
    indizes = []
    i = versatz % stride
    while i < anzahl and len(indizes) < panel_n:
        indizes.append(i)
        i += stride
    return indizes, stride


def panel_versatz(stride):
    """Panel B: Versatz `stride // 2`, mindestens 1 (src/agent.rs:2743)."""
    return max(1, stride // 2)


# -------------------------------------------------------------------------
# 6 Typnamen und Textabschnitte eines Datensatzes
# -------------------------------------------------------------------------


def abschnitt(rec, auf, zu):
    i = rec.find(bytes([auf]))
    if i < 0:
        return ""
    j = rec.find(bytes([zu]), i + 1)
    return rec[i + 1:j].decode("utf-8", "replace") if j > 0 else ""


def aufgaben_text(rec):
    return abschnitt(rec, B_TASK, B_THINK)


def antwort_text(rec):
    return abschnitt(rec, B_ANSWER, B_END).strip()


def prompt_laenge(rec):
    return rec.find(bytes([B_ANSWER]))


def passt_ins_fenster(rec):
    p = prompt_laenge(rec)
    return p >= 0 and p + 2 < MAX_SEQ_LEN
# -------------------------------------------------------------------------
# 7 Messpaare: welcher Stand traegt welche Zeile
# -------------------------------------------------------------------------
# `form` ist die ANTWORTFORM des Panels:
#   absolut - Panel unveraendert
#   delta   - reparatur-Faelle in der Aenderungsform (D-049)
# Die Delta-Form existiert NUR ueber `agent capability run --pack
# reparatur-delta`; sie wird deshalb aus dem Loop gelesen und nicht
# nachgebaut (eine zweite Umsetzung koennte von der geprueften abweichen).

PAARE = [
    {"id": "seq-abs", "stand": "absolut", "familie": "seq", "form": "absolut",
     "rolle": "PFLICHT: absolute Linie (Kombi-Korpus, absolute Form)", "pflicht": True},
    {"id": "seq-delta", "stand": "delta", "familie": "seq", "form": "delta",
     "rolle": "PFLICHT: Delta-Form (eingefrorener Delta-Stand)", "pflicht": True},
    {"id": "quant-abs", "stand": "absolut", "familie": "quant", "form": "absolut",
     "rolle": "PFLICHT: Quantoren/Syllogistik/Mengen", "pflicht": True},
    {"id": "logic-abs", "stand": "absolut", "familie": "logic", "form": "absolut",
     "rolle": "PFLICHT: Logik Stufe 1", "pflicht": True},
    {"id": "alg-mathe", "stand": "mathe", "familie": "algebra", "form": "absolut",
     "rolle": "KONTROLLARM: Algebra-Stand OHNE formel-Shard", "pflicht": True},
    {"id": "alg-mathe2", "stand": "mathe2", "familie": "algebra", "form": "absolut",
     "rolle": "ARM MIT formel-Shard", "pflicht": True},
    {"id": "fkt-mathe", "stand": "mathe", "familie": "formel", "form": "absolut",
     "rolle": "KONTROLLARM: Stand VOR der formel-Familie (Runde 39/40, Arm A)", "pflicht": True},
    {"id": "fkt-mathe2", "stand": "mathe2", "familie": "formel", "form": "absolut",
     "rolle": "ARM MIT formel-Familie (Runde 40, Arm B)", "pflicht": True},
    {"id": "sem-fam", "stand": "sem200", "familie": "semantics", "form": "absolut",
     "rolle": "Einzelfamilien-Stand (einziger auf semantics trainierter Stand)", "pflicht": True},
    {"id": "sem-transfer", "stand": "absolut", "familie": "semantics", "form": "absolut",
     "rolle": "TRANSFER: Phasenstand, auf dem semantics NIE trainiert wurde", "pflicht": True},
    {"id": "fmt-fam", "stand": "fmt200", "familie": "format", "form": "absolut",
     "rolle": "Einzelfamilien-Stand (format)", "pflicht": True},
    {"id": "fmt-transfer", "stand": "absolut", "familie": "format", "form": "absolut",
     "rolle": "TRANSFER: Phasenstand, auf dem format NIE trainiert wurde", "pflicht": True},
]

# Zusatzpaare (nur mit --mit-formvergleich): beide Antwortformen auf DEMSELBEN
# Stand - der faire Formvergleich, den D-049 noch schuldet.
PAARE_FORMVERGLEICH = [
    {"id": "seq-abs-deltaform", "stand": "absolut", "familie": "seq", "form": "delta",
     "rolle": "KUER: Delta-Form auf dem ABSOLUTEN Stand (fairer Formvergleich)", "pflicht": False},
    {"id": "seq-delta-absform", "stand": "delta", "familie": "seq", "form": "absolut",
     "rolle": "KUER: absolute Form auf dem DELTA-Stand (fairer Formvergleich)", "pflicht": False},
]

LOOP_CAPABILITY = "sequence_reasoning"
LOOP_ABS_JOBS = [
    {"jobid": "loop-seq-abs", "stand": "absolut", "form": "absolut", "wdh": False,
     "rolle": "Gegenprobe: Panelzahlen des Projekts auf dem absoluten Stand"},
    {"jobid": "loop-seq-abs-wdh", "stand": "absolut", "form": "absolut", "wdh": True,
     "rolle": "Determinismus-Probe des Loops (Panel A zweimal)"},
    {"jobid": "loop-seq-delta", "stand": "delta", "form": "delta", "wdh": False,
     "rolle": "EINZIGE Quelle der Delta-Form (--pack reparatur-delta)"},
    {"jobid": "loop-seq-delta-wdh", "stand": "delta", "form": "delta", "wdh": True,
     "rolle": "Determinismus-Probe der Delta-Form"},
]


def familie_nach_id(fid):
    for f in FAMILIEN:
        if f["id"] == fid:
            return f
    raise SystemExit("unbekannte Familie: %s" % fid)


def sha256_datei(pfad):
    h = hashlib.sha256()
    with open(pfad, "rb") as fh:
        for block in iter(lambda: fh.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def stand_zeile(repo, key):
    """Pfad, SHA-256 (am Dateiende gemessen) und Rolle eines Standes."""
    s = STANDS[key]
    pfad = os.path.join(repo, s["pfad"])
    if not os.path.isfile(pfad):
        return {"key": key, "pfad": s["pfad"], "sha256": "(Datei fehlt)",
                "groesse": 0, "korpus": s["korpus"], "rolle": s["rolle"],
                "sha_soll": s["sha256"]}
    ist = sha256_datei(pfad)
    return {"key": key, "pfad": s["pfad"], "sha256": ist,
            "groesse": os.path.getsize(pfad), "korpus": s["korpus"],
            "rolle": s["rolle"], "sha_soll": s["sha256"]}


# -------------------------------------------------------------------------
# 8 Abdeckung pruefen (nur lesen) - und zwar GEGEN die Manifest-Zahlung
# -------------------------------------------------------------------------


def abdeckung(repo):
    """Familien, Typen, Panels, Laengenachsen, Fensterguete. Nur lesend."""
    bericht = []
    for fam in FAMILIEN:
        recs, man = korpus_datensaetze(repo, fam)
        n = len(recs)
        if fam["typquelle"] == "jsonl":
            typen = typen_aus_jsonl(repo, fam, n)
        else:
            regel = TYPREGELN[fam["typregel"]]
            typen = [regel(aufgaben_text(r)) for r in recs]
        ist = collections.Counter(typen)
        soll = {k.split(".", 1)[1]: v for k, v in man["tasks"].items()
                if k.startswith(fam["typ_praefix"])}
        stimmt = dict(sorted(ist.items())) == dict(sorted(soll.items()))
        indizes_a, stride = panel_indizes(n, PANEL_N, 0)
        indizes_b, _ = panel_indizes(n, PANEL_N, panel_versatz(stride))
        disjunkt = not (set(indizes_a) & set(indizes_b))
        fit = sum(1 for r in recs if passt_ins_fenster(r))
        # Antwortraeume je Typ (nur lesend) - Grundlage der Kurzantwort-Pruefung
        raeume = {}
        gruppen = collections.defaultdict(list)
        for r, t in zip(recs, typen):
            gruppen[t].append(antwort_text(r))
        for t in sorted(gruppen):
            raeume[t] = len(set(gruppen[t]))
        # Laengenachse: welche Typen haben mindestens MIN_KLASSE Datensaetze in
        # mindestens DREI Klassen?
        achsen = {}
        for r, t in zip(recs, typen):
            k = laengen_klasse(fam, t, r)
            if k is None:
                continue
            achsen.setdefault(t, collections.defaultdict(int))
            achsen[t][k[0] + " | " + k[1]] += 1
        achsen_ok = {}
        for t in sorted(achsen):
            gut = [k for k, v in achsen[t].items() if v >= MIN_KLASSE]
            achsen_ok[t] = sorted(gut)
        bericht.append({
            "familie": fam,
            "n": n,
            "typen": dict(sorted(ist.items())),
            "soll": dict(sorted(soll.items())),
            "stimmt": stimmt,
            "stride": stride,
            "versatz_a": 0,
            "versatz_b": panel_versatz(stride),
            "panel_a": len(indizes_a),
            "panel_b": len(indizes_b),
            "disjunkt": disjunkt,
            "passt_ins_fenster": fit,
            "antwortraeume": raeume,
            "achsen": achsen_ok,
            "manifest": man,
            "records": recs,
            "typen_liste": typen,
        })
    return bericht
# -------------------------------------------------------------------------
# 9 Bereitstellung der Panels und Unterkorpora
# -------------------------------------------------------------------------


def san(s):
    return re.sub(r"[^A-Za-z0-9]+", "_", s).strip("_")


def typ_tasks(typen):
    t = collections.Counter()
    for x in typen:
        t["typ." + x] += 1
    return t


def stelle_bereit(repo, work, nur=None, mit_formvergleich=False, mit_poolprobe=False):
    """Schreibt Panels, Typ- und Klassen-Unterkorpora und die Jobliste."""
    ab = abdeckung(repo)
    fehler = []
    for e in ab:
        if not e["stimmt"]:
            fehler.append("%s: Typregel reproduziert die Manifest-Zahlung NICHT\n  erkannt: %s\n  manifest: %s"
                          % (e["familie"]["id"], e["typen"], e["soll"]))
        if not e["disjunkt"]:
            fehler.append("%s: Panel A und B sind NICHT disjunkt" % e["familie"]["id"])
        if e["panel_a"] < PANEL_N or e["panel_b"] < PANEL_N:
            fehler.append("%s: Panel kleiner als %d (A %d, B %d)"
                          % (e["familie"]["id"], PANEL_N, e["panel_a"], e["panel_b"]))
    if fehler:
        raise SystemExit("Abbruch, die Messvorschrift ist nicht erfuellbar:\n\n" + "\n".join(fehler))

    paare = list(PAARE)
    if mit_formvergleich:
        paare = paare + list(PAARE_FORMVERGLEICH)
    if nur:
        paare = [p for p in paare if p["id"] in nur]

    jobs = []
    korpora = os.path.join(work, "korpora")
    pool_probe_familien = set()

    for e in ab:
        fam = e["familie"]
        fid = fam["id"]
        recs = e["records"]
        typen = e["typen_liste"]
        stride = e["stride"]
        versatz_b = e["versatz_b"]
        ia, _ = panel_indizes(len(recs), PANEL_N, 0)
        ib, _ = panel_indizes(len(recs), PANEL_N, versatz_b)
        ra, rb = [recs[i] for i in ia], [recs[i] for i in ib]
        ta, tb = [typen[i] for i in ia], [typen[i] for i in ib]
        seed = e["manifest"].get("seed", 0)
        base = os.path.join(korpora, fid)
        lege_korpus_an(os.path.join(base, "A"), ra, seed, "benchA", typ_tasks(ta))
        lege_korpus_an(os.path.join(base, "B"), rb, seed, "benchB", typ_tasks(tb))
        # Typ-Unterkorpora je Panel
        for panel, rr, tt in (("A", ra, ta), ("B", rb, tb)):
            for t in sorted(set(tt)):
                xr = [r for r, y in zip(rr, tt) if y == t]
                lege_korpus_an(os.path.join(base, panel, "typ-" + san(t)), xr, seed,
                               "bench" + panel, collections.Counter({"typ." + t: len(xr)}))
        # Laengenklassen auf dem GEPOOLTEN Panel (A + B)
        gepoolt = list(zip(ra + rb, ta + tb))
        achsen = collections.defaultdict(list)
        for r, t in gepoolt:
            k = laengen_klasse(fam, t, r)
            if k is not None:
                achsen[(k[0], k[1])].append((r, t))
        je_typ = fam["laengenachse"] == "je_typ"
        for (achse, klasse) in sorted(achsen):
            gruppe = achsen[(achse, klasse)]
            if je_typ:
                nach_typ = collections.defaultdict(list)
                for r, t in gruppe:
                    nach_typ[t].append(r)
                for t in sorted(nach_typ):
                    if len(nach_typ[t]) < MIN_KLASSE:
                        continue
                    ziel = os.path.join(base, "klasse", san(achse) + "-" + san(klasse), "typ-" + san(t))
                    lege_korpus_an(ziel, nach_typ[t], seed, "benchK",
                                   collections.Counter({"typ." + t: len(nach_typ[t])}))
            else:
                if len(gruppe) < MIN_KLASSE:
                    continue
                ziel = os.path.join(base, "klasse", san(achse) + "-" + san(klasse))
                tt = collections.Counter("typ." + t for _, t in gruppe)
                lege_korpus_an(ziel, [r for r, _ in gruppe], seed, "benchK", tt)
        if mit_poolprobe:
            tt = collections.Counter("typ." + t for _, t in gepoolt)
            lege_korpus_an(os.path.join(base, "AB"), [r for r, _ in gepoolt], seed, "benchAB", tt)
            pool_probe_familien.add(fid)

    # --- Jobliste ---------------------------------------------------------
    for p in paare:
        fam = familie_nach_id(p["familie"])
        fid = fam["id"]
        e = [x for x in ab if x["familie"]["id"] == fid][0]
        base = "korpora/" + fid
        # Die Panels dieser Familie erneut ableiten - EINE Quelle fuer Auswahl
        # und Fragenzahl (die Schleifenvariablen der Bereitstellung gelten hier
        # nicht mehr).
        ia, _ = panel_indizes(e["n"], PANEL_N, 0)
        ib, _ = panel_indizes(e["n"], PANEL_N, e["versatz_b"])
        ta = [e["typen_liste"][i] for i in ia]
        tb = [e["typen_liste"][i] for i in ib]
        if p["form"] == "delta":
            # Die Delta-Form gibt es NUR im Loop (--pack reparatur-delta).
            # Ein eval-Lauf auf dem absoluten Panel wuerde die absolute Form
            # messen und die Zeile falsch beschriften.
            continue
        jobs.append({"jobid": p["id"] + "-A", "kind": "eval", "corpus": base + "/A",
                     "answers": e["panel_a"], "stand": p["stand"], "familie": fid,
                     "form": p["form"], "ebene": "panel", "typ": "", "klasse": "",
                     "achse": "", "rolle": p["rolle"]})
        jobs.append({"jobid": p["id"] + "-B", "kind": "eval", "corpus": base + "/B",
                     "answers": e["panel_b"], "stand": p["stand"], "familie": fid,
                     "form": p["form"], "ebene": "panel", "typ": "", "klasse": "",
                     "achse": "", "rolle": "Panel B (Versatz %d)" % e["versatz_b"]})
        jobs.append({"jobid": p["id"] + "-A-wdh", "kind": "eval", "corpus": base + "/A",
                     "answers": e["panel_a"], "stand": p["stand"], "familie": fid,
                     "form": p["form"], "ebene": "wiederholung", "typ": "", "klasse": "",
                     "achse": "", "rolle": "Determinismus-Probe: Panel A zum zweiten Mal"})
        # Die Fragenzahl einer Typzeile ist die Zahl der Datensaetze DIESES Typs
        # IM PANEL - nicht die des ganzen Shards. Sonst meldet der Bericht mehr
        # gestellte Fragen als das Panel hat.
        zahl_a = collections.Counter(ta)
        zahl_b = collections.Counter(tb)
        for t in sorted(e["typen"]):
            for panel, zahl in (("A", zahl_a), ("B", zahl_b)):
                n = zahl.get(t, 0)
                if n == 0:
                    continue
                jobs.append({"jobid": p["id"] + "-" + panel + "-t-" + san(t), "kind": "eval",
                             "corpus": base + "/" + panel + "/typ-" + san(t),
                             "answers": n, "stand": p["stand"], "familie": fid,
                             "form": p["form"], "ebene": "typ", "typ": t, "klasse": "",
                             "achse": "", "rolle": "Typzeile Panel " + panel})
        for (achse, klasse, t, anz) in klassen_jobs(e, fam):
            jobs.append({"jobid": p["id"] + "-k-" + san(achse) + "-" + san(klasse) + ("-t-" + san(t) if t else ""),
                         "kind": "eval",
                         "corpus": base + "/klasse/" + san(achse) + "-" + san(klasse) + ("/typ-" + san(t) if t else ""),
                         "answers": anz, "stand": p["stand"], "familie": fid,
                         "form": p["form"], "ebene": "klasse", "typ": t, "klasse": klasse,
                         "achse": achse, "rolle": "Laengenachse " + achse})
        if mit_poolprobe and fid in pool_probe_familien:
            jobs.append({"jobid": p["id"] + "-AB", "kind": "eval", "corpus": base + "/AB",
                         "answers": e["panel_a"] + e["panel_b"], "stand": p["stand"],
                         "familie": fid, "form": p["form"], "ebene": "poolprobe", "typ": "",
                         "klasse": "", "achse": "",
                         "rolle": "Pool-Probe: A+B in EINEM Lauf gegen die Summe aus zwei Laeufen"})
    for j in LOOP_ABS_JOBS:
        fids = [p for p in paare if p["familie"] == "seq"]
        if not fids:
            continue
        jobs.append({"jobid": j["jobid"], "kind": "agent", "corpus": "",
                     "answers": PANEL_N, "stand": j["stand"], "familie": "seq",
                     "form": j["form"], "ebene": "loop", "typ": "", "klasse": "",
                     "achse": "", "rolle": j["rolle"]})

    os.makedirs(work, exist_ok=True)
    felder = ["jobid", "kind", "corpus", "answers", "stand", "familie", "form",
              "ebene", "typ", "klasse", "achse", "rolle"]
    with open(os.path.join(work, "jobs.tsv"), "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\t".join(felder) + "\n")
        for j in jobs:
            fh.write("\t".join(str(j[f]) for f in felder) + "\n")
    return jobs, ab


def klassen_jobs(e, fam):
    """(Achse, Klasse, Typ, Anzahl) der bereitgestellten Klassenkorpora."""
    out = []
    if fam["laengenachse"] == "keine":
        return []
    gepoolt = []
    ia, _ = panel_indizes(len(e["records"]), PANEL_N, 0)
    ib, _ = panel_indizes(len(e["records"]), PANEL_N, e["versatz_b"])
    for i in ia + ib:
        gepoolt.append((e["records"][i], e["typen_liste"][i]))
    gruppen = collections.defaultdict(list)
    for r, t in gepoolt:
        k = laengen_klasse(fam, t, r)
        if k is not None:
            gruppen[k].append((r, t))
    je_typ = fam["laengenachse"] == "je_typ"
    for (achse, klasse) in sorted(gruppen):
        gruppe = gruppen[(achse, klasse)]
        if je_typ:
            nach_typ = collections.defaultdict(list)
            for r, t in gruppe:
                nach_typ[t].append(r)
            for t in sorted(nach_typ):
                if len(nach_typ[t]) >= MIN_KLASSE:
                    out.append((achse, klasse, t, len(nach_typ[t])))
        else:
            if len(gruppe) >= MIN_KLASSE:
                out.append((achse, klasse, "", len(gruppe)))
    return out
# -------------------------------------------------------------------------
# 10 Kurzstrategie-Baselines - an den BYTES gerechnet
# -------------------------------------------------------------------------
# Vorbild: /home/changchi/r39/kurzstrategie.py und die 4-Typ-Fassung
# kurzstrategie4.py. Die Lehre der 4-Typ-Fassung ist hier eingebaut: bei
# Familien, deren Typen dieselbe erste Aufgabenzeile haben (formel: `Punkt A:`),
# wird der Typ NICHT aus der ersten Zeile, sondern aus der FRAGEZEILE bestimmt.

def erste_zahl(t):
    m = re.search(r"\d+", t)
    return m.group(0) if m else ""


def erste_eingabe(t):
    m = re.search(r"(?:Eingaben: x =|Belegung: x =|Eingabe x =) (-?\d+)", t)
    if m:
        return m.group(1)
    m = re.search(r"x = (-?\d+)", t)
    return m.group(1) if m else ""


def kurz_baselines(repo, bericht):
    """Kurzstrategie-Baselines je Familie, Panel und Typ - nur aus Bytes."""
    kurzverfahren = {}
    for fam in FAMILIEN:
        if fam["jsonl"]:
            kurzverfahren[fam["id"]] = lies_kurzverfahren(os.path.join(repo, fam["jsonl"]))
    ergebnis = []
    for e in bericht:
        fam = e["familie"]
        fid = fam["id"]
        recs, typen = e["records"], e["typen_liste"]
        ia, _ = panel_indizes(len(recs), PANEL_N, 0)
        ib, _ = panel_indizes(len(recs), PANEL_N, e["versatz_b"])
        kv = kurzverfahren.get(fid, [])
        # V = ganzer Held-out-Shard: nur fuer den Quervergleich mit dem
        # Projekt-Vorbild /home/changchi/r39/kurzstrategie.py, nicht fuer die Linien.
        for panel, idx in (("A", ia), ("B", ib), ("AB", ia + ib),
                           ("V", list(range(len(recs))))):
            je_typ = collections.defaultdict(list)
            for i in idx:
                je_typ[typen[i]].append(i)
            for t in sorted(je_typ):
                ids = je_typ[t]
                antworten = [antwort_text(recs[i]) for i in ids]
                c = collections.Counter(antworten)
                top, topn = c.most_common(1)[0]
                n = len(ids)
                treffer = {
                    "Mehrheitsantwort": topn,
                    "immer 0": sum(1 for a in antworten if a == "0"),
                    "erste Zahl": sum(1 for i, a in zip(ids, antworten)
                                      if erste_zahl(aufgaben_text(recs[i])) == a and a != ""),
                    "erste Eingabe": sum(1 for i, a in zip(ids, antworten)
                                         if erste_eingabe(aufgaben_text(recs[i])) == a and a != ""),
                    "Schritt 1": sum(1 for a in antworten if a == "Schritt 1"),
                }
                # Nur Familien, deren Begleitung das Feld WIRKLICH traegt (seq).
                if any(x.strip() for x in kv):
                    treffer["Kurzverfahren"] = sum(
                        1 for i, a in zip(ids, antworten)
                        if 0 <= i < len(kv) and kv[i].strip() == a and a != "")
                ergebnis.append({
                    "familie": fid, "panel": panel, "typ": t, "n": n,
                    "antwortraum": len(c), "mehrheit": top, "mehrheit_n": topn,
                    "treffer": treffer,
                })
    return ergebnis


def lies_kurzverfahren(pfad):
    out = []
    with open(pfad, "r", encoding="utf-8") as fh:
        for zeile in fh:
            zeile = zeile.strip()
            if zeile:
                out.append(json.loads(zeile).get("kurzverfahren", ""))
    return out


# -------------------------------------------------------------------------
# 11 Log-Parser
# -------------------------------------------------------------------------

RE_GEFRAGT = re.compile(r"^Gefragt\s*:\s*(\d+) Datensaetze, davon richtig:\s*(\d+)\s*\(([\d.]+) %\)", re.M)
RE_AUSWAHL = re.compile(r"^Auswahl\s*:\s*jeder (\d+)\. Datensatz des Shards", re.M)
RE_PROBE_TYP = re.compile(r"^(\S+)\s+(\d+)\s+([\d.]+)\s+(\d+)\s+(\d+)\s+(\d+)\s*$", re.M)
RE_LOOP_PANELA = re.compile(r"^Referenzsatz\s*:\s*Panel A (\d+) Datensaetze, jeder (\d+)\. aus (\S+)", re.M)
RE_LOOP_PANELB = re.compile(r"^\s+Panel B (\d+) Datensaetze, jeder (\d+)\. mit Versatz (\d+)", re.M)
RE_LOOP_FORM = re.compile(r"^Antwortform\s*:\s*(.+)$", re.M)
RE_LOOP_TYP_A = re.compile(r"^   Typ (\S+)\s+(\d+)/(\d+) richtig\s*$", re.M)
RE_LOOP_TYP_B = re.compile(r"^   Typ B (\S+)\s+(\d+)/(\d+) richtig\s*$", re.M)
RE_LOOP_SUM_A = re.compile(r"^Baseline\s*:\s*inhaltlich ([\d.]+) % \((\d+) von (\d+)\)"
                           r"(?: \| Kurzantwort ([\d.]+) % \((\d+) von (\d+)\))?"
                           r"(?: \| gesamt ([\d.]+) %)?", re.M)
RE_LOOP_SUM_B = re.compile(r"^Baseline B\s*:\s*inhaltlich ([\d.]+) % \((\d+) von (\d+)\)"
                           r"(?: \| gesamt ([\d.]+) %)?", re.M)
RE_LOOP_BAND = re.compile(r"^Trainingsband\s*:\s*(.+)$", re.M)


def parse_eval(text):
    """Parser fuer `shiro eval --answers N`. Gibt None zurueck, wenn kein Lauf."""
    m = RE_GEFRAGT.search(text)
    if not m:
        return None
    a = RE_AUSWAHL.search(text)
    return {
        "gefragt": int(m.group(1)),
        "richtig": int(m.group(2)),
        "prozent": float(m.group(3)),
        "stride": int(a.group(1)) if a else None,
    }


def parse_loop(text):
    """Parser fuer `shiro agent capability run ... --dry-run`."""
    a = RE_LOOP_PANELA.search(text)
    if not a:
        return None
    b = RE_LOOP_PANELB.search(text)
    form = RE_LOOP_FORM.search(text)
    ergebnis = {
        "panel_a_groesse": int(a.group(1)),
        "stride": int(a.group(2)),
        "korpus": a.group(3),
        "panel_b_groesse": int(b.group(1)) if b else 0,
        "versatz_b": int(b.group(3)) if b else None,
        "antwortform": form.group(1).strip() if form else "(keine Zeile)",
        "typ_a": [(t, int(c), int(n)) for t, c, n in RE_LOOP_TYP_A.findall(text)],
        "typ_b": [(t, int(c), int(n)) for t, c, n in RE_LOOP_TYP_B.findall(text)],
        "band": (RE_LOOP_BAND.search(text).group(1).strip() if RE_LOOP_BAND.search(text) else ""),
    }
    for schluessel, muster in (("summe_a", RE_LOOP_SUM_A), ("summe_b", RE_LOOP_SUM_B)):
        m = muster.search(text)
        ergebnis[schluessel] = None
        if m:
            g = m.groups()
            if schluessel == "summe_a":
                ergebnis[schluessel] = {
                    "inhaltlich": float(g[0]), "inhaltlich_c": int(g[1]), "inhaltlich_n": int(g[2]),
                    "kurz": float(g[3]) if g[3] else None,
                    "kurz_c": int(g[4]) if g[4] else 0, "kurz_n": int(g[5]) if g[5] else 0,
                    "gesamt": float(g[6]) if g[6] else None,
                }
            else:
                ergebnis[schluessel] = {
                    "inhaltlich": float(g[0]), "inhaltlich_c": int(g[1]), "inhaltlich_n": int(g[2]),
                    "gesamt": float(g[3]) if g[3] else None,
                }
    return ergebnis


def lies_log(work, name):
    pfad = os.path.join(work, "logs", name + ".log")
    if not os.path.isfile(pfad):
        return None, None
    with open(pfad, "rb") as fh:
        roh = fh.read()
    return roh, roh.decode("utf-8", "replace")


def lies_jobs(work):
    pfad = os.path.join(work, "jobs.tsv")
    if not os.path.isfile(pfad):
        raise SystemExit("Jobliste fehlt: %s - zuerst `stage` laufen lassen" % pfad)
    with open(pfad, "r", encoding="utf-8") as fh:
        zeilen = [z.rstrip("\n").split("\t") for z in fh if z.strip()]
    kopf = zeilen[0]
    return [dict(zip(kopf, z)) for z in zeilen[1:]]
# -------------------------------------------------------------------------
# 12 Auswertung
# -------------------------------------------------------------------------


def kurzantwort_typen(repo):
    """Liest die Kurzantwort-Typen AUS DER CAPABILITY-DATEI (nicht aus dem Code)."""
    pfad = os.path.join(repo, "configs", "capabilities", "sequence_reasoning.toml")
    if not os.path.isfile(pfad):
        return [], "(Capability-Datei fehlt)"
    text = open(pfad, "r", encoding="utf-8").read()
    namen = []
    m = re.search(r"kurzantwort_typen\s*=\s*\[(.*?)\]", text, re.S)
    if m:
        namen = re.findall(r"\"([^\"]+)\"", m.group(1))
    return namen, "(configs/capabilities/sequence_reasoning.toml)"


def quote(c, n):
    return (100.0 * c / n) if n else 0.0


def urteil_linien(kopf_c, kopf_n, typen_zeilen, inhaltliche, typ_min, kopf_min):
    """D-047-Regel: Kopfzahl >= kopf_min UND kein inhaltlicher Typ < typ_min.

    kopf_min und typ_min sind ANTEILE (0.90 / 0.75), `quote` liefert Prozent -
    verglichen wird deshalb durchgehend als Anteil.
    """
    kopf = (kopf_c / kopf_n) if kopf_n else 0.0
    verstoesse = []
    for t in sorted(typen_zeilen):
        if t not in inhaltliche:
            continue
        c, n = typen_zeilen[t]
        if n and (c / n) < typ_min:
            verstoesse.append((t, 100.0 * c / n, c, n))
    return {
        "kopf_prozent": kopf * 100.0,
        "kopf_c": kopf_c,
        "kopf_n": kopf_n,
        "kopf_ok": kopf >= kopf_min,
        "typ_verstoesse": verstoesse,
        "erfuellt": kopf >= kopf_min and not verstoesse,
    }


def auswertung(repo, work):
    jobs = lies_jobs(work)
    nach_paar = collections.OrderedDict()
    for j in jobs:
        pid = j["jobid"]
        for p in PAARE + PAARE_FORMVERGLEICH:
            if (pid.startswith(p["id"] + "-")
                    or pid == "loop-" + p["id"]
                    or pid.startswith("loop-" + p["id"] + "-")):
                nach_paar.setdefault(p["id"], []).append(j)
                break
    ergebnis = collections.OrderedDict()
    alle = {p["id"]: p for p in PAARE + PAARE_FORMVERGLEICH}
    for pid in sorted(nach_paar):
        if pid not in alle:
            continue
        p = alle[pid]
        eintrag = {
            "paar": p, "ebenen": {}, "panel": {}, "klassen": [], "det": [],
            "fehlend": [], "loop": None, "poolprobe": None,
        }
        for j in sorted(nach_paar[pid], key=lambda x: x["jobid"]):
            roh, text = lies_log(work, j["jobid"])
            if text is None:
                eintrag["fehlend"].append(j["jobid"])
                continue
            if j["kind"] == "eval":
                d = parse_eval(text)
                if d is None:
                    eintrag["fehlend"].append(j["jobid"] + " (kein Trefferquoten-Block)")
                    continue
                d.update({"jobid": j["jobid"], "ebene": j["ebene"], "typ": j["typ"],
                          "klasse": j["klasse"], "achse": j["achse"],
                          "gestellt": int(j["answers"])})
                if j["ebene"] == "panel":
                    eintrag["panel"][j["jobid"][-1]] = d
                elif j["ebene"] == "klasse":
                    eintrag["klassen"].append(d)
                elif j["ebene"] == "wiederholung":
                    eintrag["det"].append((j["jobid"], d, roh))
                elif j["ebene"] == "poolprobe":
                    eintrag["poolprobe"] = d
                else:
                    eintrag["ebenen"][j["jobid"]] = d
            else:
                lo = parse_loop(text)
                if lo is None:
                    eintrag["fehlend"].append(j["jobid"] + " (kein Panelblock)")
                    continue
                lo["jobid"] = j["jobid"]
                lo["roh"] = roh
                lo["form"] = j["form"]
                eintrag.setdefault("agent", []).append(lo)
                if lo["form"] == "delta" and not lo["jobid"].endswith("-wdh"):
                    eintrag["loop"] = lo
                elif lo["form"] == "absolut":
                    eintrag.setdefault("loop_abs", []).append(lo)
        ergebnis[pid] = eintrag
    return ergebnis


def typzeilen(z, panel):
    """Typzeilen eines Panels: {typ: (richtig, gefragt)} samt Kopfzahl."""
    zeilen = collections.OrderedDict()
    for jobid in sorted(z["ebenen"]):
        d = z["ebenen"][jobid]
        if d["ebene"] != "typ":
            continue
        marke = "-A-t-"
        if marke not in jobid:
            marke = "-B-t-"
        ist_a = "-A-t-" in jobid
        if (panel == "A") != ist_a:
            continue
        zeilen[d["typ"]] = (d["richtig"], d["gefragt"])
    return zeilen

def panel_daten(z, panel):
    """Panel-Zahlen eines Messpaares - aus eval-Logs oder aus dem Loop-Log."""
    p = z["paar"]
    lo = z.get("loop")
    if p["form"] == "delta" and lo is not None:
        liste = lo["typ_a"] if panel == "A" else lo["typ_b"]
        typen = collections.OrderedDict((t, (c, n)) for t, c, n in liste)
        groesse = lo["panel_a_groesse"] if panel == "A" else lo["panel_b_groesse"]
        return {"quelle": "loop", "gestellt": groesse,
                "gefragt": sum(n for _, n in typen.values()),
                "richtig": sum(c for c, _ in typen.values()), "typen": typen}
    d = z["panel"].get(panel)
    typen = typzeilen(z, panel)
    if d is None:
        return {"quelle": "fehlt", "gestellt": 0, "gefragt": 0, "richtig": 0, "typen": typen}
    return {"quelle": "eval", "gestellt": d["gestellt"], "gefragt": d["gefragt"],
            "richtig": d["richtig"], "typen": typen}


def pool(a, b, kurz):
    typen = collections.OrderedDict()
    for quelle in (a, b):
        for t, (c, n) in quelle["typen"].items():
            x = typen.get(t, (0, 0))
            typen[t] = (x[0] + c, x[1] + n)
    inhaltlich = collections.OrderedDict((t, v) for t, v in typen.items() if t not in kurz)
    return {
        "gestellt": a["gestellt"] + b["gestellt"],
        "gefragt": a["gefragt"] + b["gefragt"],
        "richtig": a["richtig"] + b["richtig"],
        "typen": typen,
        "inhaltlich": inhaltlich,
        "inhaltlich_c": sum(c for c, _ in inhaltlich.values()),
        "inhaltlich_n": sum(n for _, n in inhaltlich.values()),
        "kurz_c": sum(c for t, (c, n) in typen.items() if t in kurz),
        "kurz_n": sum(n for t, (c, n) in typen.items() if t in kurz),
    }


def zelle(s):
    """Tabellenzelle: Pipes escapen, Zeilenumbrueche entfernen."""
    return str(s).replace("|", "\\|").replace("\n", " ").strip()


def proz(c, n):
    return ("%.2f" % quote(c, n)) if n else "n/a"


def schreibe_bericht(repo, work, bericht, baselines, aus, kurz_namen, kurz_quelle):
    stands = collections.OrderedDict()
    for p in PAARE + PAARE_FORMVERGLEICH:
        if p["stand"] not in stands:
            stands[p["stand"]] = stand_zeile(repo, p["stand"])
    L = []
    A = L.append
    A("# Voll-Benchmark Logisches und Analytisches Denken - Bericht")
    A("")
    A("Erzeugt von `scripts/benchmark-logik.sh` / `scripts/benchmark-logik.py`.")
    A("**Ohne Zeitstempel** - der Bericht ist bei gleichem Stand und gleicher")
    A("Messvorschrift byte-identisch.")
    A("")
    A("## 1 Die zwei Linien")
    A("")
    A("* **Pflichtlinie 90 % (D-047, Projektregel):** gepoolte KOPFZAHL >= 90 %")
    A("  UND kein inhaltlicher Aufgabentyp < 75 %.")
    A("* **Ziellinie 98 %:** gepoolte Kopfzahl >= 98 %. Die Projektregel nennt")
    A("  keine Typbedingung fuer 98 %; dieses Werkzeug berichtet deshalb BEIDE")
    A("  Lesarten (nur Kopfzahl und Kopfzahl plus kein Typ < 98 %).")
    A("")
    A("Kurzantwort-Typen (aus der Capability-Datei, nicht aus dem Code): %s %s"
      % (", ".join("`%s`" % k for k in kurz_namen) or "(keine)", kurz_quelle))
    A("")
    A("## 2 Gemessene Staende")
    A("")
    A("| Schluessel | Pfad | SHA-256 | Bytes | trainiert auf | Rolle |")
    A("|---|---|---|---|---|---|")
    for k, s in stands.items():
        warn = ""
        if s["sha_soll"] and s["sha256"] != s["sha_soll"]:
            warn = " **ABWEICHUNG zur Auftragsvorgabe %s**" % s["sha_soll"][:16]
        A("| `%s` | `%s` | `%s` | %d | %s | %s%s |"
          % (k, zelle(s["pfad"]), s["sha256"], s["groesse"], zelle(s["korpus"]),
             zelle(s["rolle"]), warn))
    A("")
    A("## 3 Abdeckung")
    A("")
    A("| Familie | Typen | Datensaetze | stride | Versatz A | Versatz B | Panel A | Panel B | disjunkt | Typregel | Laengenachse |")
    A("|---|---|---|---|---|---|---|---|---|---|---|")
    for e in bericht:
        fam = e["familie"]
        A("| `%s` | %d | %d | %d | %d | %d | %d | %d | %s | %s | %s |"
          % (fam["id"], len(e["typen"]), e["n"], e["stride"], e["versatz_a"], e["versatz_b"],
             e["panel_a"], e["panel_b"], "ja" if e["disjunkt"] else "NEIN",
             "Manifest-Zahlung" if e["stimmt"] else "ABWEICHUNG",
             zelle(fam["achsen_text"])))
    A("")
    A("| Familie | Datensaetze | davon ins Fenster (%d Byte) | Anteil |" % MAX_SEQ_LEN)
    A("|---|---|---|---|")
    for e in bericht:
        A("| `%s` | %d | %d | %s %% |"
          % (e["familie"]["id"], e["n"], e["passt_ins_fenster"],
             proz(e["passt_ins_fenster"], e["n"])))
    A("")
    A("**Warum die Fensterspalte zaehlt:** `shiro eval --answers` ueberspringt")
    A("Datensaetze, deren Prompt nicht in `max_seq_len = %d` passt" % MAX_SEQ_LEN)
    A("(`src/main.rs answers_core`). Bei Quantoren, Semantik und Format faellt damit")
    A("ein Teil des Panels weg - die Zeile `gestellt` gegen `gefragt` weist das aus.")
    A("")
    A("### 3.1 Typen je Familie (aus den Bytes erkannt)")
    A("")
    A("| Familie | Typ | Datensaetze | Antwortraum |")
    A("|---|---|---|---|")
    for e in bericht:
        for t in sorted(e["typen"]):
            A("| `%s` | `%s` | %d | %d |"
              % (e["familie"]["id"], t, e["typen"][t], e["antwortraeume"].get(t, 0)))
    A("")
    A("### 3.2 Typen mit kleinem Antwortraum, die NICHT als Kurzantwort gefuehrt werden")
    A("")
    A("Die Projektregel nennt Kurzantwort-Typen in der Capability-Datei. Typen mit")
    A("einem Antwortraum von 1 bis 2 Bytes sind dort NICHT alle erfasst - ihre Quote")
    A("ist billig und darf nicht als Faehigkeitsnachweis gelesen werden.")
    A("")
    A("| Familie | Typ | Antwortraum | in der Kurzantwort-Liste |")
    A("|---|---|---|---|")
    for e in bericht:
        for t in sorted(e["antwortraeume"]):
            if e["antwortraeume"][t] <= 2:
                A("| `%s` | `%s` | %d | %s |"
                  % (e["familie"]["id"], t, e["antwortraeume"][t],
                     "ja" if t in kurz_namen else "**NEIN**"))
    A("")
    A("### 3.3 Nicht gebaut")
    A("")
    A("| Familie | Offener Punkt | Grund |")
    A("|---|---|---|")
    for n in NICHT_GEBAUT:
        A("| `%s` | %s | %s |" % (n["id"], n["punkt"], zelle(n["grund"])))
    A("")
    A("Diese Familie wird **nicht als 0 % gerechnet**: eine fehlende Familie ist")
    A("keine Quote. Sie fehlt in der Kopfzahl und in der Typzeilen-Tafel.")
    A("")

    # --- Ergebnis je Messpaar ------------------------------------------
    A("## 4 Ergebnis je Messpaar")
    A("")
    kurz_je_familie = {}
    for e in bericht:
        kurz_je_familie[e["familie"]["id"]] = set(kurz_namen) & set(e["typen"])
    for pid, z in aus.items():
        p = z["paar"]
        A("### 4.%d `%s` - Stand `%s`, Familie `%s`, Form `%s`"
          % (list(aus).index(pid) + 1, pid, p["stand"], p["familie"], p["form"]))
        A("")
        A("%s" % zelle(p["rolle"]))
        A("")
        kurz = kurz_je_familie.get(p["familie"], set())
        pa, pb = panel_daten(z, "A"), panel_daten(z, "B")
        pq = pool(pa, pb, kurz)
        A("| Kennzahl | Panel A | Panel B | gepoolt |")
        A("|---|---|---|---|")
        A("| gestellt | %d | %d | %d |" % (pa["gestellt"], pb["gestellt"], pq["gestellt"]))
        A("| gefragt | %d | %d | %d |" % (pa["gefragt"], pb["gefragt"], pq["gefragt"]))
        A("| richtig | %d | %d | %d |" % (pa["richtig"], pb["richtig"], pq["richtig"]))
        A("| **Kopfzahl** | **%s %%** | **%s %%** | **%s %%** |"
          % (proz(pa["richtig"], pa["gefragt"]), proz(pb["richtig"], pb["gefragt"]),
             proz(pq["richtig"], pq["gefragt"])))
        A("| inhaltlich | %s %% | %s %% | %s %% |"
          % (proz(pa["richtig"] - sum(c for t, (c, n) in pa["typen"].items() if t in kurz),
                  pa["gefragt"] - sum(n for t, (c, n) in pa["typen"].items() if t in kurz)),
             proz(pb["richtig"] - sum(c for t, (c, n) in pb["typen"].items() if t in kurz),
                  pb["gefragt"] - sum(n for t, (c, n) in pb["typen"].items() if t in kurz)),
             proz(pq["inhaltlich_c"], pq["inhaltlich_n"])))
        A("")
        A("| Typ | gefragt A | richtig A | Quote A | gefragt B | richtig B | Quote B | gefragt gepoolt | richtig gepoolt | Quote gepoolt |")
        A("|---|---|---|---|---|---|---|---|---|---|")
        for t in sorted(pq["typen"]):
            ca, na = pa["typen"].get(t, (0, 0))
            cb, nb = pb["typen"].get(t, (0, 0))
            marke = " (Kurzantwort)" if t in kurz else ""
            A("| `%s`%s | %d | %d | %s %% | %d | %d | %s %% | %d | %d | %s %% |"
              % (t, marke, na, ca, proz(ca, na), nb, cb, proz(cb, nb),
                 na + nb, ca + cb, proz(ca + cb, na + nb)))
        A("")
        u = urteil_linien(pq["richtig"], pq["gefragt"], pq["typen"], pq["inhaltlich"], TYP_MIN, KOPF_MIN)
        A("* Pflichtlinie 90 %% (D-047): Kopfzahl %s %%, Typverstoesse %s -> **%s**"
          % (proz(u["kopf_c"], u["kopf_n"]),
             ", ".join("`%s` %s %%" % (t, proz(c, n)) for t, _, c, n in u["typ_verstoesse"]) or "keine",
             "ERFUELLT" if u["erfuellt"] else "NICHT ERFUELLT"))
        ziel_kopf = quote(pq["richtig"], pq["gefragt"]) >= ZIEL_MIN * 100.0
        ziel_typ = [t for t in sorted(pq["inhaltlich"])
                    if pq["inhaltlich"][t][1] and quote(*pq["inhaltlich"][t]) < ZIEL_MIN * 100.0]
        A("* Ziellinie 98 %% (nur Kopfzahl): **%s**; zusaetzlich kein Typ < 98 %%: **%s**"
          % ("erreicht" if ziel_kopf else "nicht erreicht",
             "erfuellt" if not ziel_typ else "nicht erfuellt (%d Typen)" % len(ziel_typ)))
        if z["fehlend"]:
            A("")
            A("* Fehlende Messungen in diesem Paar: %s" % ", ".join("`%s`" % f for f in sorted(z["fehlend"])))
        A("")
        if z["klassen"]:
            A("| Laengenachse | Klasse | Typ | gestellt | gefragt | richtig | Quote |")
            A("|---|---|---|---|---|---|---|")
            for d in sorted(z["klassen"], key=lambda x: (x["achse"], x["klasse"], x["typ"])):
                A("| %s | %s | %s | %d | %d | %d | %s %% |"
                  % (zelle(d["achse"]), zelle(d["klasse"]),
                     "`%s`" % d["typ"] if d["typ"] else "(alle)",
                     d["gestellt"], d["gefragt"], d["richtig"], proz(d["richtig"], d["gefragt"])))
            A("")
    return "\n".join(L) + "\n"

def bericht_baselines(baselines, kurz_namen):
    L = []
    A = L.append
    A("## 5 Kurzstrategie-Baselines - an den BYTES gerechnet")
    A("")
    A("Vorbild sind `/home/changchi/r39/kurzstrategie.py` und die 4-Typ-Fassung")
    A("`kurzstrategie4.py`. Gerechnet wird auf **denselben Fragen wie die Messung**")
    A("(Panel A, Panel B und gepoolt). **Panel V** ist der GANZE Held-out-Shard; er")
    A("dient allein dem Quervergleich mit dem Projekt-Vorbild und traegt die Linien NICHT.")
    A("Die Lehre der 4-Typ-Fassung ist eingebaut:")
    A("der Typ kommt bei formel/algebra/logik aus der FRAGEZEILE, nicht aus der")
    A("ersten Aufgabenzeile - sonst fallen `fkt-mittel` und `fkt-steig` zusammen.")
    A("")
    A("Strategien: **Mehrheitsantwort** (haeufigste Antwort der Gruppe), **immer 0**,")
    A("**erste Zahl** der Aufgabe, **erste Eingabe** (`x = ...` bzw. `Belegung: x = ...`),")
    A("**Schritt 1**, und - wo der Korpus es mitbringt - das **Kurzverfahren** des")
    A("Generators (seq).")
    A("")
    for fam_id in sorted(set(b["familie"] for b in baselines)):
        A("### 5.%s Familie `%s`" % (str(sorted(set(b["familie"] for b in baselines)).index(fam_id) + 1), fam_id))
        A("")
        spalten = ["Mehrheitsantwort", "immer 0", "erste Zahl", "erste Eingabe", "Schritt 1"]
        if any("Kurzverfahren" in b["treffer"] for b in baselines if b["familie"] == fam_id):
            spalten.append("Kurzverfahren")
        A("| Panel | Typ | n | Antwortraum | " + " | ".join(spalten) + " |")
        A("|---|---|---|---|" + "---|" * len(spalten))
        for b in baselines:
            if b["familie"] != fam_id:
                continue
            werte = []
            for s in spalten:
                c = b["treffer"].get(s)
                werte.append(proz(c, b["n"]) + " %" if c is not None else "n/a")
            A("| %s | `%s` | %d | %d | %s |"
              % (b["panel"], b["typ"], b["n"], b["antwortraum"], " | ".join(werte)))
        A("")
        klein = [b for b in baselines if b["familie"] == fam_id and b["panel"] == "AB" and b["antwortraum"] <= 1]
        A("Antwortraum 1 (Verstoss gegen ein Baugate): %s"
          % (", ".join("`%s`" % b["typ"] for b in klein) if klein else "keiner"))
        A("")
    return "\n".join(L) + "\n"


def bericht_determinismus(aus, work):
    L = []
    A = L.append
    A("## 6 Determinismus-Proben")
    A("")
    A("Projektregel (exp-013/015): Panel A wird ZWEIMAL gemessen; die Ergebnisse")
    A("muessen byte-identisch sein, sonst ist die Messung **UNGUELTIG**.")
    A("")
    A("| Paar | Quelle | erster Lauf | zweiter Lauf | Logdatei byte-identisch | Messwerte identisch | Urteil |")
    A("|---|---|---|---|---|---|---|")
    for pid, z in aus.items():
        for j in sorted(z["ebenen"]):
            pass
        # eval-Paare: -A gegen -A-wdh
        a = z["panel"].get("A")
        w = None
        for jobid, d, roh in z["det"]:
            w = (jobid, d, roh)
        if a is not None:
            if w is None:
                A("| `%s` | eval | `%s-A` | `%s-A-wdh` fehlt | - | - | **nicht gemessen** |"
                  % (pid, pid, pid))
            else:
                gleiche_zahlen = (a["gefragt"] == w[1]["gefragt"] and a["richtig"] == w[1]["richtig"])
                byte_gleich = None
                roh_a, _ = lies_log(work, w[0].replace("-wdh", ""))
                if roh_a is not None and w[2] is not None:
                    byte_gleich = (roh_a == w[2])
                urteil = ("GUELTIG" if (byte_gleich and gleiche_zahlen)
                          else "GUELTIG MIT VORBEHALT" if gleiche_zahlen
                          else "UNGUELTIG")
                A("| `%s` | eval | `%s` | `%s` | %s | %s | **%s** |"
                  % (pid, w[0].replace("-wdh", ""), w[0],
                     "ja" if byte_gleich else "nein",
                     "ja" if gleiche_zahlen else "NEIN", urteil))
        lo = z.get("loop")
        if lo is not None:
            agent = [x for x in z.get("agent", []) if lo["jobid"].replace("-wdh", "") in x["jobid"]]
            zw = [x for x in z.get("agent", []) if x["jobid"].endswith("-wdh")
                  and x["form"] == lo["form"]]
            if zw:
                byte_gleich = (lo["roh"] == zw[0]["roh"])
                gleich = (lo["typ_a"] == zw[0]["typ_a"] and lo["typ_b"] == zw[0]["typ_b"])
                A("| `%s` | agent capability | `%s` | `%s` | %s | %s | **%s** |"
                  % (pid, lo["jobid"], zw[0]["jobid"], "ja" if byte_gleich else "nein",
                     "ja" if gleich else "NEIN",
                     "GUELTIG" if (byte_gleich and gleich) else "UNGUELTIG"))
            else:
                A("| `%s` | agent capability | `%s` | fehlt | - | - | **nicht gemessen** |"
                  % (pid, lo["jobid"]))
    A("")
    A("Der Loop fuehrt seine eigene Determinismus-Probe mit; sie steht in der")
    A("Registry (`deterministisch`). Dieser Bericht prueft sie am LOG nach.")
    A("")
    return "\n".join(L) + "\n"


def bericht_integritaet(aus, bericht):
    L = []
    A = L.append
    A("## 7 Integritaetsproben")
    A("")
    A("1. **Typensumme gegen Kopfzahl.** Die Typ-Unterkorpora sind eine Zerlegung")
    A("   des Panels. Summe der Typzeilen muss die Kopfzahl des Panels ergeben -")
    A("   sonst hat das Werkzeug etwas anderes bereitgestellt als es behauptet.")
    A("2. **Ausgangspruefung.** Jede Typregel muss die `typ.*`-Zahlung des Manifests")
    A("   exakt reproduzieren; Panel A und B muessen disjunkt sein.")
    A("")
    A("| Paar | Panel | Kopfzahl (c/n) | Summe der Typzeilen (c/n) | stimmt |")
    A("|---|---|---|---|---|")
    for pid, z in aus.items():
        for panel in ("A", "B"):
            d = panel_daten(z, panel)
            sc = sum(c for c, _ in d["typen"].values())
            sn = sum(n for _, n in d["typen"].values())
            A("| `%s` | %s | %d/%d | %d/%d | %s |"
              % (pid, panel, d["richtig"], d["gefragt"], sc, sn,
                 "ja" if (sc == d["richtig"] and sn == d["gefragt"]) else "**NEIN**"))
    A("")
    A("| Familie | Typregel reproduziert die Manifest-Zahlung | Panel A und B disjunkt |")
    A("|---|---|---|")
    for e in bericht:
        A("| `%s` | %s | %s |" % (e["familie"]["id"],
                                  "ja" if e["stimmt"] else "**NEIN**",
                                  "ja" if e["disjunkt"] else "**NEIN**"))
    A("")
    return "\n".join(L) + "\n"


def bericht_linien(aus, kurz_je_familie):
    L = []
    A = L.append
    A("## 8 Linienuebersicht")
    A("")
    A("| Paar | Stand | Familie | Form | gefragt | Kopfzahl | Pflicht 90 % | Ziellinie 98 % | schwaechster Typ |")
    A("|---|---|---|---|---|---|---|---|---|")
    for pid, z in aus.items():
        p = z["paar"]
        kurz = kurz_je_familie.get(p["familie"], set())
        pa, pb = panel_daten(z, "A"), panel_daten(z, "B")
        pq = pool(pa, pb, kurz)
        u = urteil_linien(pq["richtig"], pq["gefragt"], pq["typen"], pq["inhaltlich"], TYP_MIN, KOPF_MIN)
        schwach = ""
        if pq["inhaltlich"]:
            t = min(pq["inhaltlich"], key=lambda x: quote(*pq["inhaltlich"][x]) if pq["inhaltlich"][x][1] else 999)
            c, n = pq["inhaltlich"][t]
            schwach = "`%s` %s %% (%d/%d)" % (t, proz(c, n), c, n)
        A("| `%s` | `%s` | `%s` | %s | %d | %s %% | %s | %s | %s |"
          % (pid, p["stand"], p["familie"], p["form"], pq["gefragt"],
             proz(pq["richtig"], pq["gefragt"]),
             "ERFUELLT" if u["erfuellt"] else "NICHT ERFUELLT",
             "erreicht" if quote(pq["richtig"], pq["gefragt"]) >= ZIEL_MIN * 100.0 else "offen",
             schwach))
    A("")
    A("**Was diese Tafel NICHT sagt.** Die Zeilen sind je Stand und Familie")
    A("getrennt; ein Vergleich zwischen zwei Staenden ist nur bei gleicher Familie,")
    A("gleicher Antwortform und gleicher Panelgroesse zulaessig.")
    A("")
    return "\n".join(L) + "\n"

# -------------------------------------------------------------------------
# 13 Plan-Ausgabe (nur lesen)
# -------------------------------------------------------------------------


def plan_ausgabe(repo, mit_formvergleich=False):
    bericht = abdeckung(repo)
    kurz_namen, kurz_quelle = kurzantwort_typen(repo)
    print("Voll-Benchmark Logisches und Analytisches Denken - Abdeckung (nur gelesen)")
    print("Pflichtlinie 90 % (D-047): Kopfzahl >= 90 % UND kein inhaltlicher Typ < 75 %")
    print("Ziellinie 98 %")
    print("Kurzantwort-Typen %s: %s" % (kurz_quelle, ", ".join(kurz_namen) or "(keine)"))
    print()
    gesamt_typen = 0
    for e in bericht:
        fam = e["familie"]
        gesamt_typen += len(e["typen"])
        print("Familie %-10s %s" % (fam["id"], fam["titel"]))
        print("  Korpus        : %s  (%d Datensaetze)" % (fam["korpus"], e["n"]))
        print("  Typen (%2d)    : %s" % (len(e["typen"]), ", ".join(sorted(e["typen"]))))
        print("  Typregel      : %s" % ("Manifest-Zahlung exakt reproduziert"
                                        if e["stimmt"] else "ABWEICHUNG - Abbruch"))
        print("  Panel         : stride %d, Versatz A %d / B %d, %d + %d Fragen, disjunkt %s"
              % (e["stride"], e["versatz_a"], e["versatz_b"], e["panel_a"], e["panel_b"],
                 "ja" if e["disjunkt"] else "NEIN"))
        print("  Fenster %d    : %d von %d Datensaetzen messbar (%s %%)"
              % (MAX_SEQ_LEN, e["passt_ins_fenster"], e["n"], proz(e["passt_ins_fenster"], e["n"])))
        raeume = e["antwortraeume"]
        klein = sorted(t for t in raeume if raeume[t] <= 2)
        print("  Antwortraeume : kleinster %s, groesster %s; Typen mit Antwortraum <= 2: %s"
              % (min(raeume.values()), max(raeume.values()), ", ".join(klein) or "keine"))
        for t in kurz_namen:
            if t in e["typen"]:
                if raeume.get(t, 0) > 2:
                    print("  WARNUNG       : `%s` steht in der Kurzantwort-Liste, hat im "
                          "Korpus aber Antwortraum %d - die Namensregel traegt hier NICHT"
                          % (t, raeume[t]))
                else:
                    print("  Kurzantwort   : `%s` Antwortraum %d - Namensregel traegt" % (t, raeume[t]))
        if fam["laengenachse"] == "keine":
            print("  Laengenachse  : KEINE (%s)" % fam["achsen_text"])
        else:
            print("  Laengenachse  : %s" % fam["achsen_text"])
            for t in sorted(e["achsen"]):
                roh = e["achsen"][t]
                ordnung = stufen_namen(fam, t)
                kl = []
                for name in ordnung:
                    for x in sorted(roh):
                        if (x == name or x.endswith(" | " + name)) and x not in kl:
                            kl.append(x)
                for x in sorted(roh):
                    if x not in kl:
                        kl.append(x)
                if len(kl) >= 3:
                    print("    `%s` : %s" % (t, " / ".join(kl)))
                else:
                    print("    `%s` : nur %d Klasse(n) %s -> keine Achse"
                          % (t, len(kl), kl or "[]"))
    print()
    paare = PAARE + (PAARE_FORMVERGLEICH if mit_formvergleich else [])
    n_typ = sum(len(e["typen"]) for e in bericht if any(p["familie"] == e["familie"]["id"] for p in paare))
    print("Familien: %d, Typnamen gesamt: %d" % (len(bericht), gesamt_typen))
    pflicht = [p for p in paare if p["pflicht"]]
    kuer = [p for p in paare if not p["pflicht"]]
    print("Messpaare: %d Pflicht, %d Zusatz (--mit-formvergleich)" % (len(pflicht), len(kuer)))
    for p in paare:
        print("  %-20s Stand %-8s Familie %-10s Form %-7s %-7s %s"
              % (p["id"], p["stand"], p["familie"], p["form"],
                 "Pflicht" if p["pflicht"] else "Kuer", p["rolle"]))
    print()
    print("NICHT GEBAUT (wird nicht als 0 % gerechnet):")
    for n in NICHT_GEBAUT:
        print("  %-12s %s (%s) - %s" % (n["id"], n["titel"], n["punkt"], n["grund"]))
    return bericht, kurz_namen, kurz_quelle


# -------------------------------------------------------------------------
# 14 Selbstprobe (ohne schwere Laeufe)
# -------------------------------------------------------------------------

BEISPIEL_EVAL = """Backend: cuda (/home/changchi/.cache/shiro-target-cuda/release/shiro)
Konfiguration : configs/train.micro.toml
Datenordner   : /tmp/bench/korpora/seq/A
Messung       : Batch 16 x 512 Bytes, pad 0, ohne Gewichtsaktualisierung
Seed          : 1337
Modell        : checkpoints/kombi-step19200.safetensors

--- Messung (ohne Gewichtsaktualisierung) ---
Modell      Mittel-Loss Perplexitaet  Batches   Zielposit. Datensaetze
Baseline         5.7449        312.6        1       1217601          480
Trainiert        0.5071          1.7        1       1217601          480
Differenz       -5.2377       -310.9

Urteil        : trainiert ist besser als die Baseline (Loss)

--- Trefferquote (greedy, byte-genau) ---
Auswahl       : jeder 1. Datensatz des Shards
Gefragt       : 480 Datensaetze, davon richtig: 437 (91.0 %)
"""

BEISPIEL_LOOP = """Backend: cuda (/home/changchi/.cache/shiro-target-cuda/release/shiro)
Antwortform   : DELTA - reparatur-Faelle der Panels umgeschrieben (A 63 von 480, B 60 von 480); alle anderen Typen unveraendert
=== Agentischer Loop: sequence_reasoning (L2/L3 (Zahlenfolgen und Regelausfuehrung)) ===
Referenzsatz  : Panel A 480 Datensaetze, jeder 10. aus data/seq/val
                Panel B 480 Datensaetze, jeder 10. mit Versatz 5 (disjunkt)
Replikation   : Annahme nur, wenn ALLE 2 Panels tragen (pessimistisch, Nutzerauflage)
Gewichte      : checkpoints/agent-sequence_reasoning-from7600-i1-step12400.safetensors
Trainingsband : 82.1 % inhaltlich (23 von 28), 100.0 % Kurzantwort (4 von 4)
   Band-Typ luecke      5/5 richtig

--- Iteration 1/3 (sequence_reasoning-i1) ---
Baseline      : inhaltlich 88.1 % (362 von 411) | Kurzantwort 98.6 % (68 von 69) | gesamt 89.6 % | Antwortform 95.0 % | Score 0.535
   Typ luecke      49/51 richtig
   Typ naechste    52/59 richtig
   Typ vergleich   68/69 richtig
Baseline B    : inhaltlich 87.1 % (372 von 427) | gesamt 88.5 % | Score 0.532
   Typ B luecke    58/58 richtig
   Typ B vergleich 53/53 richtig
"""


def selftest(repo):
    fehler = []
    def pruefe(name, ist, soll):
        if ist != soll:
            fehler.append("%s: ist %r, soll %r" % (name, ist, soll))
        print("  %-52s %s" % (name, "ok" if ist == soll else "FEHLER (%r statt %r)" % (ist, soll)))

    print("Selbstprobe ohne Messlaeufe")
    print("\n[1] Panel-Auswahl (Regel aus src/agent.rs reference_set_ab)")
    ia, stride = panel_indizes(5000, 480, 0)
    ib, _ = panel_indizes(5000, 480, panel_versatz(stride))
    pruefe("5000 Datensaetze -> stride", stride, 10)
    pruefe("Versatz Panel B", panel_versatz(10), 5)
    pruefe("Panel A Groesse", len(ia), 480)
    pruefe("Panel B Groesse", len(ib), 480)
    pruefe("Panel A erster Index", ia[0], 0)
    pruefe("Panel B erster Index", ib[0], 5)
    pruefe("disjunkt", len(set(ia) & set(ib)), 0)
    pruefe("gepoolt 960 Fragen", len(set(ia) | set(ib)), 960)
    i2, s2 = panel_indizes(3000, 480, 0)
    i2b, _ = panel_indizes(3000, 480, panel_versatz(s2))
    pruefe("3000 -> stride", s2, 6)
    pruefe("3000 -> Versatz B", panel_versatz(s2), 3)
    pruefe("3000 -> disjunkt", len(set(i2) & set(i2b)), 0)

    print("\n[2] Log-Parser")
    d = parse_eval(BEISPIEL_EVAL)
    pruefe("eval gefragt", d["gefragt"], 480)
    pruefe("eval richtig", d["richtig"], 437)
    pruefe("eval Prozent", d["prozent"], 91.0)
    pruefe("eval stride", d["stride"], 1)
    lo = parse_loop(BEISPIEL_LOOP)
    pruefe("loop Panel-A-Groesse", lo["panel_a_groesse"], 480)
    pruefe("loop Versatz B", lo["versatz_b"], 5)
    pruefe("loop Antwortform beginnt mit DELTA", lo["antwortform"].startswith("DELTA"), True)
    pruefe("loop Typzeilen A", lo["typ_a"], [("luecke", 49, 51), ("naechste", 52, 59), ("vergleich", 68, 69)])
    pruefe("loop Typzeilen B", lo["typ_b"], [("luecke", 58, 58), ("vergleich", 53, 53)])
    pruefe("loop Kurzantwort A", (lo["summe_a"]["kurz_c"], lo["summe_a"]["kurz_n"]), (68, 69))
    pruefe("loop gesamt A", lo["summe_a"]["gesamt"], 89.6)

    print("\n[3] Linien")
    u = urteil_linien(437, 480, {"a": (437, 480)}, {"a"}, 0.75, 0.90)
    pruefe("91,0 % ohne Typverstoss -> erfuellt", u["erfuellt"], True)
    u = urteil_linien(427, 480, {"a": (427, 480)}, {"a"}, 0.75, 0.90)
    pruefe("89,0 % -> nicht erfuellt", u["erfuellt"], False)
    u = urteil_linien(440, 480, {"a": (440, 480), "b": (30, 40)}, {"a", "b"}, 0.75, 0.90)
    pruefe("91,7 % aber Typ 75,0 % -> erfuellt", u["erfuellt"], True)
    u = urteil_linien(440, 480, {"a": (440, 480), "b": (29, 40)}, {"a", "b"}, 0.75, 0.90)
    pruefe("91,7 % aber Typ 72,5 % -> NICHT erfuellt", u["erfuellt"], False)
    u = urteil_linien(430, 480, {"a": (430, 480), "k": (5, 50)}, {"a"}, 0.75, 0.90)
    pruefe("Kurzantwort-Typ zaehlt nicht in den Typverstoss", len(u["typ_verstoesse"]), 0)

    print("\n[4] Pruefsumme gegen einen ECHTEN Shard")
    shard = os.path.join(repo, "data", "seq", "val", "shard-0000.bin")
    if os.path.isfile(shard):
        man = lies_manifest(os.path.join(repo, "data", "seq", "val", "manifest.json"))
        with open(shard, "rb") as fh:
            roh = fh.read()
        pruefe("FNV-1a-64 == Manifest-Pruefsumme",
               "%016x" % fnv1a64(roh), man["shards"][0]["checksum"])
    else:
        print("  (Shard fehlt - uebersprungen)")

    print("\n[5] Shard-Rundlauf")
    tmp = tempfile.mkdtemp(prefix="benchmark-selftest-")
    recs = [b"abc", bytes(range(256)), b"x" * 4096]
    pruef, nutz = lege_korpus_an(os.path.join(tmp, "k"), recs, 1, "selftest",
                                 collections.Counter({"typ.x": 3}))
    zurueck = lies_shard(os.path.join(tmp, "k", "shard-0000.bin"))
    pruefe("Rundlauf Datensaetze", zurueck, recs)
    pruefe("Nutzbytes", nutz, sum(len(r) for r in recs))
    shutil.rmtree(tmp, ignore_errors=True)

    print("\n[6] Typregeln gegen die Manifest-Zahlung")
    bericht = abdeckung(repo)
    for e in bericht:
        pruefe("Familie %-10s Typzahl" % e["familie"]["id"], e["stimmt"], True)

    print("\nErgebnis: %s" % ("alle Proben bestanden" if not fehler else "%d FEHLER" % len(fehler)))
    for f in fehler:
        print("  - %s" % f)
    return 1 if fehler else 0

# -------------------------------------------------------------------------
# 15 Hauptprogramm
# -------------------------------------------------------------------------


def cmd_baseline(repo, work):
    bericht = abdeckung(repo)
    baselines = kurz_baselines(repo, bericht)
    kurz_namen, kurz_quelle = kurzantwort_typen(repo)
    with open(os.path.join(work, "baselines.json"), "w", encoding="utf-8", newline="\n") as fh:
        json.dump(baselines, fh, ensure_ascii=True, indent=1, sort_keys=True)
        fh.write("\n")
    print(bericht_baselines(baselines, kurz_namen))
    print("Baselines geschrieben: %s" % os.path.join(work, "baselines.json"))
    return baselines


def cmd_evaluate(repo, work):
    bericht = abdeckung(repo)
    kurz_namen, kurz_quelle = kurzantwort_typen(repo)
    pfad = os.path.join(work, "baselines.json")
    if os.path.isfile(pfad):
        baselines = json.load(open(pfad, "r", encoding="utf-8"))
    else:
        baselines = kurz_baselines(repo, bericht)
    aus = auswertung(repo, work)
    kurz_je_familie = {}
    for e in bericht:
        kurz_je_familie[e["familie"]["id"]] = set(kurz_namen) & set(e["typen"])
    text = schreibe_bericht(repo, work, bericht, baselines, aus, kurz_namen, kurz_quelle)
    text += bericht_baselines(baselines, kurz_namen)
    text += bericht_determinismus(aus, work)
    text += bericht_integritaet(aus, bericht)
    text += bericht_linien(aus, kurz_je_familie)
    ziel = os.path.join(work, "report.md")
    with open(ziel, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print("Bericht geschrieben: %s (%d Bytes, %d Zeilen)"
          % (ziel, len(text.encode("utf-8")), text.count("\n")))
    print()
    print("Linienuebersicht (gepoolt, 2 x %d Fragen):" % PANEL_N)
    for pid, z in aus.items():
        p = z["paar"]
        kurz = kurz_je_familie.get(p["familie"], set())
        pq = pool(panel_daten(z, "A"), panel_daten(z, "B"), kurz)
        u = urteil_linien(pq["richtig"], pq["gefragt"], pq["typen"], pq["inhaltlich"], TYP_MIN, KOPF_MIN)
        print("  %-20s %-8s %-10s %-7s %3d/%3d = %6s %%  Pflicht %s  Ziel %s"
              % (pid, p["stand"], p["familie"], p["form"], pq["richtig"], pq["gefragt"],
                 proz(pq["richtig"], pq["gefragt"]),
                 "ERFUELLT" if u["erfuellt"] else "offen",
                 "erreicht" if quote(pq["richtig"], pq["gefragt"]) >= ZIEL_MIN * 100.0 else "offen"))
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description="Voll-Benchmark Logisches und Analytisches Denken")
    ap.add_argument("befehl", choices=["plan", "stage", "baseline", "evaluate", "selftest",
                                       "stand", "staende"])
    ap.add_argument("--repo", default=".")
    ap.add_argument("--work", default="/tmp/shiro-voll-benchmark")
    ap.add_argument("--nur", default="", help="nur diese Messpaare (Komma-getrennt)")
    ap.add_argument("--mit-formvergleich", action="store_true",
                    help="beide Antwortformen auf DEMSELBEN Stand messen (Zusatzpaare)")
    ap.add_argument("--mit-poolprobe", action="store_true",
                    help="zusaetzlich A+B in EINEM Lauf messen (Gegenprobe zur Summe)")
    ap.add_argument("--key", default="", help="Standschluessel fuer `stand`")
    a = ap.parse_args(argv)
    repo = os.path.abspath(a.repo)

    if a.befehl == "stand":
        if a.key not in STANDS:
            print("unbekannter Stand: %s (bekannt: %s)" % (a.key, ", ".join(STANDS)), file=sys.stderr)
            return 4
        print(os.path.join(repo, STANDS[a.key]["pfad"]))
        return 0
    if a.befehl == "staende":
        print("Stand      Pfad                                                  Bytes      SHA-256")
        for key in STANDS:
            z = stand_zeile(repo, key)
            print("%-10s %-52s %9d  %s" % (key, z["pfad"], z["groesse"], z["sha256"]))
            if z["sha_soll"] and z["sha256"] != z["sha_soll"]:
                print("%-10s ABWEICHUNG von der Auftragsvorgabe: %s" % ("", z["sha_soll"]))
        return 0
    if a.befehl == "plan":
        plan_ausgabe(repo, a.mit_formvergleich)
        return 0
    if a.befehl == "selftest":
        return selftest(repo)
    if a.befehl == "stage":
        nur = [x for x in a.nur.split(",") if x] or None
        jobs, bericht = stelle_bereit(repo, a.work, nur, a.mit_formvergleich, a.mit_poolprobe)
        fragen = 0
        for j in jobs:
            if j["kind"] == "eval":
                fragen += int(j["answers"])
        print("Bereitgestellt: %d Jobs, davon %d eval-Laeufe mit %d Fragen insgesamt"
              % (len(jobs), sum(1 for j in jobs if j["kind"] == "eval"), fragen))
        print("Jobliste      : %s" % os.path.join(a.work, "jobs.tsv"))
        print("Korpora       : %s" % os.path.join(a.work, "korpora"))
        for j in jobs:
            print("  %-34s %-8s %-46s %5d %s"
                  % (j["jobid"], j["kind"], j["corpus"], int(j["answers"]), j["stand"]))
        return 0
    if a.befehl == "baseline":
        os.makedirs(a.work, exist_ok=True)
        cmd_baseline(repo, a.work)
        return 0
    if a.befehl == "evaluate":
        return cmd_evaluate(repo, a.work)
    return 2


if __name__ == "__main__":
    sys.exit(main())