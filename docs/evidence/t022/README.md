# T022 证据档（M5-05 挑战预设 ≥3 数据化，席位 7）

- 日期：2026-10-07（worker-2 执行段；隔离树 t022-a，基线 commit = 0f2a977）
- 任务卡：`taskset/t022-challenges.md`；派工单：`docs/evidence/t022/dispatch-sheet.md`
- 范围：三挑战数据化（构成 + 参数 + 期望终局断言）+ 判定接入 `game.run_tests`
  （`challenges` / `m5-all` 两套件）+ 冒烟与门禁。**不改 sim / host/src/rpc.rs**
  （红线；diff 核验见「实现」节）。

## 索引

| 文件 | 内容 |
|---|---|
| `host/src/challenges.rs`（新） | 挑战注册表 3 条 + `get/names` + 模块头注（三挑战语义/终局语义分类/seed43 注/#2 敏感度注/#1 预算算式） |
| `host/src/suite.rs`（改） | `challenges`（3 断言）+ `m5-all`（12 聚合）注册；缺省 4 / m5-core 9 计数语义不变 |
| `host/src/main.rs`（改） | `mod challenges;` 声明（一行，新模块入树必备） |
| `challenge_smoke.sh` | D6 冒烟脚本（端口 15706 独占） |
| `challenge-smoke-run.log` | 冒烟运行实录（REQ/RESP 原文 + 37 判定行） |
| `t018-smoke-rerun.log` | t018 冒烟 46/46 不回归副本（现场实录） |
| `host-stdout.log` / `host-stderr.log` | challenge 冒烟宿主运行留痕 |
| `design/` | Lead 设计段锚实测档（**来源指针，正文不复制**） |

## 复测终局 vs 派工单 §2 锚对拍（BRP 驱动，seed 42）

| # | challenge | deploy_hash | winner | end_tick | alive red/blue | final_hash | 对拍 |
|---|---|---|---|---|---|---|---|
| 1 | `few-elite` | `0x583ddfef1fc446cf` | red | 2474 | 7/0 | `0xa48ce79b1e83e105` | 六字段全等 |
| 2 | `counter-militia` | `0x6a96df6f3c94e741` | blue | 1800 | 6/24 | `0x3247aae383d5d5bd` | 六字段全等 |
| 3 | `iron-wall` | `0x9a926533cbf1d445` | blue | 1800 | 15/56 | `0x81166458a5437951` | 六字段全等 |

- 双盲条款（B.2 ⑤）未触发：复测与派工单锚任何字段均一致，无改锚/改判定。
- 锚来源指针（文档引用不复制正文）：
  - seed 42：`design/measure-candidates-run.log`（#1 L8/L11、#2 L32/L35、#3 L56/L59）；
  - seed 43 敏感性：`design/measure-seed43-run.log`（L6/L12/L18）；
  - #2 敏感度旁证 ch2b（民兵 20 红胜 `0xb3215593b1fbcde5`）：`design/measure-candidates-run.log:43`。

## 判定行（代码生成，人工只解读）

BRP `game.run_tests` 实测（原文见 `challenge-smoke-run.log`）：

| 套件 | 实测判定行 | 语义 |
|---|---|---|
| `challenges` | `{"failed":0,"passed":3,"total":3}` | 新增：三挑战各一条 `challenge::<name>` 断言（判定行代码生成） |
| `m5-all` | `{"failed":0,"passed":12,"total":12}` | 新增：m5-core 9 + challenges 3 聚合（M5 完整判定面） |
| `m5-core` | `{"failed":0,"passed":9,"total":9}` | 不回归（计数语义零改动） |
| 缺省（`t018-smoke`） | `{"failed":0,"passed":4,"total":4}` | 不回归 |

挑战断言 detail（pass 写实测六字段，fail 写期望 vs 实测全字段）逐条：

- `challenge::few-elite`：`winner red end_tick 2474 alive 7/0 final 0xa48ce79b1e83e105 deploy 0x583ddfef1fc446cf == challenge anchor`
- `challenge::counter-militia`：`winner blue end_tick 1800 alive 6/24 final 0x3247aae383d5d5bd deploy 0x6a96df6f3c94e741 == challenge anchor`
- `challenge::iron-wall`：`winner blue end_tick 1800 alive 15/56 final 0x81166458a5437951 deploy 0x9a926533cbf1d445 == challenge anchor`

## 设计注（要点指针；正文在 `host/src/challenges.rs` 模块头注）

1. **终局语义分类**：#1 = 灭绝收束（end_tick 2474 < max_ticks 3600）；#2/#3 =
   上限 hp 判定（end_tick == max_ticks 1800）。判定面不区分路径、以六字段锚定收束点。
2. **seed 43 设计注（不作断言）**：三构型 seed 43 实测 win/end_tick/存活数与 seed 42
   完全相同，仅哈希不同（#1 `0x51475d4b497f541a`、#2 `0x398a2e9baf9e4d35`、
   #3 `0x356d3b0a9bba9a09`）——挑战叙事跨种子稳健；断言锚只钉 seed 42。
3. **#2 敏感度旁证注**：同构换量 ch2b（民兵 20）实测红胜——数量翻盘点在 20~30 之间
   （hp 池翻转）；#2 取 30 钉蓝胜锚。机制算式（民兵→重骑 9/击×20t、重骑→民兵
   9/击×45t）见模块头注（倍率表 `sim/src/units.rs:190-195`）。
