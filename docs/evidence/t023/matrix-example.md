# T023 批量采样示例矩阵（game.sample_outcomes，900 局）

- 生成：`matrix_example.sh`（本档判定行由脚本计算生成，人工只解读不计算；勿手改）。
- 基线：隔离树 t023-a（基线 commit bb3f4ba）+ `game.sample_outcomes`（本卡新增，M5-06 席位 8）。
- 宿主：`target/release/host.exe`（release 构建，BRP 端口 15707 回环独占）；批跑耗时 wall_ms=26651（900 局，threads=1）。
- 参数：red/blue 各为单兵种 ×100（每方 100 共 200 单位/局——表 6-0 降规模对局字面）；
  `lane_len_m=50`（沿 M0 T009 矩阵缩比道口径）；`max_ticks` 缺省（1800=`TICK_CAP_REDUCED`）；
  `games=100`/格；`seed_base=42`（每格种子 42..141）；`threads=1`。

## 口径注（D4 逐字，残余账随卡）

① 降规模口径：每方 100 共 200 单位/局、单局 ≤1800 ticks（表 6-0）——不得与全规模数据混用（残余账 #11）

② 样本量注：本档每格 100 局（<400 场）——趋势指示、非基准（判据 ±10pp / ≥400 场/周，R5 功效注——残余账 #12）

③ 灰盒指标不作外推依据（残余账 #9）——本卡不适用：统计面无渲染指标，如实标注

## 接敌可达性算式（附录 B.2 ②；仅核对，以算式为准）

最慢闭合对（盾兵×盾兵）：(50 − 0.5 − 0.5 − (0.5+0.5+0.2)) ÷ (0.05+0.05)
= (49.0 − 1.2) ÷ 0.10 = 478 ≤ 1800 tick ✓ —— 全部 9 格任意 seed 接敌可达
（半径/移速源 sim/src/units.rs；队头布阵式 sim/src/world.rs:601-622；
单兵种阵列洗牌不改变队头兵种——任何 seed 队头半径恒定）。

## 胜率矩阵（红方视角；判定行由脚本从响应计算）

格子格式：`win_rate_red_pp`（红胜/蓝胜/平）——pp ÷ 100 = 红方胜率百分数（整数换算）。

| red \ blue | shieldman | militia | swordsman | 行合计 red_wins |
|---|---|---|---|---|
| shieldman | 10000（100/0/0） | 10000（100/0/0） | 10000（100/0/0） | 300 |
| militia | 0（0/100/0） | 10000（100/0/0） | 0（0/100/0） | 100 |
| swordsman | 0（0/100/0） | 10000（100/0/0） | 10000（100/0/0） | 200 |

每格 n=100（胜率百分数 = pp/100，如 4700 → 47.00%）。

## 逐格判定行（脚本生成）

- cell red=shieldman blue=shieldman: games=100 red_wins=100 blue_wins=0 draws=0 win_rate_red_pp=10000 (100.00%)
- cell red=shieldman blue=militia: games=100 red_wins=100 blue_wins=0 draws=0 win_rate_red_pp=10000 (100.00%)
- cell red=shieldman blue=swordsman: games=100 red_wins=100 blue_wins=0 draws=0 win_rate_red_pp=10000 (100.00%)
- cell red=militia blue=shieldman: games=100 red_wins=0 blue_wins=100 draws=0 win_rate_red_pp=0 (0.00%)
- cell red=militia blue=militia: games=100 red_wins=100 blue_wins=0 draws=0 win_rate_red_pp=10000 (100.00%)
- cell red=militia blue=swordsman: games=100 red_wins=0 blue_wins=100 draws=0 win_rate_red_pp=0 (0.00%)
- cell red=swordsman blue=shieldman: games=100 red_wins=0 blue_wins=100 draws=0 win_rate_red_pp=0 (0.00%)
- cell red=swordsman blue=militia: games=100 red_wins=100 blue_wins=0 draws=0 win_rate_red_pp=10000 (100.00%)
- cell red=swordsman blue=swordsman: games=100 red_wins=100 blue_wins=0 draws=0 win_rate_red_pp=10000 (100.00%)

## 全表汇总（脚本生成）

- 合计 games=900 red_wins=600 blue_wins=300 draws=0（混合 9 配置——非单一命题样本，仅作量级索引）。

## M0 T009 归档 sanity 对照（只读引用；种子域不同——非断言）

T009 `matrix_per100` 层同口径（单兵种 ×100/方、lane 50m、cap 1800、100 局/格）；
种子域 1_000_000+cell*100+k ≠ 本档 42..141——两批独立 100 局抽样，Δ 属抽样误差范畴
（±10pp 判据口径），仅作量级 sanity；大幅背离即上报复核。

| cell（red vs blue） | T023 pp（n=100） | T009 pp（n=100） | Δpp |
|---|---|---|---|
| shieldman vs shieldman | 10000 | 10000 | +0 |
| shieldman vs militia | 10000 | 10000 | +0 |
| shieldman vs swordsman | 10000 | 10000 | +0 |
| militia vs shieldman | 0 | 0 | +0 |
| militia vs militia | 10000 | 10000 | +0 |
| militia vs swordsman | 0 | 0 | +0 |
| swordsman vs shieldman | 0 | 0 | +0 |
| swordsman vs militia | 10000 | 10000 | +0 |
| swordsman vs swordsman | 10000 | 10000 | +0 |

- max |Δpp| = 0（对照批 = T009 归档 3600 局 JSONL 只读引用：`../t009/runs/matrix_per100/matrix_per100.jsonl`）。

## 溯源

- 默认值口径：`host/src/rpc.rs`（`DEFAULT_LANE_LEN_M` / 缺省 `TICK_CAP_REDUCED` / `MAX_SAMPLE_GAMES=1000`）。
- 终局四元组与哈希口径：`sim/src/world.rs`（`BattleOutcome` / `run_battle_with` / `Winner::label`）。
- M0 锚（本档不涉及，冒烟档 `sample_smoke.sh` CHK-03 用）：T004 黄金 `0x958c5938c8682529`（sim/src/world.rs 单测常量）。

