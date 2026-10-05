# T010 渲染 spike 证据档（M0-09，验收⑤）

- 任务：独立 crate `render-spike`——胶囊体灰盒 1 万单位同屏渲染验证（bevy 0.19.1 全量特性）。
- 执行：worker-2 · 隔离树 `t010-a` · 基线 commit `5ab440b`（含 T009 收获）。
- 卡面验收⑤：10,000 胶囊体同屏 @60fps 且 **1% low ≥ 45**（基准机 A）——达标/未达标如实判定。

## 判定行（summarize.py 脚本生成，禁手改；原文见 `summary.md`）

**t10000: avg_fps = 333.15（≥60 ? 是）、1% low = 202.48（≥45 ? 是）→ PASS**

- 预注册预期（任务卡 D10）：PASS（avg ≫ 60、1% low ≥ 45 裕度充足）——实测与预期方向一致；
  未触发 FAIL 路径，故无「渲染腿后果记录行」。
- 结论：渲染灰盒口径下，1 万胶囊（静态、共享网格、无动画/LOD/分层）在基准机 A 上远超验收线
  （avg 5.55× 阈值、1% low 4.5× 阈值）。M2/M3 常态渲染不可外推（灰盒指标偏乐观，表 6-0 纪律）。

## 索引

| 文件 | 内容 |
| --- | --- |
| `summary.md` | 四档指标 + vsync 旁证 + 判定行（脚本 `summarize.py` 生成） |
| `summarize.py` | 汇总脚本（含 `--selftest` W4 跨语言对齐） |
| `api-notes.md` | bevy 0.19 API 逐条查证（docs.rs URL + 签名摘录 + 源码行号核对） |
| `rng_double_blind.py` | W1 python 独立重实现（SplitMix64） |
| `run_t010.sh` | 采集跑批（断点续跑；gate/smoke/tier/collect/summary/env 子命令） |
| `collect_environment.ps1` / `environment.txt` | R7 环境档（零机器绝对路径） |
| `runs/` | 一命令一档：`cmd.txt` + stdout/stderr + `REAL_EXIT` + time（+ 采集输出） |
| `runs/w1_rust_first8/`、`runs/w1_python_first8/` | W1 双盲对拍两侧原始输出 + `comparison_note.txt` |
| `runs/r0_lock_diff/` | Cargo.lock 变更 diff（before/after/diff 三件套） |
| `runs/scrub_note.txt` | 日志机器路径前缀清理记录（T003/T007 先例） |

## 场景与几何算式（确定性，不依赖 sim）

- 布阵：`gx = i % 100`、`gz = i / 100`；`x = (gx − 49.5) × 1.6 + jx`、`z = (gz − 49.5) × 1.6 + jz`；
  jitter ∈ (−0.25, +0.25) 严格开区间（高 23 位 +0.5 中点映射，逐样本断言）；y = 1.0 落地
  （胶囊 radius 0.5 + half_length 0.5，总高 2.0）。
- 不重叠算式：格距 1.6 − 2×0.25 = **1.1 m > 胶囊直径 1.0 m**（对角距 ≥ 1.556 m 更宽裕）。
- 场地：99 × 1.6 = **158.4 m ∈ ±79.2 m**（jitter 后极值 ±79.45 m）。
- 前缀性质：单位 i 位置只依赖 i 与 seed ⇒ 四档同 seed（42）为前缀布局。
- 相机固定 (0, 180, 180) `looking_at` 原点（fov 45° 默认）。
  **覆盖校验（角点精确算式，粗算式仅参考）**：前向轴 f=(0,−1,−1)/√2；角点 (79.45, y, 79.45)
  沿视轴深度 cz=(360−y−z)/√2 ≈ 197–310 m；垂直占用率 ≤ 68.4%（<1）、水平 ≤ 54.8%（<1）
  ⇒ 全场地 8 角点均在视锥内（y∈{0,2} 全枚举），全部单位同屏。
  粗算式：视距 √(180²+180²) ≈ 254.6 m、2×254.6×tan(22.5°) ≈ 210.9 m > 158.4 m 场地跨度。
