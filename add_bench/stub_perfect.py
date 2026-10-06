#!/usr/bin/env python3
"""Test-Stub: verhaelt sich wie ein perfektes Additionsmodell (ohne Modell).

Liest die Prompt-Bytes von stdin, holt die Aufgabe 'a+b' heraus, rechnet die Summe
und gibt sie plus <ENDE> (0xFF) aus. Schema ueber Umgebungsvariable ADD_SCHEMA.
Reines Testwerkzeug - kein Messergebnis.
"""
import os, sys
schema = os.environ.get("ADD_SCHEMA", "normal")
prompt = sys.stdin.buffer.read().split(bytes([0xFE]), 1)[-1]
text = prompt.split(bytes([0xFD]), 1)[0].decode("ascii", "replace").strip()
links, rechts = text.split("+")
if schema == "lsd":
    links, rechts = links[::-1], rechts[::-1]
summe = int(links) + int(rechts)
antwort = str(summe) if schema == "normal" else str(summe)[::-1]
sys.stdout.buffer.write(antwort.encode("ascii") + bytes([0xFF]))