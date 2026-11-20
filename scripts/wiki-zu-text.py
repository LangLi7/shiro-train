#!/usr/bin/env python3
"""Wikipedia-Dump zu sauberem Fliesstext (Runde 456).

**Warum zeilenweise und nicht per Ersetzung.** Der erste Versuch entfernte nur
einfache `{{...}}`-Vorlagen und liess **33 % Markup** stehen - gemessen an 2,8
Millionen Zeilen. Beispiele:

    | 1012 = 10004
    |-
    |style="text-align:left;"|

Bei 16 GB Korpus waeren das **5 GB Tabellensyntax**. Ein Modell, das darauf
trainiert, lernt, dass deutsche Texte aus `|-` und `|style=` bestehen - genau die
Art Muster, die spaeter als "Form gelernt, Inhalt nicht" endet.

**Diese Fassung filtert Zeile fuer Zeile:** was nach Markup aussieht, faellt ganz
weg. Das kostet etwas Text, ist aber sauberer als jede Ersetzung.

Belegung: ein Durchlauf liest den Strom, die Artikel gehen **reihum** auf N
Dateien. Acht Prozesse auf dieselbe Datei waeren achtmal derselbe Text gewesen -
`iterparse` liest immer von vorn.

Aufruf:
    python3 scripts/wiki-zu-text.py <dump.xml.bz2> <zielordner> [teile]

Ergebnis: <zielordner>/sauber-0.txt bis sauber-N.txt, reiner Fliesstext.
"""
import bz2
import re
import sys
import os
import xml.etree.ElementTree as ET

# Zeilen, die ganz wegfallen: Tabellen, Vorlagen, Attribute
WEG = re.compile(r"^\s*([|!{}]|\|-|\[\[|\]\]|style=|class=|rowspan|colspan|align=)")
# Zeilen aus lauter Sonderzeichen
KAUDER = re.compile(r"^[\s\|\-\+\=\!\{\}\[\]\#\*:;.,]*$")
# Reste, die nach dem Aufloesen noch Markup tragen
REST = ("{{", "}}", "[[", "]]")


def zeilen_saeubern(text: str) -> list[str]:
    """Zerlegt einen Artikel in brauchbare Zeilen."""
    text = re.sub(r"<ref[^>]*>.*?</ref>", "", text, flags=re.S)
    text = re.sub(r"<ref[^>]*/>", "", text)
    text = re.sub(r"<[^>]+>", "", text)
    raus = []
    for ln in text.split("\n"):
        s = ln.strip()
        if not s or WEG.match(s) or KAUDER.match(s):
            continue
        s = re.sub(r"\{\{[^{}]*\}\}", "", s)
        s = re.sub(r"\[\[([^\]|]*\|)?([^\]]*)\]\]", r"\2", s)
        s = re.sub(r"'''?", "", s)
        s = re.sub(r"\s{2,}", " ", s).strip()
        if not s or any(r in s for r in REST) or len(s) < 20:
            continue
        raus.append(s)
    return raus


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    quelle, ziel = sys.argv[1], sys.argv[2]
    teile = int(sys.argv[3]) if len(sys.argv) > 3 else 8
    os.makedirs(ziel, exist_ok=True)

    aus = [open(os.path.join(ziel, f"sauber-{i}.txt"), "w", encoding="utf-8")
           for i in range(teile)]
    artikel = 0
    bytes_out = 0
    k = 0
    try:
        with bz2.open(quelle, "rb") as f:
            for _ev, el in ET.iterparse(f, events=("end",)):
                if el.tag.endswith("text"):
                    z = zeilen_saeubern(el.text or "")
                    if z:
                        block = "\n".join(z)
                        if len(block) > 200:
                            aus[k % teile].write(block + "\n\n")
                            k += 1
                            artikel += 1
                            bytes_out += len(block)
                el.clear()
    except Exception as e:  # noqa: BLE001 - Abbruch soll sichtbar sein
        print("Abbruch:", e)
    finally:
        for w in aus:
            w.close()
    print(f"Artikel: {artikel}  Bytes: {bytes_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
