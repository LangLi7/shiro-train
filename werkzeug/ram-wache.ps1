# RAM-Wache: warnt bei ueber 90 Prozent und protokolliert den Verlauf.
# Aufruf:  ram-wache.ps1 [-Grenze 90] [-IntervallSekunden 30] [-StoppBeiGrenze]
param(
    [double]$Grenze = 90,
    [int]$IntervallSekunden = 30,
    [switch]$StoppBeiGrenze,
    [string]$Protokoll = "O:\shiro\pmh\ram-verlauf.tsv"
)
while ($true) {
    $os = Get-CimInstance Win32_OperatingSystem
    $total = $os.TotalVisibleMemorySize / 1MB
    $frei = $os.FreePhysicalMemory / 1MB
    $prozent = [Math]::Round((($total - $frei) / $total) * 100, 1)
    $zeit = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$zeit`t$prozent`t$([Math]::Round($total - $frei,1))`t$([Math]::Round($frei,1))" | Add-Content -Path $Protokoll -Encoding utf8
    if ($prozent -gt $Grenze) {
        Write-Output "WARNUNG $zeit : $prozent Prozent (Grenze $Grenze)"
        if ($StoppBeiGrenze) {
            $shiro = Get-Process | Where-Object { $_.ProcessName -eq "shiro" }
            if ($shiro) {
                $shiro | Stop-Process -Force
                Write-Output "Trainer gestoppt, um den Speicher zu schuetzen"
            }
        }
    }
    Start-Sleep -Seconds $IntervallSekunden
}
