# T026 下一身份决策评审材料｜plan-code-reviewer 完整轮报告

## 裁决

**不通过（❌ 不可合并）**。发现 2 项 P0：三层执行覆盖数与卡片分类不符、拆卡预估源行号错误。tally/verify 复跑通过，四选项主问题基本对称；修正后需复审。

## 审核范围

- `docs/m5-identity-review.md`（完整材料）— 已审
- `docs/evidence/t026/tally_t026.py`（完整脚本）— 已审
- `docs/evidence/t026/verify_t026.py`（完整脚本）— 已审
- `docs/evidence/t026/runs/tally.txt` — 已审
- `docs/evidence/t026/runs/verify.txt` — 已审
- `docs/evidence/t026/runs/web-020-check.md` — 已审
- `taskset/t026-identity-review.md` — 已审
- `taskset/README.md`（T026 行及预估锚）— 已审
- 规则文件：`.zcode/skills/review-protocol/rules/default.md`、`.zcode/skills/review-protocol/rules/python.md`；另按仓库约定读取 `AGENTS.md`。未修改任何被审文件；唯一写入是本报告。审核过程中生成的 Python 缓存已清除。
- **覆盖率：共 8 个指定文件 — 已审 8 / 跳过 0 / 覆盖率 100%。**（AGENTS.md 与跨仓锚源是审核依据，不计入调度方文件清单。）

## 优点（Strengths）

- `docs/m5-identity-review.md:111-120` 的 §3 明确声明不裁决不推荐，四选项 C1/C2/B/A 均列出支持事实、成本、文档动作和首卡；按要求检查后，未发现主问题对身份路线做明示推荐。§4 的分项推荐被明确标作推荐，并对应来源/约定（如 `:141-143`）。
- grill 八题在材料 `docs/m5-identity-review.md:13-25` 与任务卡 `taskset/t026-identity-review.md:8-13` 内容相符；T026 后置节点、不计 M5 封顶的范围也在任务卡 `:4-6`、README `:37` 明示。
- tally 全部四张转写表均与台账一致：脚本 `tally_t026.py:27-105` 中 CARDS / CARD_LEVEL / MINUTES_WHOLE / REVIEW 的 25 个耗时、25 个卡级字段、17 个审核行及 20 个审核锚逐一核对，未见错录；T007 99+91=190 分钟拆分有台账 T011 D3-1 与投入权威表支撑。
- 跨仓内容按锚子串引用，没有整段复制工作流仓正文；G 组锚在规定源行命中。外部 0.20 检索仅以留档快照作状态来源，材料也注明未来真实发布窗口需再复核（`docs/evidence/t026/runs/web-020-check.md:7-8`）。

## P0（必须修复，阻塞）

1. **三层执行覆盖数与明示的 owner 决策卡分类冲突。** 置信度 92 | `docs/m5-identity-review.md:71` | 材料称“24 张开发卡全部按「Lead 规划/派工 → worker 隔离树执行 → 审核轮把关」执行”。但同一材料 `:108` 明确把 T016/T017 列为两张 owner 决策卡，并称二者无独立审核；任务卡 `taskset/t017-m5-definition.md:30-31` 记载 T017 是 owner grill、Lead 落档与批复收口，台账 `task-ledger.md:46` 也记为 owner 级内容决策；量化脚本 `docs/evidence/t026/tally_t026.py:103-105` 同样将 T016/T017 列入无 plan-code-reviewer 轮卡。任务总数是 T001~T025 共 25 张（`task-ledger.md:23-54`），排除这两张 owner 决策卡后是 23 张，而非 24 张；至少 T017 也不满足“审核轮把关”的全称断言。`task-ledger.md:48-52` 的引用范围还只覆盖部分 M5 卡，不能单独支持“全周期 24 张”。复跑旁证：量化脚本 `REAL_EXIT=0`，四表逐行核对一致，输出 `25 卡`。| 将分母和纳入条件改为与任务分类一致的明确口径（例如仅计入符合该执行流程的卡，并逐卡列出/引用覆盖清单）；不要把 owner 决策材料卡算作三层执行卡。

