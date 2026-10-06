#!/usr/bin/env python3
"""iq_test.py - logisch-analytischer Benchmark (IQ-Test) fuer Shiro.

Byte-level: jede Aufgabe wird als Byte-Sequenz im Projektformat gebaut:

    <AUFGABE> ...text... <DENKWEG> ...denkweg... <ANTWORT> ...antwort... <ENDE>

Die vier Strukturmarken stammen aus src/data/bytes.rs (0xFE, 0xFD, 0xFC, 0xFF).
Ein Tokenizer ist an keiner Stelle beteiligt - die Aufgaben sind reine ASCII-Bytes.

Modi:
  m1  Prompt endet NACH <DENKWEG>. Das Modell muss Denkweg und Antwort selbst
      erzeugen ("kann es selbst denken?").
  m2  Prompt enthaelt den offiziellen Denkweg und endet NACH <ANTWORT>. Das Modell
      erzeugt nur die Antwort ("kann es rechnen, wenn der Weg gezeigt wird?").

Aufrufe:
  python3 iq_test.py --validate --selftest
  python3 iq_test.py --runner "shiro chat --model models/shiro-s-v1" --mode m2
  python3 iq_test.py --json results.json --runner "./infer.sh"

Optionen ohne Runner: die Aufgaben werden nur aufgelistet (Antworten offiziell).
"""

import argparse
import json
import os
import shlex
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
TASKS = os.path.join(HIER, "iq_test", "tasks.jsonl")
DOKU = os.path.join(HIER, "IQ_TEST.md")

B_TASK = 0xFE
B_THINK = 0xFD
B_ANSWER = 0xFC
B_END = 0xFF
SPECIALS = (0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF)
MAX_RECORD_LEN = 4096

BANDS = [
    (12, 15, "Transfer (Stufe 4)", "Regel traegt auf laengere/neue Faelle"),
    (8, 11, "Prozedur (Stufe 3)", "wiederverwendbare Byte-Transformation"),
    (4, 7, "Muster (Stufe 2)", "Oberflaechenmuster, noch keine Regel"),
    (0, 3, "Zufallsniveau", "nicht von Raten unterscheidbar"),
]


def ist_verboten(b):
    """Gleiche Regel wie src/data/bytes.rs: 0x00 und 0xF9..0xFF ausser Spezialbytes."""
    return b == 0x00 or (0xF9 <= b <= 0xFF and b not in SPECIALS)


def erste_verbotene(daten):
    for i, b in enumerate(daten):
        if ist_verboten(b):
            return i, b
    return None


def lade_aufgaben(pfad):
    if not os.path.exists(pfad):
        sys.exit("FEHLER: Aufgabendatei fehlt: %s" % pfad)
    aufgaben = []
    with open(pfad, "r", encoding="utf-8") as f:
        for nr, zeile in enumerate(f, 1):
            zeile = zeile.strip()
            if not zeile or zeile.startswith("#"):
                continue
            try:
                aufgaben.append(json.loads(zeile))
            except json.JSONDecodeError as e:
                sys.exit("FEHLER: %s Zeile %d ist kein gueltiges JSON: %s" % (pfad, nr, e))
    return aufgaben


def prompt_bytes(a, mode):
    """Baut die Byte-Sequenz der Aufgabe (ohne Antwort)."""
    teile = [bytes([B_TASK])]
    teile.append(a["task"].encode("ascii"))
    teile.append(bytes([B_THINK]))
    if mode == "m2":
        teile.append(a["think"].encode("ascii"))
        teile.append(bytes([B_ANSWER]))
    return b"".join(teile)


def normalisiere(roh, fmt):
    """Schneidet die Antwort aus den Modell-BYTES und normalisiert sie."""
    daten = roh
    idx = daten.find(bytes([B_ANSWER]))
    if idx != -1:
        daten = daten[idx + 1:]
    for marke in (B_END, B_THINK, B_TASK):
        idx = daten.find(bytes([marke]))
        if idx != -1:
            daten = daten[:idx]
    text = daten.decode("utf-8", errors="replace").strip().strip(".").strip()
    if not text:
        return ""
    if fmt.startswith("zwei Hex"):
        t = text.split()[0].lower()
        if t.startswith("0x"):
            t = t[2:]
        return t
    if fmt == "ja oder nein":
        t = text.lower().strip(".,!?").split()[0]
        return t
    return text.split()[0]


