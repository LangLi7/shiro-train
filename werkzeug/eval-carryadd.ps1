# Auswertung des Carryadd-Laufs (Runde 343/347).
#
# Misst den neuen Checkpoint auf vier Saetzen, in dieser Reihenfolge:
#   1. carryadd eval-4d   - Auswendiglernen-Check (groessere Stellen, nie im Training)
#   2. carryadd eval-3d   - Zielgroesse (Format + Uebertrag)
#   3. alter Heldout      - Schutzmetrik (darf nicht unter ~73 % fallen)
#   4. frischer Blindtest - ehrliche Zahl ausserhalb der Trainingsverteilung
#
# Aufruf:  .\werkzeug\eval-carryadd.ps1 -Checkpoint <pfad.safetensors>

param(
    [Parameter(Mandatory = $true)][string]$Checkpoint,
    [int]$Answers = 50,
    [int]$BlindAnswers = 120
)

$ErrorActionPreference = "Stop"
$env:CARGO_TARGET_DIR = "target-win-cuda"
$exe = "target-win-cuda\release\shiro.exe"

if (-not (Test-Path $Checkpoint)) { throw "Checkpoint fehlt: $Checkpoint" }

function Invoke-Eval {
    param([string]$Name, [string]$Config, [int]$N)
    Write-Host ""
    Write-Host "=== $Name ($N Records) ===" -ForegroundColor Cyan
    $out = & $exe eval --config $Config --checkpoint $Checkpoint --answers $N --answers-mode m1 --backend cuda 2>&1
    $out | Select-String -Pattern 'Gefragt|Loesungsweg|Weg \(inhaltlich\)' | ForEach-Object { $_.Line }
    return $out
}

Write-Host "Checkpoint: $Checkpoint" -ForegroundColor Yellow

$r4d = Invoke-Eval -Name "carryadd eval-4d (Auswendiglernen-Check)" -Config "target-win\eval-carryadd4d.toml" -N $Answers
$r3d = Invoke-Eval -Name "carryadd eval-3d (Zielgroesse)" -Config "target-win\eval-carryadd3d.toml" -N $Answers
$rOld = Invoke-Eval -Name "alter Heldout (Schutzmetrik)" -Config "target-win\eval-fokus-local.toml" -N $Answers
$rNew = Invoke-Eval -Name "frischer Blindtest (ehrliche Zahl)" -Config "target-win\eval-blindtest-neu.toml" -N $BlindAnswers

Write-Host ""
Write-Host "=== ZUSAMMENFASSUNG ===" -ForegroundColor Yellow
Write-Host "Die vier Messungen stehen oben. Entscheidungsregel (vorregistriert, R336):"
Write-Host "  - Uebernehmen nur wenn eval-3d UND eval-4d steigen und der alte Heldout"
Write-Host "    innerhalb der Toleranz bleibt (> ~70 %)."
Write-Host "  - Steigt nur eval-3d, waehrend eval-4d bei 0 % bleibt: Auswendiglernen, verwerfen."
