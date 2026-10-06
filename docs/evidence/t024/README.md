# T024 观战档性能验收与 C-1 关账（M5-07 席位 9）证据档

- 任务卡：`taskset/t024-perf-c1.md`；派工单：`dispatch-sheet.md`（设计裁决 D1~D7）
- 基线 commit：`43cd5cd`（tree 导出，无 .git）；执行：worker-2 · 隔离树 `t024-a`
- 类型：★完整轮量测卡（量测窗口机器空闲独占；三段留痕强制）

## 结论先行

- **腿 A 观战档 10k：PASS**——avg_fps=304.10（≥60，裕度 5.07×）、1% low=197.22（≥45，裕度 4.38×）（判定行脚本生成；`summary-spectate.md`）。
- **腿 B C-1 关账（render-spike t10000 干净窗口复测）：PASS**——主样本 339.64/259.35；加样 ×2 同向 PASS（341.82/257.50、339.05/256.40）。**T012 P1-1 带载不可复现未复现**——avg 漂移 +1.77%~+2.60%（±25% 带内）、1% low 漂移 +26.63%~+28.09%（超带**向上**，如实披露）；归档 PASS 获确认。
- 干净窗口三段留痕（全量快照 + 每 20 s CPU/CommitFree + 复扫）两窗口齐备；窗口内零 cargo/冒烟/额外 host（pre/post 断言）。
- 插桩零侵入：七组既有冒烟全回归 46/19/30/30/37/52/89 全 PASS（SCRIPT_EXIT=0）+ 带 `--frame-capture` 冒烟 1 发自退（rc=0，frames.csv 非空）。

## 判定行（脚本生成，禁手改）

### 腿 A：观战 10k（`summary-spectate.md`；`summarize_t024.py` 生成）

指标表（脚本输出原样）：

| 档位 | 帧数 N | avg_fps | 1% low | 0.1% low | p50 (ms) | p95 (ms) | p99 (ms) | max (ms) | 非vsync旁证帧数 | 判定 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| spectate-10k | 19767 | 304.10 | 197.22 | 144.18 | 3.216 | 3.853 | 4.372 | 7.635 | 19767 | PASS |

判定行原文：

> - spectate-10k: avg_fps=304.10 (≥60 ? 是)、1% low=197.22 (≥45 ? 是) → **PASS**

- vsync 旁证：有（19767 帧 < 15.865 ms，AutoNoVsync 生效）。
- meta 现场读数（`runs/spectate-10k/meta.json`）：`units_total=10000`、`tick_at_write=2100`（采集窗尾未冻结；warmup+capture = 70 s × 30 Hz = 2100 tick 全数推进 ✓）、`window_resolution_actual=2400x1350`、seed 42、max_ticks 3600。
- 命令（D2）：`host --spectate --comp shieldman:834,heavyknight:834,pikeman:833,swordsman:833,archer:833,militia:833 --seed 42 --max-ticks 3600 --frame-capture runs/spectate-10k`（REAL_EXIT=0，墙钟 71 s）。

### 腿 B：C-1 关账（render-spike t10000 干净窗口；复用 `t010/summarize.py` 零改动）

主样本（`summary-c1-render-spike.md`；REAL_EXIT=3 = 缺 1k/2k/5k 档**预期**，t10000 判定行仍产出）：

| 档位 | 帧数 N | avg_fps | 1% low | 0.1% low | p50 (ms) | p95 (ms) | p99 (ms) | max (ms) | 非vsync旁证帧数 | 判定 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| t10000 | 22077 | 339.64 | 259.35 | 206.65 | 2.890 | 3.340 | 3.564 | 5.952 | 22077 | PASS |

> - t10000: avg_fps=339.64 (≥60 ? 是)、1% low=259.35 (≥45 ? 是) → **PASS**

加样 ×2（附录 G「复测超标即加样复测、全档披露」；判定行分别在 `summary-c1-supp-r2.md` / `summary-c1-supp-r3.md`）：

> - t10000: avg_fps=341.82 (≥60 ? 是)、1% low=257.50 (≥45 ? 是) → **PASS**（r2）
> - t10000: avg_fps=339.05 (≥60 ? 是)、1% low=256.40 (≥45 ? 是) → **PASS**（r3）

**C-1 漂移表**（`drift_c1.py` 生成，独立重算；归档常数逐字源 = `docs/evidence/t010/summary.md` L14/L31；±25% 带源 = `docs/evidence/m0/review/report.md:251`）：

