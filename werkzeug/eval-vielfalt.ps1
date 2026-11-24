# Wertet einen Checkpoint auf allen vier Saetzen aus (Runde 369).
#
# Ergebnis UND Weg, immer Modus m1. m2 wird nicht gemessen - es sagt nichts
# ueber freie Generierung (Lehre aus Runde 355: m2 100 %, m1 3,3 %).
#
#   .\werkzeug\eval-vielfalt.ps1 -Checkpoint "models\vielfalt500\vielfalt500-step15000.safetensors"
#
# Die Reihenfolge ist Absicht: erst die geschuetzte Faehigkeit (alter
# Heldout), dann die Zielgroessen. Faellt der Schutz unter 70 %, ist der
# Checkpoint verworfen, egal wie gut die Ziele sind.

param(
    [Parameter(Mandatory = $true)][string]$Checkpoint,
    [int]$Answers = 30,
    [string]$Backend = "cuda"
)

$ErrorActionPreference = "Stop"
$env:CARGO_TARGET_DIR = "target-win-cuda"
$exe = "target-win-cuda\release\shiro.exe"

if (-not (Test-Path $Checkpoint)) { throw "Checkpoint fehlt: $Checkpoint" }
if (-not (Test-Path $exe)) { throw "Binary fehlt: $exe (cargo build --release --features cuda)" }

# Name, Config, Baseline (Antwort-EM in Prozent), Rolle
$sets = @(
    @{ Name = "alter Heldout (Schutz)";  Config = "target-win\eval-fokus-local.toml";  Base = "73,0 %" },
    @{ Name = "4-5-stellig (Ziel)";      Config = "target-win\eval-gross100k.toml";    Base = "0,0 %"  },
    @{ Name = "7-stellig (Laenge)";      Config = "target-win\eval-langtest7.toml";    Base = "0,0 %"  },
    @{ Name = "8-10-stellig (Laenge)";   Config = "target-win\eval-langtest.toml";     Base = "0,0 %"  }
)

$rows = @()
foreach ($s in $sets) {
    Write-Host ""
    Write-Host ("=== {0}  (Baseline {1}) ===" -f $s.Name, $s.Base) -ForegroundColor Cyan
    $out = & $exe eval --config $s.Config --checkpoint $Checkpoint --answers $Answers --answers-mode m1 --backend $Backend 2>&1
    $ans = ($out | Select-String -Pattern 'Gefragt\s*:\s*\d+ .*richtig:\s*\d+\s*\(([\d.,]+) %\)').Matches
    $path = ($out | Select-String -Pattern 'Loesungsweg\s*:\s*richtig bei \d+ \(([\d.,]+) %\)').Matches
    $inhalt = ($out | Select-String -Pattern 'Weg \(inhaltlich\):\s*\d+ \(([\d.,]+) %\)').Matches
    $r = @{
        Satz    = $s.Name
        Baseline = $s.Base
        Antwort = if ($ans.Count) { $ans[0].Groups[1].Value } else { "?" }
        Weg     = if ($path.Count) { $path[0].Groups[1].Value } else { "?" }
        Inhalt  = if ($inhalt.Count) { $inhalt[0].Groups[1].Value } else { "?" }
    }
    $rows += [pscustomobject]$r
    Write-Host ("  Antwort {0}  Weg {1}  inhaltlich {2}" -f $r.Antwort, $r.Weg, $r.Inhalt)
}

Write-Host ""
Write-Host "=== Zusammenfassung ===" -ForegroundColor Cyan
$rows | Format-Table -AutoSize | Out-String | Write-Host

# Vorab festgelegte Uebernahmeregel (Loop-Schritt 6)
$schutz = $rows[0].Antwort -replace ',', '.'
$ziel = $rows[1].Antwort -replace ',', '.'
Write-Host "Uebernahmeregel (vor der Messung festgelegt):" -ForegroundColor Yellow
Write-Host "  1. Schutz (alter Heldout) muss >= 70 % bleiben - sonst verworfen."
Write-Host "  2. Ziel (4-5-stellig) muss ueber der Baseline 0 % liegen - sonst kein Zugewinn."
Write-Host "  3. Nur wenn beides gilt, ist der Checkpoint Kandidat."
Write-Host ""
Write-Host ("Aktuell: Schutz {0} %, Ziel {1} %" -f $schutz, $ziel)
