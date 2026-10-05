# T010 派工单 · WP-A（实现+门禁+采集+证据一体单）

[执行者] worker-2 · 隔离树 `C:\Users\Administrator\Desktop\ccc\trees\wanzhen\t010-a\`（下称 `<TREE>`，已由 Lead 建好，树内无 .git——git 命令天然不可用）

[目标] T010 渲染 spike（M0-09，验收⑤）：独立 crate `render-spike`——胶囊体灰盒 1 万单位同屏渲染验证，四档帧率数据 + 判定。

[基线 commit] 5ab440b（master，含 T009 收获）

[交付物]
1. `<TREE>\render-spike\Cargo.toml` + `<TREE>\render-spike\src\*.rs`（新 crate）
2. `<TREE>\Cargo.toml`（根 manifest，仅允许两处改动：members 增 `render-spike`；[workspace.dependencies] 增 `bevy_full = { package = "bevy", version = "0.19", default-features = true }`）
3. `<TREE>\Cargo.lock`（预期仅新增 render-spike 包条目——bevy 全图已在锁内；diff 如实入档）
4. `<TREE>\docs\evidence\t010\` 全档

[环境条款引用]（先 Read 再用；机器态文件不入库——入库产物禁止出现其绝对路径或内容转写）
`C:\Users\Administrator\Desktop\ccc\wanzhen\team-prompt\PROJECT-APPENDIX.md` 附录 A（串行/-j 3 上限/量测空闲独占/门禁清单/Bash 10min 上限）、B.1（误报清单）、B.2（数值纪律）、G（证据档规范）

**内存纪律（本单从严）**：bevy full 冷编属依赖重型足迹——涉 render-spike 的 cargo 一律 **`-j 2`**（附录 A 上限 3 的从严执行）；**每条 cargo 前**单独整句跑附录 A 的 commit 余量预检：`check/test` 需 CommitFree_G ≥ 10、`build --release` 需 ≥ 12；不足则 sleep 120 重检（≤5 次）后上报。长构建（>10 min 会触发 ZCode Bash 硬超时）一律 `run_in_background` 分步取证（附录 A「ZCode Bash 上限」条 + B.1）。

[门禁]（附录 A 清单；`<TREE>` 根执行、串行、期望退出码 0）
1. `cargo check --workspace -j 2`（0 警告——含 bevy full 冷编，长构建纪律见上）
2. `cargo test -p sim -j 2`（47 全绿——sim 侧零改动红线旁证）
3. `cargo test -p render-spike -j 2`（新增单测全绿）
4. `cargo build -p render-spike --release -j 2`

[实现规格]（设计裁决 D1~D12 已冻结于任务卡，本节自含）

## §1 crate 与依赖

- `render-spike/Cargo.toml`：`[package] name="render-spike"`，edition 与 sim 一致（Read `sim/Cargo.toml` 对照）；`[dependencies] bevy_full = { workspace = true }`。
- 代码内 `use bevy_full as bevy;`（保持惯用路径）。
- **禁依赖 sim**（卡面：不依赖 sim 战斗系统）；零其他第三方依赖。
- 根 Cargo.toml 改动仅上述两处；`sim/**` 与 `sim/Cargo.toml` 零 diff。

## §2 bevy 0.19 API 查证（硬约束 2，先查证后写码）

所用 API 逐条在 docs.rs/bevy/0.19 查证（WebFetch；失败 ≤2 次重试后上报降级），证据档 `docs api-notes.md` 留 URL + 关键签名摘录：
`Capsule3d`（构造签名——半径/半高参数序）、`Mesh::from(胶囊体)` 或等价、`Assets<Mesh>::add` / `Assets<StandardMaterial>::add`、`Mesh3d` / `MeshMaterial3d` 组件、`StandardMaterial`、`AmbientLight` / `DirectionalLight`、`Camera3d` + `Transform::looking_at`、`Window` / `WindowResolution` / `PresentMode::AutoNoVsync`、`DefaultPlugins` 的 `WindowPlugin` 定制、`App` / `add_systems` / `Update`、`EventWriter<AppExit>`（`AppExit::Success`）、`Res<Time>` 的 `delta_ns`、`Commands::spawn`。禁凭记忆写 API；0.19 与旧版差异点（组件包裹 Mesh3d 等）以 docs.rs 为准。

## §3 场景（确定性，不依赖 sim）

- 本地 SplitMix64（自实现，种子 `--seed` 默认 42；步进式：state = seed; 每取数 state += 0x9E3779B97F4A7C15 后混合——实现前与单测 W1 双盲对拍固化）。
- N 单位（`--units`，默认 10000）：`gx = i mod 100`、`gz = i div 100`；`x = (gx − 49.5)×1.6 + jx`、`z = (gz − 49.5)×1.6 + jz`（jx/jz ∈ (−0.25, +0.25) 各消耗一次 SplitMix64）；y = 落地（胶囊半高）。**前缀性质**：单位 i 位置只依赖 i 与 seed ⇒ 四档同 seed 为前缀布局（D6）。
- 几何算式（入档）：格距 1.6 m、jitter ±0.25 ⇒ 最小间距 ≥ 1.6−0.5 = 1.1 m > 胶囊直径 2×0.5 = 1.0 m（不重叠）；场地跨度 99×1.6 = 158.4 m ∈ ±79.2 m。
- 共享单 Mesh 资产（Capsule3d 半径 0.5 m）+ 共享 StandardMaterial（灰色）+ DirectionalLight + AmbientLight；相机固定 (0, 180, 180) `looking_at` 原点（视距 √(180²+180²) ≈ 254.6 m，fov 45° 垂直覆盖 ≈ 2×254.6×tan(22.5°) ≈ 210.9 m > 158.4 m 场地跨度 ⇒ 全部同屏——覆盖算式入档）；无动画/LOD/分层渲染。

## §4 窗口与帧采集

- 窗口化 1920×1080（`--res WxH` 可覆写）、PresentMode::AutoNoVsync（vsync 关）。
- Update 系统每帧读 `Res<Time>` 的 `delta_ns` 入环形缓冲（容量 240_000 帧，溢出 panic=上报）；**warmup `--warmup-sec`（默认 5）丢弃**（吸收管线编译首帧尖峰），之后 capture `--capture-sec`（默认 65）满 → 写 `AppExit::Success`；退出前写档。
- 每档目录 `<out>/t<units>/`：`frames.csv`（首行 `idx,delta_ns`，逐帧原始数据——验收断言 4）+ `meta.json`（units/seed/resolution/warmup_s/capture_s/present_mode/captured_frames）。
- CLI：`--units/--seed/--warmup-sec/--capture-sec/--res/--out`（`--out` 必选；解析错误 exit 2）；窗口/GPU 初始化失败 → 原始报错落档 + 上报（不追查）。

## §5 汇总脚本 `summarize.py`（python 3，入库；判定行脚本计算禁手算）

预注册公式（即本节，脚本实现）：
- `avg_fps = N ÷ (Σ delta_ns / 1e9)`
- `1% low = 1e9 ÷ mean(最慢 ceil(0.01·N) 帧的 delta_ns)`；`0.1% low` 同式 `ceil(0.001·N)`（下限 1 帧）
- 判定行（表 6-0 验收⑤）：`t10000: avg_fps ≥ 60 且 1% low ≥ 45 → PASS/FAIL`（如实）；四档表 1000/2000/5000/10000 全指标（avg/1%/0.1% low + p50/p95/p99/max 帧时间）
- vsync 旁证行：`delta_ns < 0.95×16.7ms` 的帧数 > 0 →「非 vsync 锁定旁证：有」，否则披露「疑似 vsync 锁定」（AutoNoVsync 不生效形态，如实披露不影响判定口径）
- FAIL 时渲染腿后果记录行（报告 6.1 原文语义）：分层渲染提前进 M1 关键路径 + 常态规模承诺减半——本卡只产数据与判定，报告级修订归 T013。

## §6 采集跑批（机器空闲独占；每档 runs/ 档：命令全文+stdout/stderr+REAL_EXIT）

- R0 门禁四条先行落档
- R1 冒烟：`render-spike --units 100 --warmup-sec 2 --capture-sec 3 --out <smoke>`（窗口创建/GPU 可用验证；失败 ≤2 次重试即上报）
- R2~R5 四档：`--units {1000,2000,5000,10000}`（默认参数：seed 42 / warmup 5 / capture 65 / 1920x1080）
- R6 `summarize.py` → summary.md
- R7 环境档：CPU/GPU+驱动（nvidia-smi 失败则 wmic path win32_VideoController）/RAM/OS/rustc/bevy 锁版本/render-spike.exe sha256；复刻 t007 collect_environment.ps1 同式，输出零机器绝对路径
- 跑批脚本 `run_t010.sh`（断点续跑、全相对路径）

## §7 单测（`render-spike` 内 `#[cfg(test)]`；窗口/渲染路径不进单测——冒烟承载）

- **W1** SplitMix64 黄金序列：本地实现首 8 输出 vs **python 独立重实现**（双盲，PIT-M-002 纪律：先实测产出再固化期望值，python 脚本入档 `docs/evidence/t010/`）逐位对拍。
- **W2** 网格映射算式：i=0 → (−79.2+jitter 范围)、i=99 → x ∈ ±79.2 边界、i=100 → gz=1 行首；jitter ∈ (−0.25, 0.25) 界内断言（构造成对样本数 ≥1000）。
- **W3** 环形缓冲：容量语义 + 溢出 panic 用例（`#[should_panic]`）。
- **W4** 百分位公式：构造样例帧序列（如 [8,8,8,16,32,64]ms 六帧）手算期望 avg/1%low/0.1%low（算式入测试注释——参考值：k1=ceil(0.06)=1、k01=1 ⇒ 1%low=1e9/64_000_000=15.625）断言脚本同式函数输出。
- sim 侧既有 47 单测零改动（门禁 2 旁证）。

## [数值纪律与双盲]（附录 B.2⑤）

本单速算参考值（§3 几何、§4 视距/覆盖、§7 W4 期望）仅供核对，以代码/脚本/实测为准；不一致即停止上报。docs.rs 查证摘录即 API 层「行号核对」。

## [写范围] / [禁改]

写范围 = 交付物四处。禁改 = 其余一切（`sim/**`、`sim/Cargo.toml`、`.gitignore`、`rust-toolchain.toml`、`taskset/**`、`AGENTS.md`、`task-ledger.md`、`docs/` 既有内容——只读；`docs/evidence/t010/` 新建除外）。

## [上报条件]（任一即停并上报，附命令+原始输出）

冒烟窗口创建失败（≤2 次重试后）；docs.rs 查证不可达（≤2 次重试后降级上报待裁）；commit 预检连续 5 次不足；cargo 假失败嫌疑（B.1 形态——自查并发后上报）；任一门禁失败；判定 FAIL（如实落档 + 上报——FAIL 是合法结果不是错误，上报供 Lead 记录渲染腿后果）；同一异常 ≤2 次尝试；配额/中断直接上报。

## [时间盒 · 预期登记]

时间盒 ≤3h（含 bevy full 冷编 25~40 min 参考——-j 2 从严口径），超盒上报。预注册预期（D10）：基准机 A（i5-12490F/RTX 3050）+ 10k 静态实例化共享网格 → **预期 PASS（avg ≫60、1% low ≥45 裕度充足）**；若 FAIL 如实判定 + 渲染腿后果记录行。

## [汇报]

按 agent 定义四要素：完成项 / 上报项 / 证据（改动文件、命令与退出码、结果、未决点）/ 偏离——含「sim 侧零 diff」「根 Cargo.toml 仅两处改动」自查声明与 API 查证清单。
