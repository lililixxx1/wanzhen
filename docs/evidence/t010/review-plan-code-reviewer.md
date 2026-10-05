# T010 完整轮审核报告（plan-code-reviewer · 量测卡口径）

- 审核对象：隔离树 `t010-a`（基线 5ab440b）全部改动；主仓基线参照。
- 审核时间：2026-10-06（W1 D1 夜~D2 晨）；机器空闲窗口（CommitFree_G=14.8 预检、无 cargo/rustc 在飞）。
- 审核模式：完整轮（量测卡）——全量证据核对 + 关键数值与判定独立复算 + ≥1 项独立复跑留痕。
- 规则加载：review-protocol default.md（Rust 无专属规则文件，走通用兜底）。

## [结论] 通过（✅ 可合并）

四条门禁审核侧独立复跑全绿（exit 0/0/0/0）；A1~A8 逐条核对全部成立；四档全部指标以审核者独立实现重算逐位一致；W1 黄金 8 值经第三实现逐位复现；端到端短采集复跑 + 复构建 exe sha256 与环境档登记指纹逐位一致。零阻塞项。六项已知披露逐条核验为如实。收获可按 pathspec（`render-spike/`、`Cargo.toml`、`Cargo.lock`、`docs/evidence/t010/`）执行。

## [已核对]（逐条 + 退出码）

### 门禁四条（审核者在树根独立复跑，串行、-j 2；预检 CommitFree_G=14.8 ≥ 12、无并发 cargo）

| # | 命令 | 退出码（审核复跑） | 声明值 | 结果 |
| --- | --- | --- | --- | --- |
| 1 | `cargo check --workspace -j 2` | 0 | 0 | 0 警告（全量输出 grep "warning" 0 命中） |
| 2 | `cargo test -p sim -j 2` | 0 | 0 | 44 lib + 3 arena = 47/47 全绿 |
| 3 | `cargo test -p render-spike -j 2` | 0 | 0 | 10/10 全绿（W1×1 + W2×4 + W3×3 + W4×1，逐 test 名单核对） |
| 4 | `cargo build -p render-spike --release -j 2` | 0 | 0 | 复构建产物 sha256 = `264A95931B74D81932DBC2B5B115BC08CF13FF74A0B97FF9AA1278FEC4605272`，与环境档登记指纹逐位一致（源码→产物链闭合） |

### 关键断言 A1~A8

