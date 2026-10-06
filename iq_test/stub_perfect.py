#!/usr/bin/env python3
"""Test-Stub: verhaelt sich wie ein perfektes Modell (Antwort aus tasks.jsonl)."""
import json, os, sys
hier = os.path.dirname(os.path.abspath(__file__))
prompt = sys.stdin.buffer.read().decode("utf-8", "replace")
with open(os.path.join(hier, "tasks.jsonl"), encoding="utf-8") as f:
    for zeile in f:
        a = json.loads(zeile)
        if a["task"] in prompt:
            sys.stdout.buffer.write(a["answer"].encode("ascii") + bytes([0xFF]))
            sys.exit(0)
sys.stdout.buffer.write(b"?" + bytes([0xFF]))