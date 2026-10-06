# T020 断言清单档（断言面与测试集全量，席位 3）

- 派工单：`docs/evidence/t020/dispatch-sheet.md`（Lead，2026-10-06；本档只引用不复制正文）。
- 判定主体：`game.run_tests`（suite=`m5-core`，进程内 9 断言）+ BRP 矩阵/错误路径脚本判定行；
  **agent 只作驱动，不作判定**（判定由脚本 `check()` 对原始响应逐条计算，人工只解读——沿席位 3 口径）。
- 本档所有「实测」列 = 2026-10-06 门禁跑（脚本原始输出在档；`replay-matrix-run.log` /
  `error-paths-run.log` / `m5core-suite-run.log`）；**2026-10-07 完整轮整改复跑**：
  m5-core 19/19 + 错误路径 30/30（整改后计数，见 §1/§3 整改注）。

## 0. 复跑入口（树根执行；端口 15714；机器基本空闲，非帧敏量测）

| 入口 | 命令 | 覆盖 |
|---|---|---|
| m5-core 套件 | `bash docs/evidence/t020/m5core_suite.sh` | 9 断言（含锚①/②、短 lane 战斗锚、seed43 伴随锚） |
| 重放矩阵 | `bash docs/evidence/t020/replay_matrix.sh` | 七配置 A~G、25 判定行 |
| 错误路径 | `bash docs/evidence/t020/error_paths.sh` | 18 发、30 判定行（含 P1-2 空构成整改段） |
| 缺省套件回归 | `bash docs/evidence/t018/brp_smoke.sh` | 46 判定行（T018 原绿 = 缺省套件不变锚） |

套件的 BRP 直调等价复跑（单发）：

```bash
curl -s -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"game.run_tests","params":{"suite":"m5-core"}}' \
  http://127.0.0.1:15714/
```

## 1. m5-core 九断言（`m5core_suite.sh`，19/19 CHK PASS，SCRIPT_EXIT=0）

单条复跑 = 运行套件脚本并核对应 `CHK-05-pass-<断言名>` 行（detail 原文见 `m5core-suite-run.log` RESP）。

| # | 断言名 | 期望 | 实测 | 判定 |
|---|---|---|---|---|
| 1 | `golden_units0_seed42_1800` | `0xd3b6408fd46c2008`（锚①，T002） | `last_hash 0xd3b6408fd46c2008 == golden` | PASS |
| 2 | `golden_default_comp_seed42_1800` | `0x958c5938c8682529`（锚②，T004） | `last_hash 0x958c5938c8682529 == golden` | PASS |
| 3 | `replay_pairwise_checkpoints` | tick 0/900/1800 两 World 逐位一致 | `tick 0/900/1800 pairwise identical; final 0x958c5938c8682529` | PASS |
| 4 | `deploy_versus_mirror_equivalence` | 镜像 `deploy_versus` == `deploy` tick0 | `tick 0 last_hash identical: 0xe2706f0b91a2be8e`（与 T021 布阵锚同值，跨卡旁证） | PASS |
| 5 | `cross_thread_pool_equivalence_900t` | t12 池 vs 串行 900t 逐位一致 | `t12 pool vs serial identical @tick 900; last_hash 0x7fbb5bd7e46ef448` | PASS |
| 6 | `battle_golden_shortlane_seed42` | red/878/1/0 且 `0xfdbc4995554ee691`（T018 CHK-16 锚） | `red/878/1/0 final_hash 0xfdbc4995554ee691 == golden` | PASS |
| 7 | `seed_sensitivity_42_43` | 42≠43 且 seed43 == `0x54611ed6ded02540`（T004） | `seed42 0x958c5938c8682529 != seed43 0x54611ed6ded02540` | PASS |
| 8 | `outcome_freeze_idempotence_shortlane` | 收束幂等 + 冻结缓存不被直接 run 重算 + **run(100) tick 前移恰 +100（P1-1 整改入 pass 条件）** | `run_battle_with x3 identical (end_tick 878 …), tick stays 878; after run(100): world.tick 878 -> 978 (expect 978 = frozen+100; 纯原语), frozen cache unchanged (end_tick 878)` | PASS（语义分解见 §5） |
| 9 | `outcome_overrun_semantics` | 越过上限：draw / end_tick 2000 / final == run(2000) 后 hash（P2-2） | `draw@2000 final_hash 0xa63d948723f692d1 == run(2000) last_hash; alive 30/30` | PASS |

