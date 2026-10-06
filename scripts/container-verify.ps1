# Container-Verifikation fuer Shiro.
#
# Aufruf (Windows blockiert .ps1-Dateien per Ausfuehrungsrichtlinie):
#   pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/container-verify.ps1
#
# Reihenfolge wie im Pod. Jede Stufe prueft den EXIT-CODE, nicht nur die Ausgabe:
# eine Binary, die nichts tut und 0 zurueckgibt, waere sonst unsichtbar. Genau
# dieser Fall ist einmal aufgetreten (Stub-Binary durch fehlerhaftes Caching im
# Dockerfile), deshalb endet der Image-Build jetzt mit `shiro version`.

param(
    [string]$Tag = 'shiro-runtime:0.1.0',
    [string]$Volume = 'shiro-verify'
)

$fail = 0
function Step {
    param([string]$Name, [scriptblock]$Body)
    Write-Host ''
    Write-Host "=== $Name ===" -ForegroundColor Cyan
    $out = & $Body 2>&1 | Out-String
    $code = $LASTEXITCODE
    Write-Host $out.TrimEnd()
    if ($code -ne 0) {
        Write-Host ">>> FEHLGESCHLAGEN (Exit $code)" -ForegroundColor Red
        $script:fail++
    } else {
        Write-Host '>>> OK' -ForegroundColor Green
    }
}

Set-Location -LiteralPath (Split-Path -Parent $PSScriptRoot)

# Alle drei Pfade liegen UNTER dem Mountpoint. Ein Mount verdeckt, was im Image an
# derselben Stelle liegt - laegen Checkpoints auf /checkpoints (Container-Schicht),
# waehrend das Volume auf /data haengt, ueberlebte kein Checkpoint einen Neustart.
$envArgs = @(
    '-e', 'SHIRO_DATA=/data',
    '-e', 'SHIRO_CHECKPOINTS=/data/checkpoints',
    '-e', 'SHIRO_LOGS=/data/logs'
)

Step '1. Image vorhanden' {
    docker image inspect $Tag --format '{{.Id}} | {{.Size}} Bytes'
}

Step '2. version meldet sich (faengt eine stille Stub-Binary)' {
    docker run --rm --entrypoint /usr/local/bin/shiro $Tag version
}

Step '3. info: Zielarchitektur Modell S' {
    docker run --rm --entrypoint /usr/local/bin/shiro $Tag info --config /app/configs/model.small.toml
}

Step '4. smoke im Image-Datensatz (volle Kette)' {
    docker run --rm --entrypoint /usr/local/bin/shiro $Tag smoke --config /app/configs/train.smoke.toml --steps 8
}

Write-Host ''
Write-Host '=== 5. Volume: Daten erzeugen, trainieren, Persistenz pruefen ===' -ForegroundColor Cyan
docker volume rm -f $Volume 2>&1 | Out-Null
docker volume create $Volume | Out-Null

# Hinweis: Docker Desktop befuellt ein NEUES benanntes Volume mit dem Inhalt des
# Image-Pfads. Ein "leeres" Volume hat hier also bereits data/sample. Deshalb wird
# bewusst neu erzeugt, statt sich auf einen leeren Zustand zu verlassen.
Step '5a. gen-sample ins Volume' {
    docker run --rm -v "${Volume}:/data" @envArgs $Tag gen-sample --records 256 --out /data | Select-Object -Last 3
}
Step '5b. train mit Checkpoint auf dem Volume' {
    docker run --rm -v "${Volume}:/data" @envArgs $Tag train --config /app/configs/train.smoke.toml --steps 8 | Select-Object -Last 2
}
Step '5c. Persistenz aus einem NEUEN Container lesen' {
    docker run --rm -v "${Volume}:/data" --entrypoint /bin/sh $Tag -c 'test -f /data/checkpoints/*.bckpt && ls /data/checkpoints && ls /data/logs'
}
Step '5d. Checkpoint-Gegenprobe aus dem Volume' {
    docker run --rm -v "${Volume}:/data" @envArgs $Tag verify-checkpoint | Select-Object -Last 4
}

Write-Host ''
if ($fail -eq 0) {
    Write-Host 'ALLE CONTAINER-PRUEFUNGEN BESTANDEN' -ForegroundColor Green
    exit 0
} else {
    Write-Host "$fail Pruefung(en) fehlgeschlagen" -ForegroundColor Red
    exit 1
}