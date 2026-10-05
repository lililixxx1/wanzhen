# T009 measurement-window load precheck (invoked by run_t009.sh before each batch;
# output -> runs/precheck-<batch>.txt). ASCII-only: Windows PowerShell 5.1 reads BOM-less
# .ps1 as ANSI and mojibake breaks parsing (first-run defect archived in runs/precheck-*.out).
# Interpretation: "processes" section empty => machine idle (no cargo/build/measure load).
$ErrorActionPreference = "Continue"
$outFile = $args[0]
$L = New-Object System.Collections.Generic.List[string]
$L.Add("T009 load precheck @ " + (Get-Date -Format "yyyy-MM-dd HH:mm:ss"))
$L.Add("policy: no other cargo/heavy-load process at batch start (empty processes section = idle).")
$L.Add("  (excluded: pythonw.exe = ZCode harness telemetry tps_stats_server.py, resident, ~0 CPU)")
$L.Add("")
$L.Add("== processes (cargo/rustc/sim/bench/arena/python match) ==")
$procs = Get-CimInstance Win32_Process | Where-Object { $_.Name -match '^(cargo|rustc|rustc_wrapper|sim|bench|arena|python).*\.exe$' -and $_.Name -ne 'pythonw.exe' }
if ($null -eq $procs) {
    $L.Add("  (no match -> idle)")
} else {
    foreach ($p in $procs) {
        $L.Add("  " + $p.Name + " pid=" + $p.ProcessId + " cmd=" + $p.CommandLine)
    }
}
$L.Add("")
$cpu = Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average
$L.Add("== cpu_load_percent == " + $cpu.Average)
$osInfo = Get-CimInstance Win32_OperatingSystem
$L.Add("== free_physical_mem_gb == " + [math]::Round($osInfo.FreePhysicalMemory / 1MB, 2))
[System.IO.File]::WriteAllLines($outFile, $L, (New-Object System.Text.UTF8Encoding($false)))
Write-Output ("precheck written: " + $outFile)