- 断言 1~4 = 冒烟集原函数复用（同一实现，T020/D1 字面；未复制粘贴）。
- **PIT-M-002 留痕（断言 6/7）**：占位 0 首测 → 实测值与派工单锚逐位核对一致 → 回填 →
  复跑全绿。首测原始档 `m5core-suite-measure.log`（当时 passed=7 / failed=2，detail 内
  `got winner red end_tick 878 alive 1/0 final_hash 0xfdbc4995554ee691` 与
  `seed43 0x54611ed6ded02540`）。

## 2. 重放矩阵七配置（`replay_matrix.sh`，25/25 CHK PASS，SCRIPT_EXIT=0）

每配置 = 完整 deploy→run→采 hash（E/G 另采 outcome）序列整段重放两遍逐位对拍；
观测值逐配置落 `replay-matrix.jsonl`。

| 配置 | deploy 参数 | K | 期望 | 实测 | 判定 |
|---|---|---|---|---|---|
| A | `{"preset":"default","seed":42,"threads":1}` | 1800 | 两遍一致 == 锚② | run1=run2=`0x958c5938c8682529`，tick 1800 | PASS |
| B | 同上 `threads:12` | 1800 | == 锚②（跨线程） | run1=run2=`0x958c5938c8682529` | PASS |
| C | `{"preset":"melee-brawl","seed":7,"threads":1}` | 1800 | 记录 H_C；两遍一致 | H_C=`0xb82a248ff23515e2`，tick 1800 | PASS |
| D | 同上 `threads:12` | 1800 | == H_C（跨线程） | `0xb82a248ff23515e2` | PASS |
| E | `{"seed":42,"red":[militia×5],"blue":[militia×5],"lane_len_m":10,"threads":1}` | 1800 | 冻结@878 + final `0xfdbc4995554ee691` + outcome red/878/1/0 | tick 878；hash 与 outcome final_hash 皆 `0xfdbc4995554ee691`；red/878/1/0 | PASS |
| F | `{"seed":2026,"red":[archer×8,militia×2],"blue":[shieldman×6,pikeman×4],"lane_len_m":100,"threads":3}` | 1800 | 记录 H_F；两遍一致 | H_F=`0x185fe8c22adeec36`，tick 1800；接敌旁证 outcome alive 2/9 | PASS |
| G | `{"preset":"default","seed":42,"threads":1}` | **2000** | tick 2000；draw/end_tick 2000/final == run 后 hash（P2-2） | tick 2000；draw/2000；final=`0xa63d948723f692d1`==run hash | PASS |

- 判定行明细（25）：`CHK-00-port-ready` + A/B/C/D 各 2（replay-bitexact + 锚命中）+ E 7
  （replay/tick878/hash/outcome-red/endtick878/alive-1-0/finalhash-anchor）+ F 3
  （replay/new-anchor/contact-losses）+ G 5（replay/tick2000/draw/endtick2000/finalhash-eq）
  + `CHK-99-process-alive`。
- **F 的 lane_len_m=100 选定算式（附录 B.2 ②，带 sim 源行号）**：最慢闭合对
  archer 0.08 m/tick（`sim/src/units.rs:129`）+ shieldman 0.05（`units.rs:85`）
  ⇒ 合闭合 0.13；射程阈值 = 0.4（`units.rs:130`）+0.5（`units.rs:86`）+0.2
  （`MELEE_MARGIN_Q32`，`units.rs:34`）= 1.1 m；队头初始距离 = 100 − (0.4+0.5)
  （`sim/src/world.rs:601-622`）⇒ (99.1 − 1.1) ÷ 0.13 ≈ 753.85 → **≤ 754 tick ≤ 1500 ✓**
  （最坏界口径；算式全文见 `replay_matrix.sh` 头注）。实测旁证：1800t 后双方战损
  （alive 2/9）⇒ 接敌真发生。
