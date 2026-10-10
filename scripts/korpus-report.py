#!/usr/bin/env python3
# -*- coding: ascii -*-
"""korpus-report.py - Inventar, Familienverteilung und GATE fuer alle Korpora.

Anlass (09.10.2026): `data/{seq,quant,algebra}/{train,val}` wurden um 15:32:02
mit Generator-DEFAULTS (512 statt 50.000 usw.) ueberschrieben. Aufgefallen ist
das nur, weil jemand die Zahlen von Hand verglichen hat. Dieses Werkzeug macht
diese Pruefung automatisch.

Was es prueft (alles nur lesend, kein Schreiben, kein Training):

  1. Manifest gegen die SHARD-DATEI: Datensatzzahl, Nutzbytes und
     FNV-1a-64-Pruefsumme werden aus den Bytes NACHGERECHNET. Ein Shard, dessen
     Manifest nicht zu seinem Inhalt passt, ist ein harter Befund.
  2. SOLLWERTE: die ausgelieferten Korpora haben feste Kennzahlen. Weicht eine
     davon ab, ist das ein harter Befund - genau die Fehlerklasse von 15:32:02.
     Wird ein Korpus ABSICHTLICH neu erzeugt, gehoert die neue Zahl im selben
     Commit in die Tabelle SOLWERTE; das ist der Zweck des Gates.
  3. Familienverteilung je Manifest (Shard-Anteile in Prozent).
  4. Mit --dubletten: Dubletten innerhalb eines Korpus und die Schnittmenge
     Training/Held-out derselben Familie (Leckage).

Rueckgabecodes: 0 = alles gruen, 1 = mindestens ein harter Befund, 2 = Aufruf.
Kein Zeitstempel in der Ausgabe (Projektregel: Berichte sind reproduzierbar).
Die Spalte "Stand" ist die Dateizeit des Manifests - sie ist Diagnose, nicht
Berichtsinhalt.

Aufruf:
    python3 scripts/korpus-report.py            # Inventar + Gate
    python3 scripts/korpus-report.py --dubletten
    python3 scripts/korpus-report.py --nur-gate
"""

import hashlib
import json
import os
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA = os.path.join(REPO, "data")

HEADER_SIZE = 32
MAGIC = b"BSHD"

# FNV-1a-64, wie src/data/manifest.rs::checksum_hex sie ueber die GANZE Datei
# rechnet (Beleg: src/main.rs cmd_data_combine -> checksum_hex(&raw)).
FNV_OFFSET = 0xCBF29CE484222325
FNV_PRIME = 0x100000001B3
MASKE = 0xFFFFFFFFFFFFFFFF


def fnv1a64(daten):
    h = FNV_OFFSET
    for b in daten:
        h ^= b
        h = (h * FNV_PRIME) & MASKE
    return "%016x" % h


def lies_shard(pfad):
    """Liest einen .bshard und rechnet Kennzahlen aus den BYTES nach."""
    roh = open(pfad, "rb").read()
    if roh[:4] != MAGIC:
        raise ValueError("kein Shard: Magic %r statt %r" % (roh[:4], MAGIC))
    if len(roh) < HEADER_SIZE:
        raise ValueError("Header abgeschnitten")
    kopf = int.from_bytes(roh[6:8], "little")
    if kopf != HEADER_SIZE:
        raise ValueError("Header-Groesse %d statt %d" % (kopf, HEADER_SIZE))
    off = HEADER_SIZE
    n = 0
    nutz = 0
    while off < len(roh):
        if off + 2 > len(roh):
            raise ValueError("Laengenfeld abgeschnitten bei %d" % off)
        ln = int.from_bytes(roh[off:off + 2], "little")
        off += 2
        if off + ln > len(roh):
            raise ValueError("Datensatz %d laeuft ueber das Dateiende" % (n + 1))
        off += ln
        n += 1
        nutz += ln
    if off != len(roh):
        raise ValueError("Datei endet nicht auf einer Datensatzgrenze")
    return {"records": n, "payload_bytes": nutz, "file_size": len(roh),
            "checksum": fnv1a64(roh), "roh": None}


def familie_von_shard(eintrag, ordner):
    """Familienname: Praefix vor '-shard' im Dateinamen, sonst Ordnername."""
    name = os.path.basename(eintrag["path"])
    if "-shard" in name:
        return name.split("-shard")[0]
    teile = ordner.rstrip("/").split("/")
    if teile[-1] in ("train", "val") and len(teile) >= 2:
        return teile[-2]
    return teile[-1]


