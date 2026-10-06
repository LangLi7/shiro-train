#!/usr/bin/env python3
"""add_test.py - Runner und Pruefer fuer den Byte-Level-Additionsbenchmark.

Liest die mit add_data.py erzeugten `.bshard`-Dateien (dasselbe Format wie
src/data/format.rs: 32-Byte-Header, danach je Datensatz u16-Laenge + Bytes),
baut daraus die Byte-Prompts und bewertet die Antwort.

Modi (--mode), identisch zu iq_test.py:
  m1  Prompt endet NACH <DENKWEG>. Das Modell muss den Spaltenweg UND die Antwort
      erzeugen - die eigentliche Denkfaehigkeits-Messung.
  m2  Prompt enthaelt den offiziellen Spaltenweg und endet NACH <ANTWORT>. Das Modell
      erzeugt nur die Antwort - misst Rechnen bei vorgegebenem Weg.

Splits (--split): train-1 | train-2 | eval-3 | eval-4  (Verzeichnisse unter data/add)

Beispiele:
  python3 add_test.py --validate
  python3 add_test.py --split eval-3 --selftest
  python3 add_test.py --split eval-3 --mode m2 --limit 200 --runner "python3 add_bench/stub_perfect.py"
  python3 add_test.py --split eval-4 --mode m1 --runner "shiro chat --model models/shiro-s-v1" --json add_eval4.json

Bewertung: exakter Vergleich des Antwortfeldes (Ziffernfolge) mit der im Datensatz
gespeicherten Summe. Es wird ausschliesslich das Feld zwischen <ANTWORT> und <ENDE>
gewertet; der vom Modell erzeugte Denkweg wird nicht bewertet (er dient der Diagnose).
"""

import argparse
import json
import os
import re
import shlex
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))

B_TASK = 0xFE
B_THINK = 0xFD
B_ANSWER = 0xFC
B_END = 0xFF
SPECIALS = (0xFA, 0xFB, 0xFC, 0xFD, 0xFE, 0xFF)
MAGIC = b"BSHD"
HEADER_SIZE = 32
MAX_RECORD_LEN = 4096
STELLE = ["E", "Z", "H", "T", "K"]
SPALTEN_RE = re.compile(r"^([EZHTK]) (\d)\+(\d)(\+1)?=(\d+) -> (\d) u([01])$")
SPLITS = {
    "train-1": ("train-1d", 1),
    "train-2": ("train-2d", 2),
    "eval-3": ("eval-3d", 3),
    "eval-4": ("eval-4d", 4),
}


def ist_verboten(b):
    return b == 0x00 or (0xF9 <= b <= 0xFF and b not in SPECIALS)


def lies_shard(pfad):
    with open(pfad, "rb") as f:
        roh = f.read()
    if len(roh) < HEADER_SIZE or roh[0:4] != MAGIC:
        sys.exit("FEHLER: %s ist kein .bshard (Magic fehlt)" % pfad)
    if int.from_bytes(roh[4:6], "little") != 1:
        sys.exit("FEHLER: %s hat unbekannte Version" % pfad)
    if int.from_bytes(roh[6:8], "little") != HEADER_SIZE:
        sys.exit("FEHLER: %s hat unerwartete Header-Groesse" % pfad)
    if int.from_bytes(roh[16:24], "little") != 0:
        sys.exit("FEHLER: %s hat unbekannte Flags gesetzt" % pfad)
    anzahl = int.from_bytes(roh[8:16], "little")
    off = HEADER_SIZE
    records = []
    while off < len(roh):
        if off + 2 > len(roh):
            sys.exit("FEHLER: %s endet mitten im Laengenfeld" % pfad)
        laenge = int.from_bytes(roh[off:off + 2], "little")
        if laenge == 0 or laenge > MAX_RECORD_LEN:
            sys.exit("FEHLER: %s hat unzulaessige Datensatzlaenge %d" % (pfad, laenge))
        records.append(roh[off + 2:off + 2 + laenge])
        off += 2 + laenge
    if len(records) != anzahl:
        sys.exit("FEHLER: %s meldet %d, gelesen %d" % (pfad, anzahl, len(records)))
    return records


def zerlege(rec):
    """Trennt Aufgabe, Denkweg und Antwort anhand der Strukturmarken."""
    if rec[-1] != B_END:
        return None
    teile = rec.split(bytes([B_THINK]), 1)
    if len(teile) != 2 or bytes([B_TASK]) not in teile[0]:
        return None
    aufgabe = teile[0].split(bytes([B_TASK]), 1)[1].strip().decode("ascii", "replace")
    rest = teile[1].split(bytes([B_ANSWER]), 1)
    if len(rest) != 2:
        return None
    weg = rest[0].strip().decode("ascii", "replace")
    antwort = rest[1].split(bytes([B_END]), 1)[0].strip().decode("ascii", "replace")
    return aufgabe, weg, antwort