- **H_C 跨卡旁证**：C/D 实测 `0xb82a248ff23515e2` 与 T021 预设冒烟同参实测
  （`docs/evidence/t021/preset-smoke-run.log:16`）逐位一致。
- PIT-M-002 留痕（C/F 新记录值）：占位首测 `replay-matrix-measure.log` /
  `replay-matrix-measure.jsonl`（首测 C=`0xb82a248ff23515e2`、F=`0x185fe8c22adeec36`，
  回填后本表即复跑全绿值）。

## 3. 错误路径（`error_paths.sh`，30/30 CHK PASS，SCRIPT_EXIT=0——2026-10-07 整改版）

18 发：前 14 发 = 非法参数/状态错误（每发断言结构化 `error.code`）+ P1-2 整改段
CHK-17/18（空构成**接受现实**用例，见下）；尾部进程存活；复跑命令见 §0。

| # | 发次（params） | 期望码 | 实测码 + 消息（摘要） | 判定 |
|---|---|---|---|---|
| 1 | `game.state_hash {}`（未 deploy） | 4001 | 4001 `no deployed game` | PASS |
| 2 | deploy seed 传字符串 `"42"` | -32602 | -32602 `missing/invalid seed (u64 required)` | PASS |
| 3 | deploy 未知 kind `laser` | -32602 | -32602 `unknown kind "laser" (valid: …)` | PASS |
| 4 | deploy count 负数 `-1` | -32602 | -32602 `count must be a non-negative integer` | PASS |
| 5 | deploy 单方 100001（>100,000） | -32602 | -32602 `red totals 100001 units, exceeding per-side cap 100000` | PASS |
| 6 | deploy `threads:0` | -32602 | -32602 `invalid threads (1..=1024 required)` | PASS |
| 7 | deploy `threads:1025` | -32602 | -32602 同上 | PASS |
| 8 | deploy `lane_len_m:0` | -32602 | -32602 `invalid lane_len_m (integer >= 1 required)` | PASS |
| 9 | deploy `max_ticks:0` | -32602 | -32602 `invalid max_ticks (1..=14400 required)` | PASS |
| 10 | deploy `max_ticks:14401` | -32602 | -32602 同上 | PASS |
| 11 | `game.run_to_tick {}`（缺 ticks） | -32602 | -32602 `missing/invalid ticks (u64 required)` | PASS |
| 12 | deploy `preset:"nope"` | -32602 | -32602 `unknown preset "nope" (available: default, melee-brawl, last-stand)` | PASS |
| 13 | `game.run_tests {"suite":"no-such-suite"}` | -32602 | -32602 `unknown suite … (available: t018-smoke, m5-core)` | PASS |
| 14 | `game.screenshot {}`（桩） | 4101 | 4101 + `data.planned_task:"T019"` | PASS |

- 发 3/5/11/12 另核消息内容（`CHK-03b/05b/11b/12b`）；**CHK-15 复证**：前 14 发后
  `state_hash` 仍 4001 ⇒ 无一次非法 deploy 静默落地；CHK-16 进程存活（无 panic / 无击穿）。

**P1-2 整改段（CHK-17/18，2026-10-07）——空构成独立用例（接受现实口径）**：

