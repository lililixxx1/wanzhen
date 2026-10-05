# T009 环境档采集（平衡实验场 v0 与吞吐量测；基准机 A = 开发机本机口径，报告表 6-0）。
# 复刻 docs/evidence/t007/collect_environment.ps1 同式；产物指纹换 arena.exe + sim.exe。
# 读取自证 CPU/GPU+驱动/RAM/OS/rustc/Cargo.lock bevy 版本/构建 profile/产物 sha256。
# 零编译、零下载。输出：本目录 environment.txt（UTF-8 无 BOM）。零机器绝对路径。
$ErrorActionPreference = "Stop"
$ScriptDir = $PSScriptRoot
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir "..\..\..")).Path
$out = Join-Path $ScriptDir "environment.txt"
$L = New-Object System.Collections.Generic.List[string]

$L.Add("T009 环境档（平衡实验场 v0 与吞吐量测；基准机 A = 开发机本机，报告 V0.9.1 表 6-0）")
$L.Add("生成时间: " + (Get-Date -Format "yyyy-MM-dd HH:mm:ss zzz"))
$L.Add("")

$L.Add("== CPU (Win32_Processor) ==")
Get-CimInstance Win32_Processor | ForEach-Object {
    $L.Add("  Name: " + $_.Name)
    $L.Add("  Cores: " + $_.NumberOfCores + "  LogicalProcessors: " + $_.NumberOfLogicalProcessors + "  MaxClockMHz: " + $_.MaxClockSpeed)
}
$L.Add("")

$L.Add("== GPU + 驱动 (Win32_VideoController) ==")
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
$L.Add("  注: sim/arena 代码路径零 bevy API 参与（headless 模拟核心）；bevy 版本仅作仓库基线记录。")
$L.Add("")

$L.Add("== 构建 profile（读取自证） ==")
$cargoToml = Get-Content (Join-Path $RepoRoot "Cargo.toml") -Encoding UTF8
$inProfile = $false
$profileLines = @()
foreach ($ln in $cargoToml) {
    $t = $ln.Trim()
    if ($t -eq "[profile.release]") { $inProfile = $true; continue }
    if ($inProfile) {
        if ($t.StartsWith("[")) { break }
        if ($t -ne "" -and -not $t.StartsWith("#")) { $profileLines += $t }
    }
}
$profText = if ($profileLines.Count -eq 0) { "（无声明行）" } else { $profileLines -join "; " }
$L.Add("  Cargo.toml [profile.release] 声明行: " + $profText)
$L.Add("  → opt-level 未声明 = 默认 3；lto 未声明 = 默认 off；debug = false（与 release 默认一致）。")
$cfgPath = Join-Path $RepoRoot ".cargo\config.toml"
if (Test-Path $cfgPath) {
    $L.Add("  .cargo/config.toml: 存在（内容未纳入本档）")
} else {
    $L.Add("  .cargo/config.toml: 不存在")
}
$L.Add("")

$L.Add("== 量测产物（arena.exe / sim.exe） ==")
foreach ($rel in @("target\release\arena.exe", "target\release\sim.exe")) {
    $bin = Join-Path $RepoRoot $rel
    if (Test-Path $bin) {
        $fi = Get-Item $bin
        $hash = (Get-FileHash $bin -Algorithm SHA256).Hash
        $L.Add("  path: $rel  size: " + $fi.Length + " bytes  mtime: " + $fi.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss"))
        $L.Add("  sha256: " + $hash)
    } else {
        $L.Add("  $rel 不存在（应先 cargo build -p sim --release -j 3）")
    }
}

[System.IO.File]::WriteAllLines($out, $L, (New-Object System.Text.UTF8Encoding($false)))
# 显示用相对路径（仓库卫生：零机器绝对路径；文件操作仍用绝对路径）。
Write-Output ("environment.txt written: " + ($out -replace [regex]::Escape($RepoRoot + "\"), ""))
