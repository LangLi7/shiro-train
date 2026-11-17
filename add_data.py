#!/usr/bin/env python3
"""add_data.py - Generator fuer den Byte-Level-Additionsbenchmark (Spaltenrechnen mit Uebertrag).

SCHEMA (verbindlich, siehe auch README.md und IQ_TEST.md)
=========================================================
Ein Datensatz ist eine reine ASCII-Byte-Sequenz im Projektformat:

    <AUFGABE> 47+58 <DENKWEG> E 7+8=15 -> 5 u1 ; Z 4+5+1=10 -> 0 u1 ; H 0+0+1=1 -> 1 u0 ; R 105 <ANTWORT> 105 <ENDE>

Strukturmarken (src/data/bytes.rs): 0xFE AUFGABE, 0xFD DENKWEG, 0xFC ANTWORT, 0xFF ENDE.
Kein Tokenizer. Ziffern sind einzelne Bytes (0x30-0x39).

Spaltenrechnen wie in der Grundschule / im Taschenrechner-Werk:

  * Eine Spalte je Stelle, BEGINNEND BEI DER EINERSTELLE (rechts). Genau in dieser
    Reihenfolge wird der Denkweg geschrieben - also links nach rechts, wie das Modell
    generiert. Die Operanden selbst stehen in der Aufgabe in normaler Leserichtung.
  * Spaltenbuchstabe: E Einer, Z Zehner, H Hunderter, T Tausender, K Zehntausender.
  * Pro Spalte: `<Stelle> <a>+<b>[+1]=<Summe> -> <Ziffer> u<Uebertrag>`.
    Das "+1" ist der UEBERTRAG AUS DER VORIGEN SPALTE und wird immer explizit
    geschrieben, wenn er 1 ist. `u0`/`u1` ist der Uebertrag, der weitergeht.
  * Braucht die Summe eine Stelle mehr als die Operanden, kommt genau eine
    zusaetzliche Spalte dazu (z. B. H 0+0+1=1 -> 1 u0).
  * `; R <Summe>` schliesst den Denkweg ab. R steht in der Darstellung des Schemas.
  * `<ANTWORT>` wiederholt exakt denselben Wert wie R. Bewertet wird nur dieses Feld.

Zwei Schemata (--schema):

  normal (Standard)  Operanden normal (47+58), Ergebnis normal (105).
                     Der Denkweg laeuft von rechts (Einer) nach links - also gegen die
                     Leserichtung. Das Modell muss die Stellen selbst zuordnen.
  lsd                Operanden stellenweise umgedreht (74+85), Ergebnis umgedreht (501).
                     Damit laeuft der Denkweg IN Leserichtung; laut Literatur
                     (Nogueira et al. 2021) der einfachere Fall. Zum Vergleich gedacht.

SPLITS
======
  train-1d  10x10 = 100 Aufgaben, einstellig (Aufwaermstufe), vollstaendig
  train-2d  zweistellige Operanden 10..99: vollstaendiger Raum = 90*90 = 8100 Aufgaben.
            Mehr als 8100 EINDEUTIGE zweistellige Aufgaben gibt es nicht - die
            Vorgabe ">= 10.000" ist damit mathematisch nicht erfuellbar, ohne Aufgaben
            zu wiederholen (die Roadmap verbietet Wiederholungen) oder Stellen
            aufzunehmen (verfaelscht den Generalisierungstest). Mit --balance wird
            auf 3645 je Uebertragsklasse ausgeduennt (7290 Aufgaben, exakt 50/50).
  eval-3d   dreistellige Operanden 100..999, im Training NIE enthalten
  eval-4d   vierstellige Operanden 1000..9999, im Training NIE enthalten
  eval-Nd   beliebig lange Messbaender ueber --eval-stellen, z. B. 5 und 6
            Stellen. Sie sind MESSACHSE, kein Trainingsstoff: sie pruefen, ob
            die Faеhigkeit auf nie gesehene Laengen traegt (Stellenwert-
            ALGORITHMUS) oder ob nur die trainierten 1-2 Stellen sitzen
            (auswendig gelernte Tabelle).

Uebertragsklasse bezieht sich auf die EINERSTELLE:
  U1 = (a%10 + b%10) >= 10, sonst U0.

Alles deterministisch: gleicher Seed -> gleiche Bytes -> gleicher Digest.
"""

import argparse
import hashlib
import json
import os
import random
import re
import sys