# Sollwerte der ausgelieferten Korpora: (Datensaetze, Nutzbytes).
# Quelle: die Manifeste nach der Wiederherstellung vom 09.10.2026 bzw. die
# eingefrorenen Kopien in data/kombi-mathe/train.
SOLWERTE = {
    "data/sample": (256, 29107),
    "data/add/train-1d": (100, 4535),
    "data/add/train-2d": (8100, 584415),
    "data/add/eval-3d": (2000, 192263),
    "data/add/eval-4d": (2000, 238531),
    "data/logic/train": (512, 175577),
    "data/logic/val": (51, 16862),
    "data/logic-large/train": (50000, 17750168),
    "data/logic-large/val": (5000, 1761484),
    "data/logic-long/train": (2000, 764980),
    "data/logic-long/val": (2000, 759484),
    "data/seq/train": (50000, 11164275),
    "data/seq/val": (5000, 1220723),
    "data/quant/train": (30000, 14279272),
    "data/quant/val": (3000, 1503784),
    "data/algebra/train": (90720, 16475732),
    "data/algebra/val": (10000, 2844503),
    "data/formel/train": (100000, 29566789),
    "data/formel/val": (10000, 3002406),
    "data/semantics/train": (20000, 11933646),
    "data/semantics/val": (2000, 1188650),
    "data/format/train": (50000, 25740625),
    "data/format/val": (5000, 2732066),
    "data/games/train": (40000, 12179395),
    "data/games/val": (2416, 730841),
    "data/kombi/train": (130000, 43193715),
    "data/kombi-delta/train": (130000, 43863176),
    "data/kombi-mathe/train": (220720, 60338908),
    "data/kombi-mathe2/train": (320720, 89905697),
    # Phase 4 (D-071, 09.10.2026): Kombinationskorpus MIT den Grundrechenarten.
    # Gebaut mit: data combine --src data/seq/train --src data/quant/train
    #   --src data/logic-large/train --src data/algebra/train
    #   --src data/add/train-1d --src data/add/train-2d
    #   --out data/kombi-add/train --antwortform delta
    # Gegenueber data/kombi-mathe/train sind GENAU die beiden add-Shards neu
    # (8.200 Datensaetze, 588.950 B) - das ist die eine geaenderte Groesse.
    "data/kombi-add/train": (228920, 60927858),
}

# Erwartete Pruefsumme fuer die eingefrorenen Arme: wer diese Korpora neu baut,
# muss byte-identisch herauskommen (Runde-32-Beweis).
FROST_PRUEFSUMME = {
    "data/kombi-mathe/train/seq-shard-0000.bin": "cd0a49a3d0957644",
    "data/kombi-mathe/train/quant-shard-0000.bin": "e113c0a1cd1198eb",
    "data/kombi-mathe/train/logic-large-shard-0000.bin": "9c800a4a5d5680cb",
    "data/kombi-mathe/train/algebra-shard-0000.bin": "94ae26f901c765b0",
}


def sammle_manifeste():
    treffer = []
    for wurzel, _dirs, dateien in os.walk(DATA):
        if "manifest.json" in dateien:
            treffer.append(os.path.join(wurzel, "manifest.json"))
    return sorted(treffer)


