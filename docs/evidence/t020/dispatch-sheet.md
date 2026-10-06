# T020 派工单（worker 实现段）— 断言面与测试集全量（席位 3）

- 发单：Lead，2026-10-06。依据：taskset/t020-assertion-suite.md + 备忘录 §三质量红线/§五判定口径 + T018 P2-2 移交项。
- 你只做「实现清单」；门禁你跑；**不 commit、不碰主仓**（树内工作）。
- **host 改动面 = `host/src/suite.rs` 一个文件**（预备重构后套件注册单一来源已就位：`run`/`names`/`DEFAULT_SUITE`；rpc.rs 禁改）；其余全部是 `docs/evidence/t020/` 新档。sim/ 零改动。

## 已核实事实（直接使用）

1. 套件注册形态（本树基线）：`host/src/suite.rs` 的 `pub fn run(name) -> Option<Vec<AssertionResult>>` / `pub fn names() -> Vec<&'static str>` / `DEFAULT_SUITE`（= "t018-smoke"，**缺省套件保持不变**——T018 冒烟回归门禁的稳定性锚）。
2. sim 公共 API：`World::{new, deploy, deploy_versus}` / `run_with(ticks, Option<&ThreadPool>)` / `run_battle_with(max_ticks, pool) -> BattleOutcome`（幂等冻结）/ `outcome()`；`BattleOutcome { winner, end_tick, alive_red, alive_blue, final_hash }`；`Winner::label()`；`pool::ThreadPool::new(n)`；`units::{UnitKind, ONE_Q32_32}`；`world::{DEFAULT_COMPOSITION, LANE_LEN_Q32, TICK_CAP_REDUCED}`。
3. 黄金锚（逐字使用勿换算）：
   - 锚① `0xd3b6408fd46c2008`（T002：seed42 空世界 1800t）
   - 锚② `0x958c5938c8682529`（T004：seed42 默认构成 1800t）
   - seed43 伴随值 `0x54611ed6ded02540`（T004 归档，附录 C 保真锚清单）
   - 短 lane 战斗锚（T018 BRP 冒烟 CHK-16 实测归档）：seed42 民兵 5v5 lane 10m → 灭绝@tick878、red 胜、alive 1/0、final_hash `0xfdbc4995554ee691`
   - T021 布阵锚：seed42 默认构成（含 preset 路径）tick0 = `0xe2706f0b91a2be8e`
4. BRP 形态：JSON-RPC over HTTP POST；域错误码 4001/4002/4101、参数错误 -32602；`game.deploy` 支持 `"preset"`（default/melee-brawl/last-stand）+ 显式覆盖。

## 实现清单

### D1 新套件 `m5-core`（全量，进程内、全新 World、不碰 HostedGame）

`suite.rs` 注册 `"m5-core"`（`run`/`names` 同步；`DEFAULT_SUITE` 不变）。断言集 9 条（前 4 条 = 冒烟集原函数复用，勿复制粘贴）：
1. `golden_units0_seed42_1800`（锚①）
2. `golden_default_comp_seed42_1800`（锚②）
3. `replay_pairwise_checkpoints`
4. `deploy_versus_mirror_equivalence`
5. `cross_thread_pool_equivalence_900t`：两个独立 `deploy(42, DEFAULT)`，a `run_with(900, Some(&ThreadPool::new(12)))` vs b `run(900)` → last_hash 逐位一致（跨线程纪律进程内复证）。
6. `battle_golden_shortlane_seed42`：`deploy_versus(42, [(Militia,5)], [(Militia,5)], 10*ONE_Q32_32)` → `run_battle_with(TICK_CAP_REDUCED, None)` → 四元组 `winner=="red" && end_tick==878 && alive_red==1 && alive_blue==0` 且 final_hash == `0xfdbc4995554ee691`。**PIT-M-002 纪律：先占位 0 跑一次实测，与锚不符立即停手上报（BRP 路径与进程内直跑的语义分歧属 P0 级发现，不得静默改锚）**。
7. `seed_sensitivity_42_43`：deploy 默认构成 seed42/seed43 各 run(1800) → 哈希不同；detail 双 hex 记录（43 侧应 == `0x54611ed6ded02540`，同样先实测后锚定，不符上报）。
8. `outcome_freeze_idempotence_shortlane`：断言 6 同构，`run_battle_with` 连调两次四元组全等；再 `run(100)` tick 不动（冻结语义）。
9. `outcome_overrun_semantics`（**T018 P2-2 移交项钉死**）：`deploy(42, DEFAULT)` → `run(2000)`（越过 max_ticks 默认 1800，对称未接敌）→ `run_battle_with(1800, None)` → `winner=="draw" && end_tick==2000`、final_hash == run(2000) 后的 last_hash。语义留痕（写入断言 detail 或注释）：outcome 判定时点 = 收束点（灭绝）或**首次求值时点**（求值前 tick 已越过 max_ticks 时上限 hp 判定落在当前 tick）——调用序影响判定，故为本套件锚定对象。