def pruefe_aufgaben(aufgaben):
    """Byte-Regeln und Format des Projekt-Validators nachbilden."""
    fehler = []
    gesehen = set()
    for a in aufgaben:
        for feld in ("id", "category", "level", "task", "think", "answer", "space", "format"):
            if feld not in a:
                fehler.append("%s: Feld '%s' fehlt" % (a.get("id", "?"), feld))
        if a.get("id") in gesehen:
            fehler.append("%s: doppelte ID" % a["id"])
        gesehen.add(a.get("id"))
        for feld in ("task", "think", "answer"):
            try:
                daten = a[feld].encode("ascii")
            except (UnicodeEncodeError, KeyError):
                fehler.append("%s: '%s' ist nicht ASCII (Byte-Laenge sonst nicht stabil)" % (a.get("id"), feld))
                continue
            treffer = erste_verbotene(daten)
            if treffer:
                fehler.append("%s/%s: verbotenes Byte 0x%02X an Position %d" % (a["id"], feld, treffer[1], treffer[0]))
        voll = prompt_bytes(a, "m2") + a["answer"].encode("ascii") + bytes([B_END])
        if len(voll) > MAX_RECORD_LEN:
            fehler.append("%s: %d Bytes > MAX_RECORD_LEN" % (a["id"], len(voll)))
        reihen = [voll.index(bytes([m])) for m in (B_TASK, B_THINK, B_ANSWER, B_END)]
        if reihen != sorted(reihen):
            fehler.append("%s: Strukturmarken in falscher Reihenfolge" % a["id"])
        if a.get("space", 0) < 1:
            fehler.append("%s: Antwortraum 'space' muss >= 1 sein" % a["id"])
    return fehler, gesehen


def pruefe_doku(ids, pfad):
    if not os.path.exists(pfad):
        return ["Dokumentation fehlt: %s" % pfad]
    with open(pfad, "r", encoding="utf-8") as f:
        text = f.read()
    return ["ID %s kommt in %s nicht vor" % (i, os.path.basename(pfad)) for i in sorted(ids) if i not in text]


def zufallserwartung(aufgaben):
    return sum(1.0 / a["space"] for a in aufgaben)


def band(punkte):
    for unten, oben, name, beschreibung in BANDS:
        if unten <= punkte <= oben:
            return name, beschreibung
    return "ausserhalb", ""


def tabelle(zeilen, kopf):
    breiten = [max(len(str(z[i])) for z in zeilen + [kopf]) for i in range(len(kopf))]
    def fmt(z):
        return "  ".join(str(z[i]).ljust(breiten[i]) for i in range(len(z)))
    print(fmt(kopf))
    print("  ".join("-" * b for b in breiten))
    for z in zeilen:
        print(fmt(z))


