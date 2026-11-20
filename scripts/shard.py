#!/usr/bin/env python3
"""Shard-Dateien lesen und schreiben - passend zu `src/data/format.rs`.

**Warum dieses Modul (Runde 467).** Ein erster Versuch las die Nutzbytes als
zusammenhaengenden Block an Offset 12. Beide Annahmen waren falsch:

| Offset | Feld | Groesse |
|---|---|---|
| 0-4    | Magic `BSHD` | 4 |
| 4-6    | Version | 2 |
| 6-8    | Headergroesse (32) | 2 |
| **8-16** | **record_count** | **8** |
| 16-24  | Flags | 8 |
| 24-32  | reserviert | 8 |

und ab Offset 32 folgen die Datensaetze **einzeln**, jeder mit **2 Byte Laenge**
(kleines Ende) davor. Wer das als Block liest, bekommt Muell - oder, wie im Test,
**null Bytes** und einen leeren Korpus.

Der Test mit kleinen Datenmengen hat das vor dem 500M-Lauf aufgedeckt: haette er es
nicht, waeren Stunden GPU-Zeit auf einen leeren Korpus gegangen.
"""

from __future__ import annotations

import struct
from pathlib import Path

MAGIC = b"BSHD"
HEADER_SIZE = 32


def lies_records(pfad: str | Path) -> list[bytes]:
    """Liest alle Datensaetze einer Shard-Datei."""
    d = Path(pfad).read_bytes()
    if len(d) < HEADER_SIZE or d[:4] != MAGIC:
        raise ValueError(f"kein Shard: {pfad}")
    version, kopf = struct.unpack("<HH", d[4:8])
    if kopf != HEADER_SIZE:
        raise ValueError(f"Kopf {kopf} statt {HEADER_SIZE}: {pfad}")
    (anzahl,) = struct.unpack("<Q", d[8:16])
    raus: list[bytes] = []
    off = HEADER_SIZE
    for _ in range(anzahl):
        if off + 2 > len(d):
            break
        (n,) = struct.unpack("<H", d[off : off + 2])
        off += 2
        raus.append(d[off : off + n])
        off += n
    if version > 1:
        # Versionen ueber 1 kennen wir nicht - laut sagen, nicht still raten
        print(f"  Warnung: Shard-Version {version} in {pfad}")
    return raus


def schreibe_records(pfad: str | Path, records: list[bytes], note: str = "") -> int:
    """Schreibt Datensaetze als Shard. Gibt die Dateigroesse zurueck."""
    p = Path(pfad)
    p.parent.mkdir(parents=True, exist_ok=True)
    kopf = bytearray(HEADER_SIZE)
    kopf[0:4] = MAGIC
    kopf[4:6] = struct.pack("<H", 1)
    kopf[6:8] = struct.pack("<H", HEADER_SIZE)
    kopf[8:16] = struct.pack("<Q", len(records))
    with open(p, "wb") as f:
        f.write(bytes(kopf))
        for r in records:
            f.write(struct.pack("<H", len(r)))
            f.write(r)
    return p.stat().st_size