2. **拆卡预估≈39h 的来源行号错一行。** 置信度 100 | `docs/m5-identity-review.md:102` | `〔源: taskset/README.md:47〕` 实际是“依赖与并行”；预估 ≈39h 在 `taskset/README.md:48`（已定点读取该行原文）。当前指针不支持这段数字。| 将指针改为 `taskset/README.md:48`，并复核其他定点引用是否准确指向包含对应事实的行。

## Important（强烈建议修复）

无（未确认达到 80% 置信度的非阻塞问题）。

## Minor（可选）

无。

## 复跑输出、抽查与负控

- **tally 独立复跑**：命令 `python docs/evidence/t026/tally_t026.py`；`REAL_EXIT=0`；stdout 2326 字节与 `docs/evidence/t026/runs/tally.txt` **逐字节一致**；末尾 `RESULT: PASS`。确认的汇总输出：`窗口：预备周 557 min / W1 3330 min / 总 3887 min = 64.78 h`；`阶段：M0 2268 min / M5 1619 min`；卡级 25 / 严格一次通过 4 / 零返工 7 / 返工轮 23；审核 17 首跑（完整 12、轻量 5）+3 跟进；捕获 P0/P1/P2=4/12/16。
- **独立手算过程**：逐台账行提取 25 卡耗时、一次通过与返工数；PREP = T001 150 + T002 15 + T003 18 + T004 35 + T005 110 + T006 130 + T007 99 = 557；W1 = T007 91 + T008 605 + T015 125 + T009 145 + T010 140 + T011 140 + T012 150 + T013 110 + T014 165 + T016 40 + T017 75 + T018 216 + T021 180 + T020 200 + T022 115 + T023 165 + T019 378 + T024 145 + T025 145 = 3330。总计 3887；M5 T017~T025 = 1619；M0 = 3887−1619=2268。小时换算 3887/60=64.7833→64.78、557/60=9.2833→9.28、3330/60=55.5、1619/60=26.9833→26.98、2268/60=37.8。T007 99/91 来源：`task-ledger.md:36` 的 D3-1、`docs/evidence/m0/README.md:401,409-415`、`taskset/README.md:13-18` 的预备周窗口标注及 `AGENTS.md:12` 的日历切分规则。25 卡返工值求和=23；严格“是”4 卡为 T005/T008/T016/T017；零返工 7 卡为其上 4 卡加 T021/T022/T023。REVIEW 表逐项相加 P0=4、P1=12、P2=16；其中完整轮 12、轻量轮 5。
- **verify 独立复跑**：命令 `python docs/evidence/t026/verify_t026.py`，输出 `T026 锚子串双向核对：30 条`、`PASS 30 | FAIL 0`、`RESULT: PASS`。
- **审核者新挑锚（8 条，均非 CLAIMS 表锚）**：依据 `taskset/t026-identity-review.md:10`，逐项对任务卡 ↔ 主材料双向核实：`一并决：万阵身份 + 产能去向`（材料缩写表达，按对应定案语义命中）；`现在备料、近日拍板`；`如实入档缺位`；`主问题不推荐`；`C1 收官→主线 / C2 收官→新选品 / B 万阵 M6 / A 升格`；`定性+量化双面`；`任何路线都解冻`；`既定规则展开为条款`。各锚均在两份文件中语义/文本匹配，verify 已有 CLAIMS 未包含这些完整锚串。
- **跨仓 G 组**：`../bevy-ai-workflow/AGENTS.md:11,13`、`Bevy-AI开发意向文档.md:138`、`docs/m5-game-selection.md:46` 均核验到对应状态/原文。独立查到“wanzhen”在 `assets-methodology/pitfalls.md` 只有 T001 一处方法论教训引用（`:368`），支持主材料所称“仅 1 条”这一计数；未见复制跨仓整段正文。文档对 M5 备忘录封顶是短锚摘录，符合引用而非复制。
- **负控/错误路径**：不落盘内存负控删除 T022 台账行，tally 返回 1 并报 `台账行缺失: T022` 及 `台账行缺失(审核): T022`；注入不存在锚串返回 1 并报 `审核锚串未命中`。verify 模拟缺源档抛 `FileNotFoundError`、非法 UTF-8 抛 `UnicodeDecodeError`，均不能错误 PASS（失败关闭）。
- **卫生**：对 8 个交付文件逐字节扫描四种要求的机器路径形态，`SCANNED=8 FILES MATCHES=0`；8 个文件均可按 UTF-8 解码，无替换字符/疑似乱码命中。
- **Git 卫生**：`git diff --check` 无输出；审核范围为一个已修改文件（`taskset/README.md`）及 7 个新增 T026 文件（主材料、任务卡、两脚本、三运行档）。

