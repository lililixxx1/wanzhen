# T004 证据档（M0-03 索敌攻击与克制结算）

- 任务卡：taskset/t004-combat.md；执行：worker-1（2026-10-04）；派工单设计定稿 D1~D10。
- 结论：**门禁全绿**——cargo check 0 警告；cargo test 23/23（含黄金占位轮 PIT-M-002
  规定动作后回填复跑）；红线 grep 0 命中；跨进程对拍 seed 42/43 各自逐字节一致、
  互异保持；T002 黄金锚原值保持；T003 黄金按 D7 失效并以新值 0x958c5938c8682529 固化。
- **上报项（派工单数值笔误，按锁定表执行，详见 §4）**：D1 例 4/5 克制倍率方向互换
  （连带 G/H 存活推演），实现与断言一律以 T003 锁定克制表 + 派工单公式为准。

## 1) 文件索引

| 文件 | 内容 |
|---|---|
| check.txt | `cargo check --workspace -j 3` 全文 + REAL_EXIT=0（0 警告） |
| test.txt | `cargo test -p sim -j 3` 回填后复跑全文 + REAL_EXIT=0（23/23） |
| golden.txt | 新黄金实测固化过程：旧值失效留痕 → 占位轮 → CLI 交叉验证 → 回填 → 复跑 |
| crossproc.txt | 跨进程对拍（seed 42/43 双进程 stdout 逐字节 diff）+ --dump-final-units + 黄金锚复核 |
| s42_run1/2.txt、s43_run1/2.txt、dump_final.txt | 对拍原始输出（自含命令与 REAL_EXIT） |
| redline-grep.txt | 模拟态红线 token 扫描（python 剥注释/字符串后扫代码 token，方法同 T003） |
| handcalc.md | 手算推演表：例 1/2/3 逐 tick 表 + 附 G/H 重推演（倍率方向笔误的实推值依据） |
| py_recalc/ | **非本卡产物**：并行执行者的 python 独立第二实现（双盲对拍地基），时间戳早于本卡开工，未改动 |

## 2) 验收断言对照（任务卡）

1. check 0 警告 ✓（check.txt）；克制全组合伤害数值 ✓——单测
   `units::tests::damage_formula_all_6x6_pairs`：36 组合对拍锁定表字面量矩阵 +
   D1 手算例显式断言（例 4/5 按锁定表值 9/9，见 §4 上报）。
2. 手算对拍 ≥3 例 ✓——`handcalc_pair_counter_fight_to_kill`（例 1：t=151 快照 /
   t=181 击杀清除 / t=185 恢复全速）、`handcalc_equal_distance_target_tiebreak_by_index`
   （例 2：B 未被选中）、`handcalc_death_removal_compacts_indices`（例 3：索引前移）；
   推演过程 = handcalc.md，全部一次对拍通过（占位轮即绿）。
3. 同 seed 跨进程终局哈希一致 ✓（crossproc.txt §1/§2）；死亡移除后哈希仍稳定 ✓——
   `death_tick_hash_replay_pairwise`：例 1 型双 World 逐 tick 对拍 state_hash 至 t=200
   （覆盖死亡 t=181 与清除后区间，清除顺序确定性由索引序 retain 保证）。

## 3) 设计定稿落点（D1~D10 → 代码）

| 定稿 | 落点 |
|---|---|
| D1 伤害公式 | `units::damage_dealt`（i32 截断除法；溢出安全 14×98304=1,376,256 << i32::MAX 注释留痕）；hp ≤ 0 → 墓碑 |
| D2 索敌 | `world::combat` 内联：存活敌方 |dx| 最小，平局取最小索引（严格小于才更新），每 tick 重算 O(n²) |
| D3 射程 | `units::MELEE_MARGIN_Q32 = 20 * ONE / 100`（=858993459，推导注释）；闭区间 `|dx| ≤ r_i+r_j+margin` |
| D4 攻击计时 | `Unit.cd: u32`；战斗阶段先全存活单位 saturating_sub(1) 再索引序判定；出手才置 interval；未接敌保持 0 |
| D5 阶段序 | `world::step`：tick → RNG 固定消耗 → move → combat → retain 清墓碑 → state_hash（注释留痕） |
| D6 死亡移除 | 墓碑标记 → tick 末 `units.retain(|u| u.alive)` 稳定保序压缩；**跨 tick 索引不稳定**（world.rs 模块注释 + 本 README 留痕）；T009 若需稳定标识须另配不回收 uid（本卡不做） |
| D7 哈希扩展 | 每单位 +hp 4B LE +cd 4B LE（`hash::write_u32`）；units=0 折叠序列不变 → T002 锚 0xd3b6408fd46c2008 原值保持；T003 黄金失效 → 新值 0x958c5938c8682529（golden.txt） |
| D8 射程字段 | `UnitSpec.range_q32`（近战 0、archer 30m）；M0 战斗判定不读（注释留痕，T009+ 启用）；单测 F 防漂移 |
| D9 CLI | `--dump-final-units`：五行摘要后逐单位 `u{idx} {side} {kind} hp={hp} cd={cd} x={x}` |
| D10 默认不接敌 | world.rs 模块注释留痕 + crossproc.txt §3（60 单位 1800 ticks 全存活满血 cd=0 实证）；移交 T005 |

## 4) 上报项（派工单数值笔误——按锁定表执行；主会话已裁决：确认笔误，实现为终态、克制表零改动）

