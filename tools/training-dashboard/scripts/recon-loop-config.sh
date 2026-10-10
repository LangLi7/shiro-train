#!/usr/bin/env bash
# Woher nimmt der agentische Loop seine Trainingskonfiguration?
set -u
R="/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro"
cd "$R"
echo "=== ein Loop-Lauf: run_start ==="
f="$(ls logs/ | grep '^agent-' | head -1)"
head -1 "logs/$f"
echo
echo "=== data_loaded / checkpoint ==="
grep -m2 -E 'data_loaded|checkpoint_written' "logs/$f"
echo
echo "=== configs/capabilities ==="
ls configs/capabilities 2>/dev/null
echo
echo "=== schreibt der Loop eine Config? (Suche in src) ==="
grep -rn "train\..*\.toml" src/agent.rs | head -20