B_TASK = 0xFE
B_THINK = 0xFD
B_ANSWER = 0xFC
B_END = 0xFF
SPECIALS = (0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF)
MAGIC = b"BSHD"
VERSION = 1
HEADER_SIZE = 32
NOTE_SIZE = 8
MAX_RECORD_LEN = 4096
# Spaltenbuchstaben von der Einerstelle aufwaerts. Die ersten fuenf sind
# unveraendert (E Z H T K); fuer laengere Baender kommen HT (Hundert-
# tausender, 10^5) und M (Million, 10^6) dazu - damit sind bis zu
# sieben Spalten moeglich, also 6-stellige Operanden mit 7-stelliger
# Summe (999999 + 999999 = 1999998).
STELLE = ["E", "Z", "H", "T", "K", "HT", "M"]
FNV_OFFSET = 0xCBF29CE484222325
FNV_PRIME = 0x100000001B3


def fnv1a64(daten):
    h = FNV_OFFSET
    for b in daten:
        h ^= b
        h = (h * FNV_PRIME) & 0xFFFFFFFFFFFFFFFF
    return h


def ist_verboten(b):
    return b == 0x00 or (0xF9 <= b <= 0xFF and b not in SPECIALS)


def spaltenweg(a, b, antwort_darstellung):
    """Baut den Denkweg Spalte fuer Spalte, beginnend bei der Einerstelle."""
    da = [int(z) for z in reversed(str(a))]
    db = [int(z) for z in reversed(str(b))]
    n = max(len(da), len(db))
    uebertrag = 0
    teile = []
    ziffern = []
    i = 0
    while True:
        x = da[i] if i < len(da) else 0
        y = db[i] if i < len(db) else 0
        if i >= n and x == 0 and y == 0 and uebertrag == 0:
            break
        if i >= len(STELLE):
            sys.exit("FEHLER: %d Stellen ueberschreiten die %d Spaltenbuchstaben"
                     % (i + 1, len(STELLE)))
        s = x + y + uebertrag
        ziffer = s % 10
        neuer = s // 10
        op = "%d+%d" % (x, y)
        if uebertrag:
            op += "+1"
        teile.append("%s %s=%d -> %d u%d" % (STELLE[i], op, s, ziffer, neuer))
        ziffern.append(str(ziffer))
        uebertrag = neuer
        i += 1
    while len(ziffern) > 1 and ziffern[-1] == "0":
        ziffern.pop()
    weg = " ; ".join(teile)
    weg += " ; R " + antwort_darstellung
    return weg, int("".join(reversed(ziffern)))


def record_text(a, b, schema):
    summe = a + b
    if schema == "lsd":
        antwort = str(summe)[::-1]
        links = str(a)[::-1]
        rechts = str(b)[::-1]
    else:
        antwort = str(summe)
        links = str(a)
        rechts = str(b)
    weg, geprueft = spaltenweg(a, b, antwort)
    if geprueft != summe:
        sys.exit("FEHLER: Spaltenweg liefert %d, erwartet %d" % (geprueft, summe))
    return (bytes([B_TASK]) + (" " + links + "+" + rechts + " ").encode("ascii")
            + bytes([B_THINK]) + (" " + weg + " ").encode("ascii")
            + bytes([B_ANSWER]) + (" " + antwort + " ").encode("ascii")
            + bytes([B_END]))


def klasse(a, b):
    return "U1" if (a % 10) + (b % 10) >= 10 else "U0"


def aufgabe(a, b, schema, idx, split):
    text = record_text(a, b, schema)
    return {
        "id": "%s-%05d" % (split, idx),
        "split": split,
        "a": a,
        "b": b,
        "summe": a + b,
        "klasse": klasse(a, b),
        "text": text,
    }


def baue_split(name, paare, schema):
    return [aufgabe(a, b, schema, i, name) for i, (a, b) in enumerate(paare)]


def paare_1d():
    return [(a, b) for a in range(10) for b in range(10)]


def paare_2d(balance, rng):
    alle = [(a, b) for a in range(10, 100) for b in range(10, 100)]
    if not balance:
        return alle
    u1 = [p for p in alle if klasse(*p) == "U1"]
    u0 = [p for p in alle if klasse(*p) == "U0"]
    n = min(len(u1), len(u0))
    rng.shuffle(u1)
    rng.shuffle(u0)
    gemischt = u1[:n] + u0[:n]
    rng.shuffle(gemischt)
    return gemischt


def paare_nd(stellen, anzahl, rng):
    lo = 10 ** (stellen - 1)
    hi = 10 ** stellen - 1
    gesehen = set()
    ergebnis = []
    versuche = 0
    while len(ergebnis) < anzahl and versuche < anzahl * 100:
        versuche += 1
        a = rng.randrange(lo, hi + 1)
        b = rng.randrange(lo, hi + 1)
        if (a, b) in gesehen:
            continue
        gesehen.add((a, b))
        ergebnis.append((a, b))
    if len(ergebnis) != anzahl:
        sys.exit("FEHLER: nur %d von %d Paaren erzeugt" % (len(ergebnis), anzahl))
    return ergebnis


