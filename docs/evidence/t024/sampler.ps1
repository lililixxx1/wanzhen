# T024 measurement-window load sampler (D5: sample CPU / CommitFree every 20 s).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File sampler.ps1 -Out <csv> [-IntervalSec 20]
# Started by measure_t024.sh at window start; killed at window end. CSV columns:
#   timestamp,elapsed_s,cpu_pct,commit_free_g
# - cpu_pct       = Win32_PerfFormattedData_PerfOS_Processor(_Total).PercentProcessorTime (0..100)
# - commit_free_g = (TotalVirtualMemorySize - FreeVirtualMemory)/1MB (same formula as appendix A precheck)
# NOTE: ASCII-only source on purpose -- PowerShell 5.1 reads BOM-less files as ANSI and
# mis-parses non-ASCII comments (attempt-1 parser failure, 2026-10-07); keep this file ASCII.
# NOTE: cmdlet parameter surfaces pinned against Windows PowerShell 5.1.19041 (verified
# 2026-10-07): Out-File has -FilePath (no -Path), Add-Content has -Path (no -FilePath).
# Keep this exact combination (attempt-1b/1c parameter-binding failures).
param(
  [Parameter(Mandatory = $true)][string]$Out,
  [int]$IntervalSec = 20
)
"timestamp,elapsed_s,cpu_pct,commit_free_g" | Out-File -FilePath $Out -Encoding ascii
$t0 = Get-Date
while ($true) {
  $cpu = $null
  try {
    $cpu = (Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'" -ErrorAction Stop).PercentProcessorTime
  } catch {
    $cpu = "ERR"
  }
  try {
    $os = Get-CimInstance Win32_OperatingSystem -ErrorAction Stop
    $commitFree = [math]::Round(($os.TotalVirtualMemorySize - $os.FreeVirtualMemory) / 1MB, 1)
  } catch {
    $commitFree = "ERR"
  }
  $elapsed = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
  "$(Get-Date -Format 'HH:mm:ss'),$elapsed,$cpu,$commitFree" | Add-Content -Path $Out
  Start-Sleep -Seconds $IntervalSec
}
