# T007 验收⑥ 长跑 + 内存采样（M0-06）。由 run_matrix.sh 顺序调用（也可独立运行）。
#
# 流程（D7）：
#   1) 探针：bench 50k × ProbeTicks ticks @12t（warmup 0 / repeats 1），取 median
#      估算每 tick 墙钟 → K6 = ceil(TargetSeconds / 每tick秒)，使长跑壁钟 ≥600s；
#   2) 长跑：bench 50k × K6 ticks @12t 单局（warmup 0 / repeats 1）；
#   3) 外部采样器：每 SampleIntervalMs(5s) 读 Get-Process 的 WorkingSet64 +
#      PrivateMemorySize64 追加写 runs/longrun_memory.csv；
#   4) runs/longrun_meta.txt 记录探针/K6/壁钟/退出码；REAL_EXIT 追加到
#      runs/REAL_EXIT.txt。
#
# 进程控制说明：不用 Start-Process -PassThru（PS 5.1 实测 ExitCode 为空），
# 改 .NET System.Diagnostics.Process 直控（UseShellExecute=false + 重定向 +
# ReadToEndAsync），WaitForExit 后 ExitCode 可靠。
# 采样窗口覆盖进程全程（含启动与部署）；判定侧弃首 10% 样本即覆盖该口径。
# 全部相对路径（仓库可公开态，零机器绝对路径）。
param(
    [int]$TargetSeconds = 600,
    [int]$SampleIntervalMs = 5000,
    [int]$ProbeTicks = 10,
    [int]$Units = 50000,
    [int]$Threads = 12,
    [int]$Seed = 42
)
$ErrorActionPreference = "Stop"
$ScriptDir = $PSScriptRoot
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir "..\..\..")).Path
$Bin = Join-Path $RepoRoot "target\release\bench.exe"
$Runs = Join-Path $ScriptDir "runs"
New-Item -ItemType Directory -Force -Path $Runs | Out-Null
if (-not (Test-Path $Bin)) { throw "missing $Bin (build: cargo build -p sim --bin bench --release -j 3)" }
$enc = New-Object System.Text.UTF8Encoding($false)

function Start-BenchProcess {
    param([string[]]$BenchArgs)
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $Bin
    $psi.Arguments = ($BenchArgs -join " ")
    $psi.WorkingDirectory = $RepoRoot
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    [void]$proc.Start()
    # 先挂异步读（防 4KB 管道缓冲写满死锁）；本工具单行 JSON 输出量极小。
    $outTask = $proc.StandardOutput.ReadToEndAsync()
    $errTask = $proc.StandardError.ReadToEndAsync()
    return @{ Proc = $proc; OutTask = $outTask; ErrTask = $errTask }
}

# ---- 1) 探针 ----
$probeOut = Join-Path $Runs "longrun-probe.stdout.json"
$probeErr = Join-Path $Runs "longrun-probe.stderr.txt"
$probeArgs = @("--units", "$Units", "--threads", "$Threads", "--ticks", "$ProbeTicks", "--warmup", "0", "--repeats", "1", "--seed", "$Seed")
$probe = Start-BenchProcess -BenchArgs $probeArgs
$probe.Proc.WaitForExit()
$probeExit = $probe.Proc.ExitCode
[System.IO.File]::WriteAllText($probeOut, $probe.OutTask.Result, $enc)
[System.IO.File]::WriteAllText($probeErr, $probe.ErrTask.Result, $enc)
if ($probeExit -ne 0) { throw "probe REAL_EXIT=$probeExit (see runs/longrun-probe.stderr.txt)" }
$probeText = Get-Content $probeOut -Raw -Encoding UTF8
$m = [regex]::Match($probeText, '"median_ns":([0-9.]+)')
if (-not $m.Success) { throw "cannot parse probe median_ns from $probeOut" }
$probeMedianNs = [double]$m.Groups[1].Value
$perTickS = $probeMedianNs / $ProbeTicks / 1e9
if ($perTickS -le 0) { throw "probe per-tick seconds <= 0" }
$k6 = [int][math]::Ceiling($TargetSeconds / $perTickS)
if ($k6 -lt 1) { $k6 = 1 }
Write-Output ("probe: median_ns=" + $probeMedianNs + " per_tick_s=" + [math]::Round($perTickS, 4) + " K6=" + $k6 + " est_wall_s=" + [math]::Round($k6 * $perTickS, 1))