## 未决疑问

一项宣称的审核时间点未在当前证据中单独记录：`docs/m5-identity-review.md:74` 总结 T025 数字转述的勘误为“自查自纠，无一流入归档判定”；台账 `task-ledger.md:54` 证明曾有 `304.15→304.10` 的自查更正，最终判定档 `docs/evidence/m5/README.md:16` 使用 304.10，但这些材料不足以断定误值是否短暂进入过一个早期归档版本。故未将这点定为缺陷；如作者保留“无一流入”绝对表述，可补充确认其所指归档版本/时间点。

## 修后复审

**裁决：通过（修后可合并）**。复核范围限于初审 P0-1、P0-2 的整改及其是否引入新问题；此前未修改的其余内容不重新全面审核。

### 初审 P0 复核

1. **P0-1 三层执行统计口径：已修复。** `docs/m5-identity-review.md:72` 现限定为 T007~T015 与 T018~T025 共 17 张卡，并把 T001~T006 的制度定案前形态及 T016/T017 的 owner 决策卡分列说明。独立读取 `docs/evidence/t026/tally_t026.py:78-96` 并提取 REVIEW 卡号，所得集合恰为 T007~T015、T018~T025 共 17 张；分层计数为完整轮 12 + 轻量轮 5。`runs/tally.txt` 同样记录 `审核：17 轮首跑（完整轮 12 + 轻量轮 5）`；`task-ledger.md:44,46` 分别说明 T016/T017 是 owner 决策卡。范围说明与量化锚一致，原“24 张”计数冲突已消除。
2. **P0-2 预估源行号：已修复。** `docs/m5-identity-review.md:102` 现引用 `taskset/README.md:48`；定点读取该行确认内容为“M5 预估合计 ≈39h（T018~T025）”。全文检索主材料与任务卡中的 `taskset/README.md:` 引用，仅该处引用该文件行号，未发现其他因该行移动造成的错位。

### 回归检查

- 两项修改未造成邻近章节/分项推荐、四选项对称性或其他已有断言的可见破坏；没有发现新的 ≥80 置信度问题。
- 按用户要求独立复跑 `python docs/evidence/t026/verify_t026.py`：`REAL_EXIT=0`，`PASS 30 | FAIL 0`，且 stdout 与刷新后的 `docs/evidence/t026/runs/verify.txt` 逐字节一致。
- tally 再跑结果仍为窗口 557/3330/3887，阶段 M0/M5 2268/1619，审核 17 首跑（12 完整 + 5 轻量）、P0/P1/P2=4/12/16，`RESULT: PASS`。改动聚焦口径和源行引用，没有改变 tally 输入表。
- `git diff --check` 无输出。

**修后复审 P 项计数：P0=0 / P1=0。最终裁决：通过（修后可合并）。**

### 修后复审范围

- `docs/m5-identity-review.md:72,102` — 已审（两项整改点及上下文）
- `taskset/README.md:48` — 已审（源行定点）
- `docs/evidence/t026/tally_t026.py:78-96`、`runs/tally.txt` — 已审（17 卡集合、审核分类与复跑结果）
- `docs/evidence/t026/verify_t026.py`、`runs/verify.txt` — 已审（独立复跑与存档输出）
- **复审覆盖：上述 6 项 / 6 项已审，跳过 0。**