- 共享单 Mesh（Capsule3d r=0.5, length=1.0）+ 共享灰色 StandardMaterial + DirectionalLight
  （默认无阴影）+ GlobalAmbientLight（0.19 语义，见 api-notes.md）；无动画/LOD/分层。

## 采集方法（帧时间环形缓冲，非挂钟平均）

- 窗口化 `PresentMode::AutoNoVsync`；Update 每帧读 `Res<Time>::delta()`（0.19 无 `delta_ns()`，
  以 `as_nanos()` 换算）入环形缓冲（容量 240,000 帧，满即 panic）。
- warmup 5 s 丢弃（吸收管线编译尖峰；跨线圈帧计入 warmup）；capture 65 s（≥60 s 卡面），
  以「已记录帧 delta_ns 之和」计量；满 → 写 `frames.csv`（首行 `idx,delta_ns`，逐帧原始数据）
  + `meta.json`，发 `AppExit::Success` 退出。
- 四档命令（默认参数，见各档 `cmd.txt`）：`--units {1000,2000,5000,10000}`、seed 42、1920x1080。
- CLI 错误路径旁证：`runs/r8_cli_errors/`（`--units 0`、`--res 0x0`、未知参数 → usage 落 stderr、
  exit 2，无窗口创建）。

## 门禁（四条，串行；-j 2 从严；详档见 runs/r0_*）

| 门禁 | 命令 | REAL_EXIT | 结果 |
| --- | --- | --- | --- |
| 1 | `cargo check --workspace -j 2` | 0 | 0 警告（含 bevy full 冷编） |
| 2 | `cargo test -p sim -j 2` | 0 | 47 全绿（44 lib + 3 arena bin）；test profile 2 条既有 `unused_mut` 警告（`sim/src/world.rs:1804/1868`，基线既有，红线上未改） |
| 3 | `cargo test -p render-spike -j 2` | 0 | 10 全绿（W1 黄金 + W2×4 + W3×3 + W4×1） |
| 4 | `cargo build -p render-spike --release -j 2` | 0 | release 冷编 19m45s |

- 门禁 3 首跑为 W1 预期失败（占位黄金值→实测提取），档案移至 `runs/w1_rust_first8/attempt_failing/`；
  对拍 8/8 一致后固化并转绿（本表 REAL_EXIT=0 为转绿记录）。
- 门禁 1 首次尝试另有两次失败记录，见「事件记录」。

## 四档结果（详表与判定见 `summary.md`；此处为摘要）

| 档位 | 帧数 N | avg_fps | 1% low | 0.1% low | p50 ms | p95 ms | p99 ms | max ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| t1000 | 34843 | 536.03 | 260.07 | 151.61 | 1.745 | 2.567 | 3.249 | 24.113 |
| t2000 | 33933 | 522.03 | 268.28 | 213.06 | 1.800 | 2.666 | 3.324 | 6.026 |
| t5000 | 32936 | 506.70 | 242.93 | 191.98 | 1.824 | 2.866 | 3.650 | 6.163 |
| **t10000** | **21656** | **333.15** | **202.48** | **153.40** | **2.935** | **3.727** | **4.395** | **7.743** |
| t100（冒烟，参考） | 1423 | 474.12 | 251.14 | 189.92 | 1.958 | 3.009 | 3.538 | 6.282 |

- vsync 旁证：四档「非 vsync 锁定旁证：有」（每帧 delta < 15.865 ms 计数 = N，全程无 16.7 ms 锁定形态）。
- t1000 单帧 max 24.1 ms 为孤立毛刺（1/34843 帧，未影响 1% low），t10000 max 7.743 ms 无毛刺。
- 原始帧数据：各档 `runs/r{2..5}_t*/out/t*/frames.csv`（行数 = 帧数 + 1 表头，已复核对账）。

## 环境与窗口口径披露（R7）