def write_shard(pfad, aufgaben, note):
    if len(note) > NOTE_SIZE or not note.isascii():
        sys.exit("FEHLER: Shard-Notiz muss <= %d ASCII-Zeichen sein" % NOTE_SIZE)
    header = bytearray(HEADER_SIZE)
    header[0:4] = MAGIC
    header[4:6] = VERSION.to_bytes(2, "little")
    header[6:8] = HEADER_SIZE.to_bytes(2, "little")
    header[8:16] = len(aufgaben).to_bytes(8, "little")
    header[16:24] = (0).to_bytes(8, "little")
    header[24:24 + len(note)] = note.encode("ascii")
    os.makedirs(os.path.dirname(pfad), exist_ok=True)
    with open(pfad, "wb") as f:
        f.write(bytes(header))
        for a in aufgaben:
            daten = a["text"]
            if not daten or len(daten) > MAX_RECORD_LEN:
                sys.exit("FEHLER: Datensatz %s hat %d Bytes" % (a["id"], len(daten)))
            f.write(len(daten).to_bytes(2, "little"))
            f.write(daten)
    return os.path.getsize(pfad)


def write_manifest(ordner, seed, note, tasks, eintraege):
    manifest = {
        "format_version": 1,
        "vocab_size": 256,
        "byte_order": "little",
        "record_encoding": "raw-bytes-with-special-tokens",
        "seed": seed,
        "note": note,
        "tasks": tasks,
        "shards": eintraege,
    }
    pfad = os.path.join(ordner, "manifest.json")
    with open(pfad, "w", encoding="utf-8") as f:
        f.write(json.dumps(manifest, indent=2) + "\n")
    return pfad


def read_shard(pfad):
    with open(pfad, "rb") as f:
        roh = f.read()
    if roh[0:4] != MAGIC:
        sys.exit("FEHLER: %s hat keine BSHD-Magic" % pfad)
    anzahl = int.from_bytes(roh[8:16], "little")
    off = HEADER_SIZE
    records = []
    while off < len(roh):
        laenge = int.from_bytes(roh[off:off + 2], "little")
        records.append(roh[off + 2:off + 2 + laenge])
        off = off + 2 + laenge
    if len(records) != anzahl:
        sys.exit("FEHLER: %s meldet %d Datensaetze, gelesen %d" % (pfad, anzahl, len(records)))
    return records


def pruefe_split(name, aufgaben):
    fehler = []
    gesehen = set()
    for a in aufgaben:
        daten = a["text"]
        if a["text"] in gesehen:
            fehler.append("%s: doppelter Datensatz %s" % (name, a["id"]))
        gesehen.add(a["text"])
        for b in daten:
            if ist_verboten(b):
                fehler.append("%s/%s: verbotenes Byte 0x%02X" % (name, a["id"], b))
        positionen = [daten.index(bytes([m])) for m in (B_TASK, B_THINK, B_ANSWER, B_END)]
        if positionen != sorted(positionen):
            fehler.append("%s/%s: Strukturmarken falsch angeordnet" % (name, a["id"]))
        if daten[-1] != B_END:
            fehler.append("%s/%s: endet nicht mit <ENDE>" % (name, a["id"]))
        rumpf = daten.split(bytes([B_ANSWER]))[1].split(bytes([B_END]))[0].strip().decode("ascii")
        if rumpf != str(a["summe"]):
            fehler.append("%s/%s: Antwortfeld '%s' passt nicht zur Summe %d" % (name, a["id"], rumpf, a["summe"]))
    return fehler


