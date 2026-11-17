# Paraphrase-Transfer: dieselbe Aufgabe mit einer Formulierung, die nie im Training war.
#
# Warum das zaehlt: das Modell soll die Aufgabe loesen, nicht eine Zeichenkette wiedererkennen.
# Wenn ein Praefix wie "Bitte loese:" die Trefferquote zusammenbrechen laesst, hat es die
# trainierte Form auswendig gelernt.
#
# Aufruf:
#   .\paraphrase.ps1 -Checkpoint <pfad> -Shard <pfad> -Name Gleichungen -N 20
param(
    [Parameter(Mandatory = $true)][string]$Checkpoint,
    [Parameter(Mandatory = $true)][string]$Shard,
    [string]$Name = "familie",
    [int]$N = 20,
    [int]$MaxNew = 256,
    [string]$Config = "O:\shiro\pmh\eval25m.toml"
)

$ErrorActionPreference = "Stop"
$bin = "O:\shiro\target-moe\release\shiro.exe"
$mk = "O:\shiro\pmh\mktarget\release\mktool.exe"
$arbeit = Join-Path $env:TEMP ("paraph-" + [guid]::NewGuid().ToString("N").Substring(0, 8))

# Vier Formulierungen. Die erste ist die trainierte (Referenz), die anderen drei sind fremd.
$varianten = [ordered]@{
    "trainiert"  = ""
    "Bitte"      = "Bitte loese: "
    "Rechne"     = "Rechne: "
    "Frage"      = "Frage: "
}

function Wert([string]$s) {
    if (-not $s) { return $null }
    $t = ($s -replace '\s+', '').Trim()
    if ($t -match '^(-?\d+)[/,](\d+)$') { return [double]$matches[1] / [double]$matches[2] }
    if ($t -match '^(-?\d+)[.,](\d+)$') { return [double]("$($matches[1]).$($matches[2])") }
    if ($t -match '^-?\d+$') { return [double]$t }
    return $null
}

& $mk --wegextract $Shard $arbeit $N | Out-Null
if (-not (Test-Path (Join-Path $arbeit "p0.txt"))) { "$Name : keine Aufgaben"; exit 1 }

$ergebnis = [ordered]@{}
foreach ($v in $varianten.Keys) {
    $praefix = $varianten[$v]
    $ok = 0; $gesamt = 0
    for ($i = 0; $i -lt $N; $i++) {
        $pf = Join-Path $arbeit "p$i.txt"
        $af = Join-Path $arbeit "a$i.txt"
        if (-not (Test-Path $pf)) { continue }
        # Der Prompt besteht aus Marker-Bytes: <AUFGABE> Text <DENKWEG>.
        # Die Paraphrase muss den TEXT ersetzen, nicht vor den Marker treten - sonst steht sie
        # ausserhalb der Aufgabe und das Modell sieht sie gar nicht (erster Versuch: 8 Prozent
        # in der trainierten Form selbst).
        $bytes = [System.IO.File]::ReadAllBytes($pf)
        $gold = (Get-Content $af -Raw -ErrorAction SilentlyContinue)
        $neu = Join-Path $arbeit "pv.txt"
        $praefixBytes = [System.Text.Encoding]::UTF8.GetBytes($praefix)
        $out = New-Object System.Collections.Generic.List[byte]
        $out.Add($bytes[0])
        $out.AddRange($praefixBytes)
        if ($bytes.Length -gt 1) {
            $rest = New-Object byte[] ($bytes.Length - 1)
            [Array]::Copy($bytes, 1, $rest, 0, $bytes.Length - 1)
            $out.AddRange($rest)
        }
        [System.IO.File]::WriteAllBytes($neu, $out.ToArray())
        $env:SHIRO_PROMPT_FILE = $neu
        $aus = & $bin chat --config $Config --checkpoint $Checkpoint --max-new $MaxNew --backend cuda 2>&1
        $gen = ($aus | Where-Object { $_ -like "Generiert*" }) -replace '^Generiert\s*:\s*', ''
        $antwort = ""
        if ($gen -match '<ANTWORT>(.*?)<ENDE>') { $antwort = $matches[1].Trim() }
        elseif ($gen -match '<ANTWORT>(.*)') { $antwort = $matches[1].Trim() }
        $gesamt++
        if ((Wert $antwort) -ne $null -and (Wert $antwort) -eq (Wert $gold)) { $ok++ }
    }
    Remove-Item Env:SHIRO_PROMPT_FILE -ErrorAction SilentlyContinue
    $ergebnis[$v] = [pscustomobject]@{ Treffer = $ok; Gesamt = $gesamt }
}

Remove-Item $arbeit -Recurse -Force -ErrorAction SilentlyContinue
$zeile = "$Name n=$N  "
foreach ($v in $ergebnis.Keys) {
    $e = $ergebnis[$v]
    $p = if ($e.Gesamt -gt 0) { [int](100 * $e.Treffer / $e.Gesamt) } else { 0 }
    $zeile += ("{0}: {1}%  " -f $v, $p)
}
$zeile.Trim()