def bericht(mit_dubletten, nur_gate):
    befunde = []
    zeilen = []
    for mf in sammle_manifeste():
        ordner = os.path.dirname(mf)
        rel_ordner = os.path.relpath(ordner, REPO).replace("\\", "/")
        rel_mf = os.path.relpath(mf, REPO).replace("\\", "/")
        try:
            m = json.load(open(mf, encoding="utf-8"))
        except Exception as e:
            befunde.append("HART  %s: Manifest nicht lesbar (%s)" % (rel_mf, e))
            continue
        rec_sum = 0
        pay_sum = 0
        fams = []
        for e in m.get("shards", []):
            pfad = os.path.join(ordner, e["path"])
            rel_pfad = os.path.relpath(pfad, REPO).replace("\\", "/")
            if not os.path.isfile(pfad):
                befunde.append("HART  %s: Shard fehlt: %s" % (rel_ordner, e["path"]))
                continue
            try:
                ist = lies_shard(pfad)
            except Exception as ex:
                befunde.append("HART  %s: %s nicht lesbar (%s)" % (rel_ordner, e["path"], ex))
                continue
            for feld in ("records", "payload_bytes", "file_size", "checksum"):
                if e.get(feld) != ist[feld]:
                    befunde.append(
                        "HART  %s: Manifest und Datei widersprechen sich in %s "
                        "(%r im Manifest, %r in der Datei)"
                        % (rel_pfad, feld, e.get(feld), ist[feld]))
            soll = FROST_PRUEFSUMME.get(rel_pfad)
            if soll is not None and ist["checksum"] != soll:
                befunde.append("HART  %s: Pruefsumme %s statt der eingefrorenen %s"
                               % (rel_pfad, ist["checksum"], soll))
            rec_sum += ist["records"]
            pay_sum += ist["payload_bytes"]
            fams.append((familie_von_shard(e, rel_ordner), ist["records"], ist["payload_bytes"]))
        soll = SOLWERTE.get(rel_ordner)
        if soll is None:
            befunde.append("HINWEIS %s: kein Sollwert hinterlegt (%d Datensaetze, %d B)"
                           % (rel_ordner, rec_sum, pay_sum))
        elif (rec_sum, pay_sum) != soll:
            befunde.append(
                "HART  %s: %d Datensaetze / %d B, erwartet %d / %d  (Faktor Datensaetze %.3f)"
                % (rel_ordner, rec_sum, pay_sum, soll[0], soll[1],
                   (rec_sum / soll[0]) if soll[0] else 0.0))
        stand = time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(os.path.getmtime(mf)))
        zeilen.append((rel_ordner, rec_sum, pay_sum, len(m.get("shards", [])),
                       m.get("seed"), stand, fams))
        if mit_dubletten:
            befunde.extend(dubletten_und_leckage(rel_ordner, ordner, m))

    if not nur_gate:
        print("Korpus-Inventar (nur gelesen; Kennzahlen aus den BYTES nachgerechnet)")
        print("%-30s %8s %12s %6s %5s %-19s %s" % ("Ordner", "records", "payload", "shards", "seed", "Stand", "Familienanteile"))
        for rel, rec, pay, ns, seed, stand, fams in zeilen:
            anteile = " ".join("%s %.2f%%" % (f, 100.0 * r / rec) if rec else f for f, r, _p in fams)
            print("%-30s %8d %12d %6d %5s %-19s %s" % (rel, rec, pay, ns, seed, stand, anteile))
        print()
        print("Summe ueber alle Manifeste: %d Datensaetze, %d B (%.2f MiB)"
              % (sum(z[1] for z in zeilen), sum(z[2] for z in zeilen),
                 sum(z[2] for z in zeilen) / 1048576.0))
        print()

    harte = [b for b in befunde if b.startswith("HART")]
    hinweise = [b for b in befunde if b.startswith("HINWEIS")]
    print("Gate: %d harte Befunde, %d Hinweise" % (len(harte), len(hinweise)))
    for b in harte:
        print("  " + b)
    for b in hinweise:
        print("  " + b)
    return 1 if harte else 0


def lade_datensaetze(ordner, m):
    """Alle Datensaetze eines Korpus als Liste von Bytes (fuer Dubletten)."""
    raus = []
    for e in m.get("shards", []):
        pfad = os.path.join(ordner, e["path"])
        roh = open(pfad, "rb").read()
        off = HEADER_SIZE
        while off < len(roh):
            ln = int.from_bytes(roh[off:off + 2], "little")
            off += 2
            raus.append(roh[off:off + ln])
            off += ln
    return raus


def dubletten_und_leckage(rel_ordner, ordner, m):
    """Dubletten innerhalb eines Korpus; Leckage Training gegen Held-out."""
    befunde = []
    try:
        recs = lade_datensaetze(ordner, m)
    except Exception as e:
        return ["HART  %s: Dublettenpruefung nicht moeglich (%s)" % (rel_ordner, e)]
    hashes = [hashlib.sha256(r).digest() for r in recs]
    eindeutig = len(set(hashes))
    dpl = len(hashes) - eindeutig
    if dpl > 0:
        befunde.append("HINWEIS %s: %d Dubletten innerhalb des Korpus (%d von %d eindeutig)"
                       % (rel_ordner, dpl, eindeutig, len(hashes)))
    eltern = os.path.dirname(ordner)
    if os.path.basename(ordner) == "train":
        val = os.path.join(eltern, "val", "manifest.json")
        if os.path.isfile(val):
            try:
                mv = json.load(open(val, encoding="utf-8"))
                vals = lade_datensaatze_set(os.path.join(eltern, "val"), mv)
                gemeinsam = set(hashes) & vals
                if gemeinsam:
                    befunde.append("HART  %s: %d Datensaetze sind gleichzeitig in Training und Held-out (Leckage)"
                                   % (rel_ordner, len(gemeinsam)))
                else:
                    befunde.append("OK    %s: Schnittmenge Training/Held-out = 0" % rel_ordner)
            except Exception as e:
                befunde.append("HINWEIS %s: Leckagepruefung nicht moeglich (%s)" % (rel_ordner, e))
    return befunde


def lade_datensaatze_set(ordner, m):
    return set(hashlib.sha256(r).digest() for r in lade_datensaetze(ordner, m))


def main(argv):
    mit_dubletten = "--dubletten" in argv
    nur_gate = "--nur-gate" in argv
    for a in argv[1:]:
        if a not in ("--dubletten", "--nur-gate"):
            print("Unbekannter Schalter: %s" % a)
            print(__doc__)
            return 2
    return bericht(mit_dubletten, nur_gate)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
