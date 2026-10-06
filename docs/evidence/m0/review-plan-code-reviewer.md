# T011 完整审核轮报告（plan-code-reviewer，2026-10-06）

> 存档说明（Lead）：本报告为 plan-code-reviewer 完整轮原文存档；裁决 = **有条件通过（P0=0、P1=0、P2×2、Minor×5）**，验收断言 1/2/3 全 PASS。P2×2 + M-2/M-5 已于同日整改闭环（见文末「整改闭环记录」）；M-1 随收口统一处理、M-3/M-4 留痕不动。整改后 verify_claims.py 复跑 81/81（runs/verify-g1.txt）。

## 总裁决

**裁决：有条件通过（⚠️ 修后可合并）+ 整改项 2（P2×2）**。全部验收判定行逐字忠实、口径引用七处行号全对、投入校准拆分依据经独立硬证成立、抽查三档自洽；两处整改项均为引用指针精度问题，零判定行数字受影响。P0=0，P1=0。

独立复跑输出摘要（只读复跑，未改任何文件）：
- `python docs/evidence/m0/verify_claims.py` → `汇总: 81 条 | PASS 81 | FAIL 0`，REAL_EXIT=0（与归档 `runs/verify.txt` 逐行一致）
- `python docs/evidence/m0/tally_hours.py` → `预备周小计: 557 min = 9.28 h` / `W1 累计: 1106 min = 18.43 h`，REAL_EXIT=0（与 G1 修正段一致）

## 审核范围

被审交付物 21 个文件：`docs/evidence/m0/` 20 档（README.md、tally_hours.py、verify_claims.py、selfcheck.md、g1-lead-recheck.md、dispatches/wp-a.md、runs/ 14 档——tally.txt/verify.txt/changed-files.txt + spot-t008 四件 + spot-bench-10k-t1 三件 + spot-t009-simt1 四件）+ `taskset/t011-data-pack.md` 修改（git diff 14 行插入 = 开工裁决 D1~D9+D3-1，已逐字比对）。
加载规则：review-protocol `rules/python.md` + `default.md`（在审含两份 Python 脚本）。
对照参考档（交叉核对用，20 份）：task-ledger.md、AGENTS.md、`docs/万阵-游戏前期策划报告.html`（L460-560 + 定点 grep）、t007（summary/README/environment/longrun_meta/takeover_preflight）、t008（matrix/README/environment/plan.txt/red200-th1-s42.cmd）、t009（summary/README/run_t009.sh/crosscheck_r5.sh/r5-sim-t1.cmd/environment）、t010（summary/README/environment）、t015（summary/README/run_matrix_t015.sh/environment）、taskset/t015-opt-round-1.md。

覆盖率：共 21 个文件 — 已审 21 / 跳过 0 / 覆盖率 100%（runs/spot-t008-…diff 为 0 字节空档，以 find 尺寸 0 + diff 语义核实）。加载规则 2 份（限 1-2 份，未全量）。

说明一处委托口径差：审核委托称「19 档 / runs 13 档 / spot 三档 ×4」，实有 20 档 / runs 14 档——spot-bench-10k-t1 仅 check/stdout/stderr 三件（无 .cmd，命令与 REAL_EXIT 内嵌于 .check）。交付物自身账目自洽（changed-files.txt 18 档 = worker 产出口径，g1-lead-recheck「18 档」一致），差 1 出在委托描述，非交付缺陷。

## 逐项审核结果（审核要点 1~10）