def operanden(aufgabe, schema):
    a_text, b_text = aufgabe.split("+")
    if schema == "lsd":
        a_text = a_text[::-1]
        b_text = b_text[::-1]
    return int(a_text), int(b_text)


def pruefe_spaltenweg(weg, a, b, antwort, schema):
    """Prueft die gespeicherte Rechnung unabhaengig nach: Ziffern, Summe, Uebertragskette."""
    fehler = []
    if " ; R " not in weg:
        return ["Denkweg enthaelt kein ' ; R '"]
    spalten_text, r_wert = weg.rsplit(" ; R ", 1)
    erwartete_antwort = antwort if schema == "normal" else antwort[::-1]
    if r_wert != erwartete_antwort:
        fehler.append("R-Wert '%s' passt nicht zu <ANTWORT> '%s'" % (r_wert, antwort))
    da = [int(z) for z in reversed(str(a))]
    db = [int(z) for z in reversed(str(b))]
    erwartete_spalten = max(len(da), len(db)) + (1 if a + b >= 10 ** max(len(da), len(db)) else 0)
    spalten = [s.strip() for s in spalten_text.split(";")]
    if len(spalten) < max(len(da), len(db)):
        fehler.append("zu wenige Spalten: %d" % len(spalten))
    uebertrag = 0
    ziffern = []
    for i, spalte in enumerate(spalten):
        m = SPALTEN_RE.match(spalte)
        if not m:
            fehler.append("Spalte %d '%s' passt nicht ins Schema" % (i, spalte))
            continue
        buchstabe, x, y, plus1, summe, ziffer, neuer = m.groups()
        x, y, summe, ziffer, neuer = int(x), int(y), int(summe), int(ziffer), int(neuer)
        if buchstabe != STELLE[i]:
            fehler.append("Spalte %d hat Buchstabe %s statt %s" % (i, buchstabe, STELLE[i]))
        if (x, y) != (da[i] if i < len(da) else 0, db[i] if i < len(db) else 0):
            fehler.append("Spalte %d nutzt Ziffern %d+%d, erwartet %d+%d"
                          % (i, x, y, da[i] if i < len(da) else 0, db[i] if i < len(db) else 0))
        if bool(plus1) != bool(uebertrag):
            fehler.append("Spalte %d: Uebertrag aus der Vorspalte nicht korrekt uebernommen" % i)
        erwartete_summe = x + y + uebertrag
        if summe != erwartete_summe:
            fehler.append("Spalte %d: Summe %d statt %d" % (i, summe, erwartete_summe))
        if ziffer != erwartete_summe % 10 or neuer != erwartete_summe // 10:
            fehler.append("Spalte %d: Ziffer/Uebertrag falsch (%d/%d)" % (i, ziffer, neuer))
        ziffern.append(str(ziffer))
        uebertrag = neuer
    if len(spalten) > erwartete_spalten:
        fehler.append("zu viele Spalten: %d statt %d" % (len(spalten), erwartete_spalten))
    while len(ziffern) > 1 and ziffern[-1] == "0":
        ziffern.pop()
    aus_spalten = int("".join(reversed(ziffern))) if ziffern else 0
    if aus_spalten != a + b:
        fehler.append("Spalten ergeben %d, tatsaechlich %d" % (aus_spalten, a + b))
    return fehler


def pruefe_split(name, records, schema, stellen):
    fehler = []
    gesehen = set()
    summen = set()
    for i, rec in enumerate(records):
        for b in rec:
            if ist_verboten(b):
                fehler.append("%s/%d: verbotenes Byte 0x%02X" % (name, i, b))
                break
        if len(rec) > MAX_RECORD_LEN:
            fehler.append("%s/%d: ueber MAX_RECORD_LEN" % (name, i))
        positionen = [rec.find(bytes([m])) for m in (B_TASK, B_THINK, B_ANSWER, B_END)]
        if -1 in positionen or positionen != sorted(positionen):
            fehler.append("%s/%d: Strukturmarken fehlen oder sind falsch angeordnet" % (name, i))
            continue
        if rec in gesehen:
            fehler.append("%s/%d: doppelter Datensatz" % (name, i))
        gesehen.add(rec)
        z = zerlege(rec)
        if z is None:
            fehler.append("%s/%d: nicht zerlegbar" % (name, i))
            continue
        aufgabe, weg, antwort = z
        try:
            a, b = operanden(aufgabe, schema)
        except ValueError:
            fehler.append("%s/%d: Aufgabe '%s' nicht als a+b lesbar" % (name, i, aufgabe))
            continue
        if len(str(a)) != stellen or len(str(b)) != stellen:
            fehler.append("%s/%d: Stellen %d/%d statt %d (Split-Disziplin verletzt)"
                          % (name, i, len(str(a)), len(str(b)), stellen))
        if antwort != str(a + b):
            fehler.append("%s/%d: Antwort '%s' statt %d" % (name, i, antwort, a + b))
        fehler += ["%s/%d: %s" % (name, i, f) for f in pruefe_spaltenweg(weg, a, b, antwort, schema)]
        summen.add(a + b)
        if len(fehler) > 25:
            return fehler
    return fehler, summen


