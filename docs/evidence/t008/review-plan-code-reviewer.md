# T008 完整审核轮报告存档（plan-code-reviewer，2026-10-05 19:04~19:26）

> 存档说明：以下为 plan-code-reviewer 完整审核轮（量测/复测卡口径）最终报告全文（主会话存档；仅第 1 节的卫生扫描模式串一行按 P2-1 口径脱敏为描述，其余逐字保留）。审核独立复现产物在 target/review-t008/（不入库）。

---

### 合并裁决
**有条件通过（⚠️ 修后可合并）** —— 验收④三断言的全部数据与判定经独立复算、双锚 diff、双抽查复跑验证成立，证据链完整；唯一 P1 为解析契约源行号引用 off-by-one（3 处，须整改后归档），无 P0。

### 审核范围
- **被审 diff**：`git status` 3 个已跟踪修改（`docs/evidence/t006/batch.sh`、`task-ledger.md`、`taskset/t008-replay-hash.md`）+ 新增 `docs/evidence/t008/` 全目录（11 顶层文件 + dispatches/4 单 + runs/92 档）。
- **加载规则**：review-protocol 七条防误报协议；按在审文件叠加 python 场景规则（compare_matrix.py / gen_comp.py 为 python 脚本）。
- **独立复现产物**（全部写 target/，仓库零写入）：`target/review-t008/`（matrix 重算、双抽查复跑、reviewer 自建 FAIL/缺局夹具）。
- **覆盖率**：共 110 个文件 — 已审 110 / 跳过 0 / 覆盖率 100%。其中 runs/ 92 档采用程序化全量核对（29 stdout 全部经复算器解析 + 29 cmd 全量逐字节比对 plan.txt + 29 stderr 计数与抽查 + 双锚/双 diff 全文），其余文件逐一人工审读。

---

## 1. diff 卫生：机器路径/密钥/个人信息 + t006/batch.sh 最小改动

**证据**
- `git diff docs/evidence/t006/batch.sh`：**恰好 1 行**（第 4 行机器绝对路径 → `cd "$(dirname "$0")/../../.."` + 留痕注释），diff 总计 ±4 行（1 行改动的 +/− 对）；`grep` 确认 COMP500/COMP5000 串、循环体、watchdog 数值零变化。功能核验：`bash -c 'cd "$(dirname "docs/evidence/t006/batch.sh")/../../.." && pwd -P'` → 正确落到仓库根 ✓。
- 全目录路径扫描（机器路径/账号名/密钥类模式，模式串按 P2-1 口径脱敏不复述）：dispatches/ 4 处（流程文档，按调度口径豁免路径类）+ `recheck-main-session.txt:23` 一处（见 P2-1）+ task-ledger T004 既有行（非本次改动）。**无任何真实机器绝对路径、零密钥、零个人信息**（dispatches 内密钥类扫描同样零命中）。
- 新增文件中无 `.exe`/二进制入库（environment.txt 只记 sha256）。

**结论：OK**（1 个 P2 见下）

## 2. 零代码变更红线 + cargo check

**证据**
- `git status --porcelain sim/` → 0 行；`git diff --stat sim/` → 空。`sim/src/` 8 文件原样（D8 红线成立）。
- 独立复跑门禁：`cargo check --workspace -j 3` → `Finished dev profile`，`CHECK_EXIT=0`，二次运行 `grep -c "^warning"` → **0**。

**结论：OK** — 验证卡零代码变更红线成立，0 警告门禁复核通过。

## 3. 独立复算（重点）