| 样本 | N | avg_fps | avg 漂移 vs 归档 | 1% low | 1% low 漂移 vs 归档 | ±25% 带 |
| --- | --- | --- | --- | --- | --- | --- |
| 归档（t010/summary.md） | 21656 | 333.15 | — | 202.48 | — | — |
| r1 (主样本) | 22077 | 339.64 | +1.95%（带内） | 259.35 | +28.09%（超带（向上）） | ±25% |
| r2 (加样 #1) | 22219 | 341.82 | +2.60%（带内） | 257.50 | +27.17%（超带（向上）） | ±25% |
| r3 (加样 #2) | 22039 | 339.05 | +1.77%（带内） | 256.40 | +26.63%（超带（向上）） | ±25% |

- **披露（禁粉饰）**：avg 侧三样本带内；1% low 侧三样本一致超带**向上**（+26.63%~+28.09%，三样本间极差仅 1.5 个百分点——系统性上偏而非随机噪声）。可披露事实：main 分布几乎一致（N +1.9%、p50 −1.5%），差异集中在尾部（归档最慢 1% 帧均 4.94 ms vs 本窗 3.86 ms）——归档窗口尾部抖动更大；成因不归因（无直接证据），红绿判定行口径零改动、归档值零改动。
- **C-1 关账语义（D4③）**：干净窗口实测入档——T012 P1-1「带载不可复现（-79%~-89% + window_resolution_actual=0x0 + 窗口非空闲）」在本卡两窗三样本中**未复现**；归档 PASS（333.15/202.48）获干净窗口一致性确认。**非「翻绿」**：归档原本即 PASS，本卡为干净窗口复核。
- 命令（D4②）：`render-spike --units 10000 --seed 42 --warmup-sec 5 --capture-sec 65 --res 1920x1080 --out runs/c1-render-spike/r1/out`（REAL_EXIT=0，墙钟 71 s；加样 r2/r3 同参）。

## 预注册 vs 实测（D6②）

- **预注册（派工单原文，量测前登记）**：PASS 预期方向——「T010 归档 render-spike t10000 avg 333.15 / 1% low 202.48——spectate 版叠加 HUD/表现映射/宿主循环，裕度收窄但仍应远超阈值」；**不设数值区间**，只注册方向 + 阈值（avg ≥60 / 1% low ≥45）；实测偏离如实披露禁事后改口。
- **实测对照**：
  - 腿 A：**PASS**，方向一致。同窗口交叉对照（观战 vs render-spike）：avg −10.46%、1% low −23.96%——与「叠加表现层后裕度收窄」方向一致，裕度仍 5.07×/4.38×。
  - 腿 B：**PASS**，方向一致（归档 PASS 复现）。
  - 无方向性偏离；唯一的带状披露 = C-1 1% low 上偏 +26.6~+28.1%（见上节，超 ±25% 口径向上）。
- 事后零改口：本档所有判定行由脚本在实测数据上生成（`summarize_t024.py` / `t010/summarize.py`），阈值常量未改动。

## 口径注（D6①③；灰盒边界）

1. **灰盒边界注（残余账 #9，沿 T010 口径）**：本卡渲染面为灰盒——无动画/LOD/特效/多分辨率（范围预裁剪），观战形态虽含 HUD/表现映射/宿主 30 Hz 驱动（比 T010 静态灰盒更接近产品形态），仍**高于 M2/M3 表现层常态属预期，指标不作 M2/M3 外推依据**。
2. **观战帧率 ≠ 模拟吞吐（表 6-0 口径分立）**：本卡判定对象 = 观战形态帧率（渲染 + 宿主循环 + 表现映射）；模拟侧 µs/加速比口径见 t015/m0 档（sim/ 零改动，本卡不产模拟侧结论）。
3. **10k 构成算式**（D2；等比缩放 + 整数均分）：sim 默认构成六兵种各 5 = 每方 30（`sim/src/world.rs:166` `DEFAULT_COMPOSITION`）等比缩放至每方 5000；整数算式沿 `sim/src/bin/bench.rs:284-302` `composition_for`：per_side=5000 → q=5000/6=833、r=5000%6=2 → 表序前 r 个 +1（表序 bench.rs:289-296）⇒ `834/834/833/833/833/833`（和 5000/方；`--comp` 双方对称 ⇒ 总数 10,000 = 备忘录「万人常态 10k」字面）。
4. **判定口径**：avg/1% low/0.1% low 算式与 vsync 旁证逐字沿 `docs/evidence/t010/summarize.py`（`summarize_t024.py` 同式；`--selftest` 与 t010 输出**逐字节一致**实证）。阈值注源 = 表 6-0 验收⑤ / T010 先例（`t010/summarize.py:34-35`）。
5. **窗口与呈现**：两腿同口径 warmup 5 s 丢弃 + capture 65 s 判定窗；实际物理窗口 2400×1350（125% DPI——T010 披露先例，负载更重为保守方向）。

## 索引

| 文件 | 内容 |
| --- | --- |
| `dispatch-sheet.md` | 派工单（Lead 设计 D1~D7；基线拷贝） |
| `README.md` / `env.md` | 本档索引与判定 / 环境档（机器/工具链/二进制指纹/构建耗时） |
| `measure_t024.sh` | 量测主脚本（两腿 + 三段留痕钩子 + post-window 判定生成；窗口 1） |
| `measure_c1_supp.sh` | C-1 加样脚本（r2/r3 + 三段留痕；窗口 2） |
| `sampler.ps1` | 负载抽样器（每 20 s CPU/CommitFree；ASCII-only + PS5.1 参数面注） |
| `summarize_t024.py` | 腿 A 判定脚本（口径逐字沿 t010；含 `--selftest`） |
| `drift_c1.py` / `c1-drift-table.md` | C-1 漂移表（独立重算；加样披露） |
| `summary-spectate.md` | 腿 A 判定（脚本生成） |
| `summary-c1-render-spike.md`、`summary-c1-supp-r2.md`、`summary-c1-supp-r3.md` | 腿 B 主样本 + 加样 ×2 判定（t010 脚本生成） |
| `runs/spectate-10k/` | 腿 A 落档（frames.csv + meta.json；判定用样本） |
| `runs/c1-render-spike/r1/out/t10000/` | 腿 B 主样本落档（判定用样本） |
| `runs/c1-supp-r2/…`、`runs/c1-supp-r3/…` | 腿 B 加样落档 |
| `runs/fc-smoke/` | `--frame-capture` 冒烟自退产物（frames.csv 35531 帧 + meta） |
| `runs/leg*-*.log`、`runs/*-exit.txt` | 各腿 stdout/stderr/退出码/墙钟 |
| `runs/gate1-check.log`、`gate1b-check-final-source.log`、`gate2-host-release.log`、`gate3-*` | 门禁日志（check/host release/render-spike release/commit 预检/终源复验） |
| `runs/smoke-*-console.log` | 七组既有冒烟回归控制台实录（各脚本自身日志见对应 evidence 目录） |
| `window/pre-process*.txt`、`post-process*.txt`、`load-samples*.csv`、`events*.txt` | 干净窗口三段留痕（窗口 1/2；全量进程表 + 声明 + 20 s 抽样 + 时间线） |
| `attempt1-sampler-failure/` | 事件归档：attempt-1 双腿绿但抽样器故障（见「事件记录」） |

## 干净窗口实录（摘要）

- 窗口 1（腿 A+B）：`2026-10-07 06:24:27 ~ 06:27:34`（legs 06:24:35-06:25:46 / 06:26:17-06:27:28，中间 30 s 空档）；窗口 2（加样 r2/r3）：`06:29:15 ~ 06:32:20`。
- 跑前快照 + 声明：`window/pre-process.txt` / `pre-process-supp.txt`——全量进程表 + `rustc/cargo/host/render-spike = 0` + CommitFree + 端口 15702 空闲 + 「无并发负载」声明（Lead 承诺窗口内零负载）。
- 量测中抽样（每 20 s）：`window/load-samples.csv`（10 行 = 表头 + 9 样本；腿间空档样本 0~6%）/ `load-samples-supp.csv`（同规格）。
  - 窗口 1：腿 A 内 30/31/49%（被测负载自身主导）、空档 5/0%、腿 B 内 10/21/14%。
  - 窗口 2：r2 内 22/23/16%、空档 0/6%、r3 内 17/11/21%。
- 结束后复扫：`window/post-process.txt` / `post-process-supp.txt`——零残留（legs 进程全退、逐项断言）。
- 第三方负载取证（快照 CPU 列 diff，两窗一致）：窗口内全部第三方进程累计 CPU ≈ 70~75 s / 187 s（≈ 单核 40%，整机占比 ≤3.3%），且为常驻后台基线（ZCode 主进程 ~25 s、dwm ~12 s——桌面合成含被测窗口本身、sqlservr ~9 s 等）；**无 SLDWORKS/AutoCAD 系活跃进程**（T012 故障窗特征物）、`window_resolution_actual=2400x1350` 正常（非 0x0）。
- 构建时序纪律：全部构建腿（gate1~gate3）于窗口前完成，最后一笔构建（06:14）与窗口 1（06:24）间隔 ≈10 分钟；窗口内禁一切 cargo/冒烟/额外 host 实例（脚本 pre/post 断言实证）。

## 事件记录（异常处置；≤2 次自查纪律）

1. **attempt-1 抽样器故障（06:16~06:19，双腿绿但窗口证据不完整）**：`sampler.ps1` 两处 5.1 兼容缺陷（① BOM-less UTF-8 中文注释被 PS 5.1 按 ANSI 读→param 块解析失败；② `Out-File -FilePath` / `Add-Content -Path` 参数面倒挂——本机 PS 5.1.19041 实测：Out-File 只有 `-FilePath`、Add-Content 只有 `-Path`）。attempt-1 双腿 rc=0、采集产物完整，但**零负载样本 → 窗口不合规**；按纪律归档（`attempt1-sampler-failure/`，含 note）并重排 attempt-2 为判定用窗口。修复后抽样器经 45 s/30 s 两次独立自检（header + 20 s 行距 + 零 stderr）＋窗口覆盖校验（≥8 行红线入脚本）。
2. **attempt-2 重排（06:24）**：窗口 1 判定用样本即 attempt-2 产物；attempt-1 与 attempt-2 的腿指标互证一致（spectate avg 304.21→304.10）。

## 门禁与冒烟回归（插桩零侵入证明）

| # | 项 | 命令 | 实测 | 判定 |
| --- | --- | --- | --- | --- |
| 1 | check 0 警告 | `cargo check --workspace -j 3` | REAL_EXIT=0；0 警告；Finished 1.75s；终源复验 0.88s（`runs/gate1-check.log` / `runs/gate1b-check-final-source.log`） | PASS |
| 2 | host release | `cargo build -p host --release -j 3` | REAL_EXIT=0；11.91s（增量） | PASS |
| 3 | render-spike release | `cargo build -p render-spike --release -j 2`（commit 预检 11.7 G 附增量事实） | REAL_EXIT=0；8.86s——**仅 Compiling render-spike，无 bevy 冷编签名** | PASS |
| 4 | t018 BRP 冒烟 | `bash docs/evidence/t018/brp_smoke.sh` | **PASS=46 FAIL=0**；SCRIPT_EXIT=0 | PASS |
| 5 | t020 m5core 套件 | `bash docs/evidence/t020/m5core_suite.sh` | **PASS=19 FAIL=0** | PASS |
| 6 | t020 错误路径 | `bash docs/evidence/t020/error_paths.sh` | **PASS=30 FAIL=0** | PASS |
| 7 | t021 配置面冒烟 | `bash docs/evidence/t021/preset_smoke.sh` | **PASS=30 FAIL=0** | PASS |
| 8 | t022 挑战面冒烟 | `bash docs/evidence/t022/challenge_smoke.sh` | **PASS=37 FAIL=0** | PASS |
| 9 | t023 统计面冒烟 | `bash docs/evidence/t023/sample_smoke.sh` | **PASS=52 FAIL=0** | PASS |
| 10 | t019 观战冒烟 | `bash docs/evidence/t019/spectate_smoke.sh` | **PASS=89 FAIL=0**；含跨模式黄金锚 `0xb82a248ff23515e2` | PASS |
| 11 | `--frame-capture` 冒烟 1 发 | `host --spectate --preset melee-brawl --seed 7 --frame-capture runs/fc-smoke` | 自退 rc=0；frames.csv 非空（35531 帧）；meta 齐（`runs/fc-smoke/`；banner 行在 stderr） | PASS |

- 端口纪律：各冒烟端口固定（15702/15706/15707/15712/15713/15714/15715-15717），跑前 `Get-NetTCPConnection` 全空闲检查 + 各脚本自检；串行执行零冲突。
- 冒烟脚本按其设计自写运行日志至各自 evidence 目录（41 个文件被刷新，清单见完成报告；判定门禁以 `runs/smoke-*-console.log` 控制台副本与脚本自身 SUMMARY 行为准）。

## 环境与二进制指纹

见 `env.md`：host.exe sha256 `602b491f…9f8dd`（95,966,208 B）/ render-spike.exe sha256 `ff4b990f…6767c5`（85,301,760 B）；构建命令与耗时、工具链 1.98.1 钉版、基准机 A 事实。

- **构建可复现性注**：本机 release 链接**非位级可复现**（受控对照：源字节相同的两次重建哈希不同，实测 `33ddcf6a…` vs `a2826dcc…`）——指纹核验对象 = **档存测量用产物文件本身**（判定数据全部由该文件产生），重建哈希不作核对基准；测量后源改动仅 `frame_capture.rs` 注释内一处行号引用修正（26-33↔26-34 一次往返，无代码/语义面改动）。详见 `env.md` + `runs/build-reproducibility-note.txt`。

## 锚来源指针（可复核性）

- 阈值/验收线：表 6-0 验收⑤（`docs/万阵-游戏前期策划报告.html:498` 转述见 `docs/evidence/m0/README.md` §5①）；T010 先例常量 `docs/evidence/t010/summarize.py:34-35`。
- 判定公式：`docs/evidence/t010/summarize.py`（T024 同式；selftest 逐字节一致）。
- C-1 归档基准：`docs/evidence/t010/summary.md` L14/L31（N=21656 / 333.15 / 202.48）。
- ±25% 带 / 加样处方先例：`docs/evidence/m0/review/report.md:251`；T012 卡 `taskset/t012-independent-recheck.md`。
- 10k 构成：`sim/src/world.rs:166` + `sim/src/bin/bench.rs:284-302`。
- 跨模式确定性锚（冒烟 10 内引用）：`0xb82a248ff23515e2`（T021 归档）。

## m0 侧回写建议（D4④；文本草案，待 Lead 搬入 `docs/evidence/m0/README.md`）

> **⑤ 条件账 C-1 关账（T024，2026-10-07）**
> - 关账动作：干净窗口（三段留痕：`docs/evidence/t024/window/`——跑前全量快照+无并发负载声明 / 量测中每 20 s CPU·CommitFree 抽样 / 结束复扫；窗口内零 cargo·冒烟·额外 host）render-spike t10000 复测——判定行（脚本生成）`t10000: avg_fps=339.64 (≥60 ? 是)、1% low=259.35 (≥45 ? 是) → PASS`（`docs/evidence/t024/summary-c1-render-spike.md`）；加样 ×2 同向 PASS（341.82/257.50、339.05/256.40；`summary-c1-supp-r2.md`/`-r3.md`）。命令与产物：`docs/evidence/t024/runs/c1-render-spike/`、`runs/c1-supp-r2/`、`runs/c1-supp-r3/`。
> - T012 P1-1（带载不可复现：-79%~-89% 漂移 + `window_resolution_actual=0x0` + 窗口非空闲）在干净窗口**未复现**——三样本 avg 漂移 +1.77%~+2.60%（±25% 带内）、1% low 漂移 +26.63%~+28.09%（超带向上，如实披露）；归档 PASS 获确认（判定口径/归档值零改动）。条件账 C-1 具备正式闭合条件。
> - T013 条件性披露（V1.0 6.2 ⑤：渲染腿后果仅 ⑤ 最终判 FAIL 时触发）以本行为事实依据：干净窗口 ⑤ 维持 PASS，后果路径未触发；M1 复测复核点的最终关闭归 T025 对账裁决。

## 上报项（待主会话裁决）

1. **C-1 1% low 超带向上（+26.6~+28.1%，三样本一致）**——已按先例加样并全档披露；是否需要口径侧任何跟进动作（如 m0 侧附加注记）由 Lead 定。
2. **commit 预检 11.7 G < 12 G 阈值**（render-spike 构建前置）——按派工单「附增量事实」条款放行：共享 target 工件在位、实测零依赖栈编译行（8.86s 完成）；是否需要在纪律文档中固化「增量场景阈值口径」由 Lead 定。
3. **冒烟脚本自写日志刷新**（41 个文件，t018~t023 evidence 目录内）——收获 pathspec 决策归 Lead（建议只收 `docs/evidence/t024/**` 与 `host/**` 改动；刷新日志如需要可一并收）。