4. **#1 灭绝预算算式（B.2 ②，仅核对）**：3 击×25t = 75t/杀 × 30 杀 = 2250t + 开销
   ≈ 实测 2474 < 3600 ✓。

## 实现（对照派工单 D1~D6）

- **D1/D2**：`ChallengeDef`（name/title_zh/brief_zh/red/blue/lane_len_m/max_ticks/
  seed/expected）+ `ExpectedOutcome`（winner/end_tick/alive_red/alive_blue/final_hash/
  deploy_hash）；常量十六进制字面量 + 来源档行注；seed 42 固定、threads 不入定义
  （判定路径固定串行）。`title_zh`/`brief_zh` 为数据档字段（当前无代码读者，
  `#[allow(dead_code)]` 结构性豁免，沿 `GameConfig` 先例）。
- **D3**：`suite.rs` 注册 `challenges`（3）+ `m5-all`（12 聚合），`names()` 增至 4；
  缺省套件 4 / `m5-core` 9 不动；`rpc.rs` 零改动（`run_tests` handler 只委托
  suite 模块）。
- **D4**：每挑战 = 全新 `World` → `deploy_versus`（seed/构成/lane 取自注册表）→
  比对 `deploy_hash` → `run_battle_with(max_ticks, None)` → 比对终局五字段；
  期望值只在 `challenges.rs` 常量（判定代码无第二份拷贝）；fail detail 含期望 vs
  实测全字段。
- **D5**：模块头注含三挑战语义 + 终局语义分类 + seed43 注 + #2 敏感度注 + #1 预算
  算式；AGENTS.md 收口句由 Lead 回写（worker 不动）。`main.rs` 仅加 `mod challenges;`
  一行（新模块必需）。
- **D6**：`challenge_smoke.sh`（15706）；断言集 37 条 = 端口就绪 1 + discover 面
  run_tests 不变 1 + 三挑战 ×6 字段对拍 18 + challenges 套件 7（名/total/passed/
  failed + 三断言名）+ m5-all 4 + m5-core 2 + 缺省 3 + 宿主存活 1。
- **红线核验**：`diff -rq` 树内 `sim/` vs 主仓 `sim/` → 零差异；`host/src/rpc.rs`、
  `host/src/presets.rs` vs 主仓 → 零差异（git 层面核验留收获段）。

## 门禁记录（2026-10-07，树根执行；逐条单独整句；`CARGO_TARGET_DIR` 共享主仓 target）

| # | 命令 | 结果 |
|---|---|---|
| 0a | 进程门 `(Get-Process rustc,cargo \| Measure-Object).Count` | `0`（每次 cargo 前复验） |
| 0b | 内存门（CommitFree） | check/test 前 `11.7~11.8G`（≥11G ✓）；release 前 `11.8 → 11.7G`（<13G，见下） |
| 1 | `cargo check --workspace -j 3` | `Finished dev profile … in 1.05s`，EXIT=0，**0 警告** |
| 2 | `cargo test -p sim -j 3` | EXIT=0：sim lib **44 passed / 0 failed** + arena 3 + bench/main/doc 0——全绿 |
| 3 | `cargo build -p host --release -j 3` | `Finished release profile … in 7.48s`，EXIT=0；产物 host.exe 拷贝至树根 `target/release/`（运行件，非入库） |
| 4 | `bash docs/evidence/t018/brp_smoke.sh`（15702，跑前确认空闲） | **PASS=46 FAIL=0，SCRIPT_EXIT=0**（副本 `t018-smoke-rerun.log`） |
| 5 | `bash docs/evidence/t022/challenge_smoke.sh`（15706） | **PASS=37 FAIL=0，SCRIPT_EXIT=0** |

- **增量足迹授权留痕（门禁 3）**：release 段 13G 门为冷建口径；实测 CommitFree
  11.8G（sleep 120 复测 11.7G）不足 → 上报等待。Lead 裁决 B（附条件增量放行）：
  host release 全依赖树 20 分钟前刚由 Lead 构建（增量 5.26s），本单改动只触
  `host/src` ⇒ 实际足迹 ≈1~2G、秒级。**「增量足迹授权——13G 门按冷建口径、
  Lead 复核增量事实后放行」**。执行条件逐条照办：① 跑前即时复验进程门 = 0；
  ② 起跑后无「在编 bevy 依赖树 / 超 90s」迹象（实测 7.48s、仅编 sim+host 两个本地
  crate，且 90s 硬超时护栏未触发）；③ 完成后照单续跑两项冒烟。
- 冒烟端口纪律：只用 15702（t018 不回归）/ 15706（challenge）；跑完无 host 进程残留。

## 运行副产物说明

- `t018/brp-smoke-run.log` 及 `t018/host-{stdout,stderr}.log` 由 t018 脚本按设计重写
  （运行副产物，t018 档内文件零人工改动——同 T020/T021 先例）；现场实录已复副本
  `t018-smoke-rerun.log` 于本档自含。
- 本档 `design/` 与 `dispatch-sheet.md` 全程零写入。
- `target/` 为运行件目录（`.gitignore` 含 `/target`），不入库。