**证据**
- ① `python docs/evidence/t008/compare_matrix.py docs/evidence/t008/runs target/review-t008/matrix.md` → **COMPARE_EXIT=0**；`diff docs/evidence/t008/matrix.md target/review-t008/matrix.md` → **仅第 3 行**「生成命令」自引用差异（归档版无 out 参数、复算版带 out 参数），**判定行/矩阵表/§5~§8 全部逐字节一致**（DIFF 输出仅 `3c3` 一段）。
- ② 双锚独立 diff（不经分析器）：`diff runs/anchor1-t006-fullscale.stdout docs/evidence/t006/runs/fullscale.stdout` → A1=0；`diff runs/anchor2-t006-10k300.stdout docs/evidence/t006/runs/10k-s42-t12.stdout` → A2=0。
- ③ 抽查复跑（写 target/）：从 plan.txt 原文命令重跑 **red200-th12-s42**（RERUN_EXIT=0，`diff` 归档 stdout → 0）与 **anchor2 10k×300**（RERUN2_EXIT=0，`diff` → 0），两局均全新独立进程逐字节一致（第 3、第 4 次进程级复现）。
- ④ 哈希溯源抽查 4 局 > 要求 3 局：`full10000-th1-s44 / red200-th3-s44 / red200-th12-s42-r3 / full10000-th6-s43` 各自 stdout 全部 `sample=`/`hash=` 行逐值 grep matrix.md → 每值命中 4 次（= 同组合 4 线程档行），**26 个哈希值全数溯源闭合**；另经复算器全量解析 29 局 stdout（非抽查）。

**结论：OK** — 复算、锚、复跑、溯源四路独立证据全部闭合。

## 4. 判定口径审查

**证据**
- **断言 1/2/3 映射完整**：matrix.md §8 逐条对应——断言 1 = 6/6 组合（24 局 × 全部采样列 + 终局列）；断言 2 = red 12/12 + full 15/15 判定点；断言 3 = 「未触发（无 mismatch）」。§5 明文「无 mismatch（无哈希不一致、无锚局逐字节差异、无种子互异失效）」、§6「无缺局无失败局」，与 exits 29 行全 0 互证，**无粉饰空间**。
- **退出码语义与断言 3 对齐**（compare_matrix.py:49-53, 869-903）：mismatch→5 且 §5 全量列出（`mismatch_blocks` 逐组合 4 档全值、禁止折叠，比较器 ANCHOR_DIFF_LIST_CAP=512 超限也以计数披露）；缺局/REAL_EXIT≠0/断言 2 数据不足→3 且 §6 全量披露；格式错→2；优先级 1>2>5>3>0——**reviewer 独立构造反例实测**：自建夹具改 1 个哈希字符 → **exit 5** + §5 列出全部 4 档值（含被改值 0x…e0 与 3 个原值）；删 1 个 stdout → **exit 3** + §6 缺局行。夹具自测档另证 PASS/FAIL/缺局/锚 diff/格式错/失败局 6 路径实跑退出码 3/5/3、3/5/2/3（fixtures-selftest.txt:27,50,69,82,100,117,128，且注明种子互异分支未覆盖——如实披露）。
- **覆盖域披露如实**：README §1.3「red200 只覆盖移动段…不得外推」；matrix §7.2 = 14/14 full 局 units<10000（9959/9969/9970，复算版逐值一致）；§3 两套构成串用途差异（锚局 T006 COMP5000 vs 矩阵局 bench 映射串，两串 total 均 10000 但分布不同、哈希不可互比）明文披露。**reviewer 独立验证覆盖域声明的数学性**：`deploy` 布阵（world.rs:473-521）红方队头 x=r₀、蓝方镜像 1000−r₀，**队首距离与单位数无关恒 ≈999m**；速度表（units.rs:85-140）最大 0.20 m/tick（HeavyKnight），合速上界 0.2+0.2=**0.4 m/tick** → 接敌 ≥ 999/0.4 ≈ 2497 ticks > 1800，red200 不接敌声明对任意构成/种子成立；「T003 观察」出处核验 = task-ledger.md:27 T003 行逐字（「LANE 1000m、队首相距 999m、合速≤0.4m/tick」），引用忠实；14 局 red 终局 units=200 实证双保险。

**结论：OK** — 判定口径与三断言严格对齐，退出码语义经 reviewer 反例实测，覆盖域披露诚实且数学成立。

## 5. 证据完备性