**R1 判定行忠实性 — PASS（含 §0 三方一致 PASS）**
独立复跑 verify_claims.py 81/81 后另做 grep -F 独立抽验 13 条，覆盖 ①②③④⑤⑥+换算表行，全部「源档命中 ∧ README 引用逐字 ∧ §0 快照值一致」三方闭合：① `0.070617` 判定行 t015/summary.md:20；换算表行 `| 10000 | 0.070617 | 0.088271 …` t015/summary.md:61；② `1.001913/TRIPPED` t015/summary.md:42；③ `games_per_hour@12t = 1099638.6` t009/summary.md:180；④ 断言 1 t008/matrix.md:135 + 终局哈希 `0xde91d6a5a6e84d43` t008/README.md:84；⑤ `avg_fps=333.15…` t010/summary.md:31；⑥ `[⑥] 判定：达标` t007/summary.md:77 + 窗口 670.6 s t007/summary.md:73。§0 快照表全部数字（含 `6.8 MiB ≤ 2048 MiB`→t007/summary.md:74 源证、`17.532997/3907.133373`、`0.060651/0.072196/0.070617/0.108681`、`7.221411`）与各节判定行、源档三方一致。§3/§4 转述数字亦全溯源：cell 29/34/35 翻转 + n=2 → t009/summary.md:225/230/231（三行恰为「不一致」三格）；`14/14 局 units<10000（9959/9969/9970）` → t008/README.md:52 + t008/matrix.md:126；`对角 6 格 z=−10.000` → t009/summary.md:133-148/152-167（双层 12 格口径正确）。§0 总注 1 对 taskset/t015-opt-round-1.md 的长引文经 L129-131 比对忠实（源档硬换行处以空格拼接、粗体标记剥离，属引文常规处理，措辞数字零改动）。唯一形态例外见 Minor M-1（③ 终态判定列千分位）。

**R2 口径引用正确性 — PASS**
HTML 行号引用 7 处全部命中原文（grep -noF 定点确认）：:471 含「首周/降档/20h」（§8 R7 规则）、:485 = 降规模定义全文（X19 尾句「两套口径不得混用。」在内）、:490 = 表 6-0 题注、:498 一线上同含表 6-0 ①③④⑤⑥ 与 ②附则「外推 10 万…方保留「极限十万」目标」、:525 = 6.1 换算公式、:538 = R2 吞吐线**全括号段逐字**（「（降规模口径，表 6-0——全规模对局按止损线约 530 场/小时，仅作质量校验、不计吞吐）」整段命中）、:543 = R7 补充（20h/无限期挂起）。AGENTS.md 引用行号亦准：投入口径 L12、render-spike 门禁 L34、bench 用法 L32。D2 编号澄清如实：t007/summary.md:44 与 t015/summary.md:44 均含内部节名 `## ③ 外推十万`，README §0 总注 3 + §2⑤节首双重澄清「报告验收②附则、非报告验收③」，防混淆目的达成。

**R3 投入校准 — PASS**
（a）D3-1 99/91 拆分依据成立且超委托范围加证：台账 T007 行阶段分解「worker-2 ≈99 + 接管 ≈65 + 审核 ≈25」在档（task-ledger.md T007 行）；`docs/evidence/t007/environment.txt` 生成时间 2026-10-04 15:28:02、`docs/evidence/t007/runs/takeover_preflight.txt` 首行 2026-10-05 08:08:59 双时间戳硬证跨界；另以 git 提交时间线独立复核前提「T007 是唯一跨预备周/W1 边界的卡」为真——T002/T003（09-30）、T004（10-04 11:07）、T005（13:24）、T006（15:01）全部收口于预备周内，T008/T015/T009 提交均 10-05、T010 提交 10-06 01:09（跨两日均在 W1 窗口）。（b）tally_hours.py 修正版独立重跑 557/1106（9.28 h/18.43 h），与 runs/tally.txt G1 修正段、手工算式（458+99；91+605+145+125+140）一致。（c）预备周+W1 逐任务 11 项耗时与台账逐行一致（150/15/18/35/110/130/190/605/145/125/140）。（d）快照+终判条款形态符合 D3：判定快照行明示「未到期——不可终判」、终判条款明示判定式与回写责任，W1 未终判正确。

**R4 脚本防假绿 — PASS**
tally_hours.py 耗时列解析正确：`re.split(r"(?<!\\)\|", …)` 转义竖线处理（台账 T004 行 `\|dx\|` 实证存在）+ 只取第 6 列（与台账表头「编号/任务/类型/一次通过/返工次数/耗时/原因/证据」索引对应，逐字核对）——T010 任务列「≈92 min 零中断」陷阱被双盲实拦（v1 W1=1157≠1205 停核，留痕 runs/tally.txt 第三段 + docstring），防假绿制度真实生效。verify_claims.py 双向子串核对逻辑透明无隐藏分支（needle ∈ 源档 ∧ needle ∈ README）。CLAIMS 覆盖核实：A~P = 42 条（A4+B4+C4+D1+E1+F1+G1+H1+I2+J7+K6+L2+M1+N4+O1+P2）与派工单 wp-a.md §2 清单逐条对应（J/K/L 按其自身预授权补全行），+ 引用点扩充 X01-X39 = 39 条，合计 81 与 §9 声明一致。覆盖缺口（§0 快照/转述数字不在 CLAIMS 内）由本审核人工补验，见 R1，结果全绿。