- **A1 验收⑤判定 — 通过**。审核者以独立 Python 实现（`math.ceil` 路径，未复用 summarize.py）从 `runs/r5_t10000/out/t10000/frames.csv`（21656 帧）重算：avg_fps = 333.15（≥60 是）、1% low = 202.48（k1=⌈21656/100⌉=217，≥45 是）→ **PASS**，与 summary.md 判定行逐位一致。0.1% low=153.40（k01=22）同档一致。
- **A2 四档降档扫描 — 通过**。1k/2k/5k/10k 四档 frames.csv 原始数据在档（34843/33933/32936/21656 帧）；四档 avg/1% low/0.1% low + p50/p95/p99/max 全指标重算与 summary.md 四档表逐位 MATCH（容差 <0.005，实为四舍五入同值）。四档 capture 累计和 65.001~65.003 s 与 65 s 窗口口径自洽（cross-check 采集计量逻辑）。
- **A3 采样原始帧时间入档 — 通过**。五档（含冒烟 t100）CSV 行数 = captured_frames + 1 表头，逐档核对；t10000 首 3 行（idx 0/1/2 → 3290600/2963100/2233800 ns）与末 3 行（idx 21653/21654/21655 → 2832300/2927000/2987600 ns）与档内 CSV 直接对读一致。
- **A4 sim 零改动红线 — 通过**。`diff -rq 主仓/sim 树/sim` 零差异（exit 0）；根 Cargo.toml diff 恰两行（`members = ["sim", "render-spike"]` + `bevy_full = { package = "bevy", version = "0.19", default-features = true }`）；`render-spike/` 内 grep `sim::` / `use sim` 零命中。全树 diff -rq（排除 .git/target/team-prompt）改动面 = 预期四处，无越权改动；task-ledger.md / taskset / AGENTS.md / rust-toolchain.toml / .gitignore 零差异（禁改清单合规）。
- **A5 bevy 0.19 API 查证 — 通过**。api-notes.md 14 节覆盖实际所用全部 API（含 4 处 0.19 差异点与 derive 宏别名 workaround 第 14 节），逐条带 URL + 签名摘录 + 本地 registry 源码行号。抽 3 条与 docs.rs 现页核对 3/3 吻合：① `Capsule3d::new(radius, length)`（radius 在前、half_length=length/2、有 `From<Capsule3d> for Mesh`）；② `Time` 现页方法表无 `delta_ns`、`delta() -> Duration`；③ `WindowResolution::new(physical_width: u32, physical_height: u32)` 物理像素语义 + `physical_width()/physical_height()` getter 存在（与 capture.rs 用法一致）。
- **A6 W1 双盲 — 通过**。链条齐全：`rng_double_blind.py` 独立 Python 实现（掩码算术，与 Rust 两侧独立书写）→ `runs/w1_python_first8/python_first8.log` 原始输出 → Rust 侧先失败档 `runs/w1_rust_first8/attempt_failing/`（REAL_EXIT=101，PIT-M-002 占位先失败纪律）→ `comparison_note.txt` 8/8 逐位一致 → rng.rs 黄金常量回填转绿。审核者第三实现独立复算 seed=42 前 8 输出：与黄金常量逐位一致（True）。
- **A7 判定公式预注册 — 通过**。summarize.py 与派工单 §5 逐字对齐：avg = N/(Σ delta_ns/1e9)；1% low k=(N+99)//100、0.1% low k=(N+999)//100，均 max(1,·)（整数算式 = ceil，下限 1 帧）；判定常量 60.0/45.0（表 6-0 验收⑤）；vsync 旁证行阈值 int(0.95×16_700_000)=15_865_000 ns、双分支文案与派工单一致；**FAIL 后果行代码路径存在**（summarize.py:236-243，本卡未触发——PASS 为如实结果非粉饰）；退出码 0/2/3 实现并文档化。跨语言 W4 对齐：selftest.log（44.11764705882353 / 15.625 / 15.625）= Rust W4 测试期望 = 派工单 §7 手算参考值，三方一致。
- **A8 证据档自含与可公开态 — 通过**。逐档结构 = cmd.txt + stdout/stderr + REAL_EXIT（+time.txt/precheck.log）；r0~r8 + w1 + lock diff 全 REAL_EXIT 正确（门禁 0、CLI 错误档 2×3）；命令全相对路径（只看档即可复跑）；全档机器绝对路径扫描（用户目录前缀 / 树外目录段 / 用户名片段等 6 类模式）零命中；6 处前缀清理留痕 `runs/scrub_note.txt`（T003/T007 先例：只改前缀）。

### 已知披露 1~6 如实性核对（均属实）

1. **Cargo.lock +419**：审核者独立计数 135→554（+419），与派工单预期「仅 +render-spike 条目」不符属实——sim minimal 下 bevy 全图原不在锁内，设计侧预期错误、Lead 已归因派工单（g1-lead-recheck.txt §5）；diff 三件套在档；`-` 内容行 13 条全部为同名多版本消歧重写（`"bitflags"` → `"bitflags 2.13.2"` 类），无删包、bevy 单条 0.19.1（两入口同锁）。
2. **窗口 1920×1080 请求 / 物理 2400×1350**：五档 meta.json `window_resolution_actual` 逐档记录一致；125% DPI 逻辑尺寸成因引 bevy_winit 源码行；方向保守（物理画布 ≈1.56× 像素、负载更重仍 PASS ⇒ 1080p 口径结论安全）。审核者端到端复跑实测同窗口尺寸，口径可复现。
3. **D2 与派工单 §3 的 0.8 m 平移差**：grid.rs:40-41 实现按派工单 `(gx − 49.5)×1.6`（场地居中）执行，README 事件记录第 5 条如实披露供 T013 留意。
4. **sim 既有 2 条 test-profile unused_mut 警告**：sim/ 与基线字节级零差异 ⇒ 警告定义性为基线既有；审核门禁 2 复跑 exit 0/47 绿（缓存编译未重放警告，与 worker 新鲜编译档出现警告不矛盾）。
5. **三处首败修复留痕**：`r0_gate1_first_attempt/compile_errors_excerpt.txt`（脚本上溯 off-by-one + E0433 derive×别名）在档；`w1_rust_first8/attempt_failing`（REAL_EXIT=101）在档；R7 BOM 首败在 README 事件记录第 4 条披露（t007 同式修复）。
6. **bevy_full 别名 derive 宏 workaround**：main.rs:35 / capture.rs:16 `use bevy_full::ecs as bevy_ecs;`，api-notes 第 14 节引 bevy_macro_utils 0.19.1 源码注释（官方内置 workaround 原文）。