**证据**
- `exits.txt`：29 行全 `REAL_EXIT=0`、非 0 计数 0、run_id 无重复（`sort|uniq -d` → 0）；**29 局 × (.cmd/.stdout/.stderr) = 87 档齐备**（29 stdout / 29 cmd / 29 stderr，stderr 全非空且为 `threads=`/`elapsed_ms=` 契约行，抽查 red200-th12=182ms、anchor2=16844ms 与 wall_ms 吻合）；总档数 92 = 87+plan+exits+2 锚 diff+1 运营日志，与 README §5 声明逐字吻合。
- **29 个 .cmd 与 plan.txt 全量逐字节比对 → CMD_CHECK_DONE mismatch=0**。
- `run_t008.sh` 断点续跑语义（:57-79）：REAL_EXIT=0 行→SKIP 幂等；非 0 行→「PRIOR-FAIL 不覆盖不重跑」+ 结尾汇总（断言 3 一致）；无 exit 行有残 stdout→覆盖重跑（半局非证据）；watchdog 杀=REAL_EXIT=124 如实入档。check.txt 门禁 3 实测 15/15 SKIP、exits 15→15 无新增。
- **三方一致性**：README §2 引用的 matrix §8 与实档逐字一致；events.md 与 exits/wall 交叉验算——WP-C 开工 10:12:22 + 挂钟 8h32m46s = **18:45:08** 与收官时间精确吻合，14 局 wall_ms 求和 30,763.8s ≈ 8h32m44s（含约 2s 局间开销）✓；sim.exe sha256 `c8006c0a…` 与 environment.txt 逐字一致；compare_matrix.py sha256 `cbefb2b0…` 与 fixtures-selftest.txt:3 一致（自测后未改动）；git HEAD b2dad10 与 environment.txt 一致；wp-c-batch.log 尾部 `BATCH_DONE rc=0 / executed_ok=14` 与 events §7 吻合。
- `compare_matrix.py` 代码质量（1005 行全文审读）：**判定全代码计算、`grep -cE "0x[0-9a-f]{16}"` → 0 硬编码哈希**；锚参照现场 `read_bytes()`（:912-917）；缺局/失败局一律 INCOMP→3 不可被 PASS 掩盖；`decide_exit` 中断言 2 的列缺失分歧先经组列判定落 FAIL→5，无「粉饰成 3」的路径；夹具自测覆盖三必选路径 + 4 附加分支。

**结论：OK**

## 6. 派工单审查（笔误模式重点）

**证据（4 单全读 + 源码逐条核对）**
- `bench.rs:284-301` composition_for 引用**精确**（fn 在 284、per_side/q/r 在 285-287、kinds 表 288-295、map 在 299、return 在 301）；`world.rs:145 TICK_CAP_REDUCED=1800`、`world.rs:149 TICK_CAP_FULL=8*60*30`**精确**；`main.rs:316` sample 行、`main.rs:348-349` stderr 行**精确**；world.rs:552 state_hash、:749 run_with（D6 引用）**精确**。
- **双盲链真实闭合**：gen_comp.py 按 bench.rs 逐字转写重算（200→q=16,r=4；10000→q=833,r=2），三断言含与主会话速算参考值逐字比对 → comp.txt `RESULT: PASS`；reviewer 手算复核两串总和 100/5000、前 r 个 +1 分布 ✓；plan.txt 29 局实际 `--comp` 串 = comp.txt 两串逐字 ✓。
- 采样点/种子/线程/局数口径：D1/D3 与 plan.txt、matrix 列头、exits 局数全吻合（24+3+2=29）；wp-a 选择集 15 局 = 12 红矩阵+2 加样+anchor2 ✓；wp-c 计划 14 局明细（1 锚+12+r2）与实际 executed_ok=14 ✓；watchdog 档位在 run_t008.sh 落地一致且换算式留痕（:20-26）；wp-c 预估单局（th12≈17min 等）标注「禁止当验收判据」且实际均快于预估。
- **发现问题见 P1-1**（main.rs 摘要行行号）与 P2-4（wp-c 括注局数措辞）。