- 基准机 A：i5-12490F（6C/12T）/ 32 GiB / RTX 3050（Vulkan；驱动 537.58）/ Windows 10 IoT LTSC 19044。
- 工具链：rustc 1.98.1（钉版）；bevy 锁 0.19.1（sim minimal + render-spike full 两入口同锁）。
- 产物指纹：`target/release/render-spike.exe` sha256 `264A95931B74D81932DBC2B5B115BC08CF13FF74A0B97FF9AA1278FEC4605272`。
- **实际窗口尺寸披露**：`--res 1920x1080` 为请求值；实际物理窗口 **2400×1350**（各档 meta.json
  `window_resolution_actual` 一致）。成因：显示器 125% DPI 缩放，bevy_winit 无 scale override 时按
  **逻辑尺寸**请求、由 OS 套用缩放（源码 `bevy_winit-0.19.1/src/winit_windows.rs:114-120`）。
  即实测画布面积 ≈ 3.24 Mpix = 1080p 的 1.56×——**渲染负载更重，判定为保守方向**；
  判定口径（avg/1% low）与阈值不变，不改判。详见 api-notes.md。
- 量测窗口机器空闲：采集前后 `runs/r0_precheck/idle.log`（CPU 负载 11–21%、无 cargo/rustc 等
  相关进程、commit 余量 13.5–14.5 G）。

## API 查证与 0.19 差异（硬约束 2；详 `api-notes.md`）

查证方式：docs.rs/bevy/0.19.1 逐条 WebFetch（URL+签名摘录）+ cargo registry 同版本源码行号核对。
差异点（4）：① 无 `Time::delta_ns()` → `delta().as_nanos()`；② `AmbientLight` 已是相机组件、
全局资源为 `GlobalAmbientLight`；③ `AppExit` 是 Message（`MessageWriter::write`）；
④ 别名依赖 `bevy_full` 下 derive 宏路径需补 `use bevy_full::ecs as bevy_ecs;`（官方内置 workaround）。

## 事件记录（如实披露）

1. **门禁 1 首败双因**（`runs/r0_gate1_first_attempt/`）：
   (a) `run_t010.sh` 树根路径 off-by-one（上溯两级，应为三级）→ 档案误落 `docs/docs/`，已修复并删除误落目录；
   (b) 真实编译错误 E0433（derive 宏 × 别名依赖，见上方差异点④）→ 补别名后转绿。
2. **Cargo.lock 预期不符**（`runs/r0_lock_diff/`）：卡面预期「仅新增 render-spike 条目（bevy 全图已在锁内）」，
   实况：**+419 包、-0 包、既有版本零变更**（135 → 554 包；bevy 保持 0.19.1）。成因：sim 走 minimal
   （default-features=false），锁内本无 bevy 全图；diff 中 13 条 `-` 内容行（grep 计数 14 含 diff 头行）均为同名多版本消歧重写
   （如 `"bitflags"` → `"bitflags 2.13.2"`），非依赖删除。diff 三件套如实入档，供 T013/收获复核。
3. **W1 双盲**：Rust 首测（占位失败）与 python 独立重实现 8/8 逐位一致后固化（`runs/w1_*`）。
4. **R7 首跑失败**：ps1 无 BOM，Windows PowerShell 5.1 以 ANSI 读取中文乱码致解析错误；补 UTF-8 BOM
   （t007 同式）后转绿。
5. 任务卡 D2 与派工单 §3 布阵公式存在 0.8 m 平移差（卡面 `−80 + 1.6·gx`，派工单 `(gx − 49.5)×1.6`）；
   本卡按**派工单自含口径**执行（场地居中、跨度 158.4 m），供 T013 统一措辞时留意。

## 自检声明

- `sim/**` 与 `sim/Cargo.toml` 零 diff（对 5ab440b 全树 diff 验证：除允许交付物外 ZERO DRIFT）。
- 根 `Cargo.toml` 仅两处改动：`members` + `render-spike`；`[workspace.dependencies]` + `bevy_full` 行。
- 入库产物零机器绝对路径（全档扫描 0 命中；日志前缀已按先例清理并留痕 `runs/scrub_note.txt`）。
- 本卡未 commit / 未 push（树内无 .git）；收获与闸门轮归 Lead / plan-code-reviewer。
