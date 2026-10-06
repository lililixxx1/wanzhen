# T022 轻量轮审核报告（plan-code-reviewer，2026-10-07）

> 审核对象：T022 收获提交 fe0286a。审核器按只读要求以最终消息返回报告全文（未写文件），
> 本档 = 其最终消息逐字归档（Lead 转录，零改动；其自称「报告归档目标」即本路径）。

## §0 裁决

**有条件通过（修后可合并）**。没有发现 Critical 问题；唯一待办是派工单 D5 指定由 Lead 在收口段更新的 AGENTS.md 常用命令说明。

## §1 范围方法

审核基线为 `fe0286a87f96f9fc0b7a23473676d3b702ea22ff`。读取了 `AGENTS.md`、`taskset/t022-challenges.md`、`docs/evidence/t022/dispatch-sheet.md`，以及 review-protocol 的 `rules/default.md`。逐项比对 D1–D6，核验任务卡四项验收、锚值来源和提交范围；按要求未运行 cargo 命令。

收获 diff 共 9 个文件，全部已审：

- `host/src/challenges.rs`：已审
- `host/src/suite.rs`：已审
- `host/src/main.rs`：已审
- `docs/evidence/t022/README.md`：已审
- `docs/evidence/t022/challenge_smoke.sh`：已审
- `docs/evidence/t022/challenge-smoke-run.log`：已审
- `docs/evidence/t022/host-stderr.log`：已审
- `docs/evidence/t022/host-stdout.log`：已审（空文件，已核验）
- `docs/evidence/t022/t018-smoke-rerun.log`：已审

任务卡、派工单及两份 design 锚日志作为审核依据另行核对。`git show` 显示 `sim/`、`host/src/rpc.rs`、`host/src/presets.rs` 均未进入提交 diff。审核开始时工作树已有 `docs/evidence/t018/brp-smoke-run.log` 未提交改动；本次未改动该文件。

**覆盖率：共 9 个 diff 文件 — 已审 9 / 跳过 0 / 覆盖率 100%。**

## §2 逐断言核对

### D1–D6

- **D1 数据结构**：通过。`host/src/challenges.rs:52-73` 定义了构成、参数、固定 seed 和 `ExpectedOutcome`；六个字段包含 `deploy_hash` 与五个终局字段。三个挑战的注册定义位于 `host/src/challenges.rs:116-176`。
- **D2 固定 seed 与串行判定**：通过。三项定义均使用 seed 42；`host/src/suite.rs:385-393` 通过 `run_battle_with(..., None)` 串行执行。
- **D3 套件注册与零回归**：通过。`host/src/suite.rs:49-61,183-197` 注册 `challenges`、`m5-all` 并聚合 9+3 项；缺省套件仍为 `t018-smoke`。模拟器及 RPC handler 未被修改。
- **D4 六字段比对、期望值单一来源、失败详情**：通过。`host/src/suite.rs:394-400` 比较布阵哈希及五项终局值；`host/src/suite.rs:415-429` 的失败详情同时输出期望值和实测值。断言名由 `host/src/challenges.rs:78-85` 给出。
- **D5 模块头注**：通过。`host/src/challenges.rs:6-46` 包含三挑战语义、终局分类、seed 43 设计注、ch2b 敏感度旁证和灭绝预算算式。**收口文档待办见 §4。**
- **D6 证据档**：通过。`docs/evidence/t022/README.md:8-19,65-99` 提供索引、判定记录和门禁记录；脚本及日志保存 BRP 请求/响应和结果。

### 任务卡验收 1–4

1. **三件套数据化：通过。** 三个注册项均包含构成、参数和预期终局字段，见 `host/src/challenges.rs:116-176`。
2. **接入 `game.run_tests` 且判定行代码生成：通过。** `host/src/suite.rs:183-190` 按注册表逐条生成断言；证据日志包含三个指定断言名及通过结果，见 `docs/evidence/t022/challenge-smoke-run.log:38-45`。
3. **期望值与实测一致、固定 seed 可复跑：通过。** 派工单 §2、注册常量和 `design/measure-candidates-run.log` 中的 seed 42 布阵哈希与终局字段逐位一致；seed 43 敏感性注也与 `design/measure-seed43-run.log` 的哈希一致。引用行号经 grep 复核。
4. **check 0 警告、sim 零改动：证据通过。** `docs/evidence/t022/README.md:94-98` 记录 `cargo check --workspace -j 3` EXIT=0、0 警告，以及 sim 测试通过；独立核对提交 diff 确认 `sim/` 未改动。按要求未自行运行 cargo。

### 优点（Strengths）

- 锚值使用十六进制常量，并注出来源；设计日志、注册表和派工单之间便于逐项追溯（`host/src/challenges.rs:112-175`）。
- `m5-all` 明确组合 core 与挑战断言，且复用现有套件函数，没有复制期望值（`host/src/suite.rs:193-197,385-429`）。
- BRP 复测日志保存了挑战实测字段和各套件结果，证据可独立复核（`docs/evidence/t022/challenge-smoke-run.log:7-63`）。

## §3 抽查复跑记录

端口 15706 复跑前确认空闲；直接启动已有 `target/release/host.exe --port 15706`，未运行 cargo。独立请求结果如下：

- `game.run_tests {"suite":"challenges"}`：total 3、passed 3、failed 0；包含 `challenge::few-elite`、`challenge::counter-militia`、`challenge::iron-wall`。
- `game.run_tests {"suite":"m5-all"}`：total 12、passed 12、failed 0。
- `game.run_tests {"suite":"m5-core"}`：total 9、passed 9、failed 0。
- 缺省 `game.run_tests {}`：`t018-smoke`，total 4、passed 4、failed 0。
- 手驱 `few-elite` 的 deploy/outcome：deploy hash `0x583ddfef1fc446cf`；实测 red、tick 2474、存活 7/0、final hash `0xa48ce79b1e83e105`，与注册锚一致。

复跑后进程已清理，15706 恢复空闲。

## §4 P 项清单

### Critical（必须修复，阻塞合并）

无。

### Important（强烈建议修复）

- **置信度 90｜D5 收口文档同步尚未完成｜`AGENTS.md:34`；要求见 `docs/evidence/t022/dispatch-sheet.md:34`。** 派工单要求在收口段由 Lead 更新 AGENTS.md 的 host 常用命令说明，增加对 challenges 的指引；当前该行仍只描述 BRP 初始方法集和 T018 冒烟入口。建议 Lead 在任务收口前补上挑战判定套件及证据档指引，再将 T022 标记为完成。此项属于文档闭环，不影响已复核代码行为。

### Minor（可选）

无。

## §5 意见

本次代码 diff 满足任务卡四项验收；D1–D6 的代码实现、锚值对照和指定 BRP 抽查均通过。待办是派工单明确分配给 Lead 的收口文档同步，而非挑战实现缺陷。完成 `AGENTS.md:34` 更新后，可转为通过。

---

## Lead 处置记录（转录后回填，2026-10-07）

- 报告归档：本档（审核器最终消息逐字转录）。
- **Important 项闭环**：D5 收口文档同步由 Lead 于收口提交完成——AGENTS.md 常用命令节 host 行追加挑战判定套件（challenges/m5-all）与冒烟入口指引（见本次收口提交 diff）。条件满足，**T022 转通过**。
- 附注（审核 §1 提及的未提交改动 `docs/evidence/t018/brp-smoke-run.log`）：系 T022 门禁复跑 t018 冒烟脚本的运行副产物（脚本按设计重写自写日志，同 T020/T021 先例，t018 档零人工改动），随本次收口提交一并入库保持工作树干净。
