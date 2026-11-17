# Paraphrase fuer die Sprachfamilie: die FRAGE umformulieren, nicht nur die Rahmung.
#
# Die Aufgabe ist "Satz: den Hund sucht das Kind. Wer sucht?" - die Frage nennt das Verb.
# Eine echte Umformulierung fragt nach der handelnden Person, ohne das Verb zu wiederholen.
# Wenn das Modell die Kasusregel gelernt hat, muss die Antwort dieselbe bleiben.
param(
    [Parameter(Mandatory = $true)][string]$Checkpoint,
    [Parameter(Mandatory = $true)][string]$Shard,
    [int]$N = 20,
    [int]$MaxNew = 200,
    [string]$Config = "O:\shiro\pmh\eval25m.toml"
)

$ErrorActionPreference = "Stop"
$bin = "O:\shiro\target-moe\release\shiro.exe"
$mk = "O:\shiro\pmh\mktarget\release\mktool.exe"
$arbeit = Join-Path $env:TEMP ("spr-para-" + [guid]::NewGuid().ToString("N").Substring(0, 8))

# Vier Fassungen der Frage. Die erste ist die trainierte.
$varianten = [ordered]@{
    "trainiert"     = $null
    "Wer handelt"   = "Wer handelt?"
    "Taeter"        = "Wer ist der Taeter?"
    "Bitte nennen"  = "Bitte nenne die handelnde Person."
}

& $mk --wegextract $Shard $arbeit $N | Out-Null
if (-not (Test-Path (Join-Path $arbeit "p0.txt"))) { "keine Aufgaben"; exit 1 }

$ergebnis = [ordered]@{}
foreach ($v in $varianten.Keys) {
    $ersatz = $varianten[$v]
    $ok = 0; $gesamt = 0
    for ($i = 0; $i -lt $N; $i++) {
        $pf = Join-Path $arbeit "p$i.txt"
        $af = Join-Path $arbeit "a$i.txt"
        if (-not (Test-Path $pf)) { continue }
        $bytes = [System.IO.File]::ReadAllBytes($pf)
        # Die Marker sind Einzelbytes (0xFE Aufgabe, 0xFD Denkweg) und keine gueltige
        # UTF-8-Sequenz. Wer den ganzen Prompt als Text dekodiert und neu schreibt, zerstoert
        # sie - erster Versuch: 0 Prozent in der trainierten Form.
        # Deshalb: nur den Text ZWISCHEN den Markern anfassen, die Bytes drumherum unveraendert.
        $idx = [Array]::IndexOf($bytes, [byte]0xFD)
        if ($idx -lt 2) { continue }
        $kopf = $bytes[0]
        $mitte = New-Object byte[] ($idx - 1)
        [Array]::Copy($bytes, 1, $mitte, 0, $idx - 1)
        $text = [System.Text.Encoding]::UTF8.GetString($mitte)
        if ($null -ne $ersatz) {
            $text = [regex]::Replace($text, 'Wer\s+\w+\?', $ersatz)
        }
        $rest = New-Object byte[] ($bytes.Length - $idx)
        [Array]::Copy($bytes, $idx, $rest, 0, $bytes.Length - $idx)
        $out = New-Object System.Collections.Generic.List[byte]
        $out.Add($kopf)
        $out.AddRange([System.Text.Encoding]::UTF8.GetBytes($text))
        $out.AddRange($rest)
        $neu = Join-Path $arbeit "pv.txt"
        [System.IO.File]::WriteAllBytes($neu, $out.ToArray())
        $gold = ((Get-Content $af -Raw -ErrorAction SilentlyContinue) -replace '\s+', '').Trim()
        $env:SHIRO_PROMPT_FILE = $neu
        $aus = & $bin chat --config $Config --checkpoint $Checkpoint --max-new $MaxNew --backend cuda 2>&1
        $gen = ($aus | Where-Object { $_ -like "Generiert*" }) -replace '^Generiert\s*:\s*', ''
        $antwort = ""
        if ($gen -match '<ANTWORT>(.*?)<ENDE>') { $antwort = $matches[1] }
        elseif ($gen -match '<ANTWORT>(.*)') { $antwort = $matches[1] }
        $antwort = ($antwort -replace '\s+', '').Trim()
        $gesamt++
        if ($antwort -eq $gold) { $ok++ }
    }
    Remove-Item Env:SHIRO_PROMPT_FILE -ErrorAction SilentlyContinue
    $p = if ($gesamt -gt 0) { [int](100 * $ok / $gesamt) } else { 0 }
    $ergebnis[$v] = $p
}

Remove-Item $arbeit -Recurse -Force -ErrorAction SilentlyContinue
$zeile = "Sprachlogik n=$N  "
foreach ($v in $ergebnis.Keys) { $zeile += ("{0}: {1}%  " -f $v, $ergebnis[$v]) }
$zeile.Trim()