**发现**：派工单 D1 六个手算例中的例 4/例 5 与 T003 锁定克制表（及 D1 自身例 3）
方向互换，连带影响改造 G/H 的存活推演数值：

| 项 | 派工单速算 | T003 锁定表实值 | 差异根源 |
|---|---|---|---|
| D1 例 4：民兵(6,Unarmored) 打 重甲(Heavy) | 6×43690/65536 = **3** | 6×98304/65536 = **9**（克制 ×1.5） | 43690 是「被克」倍率，对应 Unarmored→Light；锁定表为无甲**克**重甲 |
| D1 例 5：重骑(14,Heavy) 打 民兵(Unarmored) | 14×98304/65536 = **21** | 14×43690/65536 = **9**（被克 ×2/3） | 98304 对应 Heavy→Light；锁定表重甲**被**无甲克 |
| G(b)(c)(d)：盾兵 vs 民兵 | 盾 12/击、民 3/击 → t≈135 民兵死、t=200 红 hp=102 | 盾 **5**/击、民 **9**/击 → t=200 双活（红 30、蓝 15）、首亡 = 红于 t=274 | 同上方向互换 |
| H：骑士存活 | 骑 21/击（仅垫民兵 hp=6000 即安全） | 骑 **9**/击、民兵 9/击 → hp150 骑士 t=385 死于循环内 → 骑士 hp 同垫 6000 | 同上 |

**处置**（执行口径，非拍板）：实现按派工单公式 `dmg = attack × counter_multiplier / ONE_Q16_16`
+ T003 锁定表（`counter_table_3x3_exhaustive_and_armor_classification` 为既有回归锚，
不可破坏）；单测 A 断言 36 组合公式值 + 例 1/2/3/6 派工单原值 + 例 4/5 按锁定表值
（注释留痕差异）；G/H 断言按实推值（handcalc.md 附 G/H 有逐击推演）。
**未改动**克制表任何数值与方向（平衡初值变更属主会话权限）。

**影响面核查**：派工单例 1/2/3/6、单测 B/C/D/E/F 的全部数值与锁定表一致，不受影响；
其余实现（索敌/计时/阶段序/哈希/CLI）与倍率无关，不受影响。
提示：并行执行者的 python 独立实现（本目录 py_recalc/）按派工单原文推导，若其编码了
例 4/5 的 3/21，双盲对拍将在该两格出现差异——差异源即本上报项，非移植缺陷。

## 5) 其他留痕

- 执行过程中一次编辑事故（move_units 文档注释重复块）在 check 前发现并修正，
  未进入任何门禁运行；cargo check 首轮曾报 1 条 unused variable 警告（combat 索敌
  闭包），改写为 `Option<(usize, i64)>` 元组承载后复检 0 警告——记台账返工 1。
- 每文件命令与 REAL_EXIT 均已内嵌；机器门禁期间空闲独占（AGENTS.md cargo -j 3 纪律）。
- 未执行任何 git commit（派工单规定：主会话复核后统一提交）。

## 6) 主会话复核补充（2026-10-04，commit 前终态）

- **上报项裁决**：确认派工单 D1 例 4/5 为倍率方向笔误（worker-1 处置正确——例 3 与
  例 4 同为 Unarmored→Heavy 却给出矛盾倍率即内证）；按锁定表执行为终态、克制表零改动。
  G/H 连锁修正（G：t=200 双活红 30/蓝 15、首亡红 t=274；H：骑士 hp 同垫 6000）经
  主会话独立复算确认。
- **双盲对拍全闭环（py_recalc/ 独立第二实现 × Rust）**：
  - 新黄金 seed 42 = `0x958c5938c8682529`（双实现预测/实测逐位一致）；
  - seed 43 = `0x54611ed6ded02540`（双实现 × 双 seed 交叉验证，case4.txt 附节）；
  - 手算例 1~4 全部快照检查点逐字段一致（例 4 = 真等距决胜补充例，见下）；
  - py_recalc README「seed 42 待 Rust 侧复核」系时序措辞——worker-1 交付的 CLI
    实测 0x958c5938c8682529 与 python 预测早已一致，以本节为准。
- **单测 C2 补充**（主会话裁决 worker-2 上报的等距分支覆盖缺口后亲补）：链式贴身
  5 单位构造（红0/蓝A +0.8m/蓝B -0.8m/蓝C -1.6m/红D -2.4m，全民兵）——t=1 移动
  阶段全静止，红0 对 A/B 严格等距（0.8m/0.8m）→ 平局取最小索引 A；断言 B 不被击
  （hp=50）为决胜证据。C/D 位置取 0.8m 存储值 3435973836 的 -2×/-3× 精确整数倍
  （与 py_recalc 同口径；若用 floor 负除法会差 1 Q 单位破坏全静止）。
  哈希双盲锚固化：t=1 = 0xdcd569af50a28656、t=21 = 0x77695db8b679c474
  （python `--case 4 --seed 7` 逐位一致——首轮 worker-2 以 seed 42 跑出不同哈希
  属 seed 参数差异非实现分歧，seed 7 复跑后逐位一致，过程留痕于本节）。
- **终验**：cargo check --workspace -j 3 → 0 警告；cargo test -p sim -j 3 → 24/24
  （23 交付态 + C2）。台账 T004 行同步（单测 24、耗时口径对齐 ≈35 min、裁决结论）。
