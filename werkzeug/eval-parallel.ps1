# Wertet mehrere Checkpoints GLEICHZEITIG aus (Runde 473).
#
# Die Eval erzeugt ihre Antworten sequenziell und nutzt einen Kern. Der Pod hat
# 32 vCPU und 232 GiB RAM - ungenutzt. Dieses Skript startet je Checkpoint einen
# eigenen Prozess, damit N Checkpoints N Kerne belegen statt einen.
#
#   .\werkzeug\eval-parallel.ps1 -Checkpoints "models\a\model.safetensors","models\b\model.safetensors"
#
# Standard ist `-Backend cpu`, weil das der Sinn der Parallelisierung ist: die
# GPU bleibt frei fuer das Training. Mit `-Backend cuda` laufen zwei gleichzeitig
# (12 GB reichen nicht fuer mehr), danach wird gewartet.
#
# Die Ausgabe ist eine Zeile je Checkpoint, damit man sie in eine Tabelle
# uebernehmen kann:  <Checkpoint>  Antwort  <x> %  Weg  <y> %

param(
    [Parameter(Mandatory = $true)][string[]]$Checkpoints,
    [string]$Config = "target-win\eval-gross100k.toml",
    [int]$Answers = 30,
    [string]$Backend = "cpu",
    [int]$MaxParallel = 0
)

$ErrorActionPreference = "Stop"
$exe = "target-win-cuda\release\shiro.exe"
if (-not (Test-Path $exe)) { throw "Binary fehlt: $exe" }

if ($MaxParallel -le 0) {
    $MaxParallel = [Math]::Min($Checkpoints.Count, [Environment]::ProcessorCount)
}

Write-Host ("{0} Checkpoints, hoechstens {1} gleichzeitig, Backend {2}" -f $Checkpoints.Count, $MaxParallel, $Backend)

$queue = [System.Collections.Queue]::new()
$Checkpoints | ForEach-Object { $queue.Enqueue($_) }

$running = @()
$done = @()

while ($queue.Count -gt 0 -or $running.Count -gt 0) {
    while ($running.Count -lt $MaxParallel -and $queue.Count -gt 0) {
        $ck = $queue.Dequeue()
        $name = Split-Path $ck -Leaf
        Write-Host ("  start  {0}" -f $name)
        $running += Start-Job -Name $name -ScriptBlock {
            param($c, $cfg, $a, $b, $exePath)
            & $exePath eval --config $cfg --checkpoint $c --answers $a --answers-mode m1 --backend $b 2>&1
        } -ArgumentList $ck, $Config, $Answers, $Backend, $exe
    }

    $finished = $running | Where-Object { $_.State -ne "Running" }
    foreach ($j in $finished) {
        $out = Receive-Job $j -ErrorAction SilentlyContinue
        $ans = ($out | Select-String -Pattern 'Gefragt\s*:.*\(([\d.,]+) %\)').Matches
        $path = ($out | Select-String -Pattern 'Loesungsweg\s*:\s*richtig bei \d+ \(([\d.,]+) %\)').Matches
        $done += [pscustomobject]@{
            Checkpoint = $j.Name
            Antwort    = if ($ans.Count) { $ans[0].Groups[1].Value } else { "?" }
            Weg        = if ($path.Count) { $path[0].Groups[1].Value } else { "?" }
        }
        Remove-Job $j
        $running = $running | Where-Object { $_.Id -ne $j.Id }
    }

    if ($running.Count -gt 0) { Start-Sleep -Seconds 2 }
}

Write-Host ""
$done | Format-Table -AutoSize
