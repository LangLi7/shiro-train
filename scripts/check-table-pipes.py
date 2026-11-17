"""Findet Markdown-Tabellenzeilen mit einem NICHT escapten Pipe im Code-Span.

Warum es dieses Skript gibt
---------------------------
Ein Beispiel wie ``(`10+10 | E 0+0=0 ; R 20`)`` in einer Tabellenzelle sieht
richtig aus, zerlegt die Tabelle aber in eine zusaetzliche Spalte: Markdown
trennt Zellen am Pipe-Zeichen und kennt den Code-Span dabei nicht als Schutz.
In diesem Projekt ist genau das passiert (D-017-Zeile im Entscheidungsregister),
und kein vorhandener Pruefer hat es gemerkt. Richtig ist die Schreibweise mit
Backslash: ``(`10+10 \\| E 0+0=0 ; R 20`)``.

Reines Leseskript. Aufruf:
    python3 scripts/check-table-pipes.py [datei ...]
Ohne Argumente werden alle .md unter docs/ plus die Wurzeldateien geprueft.
Exit 0 = sauber, 1 = mindestens eine zerbrochene Tabellenzeile.
"""
import glob
import os
import re
import sys

CODE_SPAN = re.compile(r"`[^`\n]*`")
UNESCAPED_PIPE = re.compile(r"(?<!\\)\|")

WURZEL = ["LOGIC_ANALYTICS.md", "README.md", "MODEL_FORMAT.md", "IQ_TEST.md",
          "docs/verifikation.md", "docs/abschluss-vorbereitungsphase.md"]


def sammle_standard():
    dateien = sorted(glob.glob("docs/**/*.md", recursive=True))
    dateien += [p for p in WURZEL if os.path.exists(p) and p not in dateien]
    return dateien


def pruefe(pfad):
    treffer = []
    try:
        with open(pfad, encoding="utf-8") as handle:
            zeilen = handle.readlines()
    except OSError as exc:
        print("  FEHLER: %s nicht lesbar (%s)" % (pfad, exc))
        return treffer
    for nummer, roh in enumerate(zeilen, 1):
        zeile = roh.rstrip("\n")
        if not zeile.lstrip().startswith("|"):
            continue
        for span in CODE_SPAN.finditer(zeile):
            if UNESCAPED_PIPE.search(span.group(0)):
                treffer.append((pfad, nummer, span.group(0).strip()[:70]))
    return treffer


def main(argv):
    dateien = argv[1:] or sammle_standard()
    treffer = []
    for pfad in dateien:
        treffer += pruefe(pfad)
    for pfad, nummer, ausschnitt in treffer:
        print("  ZERBROCHENE TABELLE: %s Z.%d  %s" % (pfad, nummer, ausschnitt))
    print("%d Dateien geprueft, %d zerbrochene Tabellenzeilen"
          % (len(dateien), len(treffer)))
    return 1 if treffer else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
