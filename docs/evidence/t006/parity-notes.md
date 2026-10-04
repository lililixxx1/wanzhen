# T006 等价性笔记（parity-notes）——两阶段重构 vs 原串行语义

- 执行：worker-1（2026-10-04）；本档为 docs/evidence/t006/ 组成件（D12）。
- 主张：T006 两阶段重构后，`step_with(None)` 与 `step_with(Some(&pool))` 对任意
  输入逐位一致（线程数无关性，构造性保证）；对原 T004/T005 串行语义，在
  「生产可达域」（deploy 布阵 + 移动 clamp 保持的严格分离不变式）内逐位一致，
  黄金锚与既有 30 单测零改动全绿为实证。

## 1. 线程数无关性（构造性，任意输入成立）

- 意图阶段：per-i 纯函数读同一快照 `Arc<ScanInput>`，无共享写——分片划分
  （`chunk_range` 连续均摊，pool.rs 单测断言覆盖/不交/均摊）与内容无关；各片
  结果按 start 升序拼装 = 索引序。
- 应用阶段：固定索引序串行（与线程数无关），reads 最新位置。
- threads=1 与多线程走同一代码路径（`step_with` 内 match 仅选执行器）。
- 单测：`parallel_parity_nearfield_battle`（近距战斗 500 单位 × 120 ticks ×
  threads{1,3,6,12} × 7 采样点全等）+ `combat_rescan_after_first_blood_switches_target`
  （threads=4 vs 串行逐字段）+ CLI 16 局对拍矩阵（matrix-1k.txt / matrix-10k.txt）。

## 2. move 两阶段 ≡ 原串行（等价域内）

应用式（实现）：`d_i = 无前方者 ? speed_i : min(speed_i, max((x[f]_now − x_i)·dir_i − (r_i + r_f), 0))`。

① **前方身份恒定**（快照 front = 原「每 i 现算最近者」）：快照时刻严格在前者
全程仍在前——每单位前进量被 clamp 于与其前方者的当前正间隙，中心不越过任何
「快照时刻在其前方」单位的中心；严格在后者全程仍在后（对称）；重合对互不为
前方者、各走各的。
② **clamp 式与原版同构**：由 ①，「当前 gap」即以快照 front 的最新位置计算之
gap ⇒ 阶段 2 逐字复现原式 `min(speed, max(当前 gap, 0))`。
③ **相向穿插防御内生于 clamp**：后结算者被先结算者已更新位置压停
（`two_phase_move_no_interpenetration_head_on` 专查；首轮实现曾漏乘 `dir` 致
蓝方恒停、门禁红——修正留痕见 check.txt 首轮记录与 world.rs 注释）。

**对派工单 D3 应用式的修正（上报留痕）**：D3 原文 `min(desire_i, ·)`（desire =
`min(speed, max(gap_old,0))` 快照预折算）在「同侧前方者先结算且本 tick 前进」
（gap_now = gap_old + d_f > gap_old，快兵追慢兵贴身瞬态）把前进截断在
last-tick 间隙，与原串行不一致；默认构成 deploy 为混速洗牌队列，瞬态必现 ⇒
黄金锚 0x958c5938c8682529 失守（该路径同时被
`contact_stop_stable_at_radius_sum` 的追近段覆盖）。按派工单 §5「算式与实测
冲突按实测执行」：应用式保留原版 `min(speed_i, gap_now⁺)`，desire 不入意图
缓冲。**后果**：意图缓冲只剩 front（并行部分 = O(N²) 前方查找，即本卡并行
v0 的目标热点）；前进 clamp 的 O(N) 保留在串行侧。

**等价域注记**：快照时刻存在「同侧同位置重合对」时前方身份可翻转（重合对互
不为前方，先行者前进后成为严格在前），该域内与原串行可分歧。生产可达域排除：
deploy 严格分离 + 移动 clamp（间隙 ≥ 0、中心不越过）逐 tick 保持严格分离；
`World::new` 裸路径（N 个重合占位，T002 冒烟原语）不承载哈希证据。

## 3. combat 两阶段 ≡ 原串行（任意输入成立）

引理（D4）：combat 阶段位置冻结（本阶段不动位置）⇒ 索敌仅依赖存活集；存活集
只缩不增；旧集 argmin（(距离, 索引) 字典序）若仍存活则必为当前集 argmin（移除
元素不影响存活最小值、平局序不变）；若已死则重索敌 ≡ 原版实时索敌同式。cd 只
被本单位出手重置（全局递减对全体一致）⇒ 意图预读无竞争。伤害 O(1) 现算（不
预计算，重索敌换目标自动正确）。
单测：`combat_rescan_after_first_blood_switches_target`（手算例：idx1 快照目标
idx2 被 idx0 先手一击死 → 重索敌切 idx3，hp/cd/x 逐字段断言 + threads=4 对拍）。

## 4. 单测 ↔ CLI 覆盖分工（任务卡裁决留痕）

| 层 | 构型 | 覆盖 |
|---|---|---|
| 单测 §3.1 | 近距 500 单位混速战斗 120 ticks | 线程{1,3,6,12}×7 采样点哈希全等——**接敌战斗路径**的线程无关性（含一击死/墓碑剥夺/重索敌/retain 压缩/clamp 链） |
| 单测 §3.2 | 4 单位重索敌手算 | 重索敌正路径 + threads=4 vs 串行逐字段 |
| 单测 §3.3 | 6 单位相向链 | 阶段 2 clamp 缺失即炸（穿插防御） |
| CLI 矩阵 1k | deploy 每方 500 × 14400 ticks × threads{1,3,6,12} × seeds{42,43} | 长程接敌对局逐采样点 + 终局 stdout 逐字节一致（D9 断言本体） |
| CLI 矩阵 10k | deploy 每方 5000 × 300 ticks × 同上 | 大规模不接敌（成本约束）下的线程无关性；**战斗语义的线程无关性由单测 §3.1/§3.2 承担**（分工留痕，任务卡 D9 裁决） |
| CLI fullscale | 10k × 14400 ticks @ threads 12, seed 42 | 验收 4：跑通不 panic 无死锁（600s 防死锁兜底，时长如实入档） |

## 5. 首轮门禁返工留痕（如实计）

首轮 `cargo test`（黄金锚回归检查点）：29/32——红三项：
`contact_stop_stable_at_radius_sum`（红 hp 39≠30）、
`golden_deploy_default_comp_seed42_1800ticks`、`battle_golden_crosscheck_default_comp`。
根因：move 应用阶段 gap 少乘 `dir`（原版 `ahead = (x_j − x_i)·dir` 方向感知；
裸差值对蓝方 dir=−1 恒负 → 蓝方全体恒停，闭合速率减半 → 首击 t=39 而非
t=14）。临时调试测试 dump 时间线定位后一行修复；复跑 32/32（黄金锚逐位复现）
后删除临时测试。返工 1 次如实计入台账。
