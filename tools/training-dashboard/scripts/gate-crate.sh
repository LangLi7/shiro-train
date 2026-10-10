# Der Gate-Befehl des Auftrags, wortgleich als Kette. In eine Datei geschrieben,
# weil die Aufrufschicht `$`-Variablen zerstoert und die Klammern in
# Pfaden mit Leerzeichen sonst als Syntaxfehler gelesen werden.
export PATH=/home/changchi/.cargo/bin:$PATH && cd '/mnt/d/.Projekt/Programmieren/AI Train Project/AI Models/Shiro/tools/training-dashboard' && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && cargo build --release
status=$?
echo "KETTEN-EXIT=$status"
exit $status