| # | 发次（params） | 期望（实测裁决） | 实测 | 判定 |
|---|---|---|---|---|
| 17 | deploy `red:[]` vs `blue:[militia×5]` lane60 mt100 | **合法布阵**：alive_red 0 / units 5 / hash `0xf5f1b5a28609daf0`；outcome 立即灭绝冻结 blue@0、final==deploy | 全部命中（`CHK-17/17b/17c/17d`） | PASS |
| 18 | deploy `red:[] blue:[]` 同参 | **合法空局**：units 0 / hash `0xa6c66902e818cd64`；outcome draw@0、final==deploy | 全部命中（`CHK-18/18b/18c/18d`） | PASS |

- 裁决依据：与 sim 层空局语义一致（T002 黄金锚① = 空局先例）；缺字段（键缺失）才是
  -32602（发 1 同族）。任务卡口径修正披露见 `taskset/t020-assertion-suite.md` 范围内节；
  审核出处 `review-plan-code-reviewer.md` P1-2。

## 4. 锚值清单与源指针

| 锚 | 值 | 源 |
|---|---|---|
| 锚① units0 seed42 1800t | `0xd3b6408fd46c2008` | `sim/src/world.rs:1065`（M0 T002 归档） |
| 锚② 默认构成 seed42 1800t | `0x958c5938c8682529` | `sim/src/world.rs:1279`（M0 T004 归档） |
| seed43 伴随值 | `0x54611ed6ded02540` | `docs/evidence/t004/README.md:90` |
| 短 lane 战斗锚（灭绝@878 red 1/0） | `0xfdbc4995554ee691` | `docs/evidence/t018/brp-smoke-run.log:90-99`（CHK-16） |
| T021 布阵锚（tick0 默认构成） | `0xe2706f0b91a2be8e` | `docs/evidence/t021/preset-smoke-run.log`（断言 4 旁证命中） |
| H_C melee-brawl seed7 1800t | `0xb82a248ff23515e2` | 本卡实测（与 t021 同参实测同值） |
| H_F 混编 seed2026 lane100 t3 1800t | `0x185fe8c22adeec36` | 本卡新锚（PIT-M-002 占位→实测→回填） |
| 跨线程 900t（默认构成） | `0x7fbb5bd7e46ef448` | 本卡实测（断言 5 detail） |
| G 越限 2000t（默认构成） | `0xa63d948723f692d1` | 本卡实测（断言 9 / 矩阵 G） |
| 空红侧布阵 tick0（militia5 对侧） | `0xf5f1b5a28609daf0` | 本卡实测（P1-2 整改 CHK-17，== final_hash 灭绝冻结@0） |
| 双侧空布阵 tick0 | `0xa6c66902e818cd64` | 本卡实测（P1-2 整改 CHK-18，== final_hash draw@0） |

## 5. 语义注（D1-8「冻结语义」的实证口径）

sim 语义边界（`sim/src/world.rs:32-34` 模块注 + `:890-897` / `:913-940` 实现）：
`run_battle_with` 收束后写入缓存并**幂等冻结**（再入不推进 tick、返回同四元组）；
`run/step` 为**纯原语**，`resolved` 不拦其推进——`world.tick` 会前进，但冻结缓存不被
重算/清除。断言 8 据此实证两件事：① 收束路径三次重入（含更小 max_ticks=100）四元组
全等且 tick 恒 878；② 直接 `run(100)` 后 world.tick 878→978（如实记录），`outcome()`
仍返回冻结于 end_tick=878 的原四元组。派工单原文「再 `run(100)` tick 不动」的
字面读法（world.tick 不前进）与模拟实现矛盾——分解口径与上报见 `README.md` 上报节。

## 6. 判定机制（agent 只作驱动）

- 套件：9 条 `AssertionResult{name, pass, detail}` 由 `host/src/suite.rs` 进程内计算，
  BRP 返回 `total/passed/failed/results`；脚本逐断言名核 `"pass":true` 与锚 detail。
- 矩阵/错误路径：脚本 `check()` 对 curl 原始响应做 `grep` 结构化判定（`SUMMARY: PASS/FAIL`
  + `SCRIPT_EXIT`）；REQ/RESP 原文逐条落档（`>&2` 体例沿 t018 整改版）。