**R5 抽查档真实性 — PASS**
三档自洽且强交叉验证：档 a 命令三处逐字一致（t008/runs/plan.txt:51 = t008/runs/red200-th1-s42.cmd = spot cmd），spot stdout 208 字节与归档 diff 空（0 字节），四采样哈希+终局哈希与 t008/matrix.md:15 表行逐值一致（0x4b8f…/0x82c8…/0x07eb…/0xe0cd…/0xde91…）；档 b final_hash 0xc5915d042208e267 与 t007/summary.md:91、t015/summary.md:82 双档附录 A 同配置行同值（D4 只核哈希口径如实注记）；档 c final_hash 0x564cf46fdf191710 与 t009/summary.md:245-247 黄金交叉三方同值一致，命令来源 run_t009.sh:85 = r5-sim-t1.cmd:1 逐字。三档 REAL_EXIT=0 全有留痕（cmd 尾行/.check 行）。覆盖 t007/t008/t009/t015 四个不同证据档（≥3 要求满足）。t010/t015 重型档不重跑的豁免执行本身按 D4，但其指针入档缺位——见 P2-2。

**R6 可公开态 — PASS**
`grep -rnIF`（模式：`C:\`、`C:/`、`/c/`、`Desktop`、`Administrator`、`<d-drive>\`、`D:/`）对 m0/ 全目录排除 wp-a.md 后 19 档零命中（SCAN_EXIT=1）。wp-a.md 按流程档豁免（含树根 `<trees-root>\t011-a\`，属派工单允许内容）。git bot 身份/密钥类亦零暴露。

**R7 环境总档 — PASS**
5 份 environment.txt 全读逐项比对：rustc 1.98.1 (48a229cea 2026-09-01)、cargo 1.98.1 (797e8a9bc 2026-08-05)、CPU i5-12490F 6C/12L/3000MHz、RAM 34187943936 bytes Kingston 3600、OS 10 IoT LTSC 19044、GPU WMI 双卡（RTX 3050 31.0.15.3758 DriverDate 10/04/2023 + GameViewer 15.6.5.199）、toolchain channel 1.98.1、bevy 0.19.1——五档同值断言全部成立。抽验指纹 ≥3 项均对上：t007 bench.exe `D8E251B0…F089`（mtime 2026-10-04 15:19）、t008 sim.exe `C8006C0A…` + bench.exe `80EBCCB0…` + git HEAD b2dad10a…、t010 nvidia-smi 旁行「RTX 3050, 537.58」+ render-spike `264A9593…`、t015 bench `5A2312…`。差异披露如实：nvidia-smi 旁行仅 t010、profile 节 t008/t010 缺省、t008 toolchain 逐行三行、HEAD 字段详略不一——均按「不合并不择优」分行列出。T007 bench 三态哈希链（D8E251→09971d→80ebcc）经 t007/README.md:99/:103 与 t008 环境档交叉闭合。

**R8 宣称-证据对齐 — PASS（附 Minor×2）**
「约」独立扫描仅 2 命中（L169 = R2 引用原文自带、L422 = 自查元表述）、「大概」1 命中（同元表述行）——与 selfcheck §2 声明一致，验收级数字零「约/大概」修饰。AGENTS.md 抽查 4 项独立复核全对：333.15/202.48 各 1 击、`0.061/0.072/0.071/0.109` 1 击、`1,099,639` 1 击、`1099638.6`/`1,099,638.6` 均 0 击（t010/summary 333.15/202.48 各 2 击、t015 `0.060651` 4 击、t009 `1099638.6` 3 击，全部与 selfcheck §3 表一致）；「1,099,639 vs 1099638.6 取整一致字面不同」如实披露并上报 Lead、未自行处理。本档自身两处计算（投入小计、670.6/60=11.18）均算式呈现；「≈」仅出现于投入耗时与引用原文。轻微项见 M-2/M-3。

**R9 D1~D9+D3-1 裁决执行 — PASS（D4 一处指针缺位→P2-2；D8 门禁形态注记→M-5）**
D1（②按字面 TRIPPED+归因链+owner 裁决点注记、不粉饰不放行、①时序对称）→ §0 总注 1 + §1③ + §2④ 全落实；D2 → §0 总注 3 + §2⑤ 落实；D3 → §8 快照+终判条款落实；D3-1 → tally 脚本 SPLIT 常量 + §8 逐任务/小计/快照三处 + tally.txt G1 段 + 任务卡留痕全落实；D5 → §7 双表（要素表+分卡指纹表，各自列出不合并）落实；D6 → 逐字纪律 + verify 双向核对 + 双盲拦截实例落实；D7 → selfcheck 四节落实；D9 → T011「收口回填」占位 + 脚本「未登记」不报错落实；D4 三档选择与执行落实、t010/t015 复跑旁证指针未入 README（P2-2）；D8 纯新增红线落实（changed-files.txt 18 档全 m0/**、sim/render-spike/既有证据档零改动、git status 佐证），门禁按派工单 §5 自我修正形态执行（M-5）。

## 优点

1. 判定行纪律执行到位：81 条双向核对 + 审核侧 13 条独立 grep -F 抽验 + §0 快照三方一致，全部零偏差。
2. 口径引用精度罕见地全对：HTML 七处行号（:471/:485/:490/:498/:525/:538/:543）+ AGENTS.md 两处（L12/L34）逐一命中原文，R2 吞吐线连括号段逐字。
3. 双盲防呆是真制度不是摆设：tally v1 误抓 T010「≈92 min」被手工速算当场拦下（1157≠1205）并留痕归因（runs/tally.txt 第三段）。
4. D3-1 拆分有硬证链且经审核侧 git 时间线独立加证（「唯一跨界卡」论断为真）；快照+终判条款形态正确拒绝提前终判。
5. 抽查三档交叉验证强：spot-t008 stdout 与 t008/matrix.md:15 采样列逐值同源、bench 哈希双档附录 A 同值、sim_t1 三方黄金交叉同值。
6. 披露诚实度高：② TRIPPED 不放行、H2 红偏如实移交、AGENTS.md 字面差异上报不自处、selfcheck §5 连「首条门禁误落主仓」都如实入档。

## Critical（P0）

无。

## Important（P2）

1. 置信度 95 | README §1⑥ 两处源档行号引用失准 | `docs/evidence/m0/README.md:91`（「构建（docs/evidence/t015/run_matrix_t015.sh L28 提示行，逐字）」）与 `:97`（「L24-25 变量定义 + L159-160 汇总步」） | 依据：`docs/evidence/t015/run_matrix_t015.sh` 实测（grep -n + Read 双证）——`EVID=` 在 L22、`BIN=` 在 L23（L24 为 `REF=`、L25 为 `mkdir`），构建提示行在 L29（L28 为 `if [ ! -x "$BIN" ]`）；引用文本本身逐字无误（X32/X33 双向命中），仅行号偏 1~2，L159-160 正确 | 修复：`L28`→`L29`、`L24-25`→`L22-23`。
2. 置信度 90 | D4「t010/t015 复跑旁证指针入档」缺位，且 g1 交叉引用失实 | `g1-lead-recheck.md:26` 称「复跑旁证指针 = 各档 g1-lead-recheck + 审核轮记录，README §5 已注」，但 README 全档 grep `复跑旁证` 零命中（唯一命中即 g1 该句）；README §5 仅 DPI/vsync 披露，§10 仅三档记录与档 b 口径注 | 依据：任务卡 D4 要求该指针入档；实际旁证档在库（t010/t015 各自 g1-lead-recheck + review 报告）但未以具体路径写入总索引，验收断言 2 的重型档豁免理由在自含档内不可定位 | 修复：README §10 末补指针行 + 不重跑理由，并把 g1:26 的「README §5 已注」改为实际落点。

## Minor

- M-1：§0 ③ 终态判定列作「PASS 1,099,638.6 场/h@12t」（千分位形态，派工单 §0 规格原文即此形态），源档字面为 `1099638.6`；值一致且相邻关键数字列给出源档字面。建议收口统一精度时（既有裁决点）一并消歧。
- M-2：runs/verify.txt 未内嵌 REAL_EXIT 行（tally.txt 与各 spot 档均有），§9 的 REAL_EXIT=0/运行时刻为文字声明；G1 修正后 verify 重跑输出未另档。经审核独立复跑证实为真，属留痕形态问题。
- M-3：selfcheck.md §2 行号引用对当前 README 有 1 行漂移（元表述行现为 L422，自记 L421；G1 §8 加行所致）；实质结论（约 2/大概 1、均合规）复核属实。
- M-4：wp-a.md §1 规定每验收节固定六段结构（含⑤原始证据档路径），实际仅 §1 完整保留；§2~§6 以行内源档路径+行号满足断言 1 可点开性——功能达标、形态偏离模板。
- M-5：D8 门禁形态与任务卡字面不一致（任务卡 D8 = `cargo check --workspace -j 3`；实际 = `cargo build -p sim --release -j 3`）。属 Lead 在派工单内自我修正并留痕（bevy full 冷编 19m45s 成本理由充分），但任务卡 D8 行未回写同步；建议收口时补修正指针。

## 验收断言判定

1. **判定行齐全、每行可点开原始证据档 — PASS**。六项验收判定行全数在档且逐字（81 条双向核对 + 13 条独立抽验零偏差），每条均带源档相对路径 + 行号（行号抽验 20+ 处全部准确；唯二例外为 §1⑥ 复现命令块两处行号——P2-1，判定行本身不受影响）。
2. **抽查任意 3 项命令可直接重跑 — PASS**。三档命令全文、相对路径、参数完整（t008 局 / bench 单配置 / t009 sim_t1），来源可溯（plan.txt:51、AGENTS L32、run_t009.sh:85），已实际执行并留痕（diff 空、双哈希 MATCH、REAL_EXIT 全 0），覆盖 4 个不同证据档。
3. **投入校准判定行在档（含台账逐任务耗时汇总表）— PASS**。§8 含逐任务表 12 行（T011 占位）、小计算式、判定快照行（明示未到期不可终判）+ 终判条款；数值经 tally_hours.py 独立重跑（557/1106）与台账逐行复核一致。

## 遗漏 / 疑问

- runs/verify.txt 的「运行时刻 2026-10-06T08:50:23」无档内时间戳可对；若需严格可复核性，建议 verify 输出附 ISO 时刻行（与 M-2 一并整改即可）。
- 已知裁决点按要求未重复上报：② owner 裁决点（呈现形态忠实：字面 TRIPPED + 归因链 + 注记出处齐全，无粉饰无放行）、T007 尾差 1 min（D3-1 留痕完整）、AGENTS.md 吞吐数字收口（G1 裁决 1 已留痕）。
- 委托描述的档数口径（19/13/spot×4）与实际（20/14/spot 3+4+4）差 1，归档对账以 changed-files.txt 18 档 worker 产出口径 + wp-a.md + g1-lead-recheck.md 为准。

---

## 整改闭环记录（Lead，2026-10-06 同日）

| 项 | 整改动作 | 复验 |
|---|---|---|
| P2-1 | README §1⑥ 两处行号勘误（`L28`→`L29`、`L24-25`→`L22-23`），勘误注记随行保留 | Lead grep -n 独立核对（EVID=L22/BIN=L23/提示行 L29）后落改 |
| P2-2 | README §10 末新增「未重跑重型档的复跑旁证指针」节（t010/t015 各 g1-lead-recheck + review 报告四指针 + 不重跑理由）；g1-lead-recheck.md 原「README §5 已注」改为实际落点 §10 并注审核轮 P2-2 整改 | 本档 grep `复跑旁证` 于 README 命中（原零命中） |
| M-2 | 新增 runs/verify-g1.txt（内嵌 ISO 时刻 + REAL_EXIT 行 + 整改后 81/81 全量截录）；README §9 补指针 | REAL_EXIT=0，`汇总: 81 条 \| PASS 81 \| FAIL 0` |
| M-5 | 任务卡 D8 行补「门禁修正」回写（派工单 §5 落地指针） | 任务卡 diff 在档 |
| M-1 | 不改档：AGENTS.md「当前状态」节收口更新时统一为全值 1,099,638.6（G1 裁决 1）；README §0 千分位与源档字面 1099638.6 值一致、判定行引用处始终逐字 | AGENTS.md 收口 commit |
| M-3/M-4 | 不动（历史档如实 + 形态偏离留本报告为证） | — |

整改后复跑：verify_claims.py 81/81 PASS（runs/verify-g1.txt）；总裁决 **有条件通过 → 整改闭环后转通过**。
