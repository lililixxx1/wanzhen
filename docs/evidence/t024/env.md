# T024 环境档（M5-07 观战档性能验收与 C-1 关账）

- 生成：worker-2 · 隔离树 `t024-a` · 基线 commit `43cd5cd`（tree 导出，无 .git）
- 基准机 A = 开发机本机（报告 V0.9.1 表 6-0 口径；同 T010 `docs/evidence/t010/environment.txt` 基线）

## 机器事实

| 项 | 值 | 来源 |
| --- | --- | --- |
| CPU | 12th Gen Intel(R) Core(TM) i5-12490F，6 核 12 线程，3.0 GHz | `Win32_Processor` 现场查询；线程数旁证 = 本卡 capture 腿 stderr bevy SystemInfo `core_count: "6"`（T010 environment.txt 同值） |
| GPU | NVIDIA GeForce RTX 3050，驱动 31.0.15.3758 | `Win32_VideoController` 现场查询（T010 同值）；capture 腿 stderr AdapterInfo：Vulkan backend / driver 537.58 |
| RAM | 32 GiB | `Win32_PhysicalMemory` 求和；bevy SystemInfo `memory: "31.8 GiB"` |
| OS | Microsoft Windows 10 IoT 企业版 LTSC，10.0.19044，64 位 | `Win32_OperatingSystem` 现场查询 |

## 工具链（钉版）

```
rustc 1.98.1 (48a229cea 2026-09-01)
cargo 1.98.1 (797e8a9bc 2026-08-05)
rust-toolchain.toml: channel = "1.98.1"   # 树内钉版文件
依赖：bevy 0.19.1（Cargo.lock 入库；host 经别名 bevy_full + bevy_remote，render-spike 经 bevy_full 全量特性）
```

## 二进制指纹（量测执行件）

| 二进制 | 尺寸（bytes） | sha256 | 来源 |
| --- | --- | --- | --- |
| `target/release/host.exe` | 95,966,208 | `602b491f9467ed4fe6cc1ba1d6390504a4e87293abbe02349c1856039199f8dd` | 本卡 T024 插桩源（gate2）构建产物；同步拷贝入树 `target/release/`（运行件，非入库——`.gitignore` 已含 `/target`） |
| `target/release/render-spike.exe` | 85,301,760 | `ff4b990f6a102639b8562571b12c9c2d077ff7637a5b1c23b6f2c21fad6767c5` | 本卡 gate3 构建产物（源 = 树内 T010 版 render-spike，无 T024 改动）；同样拷贝入树 |

指纹原文：`runs/binary-fingerprints.txt`。

## 构建命令与实测耗时（窗口前完成；CARGO_TARGET_DIR = 共享 target）

| # | 命令 | 退出码 | 耗时 | 日志 |
| --- | --- | --- | --- | --- |
| 1 | `cargo check --workspace -j 3` | 0（0 警告） | Finished in 1.75s（增量；插桩后首次静态检查） | `runs/gate1-check.log` |
| 1b | `cargo check --workspace -j 3`（终源复验——注释行号往返修正后） | 0（0 警告） | Finished in 0.88s | `runs/gate1b-check-final-source.log` |
| 2 | `cargo build -p host --release -j 3` | 0 | Finished in 11.91s（增量：仅 sim + host 重编；依赖栈复用） | `runs/gate2-host-release.log` |
| 3 | commit 预检（render-spike 构建前置） | — | CommitFree_G = 11.7（< 12 G 门槛） | `runs/gate3-commit-precheck.txt` |
| 4 | `cargo build -p render-spike --release -j 2` | 0 | Finished in 8.86s（**仅 `Compiling render-spike`——无 bevy 栈冷编签名**，增量事实成立） | `runs/gate3-render-spike-release.log` |

- 构建 #3/#4 说明（派工单附加条款）：预检 11.7 G < 12 G 门槛，按「CommitFree 不足上报附增量事实」执行——共享 target 内既有 `render-spike.exe`（85,301,760 B，2026-10-06 10:01，sha256 `ce64db4e…40fbdd`）与 render-spike fingerprint 在位；本次构建实测**零依赖栈编译行**（无 `Compiling bevy_*`），8.86s 完成——冷编风险未兑现，负载足迹与该门槛的保守裕量无关（见上报项）。
- **构建可复现性注（post-window 复核实验，2026-10-07）**：本机 release 链接**非位级可复现**——受控对照：源字节完全相同（仅 mtime 相异）的两次 release 构建输出哈希不同（`33ddcf6a…` vs `a2826dcc…`；观察全序列 `602b491f…`（测量件）→ `10dbf89e…` → `33ddcf6a…` → `a2826dcc…` 见 `runs/build-reproducibility-note.txt` + `runs/gate2b-host-rebuild-commentfix.log`）。因此二进制指纹的核验对象 = **本档存档的测量用产物文件**（树 `target/release/host.exe` = `602b491f…`；判定数据全部由该文件产生，两腿日志/窗口留痕/冒烟实录同源）；**重建哈希不保证一致，不作核对基准**。测量后源改动 = `frame_capture.rs` 注释内一处行号引用修正（26-33↔26-34 一次往返，最终保留 26-33 = 与测量时源状态一致；该范围覆盖 ring.rs push 全部语句行，闭合括号在 34；无任何代码/语义面改动）。共享 target 同名文件经复核构建覆写后已用档存测量件回填，两址 sha256 复验一致（`602b491f…`）。
- 窗口内零 cargo/rustc（三段留痕 pre/post 断言；`window/`）。

## 量测窗口环境（三段留痕，D5）

- 跑前快照 / 声明：`window/pre-process.txt`（全量进程表 + rustc/cargo/host/render-spike = 0 + CommitFree + 端口 15702 空闲 + 无并发负载声明）。
- 量测中抽样（每 20 s：CPU% / CommitFree_G）：`window/load-samples.csv`。
- 结束复扫：`window/post-process.txt`。
- 时间线（窗口开/两腿起止/空档/窗口关）：`window/events.txt`。
- 窗口语义：量测机器空闲独占（Lead 承诺窗口内零负载）；窗口内禁一切 cargo / 冒烟 / 额外 host 实例——本卡两腿前已留构建空档（最后一笔构建完成后 > 2 分钟静置）。

## 窗口 / 呈现口径

- 请求窗口化 1920×1080（D3 范围预裁剪：多分辨率不做）；`PresentMode::AutoNoVsync`。
- **实际物理窗口 2400×1350**（125% DPI 缩放，bevy_winit 按逻辑尺寸请求、OS 套用缩放——T010 披露先例）；两腿 meta `window_resolution_actual` 一致。即实测画布面积 ≈ 3.24 Mpix = 1080p 的 1.56×——渲染负载更重，判定为保守方向。
- 窗口时间参数：warmup 5 s 丢弃 + capture 65 s 判定窗（两腿同口径）。