def zufallserwartung(summen):
    """Wahrscheinlichkeit, mit einer konstanten Antwort zufaellig zu treffen."""
    if not summen:
        return 0.0
    return 1.0 / len(summen)


def baue_prompt(rec, mode):
    if mode == "m1":
        return rec.split(bytes([B_THINK]))[0] + bytes([B_THINK])
    aufgabe, weg, _ = zerlege(rec)
    return (bytes([B_TASK]) + (" " + aufgabe + " ").encode("ascii") + bytes([B_THINK])
            + (" " + weg + " ").encode("ascii") + bytes([B_ANSWER]))


def normalisiere(roh):
    daten = roh
    idx = daten.find(bytes([B_ANSWER]))
    if idx != -1:
        daten = daten[idx + 1:]
    for marke in (B_END, B_THINK, B_TASK):
        idx = daten.find(bytes([marke]))
        if idx != -1:
            daten = daten[:idx]
    return daten.decode("utf-8", "replace").strip().split()[0] if daten.strip() else ""


def tabelle(zeilen, kopf):
    breiten = [max(len(str(z[i])) for z in zeilen + [kopf]) for i in range(len(kopf))]
    print("  ".join(kopf[i].ljust(breiten[i]) for i in range(len(kopf))))
    print("  ".join("-" * b for b in breiten))
    for z in zeilen:
        print("  ".join(str(z[i]).ljust(breiten[i]) for i in range(len(z))))