# ---- 2) 长跑 + 3) 采样 ----
$longOut = Join-Path $Runs "longrun.stdout.json"
$longErr = Join-Path $Runs "longrun.stderr.txt"
$csv = Join-Path $Runs "longrun_memory.csv"
$longArgs = @("--units", "$Units", "--threads", "$Threads", "--ticks", "$k6", "--warmup", "0", "--repeats", "1", "--seed", "$Seed")
$startTime = Get-Date
$long = Start-BenchProcess -BenchArgs $longArgs
$sw = [System.Diagnostics.Stopwatch]::StartNew()
Write-Output ("longrun started: pid=" + $long.Proc.Id + " ticks=" + $k6 + " args=" + ($longArgs -join " "))
[System.IO.File]::WriteAllText($csv, "i,elapsed_ms,working_set_bytes,private_memory_bytes`r`n", $enc)
$i = 0
while ($true) {
    $proc = Get-Process -Id $long.Proc.Id -ErrorAction SilentlyContinue
    if ($null -eq $proc) { break }
    $ws = $proc.WorkingSet64
    $pm = $proc.PrivateMemorySize64
    $line = "$i,$([int64]$sw.ElapsedMilliseconds),$ws,$pm`r`n"
    [System.IO.File]::AppendAllText($csv, $line, $enc)
    $i++
    Start-Sleep -Milliseconds $SampleIntervalMs
}
$long.Proc.WaitForExit()
$exitCode = $long.Proc.ExitCode
$wallS = $sw.Elapsed.TotalSeconds
$endTime = Get-Date
[System.IO.File]::WriteAllText($longOut, $long.OutTask.Result, $enc)
[System.IO.File]::WriteAllText($longErr, $long.ErrTask.Result, $enc)
Write-Output ("longrun finished: REAL_EXIT=" + $exitCode + " wall_s=" + [math]::Round($wallS, 1) + " samples=" + $i + " csv=" + ($csv -replace [regex]::Escape($RepoRoot + "\"), ""))

# ---- 4) 留档 ----
$meta = @()
$meta += "T007 验收⑥ 长跑元数据（50k 模拟单位 × 12 线程，单局）"
$meta += "探针（ticks=$ProbeTicks, warmup=0, repeats=1）median_ns: $probeMedianNs"
$meta += "探针估算每 tick 墙钟（s）: $perTickS"
$meta += "K6（校准使壁钟 ≥ ${TargetSeconds}s）: $k6"
$meta += "长跑命令: target/release/bench.exe " + ($longArgs -join " ")
$meta += "采样器: Get-Process WorkingSet64 + PrivateMemorySize64, 间隔 ${SampleIntervalMs}ms, 共 $i 条 → runs/longrun_memory.csv"
$meta += "开始: $($startTime.ToString('yyyy-MM-dd HH:mm:ss'))  结束: $($endTime.ToString('yyyy-MM-dd HH:mm:ss'))"
$meta += "壁钟（s）: $wallS"
$meta += "REAL_EXIT: $exitCode"
[System.IO.File]::WriteAllLines((Join-Path $Runs "longrun_meta.txt"), $meta, $enc)
[System.IO.File]::AppendAllText((Join-Path $Runs "REAL_EXIT.txt"),
    "longrun-${Units}-t${Threads} ticks=$k6 REAL_EXIT=$exitCode wall_s=$([math]::Round($wallS,1)) samples=$i csv=runs/longrun_memory.csv`r`n",
    [System.Text.Encoding]::ASCII)

if ($exitCode -ne 0) { exit 1 }
exit 0
