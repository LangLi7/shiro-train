#!/usr/bin/env python3
"""Test-Stub: antwortet immer 'ja' (Negativkontrolle fuer die Bewertung)."""
import sys
sys.stdin.buffer.read()
sys.stdout.buffer.write(b"ja" + bytes([0xFF]))