def main():
    p = argparse.ArgumentParser(description="Generator fuer den Additionsbenchmark (Byte-Level)")
    p.add_argument("--out", default="data/add")
    p.add_argument("--seed", type=int, default=1337)
    p.add_argument("--schema", choices=("normal", "lsd"), default="normal")
    p.add_argument("--eval-records", type=int, default=2000)
    p.add_argument("--balance", action="store_true", help="train-2d auf exakt 50/50 ausduennen")
    p.add_argument("--verify", action="store_true", help="geschriebene Shards zuruecklesen und pruefen")
    p.add_argument("--eval-stellen", type=int, nargs="+", default=[],
                   help="NUR diese Baender erzeugen (Standardname eval-<N>d), z. B. "
                        "--eval-stellen 5 6. Ohne den Schalter entstehen die vier "
                        "Standard-Splits wie bisher.")
    p.add_argument("--stellen-praefix", choices=("eval", "train"), default="eval",
                   help="Namenspraefix der ueber --eval-stellen erzeugten Baender. "
                        "Mit 'train' entsteht z. B. train-3d - ein TRAININGSband fuer "
                        "laengere Operanden. Standard 'eval' (Messband).")
    args = p.parse_args()

    rng = random.Random(args.seed)
    splits = []
    if args.eval_stellen:
        # Messmodus: NUR die genannten Baender, nichts anderes. Die Notiz im
        # Shard bleibt bei acht ASCII-Zeichen (NOTE_SIZE).
        for n in args.eval_stellen:
            if n < 1:
                sys.exit("FEHLER: --eval-stellen verlangt Werte >= 1")
            name = "%s-%dd" % (args.stellen_praefix, n)
            splits.append((name, ("%s%dd" % (args.stellen_praefix[:2], n))[:8],
                           baue_split(name, paare_nd(n, args.eval_records, rng), args.schema)))
    else:
        splits.append(("train-1d", "add1d", baue_split("train-1d", paare_1d(), args.schema)))
        splits.append(("train-2d", "add2d", baue_split("train-2d", paare_2d(args.balance, rng), args.schema)))
        splits.append(("eval-3d", "add3d", baue_split("eval-3d", paare_nd(3, args.eval_records, rng), args.schema)))
        splits.append(("eval-4d", "add4d", baue_split("eval-4d", paare_nd(4, args.eval_records, rng), args.schema)))

    alle_fehler = []
    for name, _, aufgaben in splits:
        alle_fehler += pruefe_split(name, aufgaben)
        stellen = {len(str(a["a"])) for a in aufgaben} | {len(str(a["b"])) for a in aufgaben}
        m = re.match(r"^(?:train|eval)-(\d+)d$", name)
        if not m:
            alle_fehler.append("%s: Splitname nicht deutbar" % name)
            continue
        erwartet = {int(m.group(1))}
        if stellen != erwartet:
            alle_fehler.append("%s: Stellen %s statt %s" % (name, sorted(stellen), sorted(erwartet)))
    if alle_fehler:
        print("GENERATOR-PRUEFUNG FEHLGESCHLAGEN:")
        for f in alle_fehler[:20]:
            print("  - %s" % f)
        sys.exit(1)

    print("Schema: %s | Seed: %d | Ausgabe: %s" % (args.schema, args.seed, args.out))
    print()
    kopf = ["Split", "Aufgaben", "U1/U0", "Bytes", "Shard-Bytes", "FNV-1a-64", "SHA-256 (erste 16)"]
    zeilen = []
    for name, note, aufgaben in splits:
        ordner = os.path.join(args.out, name)
        shard_pfad = os.path.join(ordner, "shard-0000.bin")
        groesse = write_shard(shard_pfad, aufgaben, note)
        with open(shard_pfad, "rb") as f:
            roh = f.read()
        nutz = sum(len(a["text"]) for a in aufgaben)
        eintrag = {
            "id": 0,
            "path": "shard-0000.bin",
            "records": len(aufgaben),
            "payload_bytes": nutz,
            "file_size": groesse,
            "checksum": "%016x" % fnv1a64(roh),
        }
        u1 = sum(1 for a in aufgaben if a["klasse"] == "U1")
        tasks = {"addition": len(aufgaben), "mit_uebertrag_einerstelle": u1,
                 "ohne_uebertrag_einerstelle": len(aufgaben) - u1}
        write_manifest(ordner, args.seed, "Addition %s, Schema %s, Uebertrag explizit" % (name, args.schema),
                       tasks, [eintrag])
        jsonl = os.path.join(args.out, "jsonl", name + ".jsonl")
        os.makedirs(os.path.dirname(jsonl), exist_ok=True)
        with open(jsonl, "w", encoding="utf-8") as f:
            for a in aufgaben:
                f.write(json.dumps({"id": a["id"], "a": a["a"], "b": a["b"], "summe": a["summe"],
                                    "klasse": a["klasse"], "text": a["text"].decode("latin-1")}) + "\n")
        zeilen.append([name, str(len(aufgaben)), "%d/%d" % (u1, len(aufgaben) - u1), str(nutz),
                       str(groesse), eintrag["checksum"], hashlib.sha256(roh).hexdigest()[:16]])

    breiten = [max(len(z[i]) for z in zeilen + [kopf]) for i in range(len(kopf))]
    print("  ".join(kopf[i].ljust(breiten[i]) for i in range(len(kopf))))
    print("  ".join("-" * b for b in breiten))
    for z in zeilen:
        print("  ".join(z[i].ljust(breiten[i]) for i in range(len(z))))

    if args.verify:
        print()
        for name, _, aufgaben in splits:
            shard_pfad = os.path.join(args.out, name, "shard-0000.bin")
            zurueck = read_shard(shard_pfad)
            if zurueck != [a["text"] for a in aufgaben]:
                sys.exit("FEHLER: Ruecklese-Probe fuer %s stimmt nicht" % name)
        print("Ruecklese-Probe: alle Shards bytegleich rekonstruiert.")
    return 0


if __name__ == "__main__":
    sys.exit(main())