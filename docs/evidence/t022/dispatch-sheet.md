# T022 派工单（M5-05 挑战预设 ≥3 数据化，席位 7）——Lead 设计段已定，worker 执行

> 任务卡：`taskset/t022-challenges.md`。基线 = 派发时 master HEAD（建树 commit 见工单下发记录）。
> 附录引用：`C:\Users\Administrator\Desktop\ccc\wanzhen\team-prompt\PROJECT-APPENDIX.md` 附录 A（门禁/内存纪律）、B.1（误报清单）、B.2（数值三查+双盲）、D（隔离树）——只读，禁转写。

## 0. 结论先行（本单可执行边界）

三挑战**构型/参数/期望终局锚已由 Lead 实测定死**（§2 表，实测档 `docs/evidence/t022/design/measure-candidates-run.log` 与 `measure-seed43-run.log`，仓库相对路径、REQ/RESP 原文在档）。worker 做：① 落数据（challenges.rs 注册表）② 接入 run_tests 判定面（§3 D3）③ 冒烟 + 证据档 + 门禁。**期望值不一致即停手上报（B.2 ⑤ 双盲）——你复测出来的终局与 §2 锚任何字段不符，不得改锚、不得改判定，上报 Lead。**

## 1. 范围（任务卡逐字重申）

- ≥3 挑战数据化三件套：构成 + 参数 + 期望终局断言（数据档）。
- 挑战判定接入 `game.run_tests` 面；判定行代码生成（禁手算）。
- 范围外：挑战编辑/排行榜/成就、新模拟机制、难度调优迭代。

## 2. 三挑战定选表（锚值 = Lead 2026-10-07 实测，seed 42）

| # | name（注册表 id） | 题名 | 构成 red / blue | lane_len_m | max_ticks | seed | 期望终局（winner/end_tick/alive_red/alive_blue） | final_hash | deploy_hash |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `few-elite` | 以少胜多·剑士绞肉 | swordsman:10 / militia:30 | 60 | 3600 | 42 | red / 2474 / 7 / 0（**灭绝收束**） | 0xa48ce79b1e83e105 | 0x583ddfef1fc446cf |
| 2 | `counter-militia` | 无甲克重甲·农兵拒骑 | heavyknight:10 / militia:30 | 60 | 1800 | 42 | blue / 1800 / 6 / 24（**上限 hp 判定**） | 0x3247aae383d5d5bd | 0x6a96df6f3c94e741 |
| 3 | `iron-wall` | 铁壁僵局·盾墙不死 | shieldman:20 / militia:60 | 60 | 1800 | 42 | blue / 1800 / 15 / 56（**上限判定**，= T021 last-stand 实测 15/56 转正） | 0x81166458a5437951 | 0x9a926533cbf1d445 |

- **来源行**：#1 = measure-candidates-run.log REQ#1/RESP#1（deploy）+ 同段 outcome；#2/#3 同档对应段；seed 43 敏感性 = measure-seed43-run.log。
- **seed 43 设计注（入 challenges.rs 文档注 + 证据 README，不作断言）**：三构型 seed 43 实测 win/end_tick/存活数与 seed 42 **完全相同**（仅哈希不同：#1 0x51475d4b497f541a、#2 0x398a2e9baf9e4d35、#3 0x356d3b0a9bba9a09）——挑战叙事跨种子稳健；断言锚只钉 seed 42（挑战语义 = 固定种子逐位复现）。
- **B.2 ② 接敌时间预算算式（#1，以实测为准）**：贴身接敌面恒 1v1 漏斗（T021 判定注，sim/src/world.rs:342/733）。剑士→民兵 dmg = 12 × 1.5 = 18/击（克制表 units.rs COUNTER_TABLE，Light→Unarmored）；民兵 50hp ⇒ ⌈50/18⌉=3 击 × 攻击间隔 25t = 75t/杀。30 杀 × 75t = 2250t + 队列推进/接敌开销 ≈ 实测 2474 < max_ticks 3600 ✓（算式仅供核对）。
- **#2 机制注**：民兵→重骑 6 × 1.5 = 9/击 × 间隔 20t；重骑→民兵 14 × 0.67 = 9/击 × 间隔 45t（无甲克重甲 ×1.5 / 重甲攻无甲被克 ×0.67，units.rs:190-195）；判定敏感度旁证 = ch2b（同构民兵 20）实测红胜（hp 池翻转）——**此对拍入 README 设计注**，警示「数量翻盘点在 20~30 之间」。