def main():
    p = argparse.ArgumentParser(description="Logisch-analytischer Benchmark (IQ-Test) fuer Shiro")
    p.add_argument("--tasks", default=TASKS)
    p.add_argument("--mode", choices=("m1", "m2"), default="m1")
    p.add_argument("--runner", default=None, help="externer Befehl: Prompt auf stdin, Modelltext auf stdout")
    p.add_argument("--selftest", action="store_true", help="Bewertung gegen die offiziellen Antworten pruefen")
    p.add_argument("--validate", action="store_true", help="Byte-Regeln und Doku-Abgleich pruefen")
    p.add_argument("--json", dest="json_out", default=None)
    args = p.parse_args()

    aufgaben = lade_aufgaben(args.tasks)
    if not aufgaben:
        sys.exit("FEHLER: keine Aufgaben geladen")

    fehler, ids = pruefe_aufgaben(aufgaben)
    if args.validate:
        fehler += pruefe_doku(ids, DOKU)
    if fehler:
        print("VALIDIERUNG FEHLGESCHLAGEN:")
        for f in fehler:
            print("  - %s" % f)
        sys.exit(1)
    if args.validate:
        print("Validierung OK: %d Aufgaben, alle ASCII, keine verbotenen Bytes, "
              "Strukturmarken in Ordnung, Doku vollstaendig." % len(aufgaben))
        print()

    hat_antwort = args.selftest or bool(args.runner)
    if not hat_antwort:
        print("Kein --runner angegeben: nur Aufgabenliste (Antworten offiziell).")
        print("Fuer eine echte Messung: --runner \"<befehl>\" [--mode m1|m2]\n")

    zeilen = []
    punkte = 0
    pro_kategorie = {}
    for a in aufgaben:
        if args.selftest:
            roh = (bytes([B_THINK]) + a["think"].encode("ascii")
                   + bytes([B_ANSWER]) + a["answer"].encode("ascii") + bytes([B_END]))
        elif args.runner:
            prompt = prompt_bytes(a, args.mode)
            try:
                r = subprocess.run(shlex.split(args.runner), input=prompt,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            except OSError as e:
                sys.exit("FEHLER: Runner nicht ausfuehrbar: %s" % e)
            if r.returncode != 0 and not r.stdout:
                sys.exit("FEHLER: Runner brach ab (Code %d): %s" % (r.returncode, r.stderr.decode("utf-8", "replace")[:200]))
            roh = r.stdout
        else:
            roh = b""
        antwort = normalisiere(roh, a["format"]) if hat_antwort else "?"
        ok = antwort == a["answer"]
        if hat_antwort:
            punkte += 1 if ok else 0
            k = pro_kategorie.setdefault(a["category"], [0, 0])
            k[1] += 1
            if ok:
                k[0] += 1
        zeilen.append([a["id"], a["category"], "Stufe %d" % a["level"], a["answer"], antwort or "-", "OK" if ok else ("--" if not hat_antwort else "FALSCH")])

    print("IQ-Test Shiro - Modus %s%s" % (args.mode, " (Selbsttest)" if args.selftest else ""))
    print()
    tabelle(zeilen, ["ID", "Kategorie", "Niveau", "Erwartet", "Modell", "Bewertung"])
    print()
    print("Zufallserwartung (Summe der Antwortraeume): %.2f von %d Punkten" % (zufallserwartung(aufgaben), len(aufgaben)))
    if hat_antwort:
        name, beschreibung = band(punkte)
        print("Punkte: %d von %d  ->  %s (%s)" % (punkte, len(aufgaben), name, beschreibung))
        for kat in sorted(pro_kategorie):
            h, g = pro_kategorie[kat]
            print("  %-14s %d/%d" % (kat, h, g))
        zahl = pro_kategorie.get("Zahlenfolgen", [0, 0])
        must = pro_kategorie.get("Byte-Muster", [0, 0])
        if punkte >= 8 and (zahl[0] < 3 or must[0] < 3):
            print("  Hinweis: Stufe 3 (Prozedur) ist NICHT belegt - dafuer braucht es")
            print("           mindestens 3/4 bei Zahlenfolgen UND 3/4 bei Byte-Mustern.")
        print()
        print("Grenzen: 15 Aufgaben, kein genormter Vergleichsmassstab, 3 Aufgaben sind")
        print("mit ja/nein zu raten (Zufallsanteil rund 1,5 Punkte). Kein menschlicher IQ.")
    if args.json_out:
        daten = {
            "mode": args.mode,
            "runner": args.runner,
            "punkte": punkte if hat_antwort else None,
            "moeglich": len(aufgaben),
            "zufallserwartung": round(zufallserwartung(aufgaben), 3),
            "kategorien": {k: {"richtig": v[0], "gesamt": v[1]} for k, v in pro_kategorie.items()},
            "aufgaben": [dict(zip(("id", "kategorie", "niveau", "erwartet", "modell", "bewertung"), z)) for z in zeilen],
        }
        with open(args.json_out, "w", encoding="utf-8") as f:
            json.dump(daten, f, indent=2, ensure_ascii=False)
        print("\nErgebnis geschrieben: %s" % args.json_out)
    return 0


if __name__ == "__main__":
    sys.exit(main())