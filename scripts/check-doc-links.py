"""Prueft alle lokalen Markdown-Links in den angegebenen Dateien.
Reines Leseskript, aendert nichts. Aufruf:
    python3 scripts/check-doc-links.py <datei> [<datei> ...]
Exit 0 = alle Links aufloesbar, 1 = mindestens ein toter Link.
"""
import os
import re
import sys

LINK = re.compile(r"\]\(([^)]+)\)")
FENCED = re.compile(r"```.*?```", re.S)
INLINE_CODE = re.compile(r"`[^`\n]*`")


def strip_code(text):
    """Entfernt Codebloecke und Inline-Code.

    Ohne diesen Schritt liest der Link-Regex Syntax aus Codebeispielen als Link
    - beobachtet an Go-Generics: ``func Max[T cmp.Ordered](a, b T) T`` wurde als
    Linkziel 'a, b T' gedeutet. Ein Pruefer, der Code fuer Text haelt, meldet
    Fehler, die keine sind.
    """
    text = FENCED.sub("", text)
    return INLINE_CODE.sub("", text)


def check(path):
    base = os.path.dirname(path)
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except OSError as exc:
        print("  FEHLER: %s nicht lesbar (%s)" % (path, exc))
        return 1
    bad = 0
    seen = set()
    for target in LINK.findall(strip_code(text)):
        target = target.split("#", 1)[0].strip()
        if not target or target in seen:
            continue
        seen.add(target)
        if target.startswith(("http://", "https://", "mailto:")):
            continue
        resolved = os.path.normpath(os.path.join(base, target))
        if not os.path.exists(resolved):
            print("  TOTER LINK: %s -> %s" % (path, target))
            bad += 1
    print("  %-46s %3d Links geprueft, %d tot" % (path, len(seen), bad))
    return bad


def main(argv):
    if len(argv) < 2:
        print("Aufruf: check-doc-links.py <datei> [...]")
        return 2
    total = sum(check(path) for path in argv[1:])
    print("Summe tote Links: %d" % total)
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