**结论：1 个 P1 + 1 个 P2**（行号引用类，正是本卡重点盯的「数字笔误」模式）

## 7. 台账诚实性

**证据**
- T008 行（task-ledger.md:32）schema 正确（验证 / 一次通过=是 / 返工 0 / ≈620 min）：**一次通过=是**符合「返工=首次判定未通过后的重做次数」定义——全程无判定失败轮；run_t008.sh cd 上溯层级首版错误属执行前自纠（脚本注释留痕、零证据损失），已作为事件①如实入账，未冒充无事。
- **≈620 min**：挂钟全计口径 = 09:45~≈19:5x（605~615 min）+ 审核轮，与「挂钟全计含审核轮」口径一致；其中 WP-C 513 min 经 wall 求和独立验证（512.7 min）✓。五项执行事件（cd 自纠 / 11:16 harness 壳被杀·WMI 核实续跑 / 耗时快于 T006 纯计时差 / 双盲零分歧 / D10×3）逐一可溯源至 events.md、check.txt、comp.txt、batch.sh diff，无遗漏无美化。
- **审核口径行更新**：结构与 AGENTS.md「每卡过审分轻重」一致（轻量轮三要素、完整轮类别、T012 不变逐字对应）；措辞见 P2-3。

**结论：OK**（1 个 P2 措辞 + 1 个 P2 回填提醒）

---

## Critical（必须修复，阻塞合并）
**无。**

## Important / P1（必须修复后归档）

**P1-1 | 置信度 100 | 解析契约源行号引用 off-by-one（「数字笔误」类，同类问题 T007 P1-1 先例）**
- **问题**：三处归档文件将 run 路径五行摘要行出处标为 `main.rs:327-330`——实际 `seed=` 在 **326** 行（327=ticks、328=units、329=final_tick、330=hash）。所列 5 个键落入 4 个行号，内部即自相矛盾。
- **file:line**：`docs/evidence/t008/dispatches/wp-a-runner-red-matrix.md:48`、`docs/evidence/t008/dispatches/wp-b-comparator-env.md:23`、`docs/evidence/t008/compare_matrix.py:34`。
- **依据**：`grep -n 'println!("seed={}", args.seed)' sim/src/main.rs` → `326:`；对照 `main.rs:316`（sample 行）与 `main.rs:348-349`（stderr 行）两处引用均精确，唯此一处差 1 行。
- **影响面**：解析器按内容解析、不读行号，**数据与判定零影响**（已由复算/复跑独立证明）；缺陷在证据指针准确性，且为本项目连续五卡追踪的笔误模式。
- **修复建议**：三处 `327-330` → `326-330`。**连带注意**：若改 `compare_matrix.py`，fixtures-selftest.txt:3 记录的 sha256 须同步重算或按原命令重生成自测档，保持指纹链闭合；两份派工单属流程文档可直接改并留一行修订注记。

## Minor / P2（建议，不阻断）

- **P2-1** `docs/evidence/t008/recheck-main-session.txt:23`：复现记录把扫描模式串原样写入正文——非真实机器路径、无个人信息，但该行会被未来任何卫生 grep 命中（且使该行自称的「→0 命中」在复扫时自我矛盾）。建议改为脱敏描述或注明「本行为模式自述、复扫时须自排除」。
- **P2-2** `docs/evidence/t008/events.md:6` 时间占位「10:0x WP-C 开工」与同文件 :9 及 README §6.4 的精确值「10:12:22」不一致。建议统一为 10:12:22。
- **P2-3** `task-ledger.md:6` 审核口径行自述「与 AGENTS.md 一致」，但写「节点卡 / **量测复测卡** / …」，AGENTS.md 实为「节点卡 / **量测卡** / …」（taskset/README.md:5 同为量测复测卡）。语义收窄风险，建议三处统一为 AGENTS 措辞或明确「量测/复测卡」。
- **P2-4** `docs/evidence/t008/dispatches/wp-c-fullscale-matrix.md:26` 括注「`full10000-*` 全部 13 局 + `full10000-th12-s42-r2`」重复计数 r2；其下「计划 14 局」明细与实际执行 14 局均正确，纯措辞瑕疵。建议改「full10000-* 全部 13 局（含 r2 加样）」。
- **P2-5** `task-ledger.md` T008 行 ≈620 min 为审核轮进行中登记的含轮估值——审核收口时若实际结束时间显著晚于估值，按「实测耗时」口径回填终值。

