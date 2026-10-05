# T010 环境档采集（M0-09 渲染 spike；基准机 A = 开发机本机，报告 V0.9.1 表 6-0）。
# 由 run_t010.sh env 调用；读取自证 CPU/GPU+驱动/RAM/OS/rustc/bevy 锁版本/
# render-spike.exe 产物指纹。零编译、零下载。输出零机器绝对路径（仅相对路径）。
# 输出：本目录 environment.txt（UTF-8 无 BOM）。
$ErrorActionPreference = "Stop"
$ScriptDir = $PSScriptRoot
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir "..\..\..")).Path
$out = Join-Path $ScriptDir "environment.txt"
$L = New-Object System.Collections.Generic.List[string]

$L.Add("T010 环境档（M0-09 渲染 spike；基准机 A = 开发机本机，报告 V0.9.1 表 6-0）")
$L.Add("生成时间: " + (Get-Date -Format "yyyy-MM-dd HH:mm:ss zzz"))
$L.Add("")

$L.Add("== CPU (Win32_Processor) ==")
Get-CimInstance Win32_Processor | ForEach-Object {
    $L.Add("  Name: " + $_.Name)
    $L.Add("  Cores: " + $_.NumberOfCores + "  LogicalProcessors: " + $_.NumberOfLogicalProcessors + "  MaxClockMHz: " + $_.MaxClockSpeed)
}
$L.Add("")

$L.Add("== GPU + 驱动 ==")
$nv = $null
try { $nv = (& nvidia-smi --query-gpu=name,driver_version --format=csv,noheader 2>$null) } catch { $nv = $null }
if ($nv) {
    foreach ($line in $nv) { $L.Add("  nvidia-smi: " + $line) }
} else {
    $L.Add("  nvidia-smi 不可用（缺失或调用失败）→ 回退 Win32_VideoController：")
}
Get-CimInstance Win32_VideoController | ForEach-Object {
    $L.Add("  Name: " + $_.Name)
    $L.Add("  DriverVersion: " + $_.DriverVersion + "  DriverDate: " + $_.DriverDate)
}
$L.Add("")

$L.Add("== RAM (Win32_ComputerSystem / Win32_PhysicalMemory) ==")
$cs = Get-CimInstance Win32_ComputerSystem
$L.Add("  TotalPhysicalMemory: " + $cs.TotalPhysicalMemory + " bytes")
Get-CimInstance Win32_PhysicalMemory | ForEach-Object {
    $L.Add("  Module: " + [math]::Round($_.Capacity / 1GB, 1) + " GiB  SpeedMHz: " + $_.Speed + "  Manufacturer: " + $_.Manufacturer)
}
$L.Add("")

$L.Add("== OS (Win32_OperatingSystem) ==")
$os = Get-CimInstance Win32_OperatingSystem
$L.Add("  Caption: " + $os.Caption)
$L.Add("  Version: " + $os.Version + "  BuildNumber: " + $os.BuildNumber + "  Arch: " + $os.OSArchitecture)
$L.Add("")

$L.Add("== 工具链 ==")
try { $L.Add("  rustc: " + (& rustc -V)) } catch { $L.Add("  rustc: ERROR " + $_) }
try { $L.Add("  cargo: " + (& cargo -V)) } catch { $L.Add("  cargo: ERROR " + $_) }
$tc = (Get-Content (Join-Path $RepoRoot "rust-toolchain.toml") -Encoding UTF8 | Where-Object { $_ -match "channel" }) -join " "
$L.Add("  rust-toolchain.toml: " + $tc.Trim())
$L.Add("")

$L.Add("== 依赖版本（Cargo.lock） ==")
$lock = Get-Content (Join-Path $RepoRoot "Cargo.lock") -Encoding UTF8
for ($i = 0; $i -lt $lock.Count; $i++) {
    if ($lock[$i].Trim() -eq 'name = "bevy"') {
        $L.Add("  " + $lock[$i].Trim() + "  " + $lock[$i + 1].Trim())
        break
    }
}
$L.Add("  注: sim 经 'bevy'(default-features=false) 最小面、render-spike 经别名 bevy_full(默认特性全量) 同锁 0.19.1。")
$L.Add("")

$L.Add("== 窗口 / 呈现口径 ==")
$L.Add("  请求：窗口化 1920x1080（--res 默认）、PresentMode::AutoNoVsync、warmup 5s / capture 65s。")
$L.Add("  实际物理窗口尺寸与 present_mode 见各档 out/t*/meta.json（window_resolution_actual / present_mode）。")
$L.Add("")

$bin = Join-Path $RepoRoot "target\release\render-spike.exe"
$L.Add("== 量测产物（render-spike.exe） ==")
if (Test-Path $bin) {
    $fi = Get-Item $bin
    $hash = (Get-FileHash $bin -Algorithm SHA256).Hash
    $L.Add("  path: target/release/render-spike.exe  size: " + $fi.Length + " bytes  mtime: " + $fi.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss"))
    $L.Add("  sha256: " + $hash)
} else {
    $L.Add("  target/release/render-spike.exe 不存在（应先 cargo build -p render-spike --release -j 2）")
}

[System.IO.File]::WriteAllLines($out, $L, (New-Object System.Text.UTF8Encoding($false)))
# 显示用相对路径（仓库卫生：零机器绝对路径；文件操作仍用绝对路径）。
Write-Output ("environment.txt written: " + ($out -replace [regex]::Escape($RepoRoot + "\"), ""))
