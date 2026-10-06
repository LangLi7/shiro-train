#!/usr/bin/env python3
"""Test-Stub: antwortet immer '0' (Negativkontrolle).

Zeigt, dass die Bewertung nicht pauschal Erfolg meldet, und macht die
Zufallserwartung greifbar. Reines Testwerkzeug - kein Messergebnis.
"""
import sys
sys.stdin.buffer.read()
sys.stdout.buffer.write(b"0" + bytes([0xFF]))