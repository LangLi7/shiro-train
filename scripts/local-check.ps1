# Lokale Verifikation auf Windows.
#
# WICHTIG: Auf diesem Rechner blockiert die Windows-Anwendungssteuerung frisch
# gelinkte .exe-Dateien ("Eine Anwendungssteuerungsrichtlinie hat diese Datei
# blockiert", os error 4551). Rust erzeugt auf Windows immer eine .exe, weil das
# das ausfuehrbare Format von Windows ist - das ist keine Rust-Eigenheit, sondern
# die Plattformkonvention (auf Linux heisst dieselbe Datei "shiro" ohne Endung).
#
# Deshalb ist der belastbare Weg die Linux-Seite:
#   scripts/wsl-verify.sh   in WSL2 (echter Linux-Kernel, keine PE-Pruefung)
#   deploy/Containerfile    im Container
#
# Dieses Skript macht trotzdem, was auf Windows zuverlaessig geht: formatieren,
# linten und die Bibliothekstests starten. Scheitert der Start an der Richtlinie,
# wird das klar gemeldet statt als raetselhafter Fehler.

$ErrorActionPreference = 'Continue'
Set-Location -LiteralPath (Split-Path -Parent $PSScriptRoot)

Write-Host '=== cargo fmt --check ===' -ForegroundColor Cyan
cargo fmt --check

Write-Host '=== cargo clippy ===' -ForegroundColor Cyan
cargo clippy --release --all-targets

Write-Host '=== cargo test --release ===' -ForegroundColor Cyan
cargo test --release
$testExit = $LASTEXITCODE

if ($testExit -ne 0) {
  Write-Host ''
  Write-Host 'Die Testbinary konnte nicht gestartet werden oder Tests sind rot.' -ForegroundColor Yellow
  Write-Host 'Bei "Anwendungssteuerungsrichtlinie hat diese Datei blockiert":' -ForegroundColor Yellow
  Write-Host '  wsl -d Ubuntu -- bash scripts/wsl-verify.sh' -ForegroundColor Yellow
  Write-Host '  oder: docker build -f deploy/Containerfile -t shiro-dev .' -ForegroundColor Yellow
  exit $testExit
}

Write-Host '=== cargo run --release -- smoke ===' -ForegroundColor Cyan
cargo run --release --quiet -- smoke

Write-Host ''
Write-Host 'Lokal fertig. Fuer den Container:' -ForegroundColor Green
Write-Host '  cargo build --release' -ForegroundColor Green
Write-Host '  docker build -f deploy/Dockerfile -t shiro-runtime:0.1.0 .' -ForegroundColor Green