### D2 BRP 参数序列重放矩阵（`docs/evidence/t020/replay_matrix.sh`）

起 host（`--port 15714`）。矩阵七配置（每配置 = 完整 deploy→run→采hash 序列**整段重放两遍**，逐位对拍；跨线程对也在内）：

| # | 配置（deploy 参数） | K | 判定 |
|---|---|---|---|
| A | `{"preset":"default","seed":42,"threads":1}` | 1800 | state_hash == 锚② |
| B | `{"preset":"default","seed":42,"threads":12}` | 1800 | == 锚②（跨线程） |
| C | `{"preset":"melee-brawl","seed":7,"threads":1}` | 1800 | 记录 H_C；重放一致 |
| D | `{"preset":"melee-brawl","seed":7,"threads":12}` | 1800 | == H_C（跨线程） |
| E | `{"seed":42,"red":[militia×5],"blue":[militia×5],"lane_len_m":10,"threads":1}` | 1800 | 冻结@878 + final_hash `0xfdbc4995554ee691` + outcome red/878/1/0 |
| F | `{"seed":2026,"red":[archer×8,militia×2],"blue":[shieldman×6,pikeman×4],"threads":3}` | 1800 | 记录 H_F；重放一致；**lane_len_m 由你按 B.2 ② 算式选定**：最慢闭合对的 (队头距离−阈值)÷合闭合 ≤ 1500 tick（算式与 spec 行号写进脚本注释） |
| G | `{"preset":"default","seed":42,"threads":1}` | **2000** | tick==2000；outcome draw/end_tick 2000/final_hash==run 后 hash（P2-2 BRP 侧钉死） |

输出：`replay-matrix.jsonl`（逐配置记录）+ `replay-matrix-run.log`（REQ/RESP `>&2` 落档体例沿 t018 整改版）+ SUMMARY/SCRIPT_EXIT。F 的 H_F 属**新锚实测产出**（占位→实测→回填→复跑全绿，PIT-M-002），值写进脚本常量。

### D3 错误路径集（`docs/evidence/t020/error_paths.sh`，同 port）

≥10 发：seed 传字符串 / 未知 kind / count 负数 / 单方超 100,000 / threads 0 与 1025 / lane_len_m 0 / max_ticks 0 与 14401 / 缺 ticks / 未 deploy 先 state_hash / 未知 preset / 未知 suite / screenshot 桩 4101。每发断言结构化错误码 + 尾部进程存活。同样 REQ/RESP 落档。

### D4 断言清单档（`docs/evidence/t020/assertions.md` + `README.md`）

assertions.md 表：每条断言（套件 9 + 矩阵 7 + 错误路径 N）——名称 / 复跑命令 / 期望 / 实测 / 判定。README：对账表（任务卡验收断言 1~5 逐条）+ 文件清单 + 门禁记录。

## 门禁（你跑）

1. `cargo check --workspace -j 2` → 0 警告。
2. `cargo build -p host --release -j 2` → 成功。
3. `bash docs/evidence/t018/brp_smoke.sh`（树根，默认端口）→ **46/46 PASS SCRIPT_EXIT=0**（缺省套件不变 = 回归原绿）。
4. `bash docs/evidence/t020/replay_matrix.sh` → 全判定 PASS、SCRIPT_EXIT=0。
5. `bash docs/evidence/t020/error_paths.sh` → 全 PASS、SCRIPT_EXIT=0。
6. BRP 侧 `game.run_tests {"suite":"m5-core"}` → `total:9 passed:9 failed:0`（在矩阵脚本或单独小脚本里判定均可）。

## 红线

- suite.rs 外的 host 文件、sim/、docs/evidence/t018|021/、taskset/、台账零改动；不碰主仓；不 commit。
- 锚值字面量逐字；实测不符 = 停手上报（P0 级）；F 的 lane 选定算式入注释。
- 附录 B.2 检查单过一遍；行号引用先 grep -n 核对。