## 优点 (Strengths)
- 判定完全机器化且经受住反例检验：分析器零硬编码哈希（grep=0）、锚参照现场读取；reviewer 自建 FAIL 夹具实测 exit 5 + §5 四值全列、缺局夹具实测 exit 3——断言 3 的「不静默、不折叠」是可执行的而非声明性的。
- 四路独立证据互相咬合：复算逐字节一致（仅预期的自引用行）→ 双锚逐字节一致 → 双新进程复跑逐字节一致 → 26 哈希全量溯源，任一路失效都会在另三路暴露。
- 时间账与文件账全部可手工验算：10:12:22+8h32m46s=18:45:08 精确闭合、wall 求和 512.7min≈台账 513min、92 档构成公式精确、29 cmd 与 plan 全量逐字一致。
- 双盲防呆制度真实落地：gen_comp.py 独立转写 + 三断言 + 与速算值逐字比对，T003~T006 连续四卡笔误教训在本卡零复发（除 P1-1 的行号笔误外，全部数值口径零偏差）。
- 覆盖域声明经本审核从源码层独立重推（deploy 镜像布阵与单位数无关 + 速度表上界 0.4）成立，非纸面转述。

## 遗漏 / 疑问（需作者澄清）
- `runs/wp-c-batch.log` 仅审尾部（`BATCH_DONE rc=0`）与 events 引述，未逐行全文审读——该档自身标注「非 D4 契约档位」，不影响判定链；如需可补审。
- 11:16 harness 包装壳被杀事件的「WMI 核实存活」为当时快照，事后无法复验，只能以 exits.txt 29 行全 0 + log 收官行间接佐证——已由时间账闭合间接支持，采信。
- 环境空闲独占声明为承诺性声明（与 T006/T007 同口径），本审核无独立手段验证跑批窗口内无并行负载。

---

**裁决：有条件通过（⚠️ 修后可合并）**
必须整改项（唯一）：**P1-1** —— 三处 `main.rs:327-330` 行号引用改为 `326-330`（改 compare_matrix.py 时同步更新 fixtures-selftest.txt 的 sha256 指纹）。P2-1~P2-5 建议同批顺手处理，不阻断。整改落地后无需重跑任何数据：数据、判定、锚、复跑四路证据均已独立验证成立，验收④闭环与「数据放行 T013」结论维持有效。

---

## 主会话整改闭环（2026-10-05 19:3x，逐项对应）

- **P1-1 已落地**：wp-a:48 / wp-b:23 / compare_matrix.py:34 三处 `327-330` → `326-330`（派工单两处加「审核轮 P1-1 修订」注记）。compare_matrix.py 仅注释行变更——行为零变化经复算比对证实（POSTFIX_COMPARE_EXIT=0、matrix.md 除自引用行逐字节一致）；fixtures-selftest.txt 按原命令重生成（新 sha256 a9de0b64… 指纹链闭合，7 路径退出码 3/5/3、3/5/2/3 与原一致）。
- **P2-1**：recheck-main-session.txt 模式串行改为脱敏描述（注明不复述防复扫自命中）。
- **P2-2**：events.md 时间占位 → 10:12:22（与 README §6.4 一致）。
- **P2-3**：task-ledger.md:6 与 taskset/README.md:5 统一为「量测/复测卡」（AGENTS.md 措辞不动）。
- **P2-4**：wp-c 括注改「全部 13 局（含 r2 加样）」。
- **P2-5**：台账 T008 行耗时回填终值 ≈605 min（09:45~19:5x 挂钟全计）。
- 整改后复核：卫生扫描 0 命中（dispatches 豁免）、py_compile 通过。**裁决转通过**。