def main():
    p = argparse.ArgumentParser(description="Runner/Pruefer fuer den Additionsbenchmark")
    p.add_argument("--split", choices=sorted(SPLITS), default="train-2")
    p.add_argument("--data", default=os.path.join(HIER, "data", "add"))
    p.add_argument("--mode", choices=("m1", "m2"), default="m1")
    p.add_argument("--schema", choices=("normal", "lsd"), default="normal")
    p.add_argument("--runner", default=None)
    p.add_argument("--limit", type=int, default=0, help="nur die ersten N Aufgaben (0 = alle)")
    p.add_argument("--selftest", action="store_true", help="Bewertung gegen die gespeicherten Antworten")
    p.add_argument("--validate", action="store_true", help="Bytes, Rechenweg und Split-Disziplin pruefen")
    p.add_argument("--json", dest="json_out", default=None)
    p.add_argument("--samples", type=int, default=0, help="N Datensaetze ausgeben")
    args = p.parse_args()

    ordner, stellen = SPLITS[args.split]
    shard = os.path.join(args.data, ordner, "shard-0000.bin")
    if not os.path.exists(shard):
        sys.exit("FEHLER: %s fehlt - erst 'python3 add_data.py' laufen lassen" % shard)
    records = lies_shard(shard)

    if args.validate:
        alle = {}
        fehler = []
        for name, (ordnername, st) in sorted(SPLITS.items()):
            pfad = os.path.join(args.data, ordnername, "shard-0000.bin")
            if os.path.exists(pfad):
                recs = lies_shard(pfad)
                ergebnis = pruefe_split(name, recs, args.schema, st)
                if isinstance(ergebnis, tuple):
                    fehler += ergebnis[0]
                    alle[name] = ergebnis[1]
                else:
                    fehler += ergebnis
        texte = {}
        for name, (ordnername, _) in sorted(SPLITS.items()):
            pfad = os.path.join(args.data, ordnername, "shard-0000.bin")
            if os.path.exists(pfad):
                texte[name] = set(lies_shard(pfad))
        namen = sorted(texte)
        for i in range(len(namen)):
            for j in range(i + 1, len(namen)):
                gemeinsam = texte[namen[i]] & texte[namen[j]]
                if gemeinsam:
                    fehler.append("Split-Ueberschneidung %s/%s: %d Aufgaben" % (namen[i], namen[j], len(gemeinsam)))
        if fehler:
            print("VALIDIERUNG FEHLGESCHLAGEN:")
            for f in fehler[:25]:
                print("  - %s" % f)
            sys.exit(1)
        print("Validierung OK: %d Splits, Rechenwege und Uebertragsketten korrekt, "
              "Split-Disziplin eingehalten, keine verbotenen Bytes." % len(alle))
        for name in sorted(alle):
            e = zufallserwartung(alle[name])
            print("  %-8s Summen: %-4d moeglich, Zufallserwartung je Aufgabe %.4f %%"
                  % (name, len(alle[name]), 100 * e))
        return 0

    if args.samples:
        for rec in records[:args.samples]:
            aufgabe, weg, antwort = zerlege(rec)
            print("AUFGABE  : %s" % aufgabe)
            print("DENKWEG  : %s" % weg)
            print("ANTWORT  : %s" % antwort)
            lesbar = baue_prompt(rec, "m2").decode("latin-1")
            for roh, sichtbar in ((chr(B_TASK), "<AUFGABE>"), (chr(B_THINK), "<DENKWEG>"),
                                  (chr(B_ANSWER), "<ANTWORT>"), (chr(B_END), "<ENDE>")):
                lesbar = lesbar.replace(roh, sichtbar)
            print("PROMPT m2: %s" % lesbar)
            print()
        return 0

    auswahl = records[:args.limit] if args.limit else records
    hat_antwort = args.selftest or bool(args.runner)
    if not hat_antwort:
        print("Kein --runner/--selftest: nur Aufgabenliste (%d Datensaetze in %s)." % (len(auswahl), ordner))
        print("Fuer eine Messung: --runner \"python3 add_bench/stub_perfect.py\" [--mode m1|m2]\n")

    zeilen = []
    richtig = 0
    klassen = {}
    for i, rec in enumerate(auswahl):
        aufgabe, weg, erwartet = zerlege(rec)
        if args.selftest:
            roh = (bytes([B_THINK]) + (" " + weg + " ").encode("ascii")
                   + bytes([B_ANSWER]) + (" " + erwartet + " ").encode("ascii") + bytes([B_END]))
        elif args.runner:
            prompt = baue_prompt(rec, args.mode)
            try:
                r = subprocess.run(shlex.split(args.runner), input=prompt,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            except OSError as e:
                sys.exit("FEHLER: Runner nicht ausfuehrbar: %s" % e)
            if r.returncode != 0 and not r.stdout:
                sys.exit("FEHLER: Runner brach ab (Code %d): %s"
                         % (r.returncode, r.stderr.decode("utf-8", "replace")[:200]))
            roh = r.stdout
        else:
            roh = b""
        antwort = normalisiere(roh) if hat_antwort else "?"
        ok = antwort == erwartet
        if hat_antwort:
            richtig += 1 if ok else 0
            a, b = operanden(aufgabe, args.schema)
            k = "mit Uebertrag" if (a % 10) + (b % 10) >= 10 else "ohne Uebertrag"
            eintrag = klassen.setdefault(k, [0, 0])
            eintrag[1] += 1
            if ok:
                eintrag[0] += 1
        if i < 20 or not ok:
            zeilen.append([aufgabe, erwartet, antwort or "-", "OK" if ok else ("--" if not hat_antwort else "FALSCH")])
    if len(zeilen) > 40:
        print("(erste 20 Zeilen und alle Fehler)\n")

    print("Additionsbenchmark - Split %s (%s Datensaetze), Modus %s%s"
          % (args.split, len(auswahl), args.mode, " - Selbsttest" if args.selftest else ""))
    print()
    tabelle(zeilen, ["Aufgabe", "Erwartet", "Modell", "Bewertung"])
    print()
    if hat_antwort:
        summen = {int(zerlege(rec)[2]) for rec in auswahl if zerlege(rec)}
        chance = zufallserwartung(summen)
        print("Punkte: %d von %d (%.1f %%)" % (richtig, len(auswahl), 100.0 * richtig / max(1, len(auswahl))))
        for k in sorted(klassen):
            h, g = klassen[k]
            print("  %-14s %d/%d" % (k, h, g))
        print("Zufallserwartung je Aufgabe (konstante Antwort): %.4f %%  (1 von %d moeglichen Summen)"
              % (100 * chance, len(summen)))
        print("Hinweis: Die Zufallserwartung ist der Vergleichsmassstab. Ein Wert darunter ist")
        print("         von Raten nicht unterscheidbar; erst deutlich darueber ist etwas gelernt.")
    else:
        print("Ohne Runner keine Punkte - nur Aufgabenliste.")
    if args.json_out:
        daten = {
            "split": args.split,
            "mode": args.mode,
            "schema": args.schema,
            "runner": args.runner,
            "richtig": richtig if hat_antwort else None,
            "gesamt": len(auswahl),
            "kategorien": {k: {"richtig": v[0], "gesamt": v[1]} for k, v in klassen.items()},
            "aufgaben": [dict(zip(("aufgabe", "erwartet", "modell", "bewertung"), z)) for z in zeilen],
        }
        with open(args.json_out, "w", encoding="utf-8") as f:
            json.dump(daten, f, indent=2, ensure_ascii=False)
        print("\nErgebnis geschrieben: %s" % args.json_out)
    return 0


if __name__ == "__main__":
    sys.exit(main())