### 完整轮独立复算留痕（≥1 项要求，实做 4 类）

1. 四档全指标独立重算（审核者自有 Python 实现）：全部 MATCH（上文 A1/A2）。
2. SplitMix64 黄金 8 值第三实现复算：逐位 True（上文 A6）。
3. 端到端复跑：`./target/release/render-spike.exe --units 1000 --warmup-sec 2 --capture-sec 5 --out <临时目录>` → **exit 0**，frames=2333、Σdelta=5.001 s（与 5 s 窗口自洽）、avg=466.47 fps（与归档 t1000 档 536 同量级，短窗含启动方差属正常）、meta.json 字段齐全且 `window_resolution_actual=2400×1350` 与四档一致。
4. 复构建 exe sha256 与环境档登记指纹逐位一致（上门禁表第 4 行）。

另：Lead G1 档（g1-lead-recheck.txt）含 t10000 fresh 采集二证（avg 329.33 / 1% low 177.95，同球区）——判定稳健性双重独立来源。

## [阻塞项]

无。（一票否决五类——安全 / 数据丢失 / 破坏兼容 / 伪造或缺失证据 / 违反验收标准——逐类排查零命中；版本锁 0.19、确定性纪律、写范围、禁改清单、预注册判定全合规。）

## [最小修复指令]

无阻塞修复。收获时顺手项（Lead 执行，非 worker 返工）：

- 台账回写：task-ledger.md T010 行 + taskset/README 状态列 + AGENTS.md 进度节（按账本清单，本卡禁改范围内的回写归收获侧）。
- 文档 nit（可入 T013 勘误清单，不单独返工）：README 事件记录第 2 条「diff 中 14 处 `-` 行」实为 13 条内容行 + 1 条 diff 头 `---`（grep 计数 14 含头行）；「均为同名多版本消歧重写」仅对 13 条内容行成立。

## [复验命令]

树根（= 本仓仓库根）执行，串行、-j 2，每条前跑附录 A commit 余量预检：

```bash
cargo check --workspace -j 2            # 期望 exit 0 且 0 警告
cargo test -p sim -j 2                  # 期望 exit 0，44+3=47 全绿
cargo test -p render-spike -j 2         # 期望 exit 0，10 全绿
cargo build -p render-spike --release -j 2   # 期望 exit 0
# 判定独立复算（任意独立实现；公式：avg=N/(Σft/1e9)，k=⌈N/100⌉ 下限 1）：
python docs/evidence/t010/summarize.py --selftest        # W4 参考值 44.11764705882353/15.625/15.625
# 端到端抽查（≈8 s，机器空闲时）：
./target/release/render-spike.exe --units 1000 --warmup-sec 2 --capture-sec 5 --out <临时目录>   # 期望 exit 0
```

## 审核覆盖（代码审核格式对照）

- 共 120 个文件 — 已审 120（数值级深审 49：render-spike 全部 8 个源码/manifest、根 Cargo.toml diff、Cargo.lock 结构级（计数/版本/render-spike 条目/±diff 三件套）、docs/evidence/t010 顶层 9 档全读、runs 内全部判定承重件——四档 meta+frames 全量重算、W1 双盲 8 档、lock diff 3 档、selftest、REAL_EXIT 全量 10 档、precheck/idle、smoke、r8 退出码；其余 71 个机械流水档（time.txt/复述行/stderr 细节）逐档过退出码、行数计数、机器路径扫描三道机器核对）/ 跳过 0 / 覆盖率 100%。
- 五字段 ↔ 结构化报告映射：结论↔合并裁决（通过=✅ 可合并）；已核对↔审核范围+逐条验收；阻塞项↔Critical（无）；最小修复指令↔修复建议（Minor 2 条，不阻塞）；复验命令↔单列如上。
