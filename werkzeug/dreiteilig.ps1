# Prueft drei Komponenten je Aufgabe:
#   1. Weg    — ist der erzeugte Loesungsweg gleich dem Goldweg
#   2. Antwort — stimmt der WERT der Antwort (nicht die Schreibweise)
#   3. Klasse  — ist die Antwort von der richtigen Art (Logik: ja/nein/unbekannt,
#               Mathematik: Zahl, Bruch, Dezimal oder "q Rest r")
#
# Warum Wertgleichheit: 9/10 und 18/20 sind dieselbe Zahl, 6/5 und 1,2 auch.
# Eine Zeichengleichheit wuerde korrekte Antworten als falsch zaehlen.
param(
    [Parameter(Mandatory = $true)][string]$Checkpoint,
    [Parameter(Mandatory = $true)][string]$Shard,
    [string]$Name = "familie",
    [ValidateSet("logik", "zahl", "bruch", "rest", "text")][string]$Klasse = "zahl",
    [int]$N = 20,
    [int]$MaxNew = 256,
    [string]$Config = "O:\shiro\pmh\eval25m.toml"
)

$cuda = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.9"
$env:CUDA_PATH = $cuda
$env:PATH = "$cuda\bin;$env:PATH"
$bin = "O:\shiro\target-moe\release\shiro.exe"
$cfg = $Config
$mk = "O:\shiro\pmh\mktarget\release\mktool.exe"
$arbeit = Join-Path $env:TEMP ("drei-" + $Name)

function Ggt([long]$a, [long]$b) {
    while ($b -ne 0) { $r = $a % $b; $a = $b; $b = $r }
    if ($a -lt 0) { return -$a }
    return $a
}

# Bringt eine Antwort auf eine vergleichbare Form: "a/b" gekuerzt, ganze Zahl als n/1,
# Dezimal als Bruch, "q Rest r" als Paar, Logikwoerter klein.
function Wert([string]$s) {
    if ($null -eq $s) { return "" }
    $t = ($s -replace '\s+', '').Trim()
    if ($t -eq '') { return "" }
    if ($t -match '^(ja|nein|unbekannt)$') { return $t.ToLower() }
    if ($t -match '^(-?\d+)/(-?\d+)$') {
        $a = [long]$matches[1]; $b = [long]$matches[2]
        if ($b -eq 0) { return $t }
        $g = Ggt ([Math]::Abs($a)) ([Math]::Abs($b))
        if ($g -eq 0) { $g = 1 }
        $a = $a / $g; $b = $b / $g
        if ($b -lt 0) { $a = -$a; $b = -$b }
        return "$a/$b"
    }
    if ($t -match '^(-?\d+)[.,](\d+)$') {
        $ganz = [long]$matches[1]
        $bruch = $matches[2]
        $nenner = [long][Math]::Pow(10, $bruch.Length)
        $zaehler = [long]$bruch
        if ($ganz -lt 0) { $zaehler = -$zaehler }
        $zaehler = $zaehler + ($ganz * $nenner)
        $g = Ggt ([Math]::Abs($zaehler)) $nenner
        if ($g -eq 0) { $g = 1 }
        return ("" + ($zaehler / $g) + "/" + ($nenner / $g))
    }
    if ($t -match '^(-?\d+)Rest(-?\d+)$') { return ("q" + $matches[1] + ",r" + $matches[2]) }
    if ($t -match '^(-?\d+)$') { return ("" + [long]$t + "/1") }
    return $t.ToLower()
}

function KlasseOk([string]$antwort) {
    $t = ($antwort -replace '\s+', '').Trim()
    switch ($Klasse) {
        "logik" { return ($t -match '^(ja|nein|unbekannt)$') }
        "bruch" { return ($t -match '^-?\d+/-?\d+$' -or $t -match '^-?\d+$' -or $t -match '^-?\d+[.,]\d+$') }
        "rest" { return ($t -match '^-?\d+Rest-?\d+$') }
        # Language family: the answer is a single noun (Hund, Katze, Lamm...), no digits.
        "text" { return ($t -match '^[A-Za-zÄÖÜäöüß]{2,20}$') }
        default { return ($t -match '^-?\d+$' -or $t -match '^-?\d+/-?\d+$' -or $t -match '^-?\d+[.,]\d+$') }
    }
}

Remove-Item $arbeit -Recurse -Force -ErrorAction SilentlyContinue
& $mk --wegextract $Shard $arbeit $N | Out-Null
if (-not (Test-Path (Join-Path $arbeit "p0.txt"))) { "$Name : keine Aufgaben"; exit 1 }

$wegOk = 0; $antOk = 0; $klasseOk = 0; $gesamt = 0
for ($i = 0; $i -lt $N; $i++) {
    $pf = Join-Path $arbeit "p$i.txt"
    if (-not (Test-Path $pf)) { continue }
    $env:SHIRO_PROMPT_FILE = $pf
    $aus = & $bin chat --config $cfg --checkpoint $Checkpoint --max-new $MaxNew --backend cuda 2>&1
    Remove-Item Env:SHIRO_PROMPT_FILE -ErrorAction SilentlyContinue
    $gen = ($aus | Where-Object { $_ -like "Generiert*" } | Select-Object -First 1) -replace '^Generiert\s*:\s*', ''
    if (-not $gen) { continue }
    # Marker-based, order-independent: the answer may come before or after the trace.
    # Each part is read up to the NEXT marker, whichever it is.
    $pw = $gen.IndexOf("<ANTWORT>")
    $pt = $gen.IndexOf("<DENKWEG>")
    $pe = $gen.IndexOf("<ENDE>")
    if ($pe -lt 0) { $pe = $gen.Length }
    $weg = ""; $ant = ""
    if ($pt -ge 0) {
        $ende = $pe
        if ($pw -gt $pt -and $pw -lt $ende) { $ende = $pw }
        $weg = (($gen.Substring($pt + 9, [Math]::Max(0, $ende - $pt - 9))) -replace '\s+', ' ').Trim()
    }
    if ($pw -ge 0) {
        $ende = $pe
        if ($pt -gt $pw -and $pt -lt $ende) { $ende = $pt }
        $ant = (($gen.Substring($pw + 9, [Math]::Max(0, $ende - $pw - 9))) -replace '\s+', ' ').Trim()
    }

    $goldWeg = ((Get-Content (Join-Path $arbeit "w$i.txt") -Raw) -replace '\s+', ' ').Trim()
    $goldAnt = ((Get-Content (Join-Path $arbeit "a$i.txt") -Raw) -replace '\s+', ' ').Trim()

    $gesamt++
    if ($weg -eq $goldWeg) { $wegOk++ }
    # Language family: plain word comparison, case-insensitive. Numeric value equality would
    # turn every noun into 0 and count wrong answers as right.
    if ($Klasse -eq "text") {
        if ((($ant -replace '\s+','').ToLower()) -eq (($goldAnt -replace '\s+','').ToLower())) { $antOk++ }
    } elseif ((Wert $ant) -eq (Wert $goldAnt)) { $antOk++ }
    if (KlasseOk $ant) { $klasseOk++ }
}
if ($gesamt -eq 0) { "$Name : keine Ausgabe"; exit 1 }
$f = { param($v) [Math]::Round(100.0 * $v / $gesamt, 1) }
"{0,-16} n={1,2}  Weg {2,5} %  Antwort {3,5} %  Klasse {4,5} %" -f $Name, $gesamt, (& $f $wegOk), (& $f $antOk), (& $f $klasseOk)