## 3. 设计裁决（D1~D6，照单执行）

- **D1 数据结构**：新文件 `host/src/challenges.rs`（沿 presets.rs 体例）：`ChallengeDef { name, title_zh, brief_zh, red: Vec<(UnitKind, usize)>, blue: …, lane_len_m: i64, max_ticks: u64, seed: u64, expected: ExpectedOutcome }`；`ExpectedOutcome { winner: &str（"red"/"blue"——与 BRP outcome 的 winner.label() 字符串对拍）, end_tick: u64, alive_red: u32, alive_blue: u32, final_hash: u64, deploy_hash: u64 }`。常量全部**整数算式/十六进制字面量直书**并注释来源档行。注册表 3 条 + `get(name)` / `names()`。
- **D2 seed 固定**：挑战定义内含 seed 42（挑战语义 = 固定种子逐位复现，任务卡断言 3）；threads 不入挑战定义（判定路径固定串行 = suite 现状）。
- **D3 判定面接入（零回归设计）**：`suite.rs` 注册表新增两套件——`"challenges"`（3 断言：每挑战一条）+ `"m5-all"`（聚合 = m5-core 9 + challenges 3，共 12，**M5 完整判定面**，T025 三合一判定用）。**缺省套件不动**（t018-smoke 4）、m5-core 9 不动、rpc.rs 零改动（run_tests handler 已委托 suite 模块）。`names()` 增至 4。
- **D4 挑战断言实现（判定行代码生成）**：每挑战 = 全新 `World`（净副作用纪律沿 suite 现状）→ `deploy_versus`（同参三入口一致性已由 T021 构造保证）→ 比对 `deploy_hash` → `run_battle_with(max_ticks)` → 比对 winner/end_tick/alive_red/alive_blue/final_hash 六字段。断言名 = `challenge::<name>`（如 `challenge::few-elite`）；fail 时 detail 含期望 vs 实测全字段（禁只报 bool）。**期望值只在 challenges.rs 常量，判定代码不含第二份拷贝。**
- **D5 文档同步**：challenges.rs 模块头注 = 三挑战语义 + 终局语义分类（灭绝/上限 hp 判定）+ seed 43 设计注 + #2 敏感度旁证注；AGENTS.md 常用命令节 host 行追 challenges 一句（收口段 Lead 回写，worker 不动 AGENTS.md）。
- **D6 证据档**：`docs/evidence/t022/`——`challenge_smoke.sh`（BRP 端口 **15706**，独占）+ 运行 log（REQ/RESP 原文）+ `README.md`（索引/判定行/设计注/锚来源指针——design/ 档引用不复制正文）。冒烟断言集：三挑战逐条 BRP 驱动对拍（game.deploy 挑战参数 → game.outcome → 六字段 vs 数据档锚）+ `run_tests {"suite":"challenges"}` total 3/pass 3 + `{"suite":"m5-all"}` total 12/pass 12 + `{"suite":"m5-core"}` total 9（不回归）+ 缺省套件 total 4（不回归）+ `rpc.discover` 面 `run_tests` 不变。

## 4. 门禁（全绿才算完）

1. `cargo check --workspace -j 3` = 0 警告（ZCode Bash 10 分钟限内，超时拆步重跑取证）。
2. `cargo test -p sim -j 3` = 0（sim 零改动红线；若被迫改 sim → 停手上报）。
3. `cargo build -p host --release -j 3` = 0（host 无渲染依赖，足迹轻；构建前跑附录 A 预检）。
4. t018 冒烟不回归：`bash docs/evidence/t018/brp_smoke.sh` 46/46（端口 15702 空闲时）。
5. `bash docs/evidence/t022/challenge_smoke.sh` 全 PASS（判定行代码生成，人工只解读）。
6. host/src/rpc.rs 与 sim/ 零改动（git 层面核验留给收获段）。

## 5. 上报纪律

- 复测终局与 §2 锚不符 → 停手上报（B.2 ⑤）。
- 同一异常排查 ≤2 次未定位 → 上报附命令与原始报错（B.1）。
- 一切范围外决策（改锚/改构型/加挑战/动 sim）→ 上报选项不拍板。
