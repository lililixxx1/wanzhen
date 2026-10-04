# T005 证据档（M0-04 胜负判定与终局）

- 任务卡：taskset/t005-victory.md；执行：worker-1（2026-10-04）+ 主会话接管收尾；
  派工单设计定稿 D1~D12。
- 结论：**门禁全绿**——cargo check 0 警告（含收尾复跑）；cargo test 30/30（T004 的
  24 个全保持 + T005 新增 6 个）；跨进程对拍 seed 42/43 各自逐字节一致、互异保持；
  **黄金交叉成立**：`--battle` 默认构成 seed 42 终局 final_hash = 0x958c5938c8682529
  = T004 黄金锚原值（且与非 battle `run(1800)` 路径哈希一致——run_battle 控制流
  不动状态演化、resolved 字段零哈希影响，双路径实证）。本卡无新黄金值（无占位轮）。
- 执行事件如实留痕：worker-1 两度中断（①后台 agent 被用户消息打断、恢复后继续；
  ②收尾阶段配额超限）——check/test/crossproc/battle_cli 四档已由 worker 落盘，
  本 README 与台账行由主会话接管完成；代码与四档内容未因中断变动（主会话独立
  复现门禁三断言 + 黄金交叉后接管）。

## 1) 文件索引

| 文件 | 内容 |
|---|---|
| check.txt | `cargo check --workspace -j 3` 全文（0 警告）+ 收尾复跑 + REAL_EXIT=0 |
| test.txt | `cargo test -p sim -j 3` 全文 + REAL_EXIT=0（30/30） |
| crossproc.txt | `--battle` seed 42/43 双进程 stdout 逐字节 diff（各自一致、互异）+ 黄金交叉结论 + REAL_EXIT |
| battle_cli.txt | `--battle` 默认构成输出 / `--battle --dump-final-units` 组合（60 单位全存活满血 cd=0——镜像不接敌实证）/ `--battle`×`--units` 报错 exit 2 / `--battle`×`--dump-formation` 报错 exit 2 |

## 2) 验收断言对照（任务卡）

1. check 0 警告 ✓（check.txt）；必胜局判定正确 ✓——单测
   `battle_decisive_1v1_counter_kill_at_181`（全克制红盾兵 vs 全被克制蓝长矛：
   Winner::Red、end_tick=181 复用 T004 单测 B 已锁定时间线、alive (1,0)、
   final_hash 与独立手动 run(181) 的 state_hash 内聚对拍）+
   `battle_decisive_3v3_counter_queue`（多单位队列歼灭形态：Winner::Red、
   alive_blue=0、end_tick < 1800、幂等冻结）；空阵营边界不 panic、判定确定 ✓——
   `battle_empty_side_tick0_no_panic`（仅红 2 单位 → Red@tick0；双空 → Draw@tick0；
   二次调用幂等）。
2. tick 上限触发路径 ✓——`battle_cap_mirror_stalemate_draw_and_freeze`（镜像
   同速同 hp 民兵 1m/999m，接敌需 ≈5539 ticks > 1800 → 上限收束 Draw、end_tick=1800；
   **终局后 tick 停止推进**：再次 run_battle 同 outcome、tick 仍 1800、state_hash
   不变）+ `battle_cap_hp_judgment_asymmetric`（hp 100 > 50 高者胜分支）。
   判定规则（存活总 hp 高者胜、同值平局）双分支均有断言。
3. 同 seed 跨进程终局四元组逐字段一致 ✓（crossproc.txt：BattleLog 8 行含
   winner/end_tick/alive_red/alive_blue/final_hash，双进程逐字节 diff 一致；
   seed 43 互异）。

## 3) 设计定稿落点（D1~D12 → 代码）

| 定稿 | 落点 |
|---|---|
| D1 上限常量 | `world::TICK_CAP_REDUCED = 1800`（表 6-0 直接落字）；`TICK_CAP_FULL = 8×60×30 = 14400`（表 6-0 无全规模 tick 上限显式字段，按产品口径单局时长上限 8 分钟 @30Hz 换算钉死，注释留痕、V1.0 可回写）；run_battle 由调用方传 max_ticks |
| D2 终局结构 | `Winner{Red,Blue,Draw}` + label；`BattleOutcome{winner,end_tick,alive_red,alive_blue,final_hash}`（final_hash=终局点 state_hash 现算，T002 起已折叠 rng 4 状态字=含 rng_state） |
| D3 判定规则 | 全灭判定每 tick 求值且优先于上限（恰在上限 tick 全灭亦按全灭收束）；上限判定=存活 hp 按索引序 i64 累加（5.2 固定序纪律，溢出安全注释：200×150=30,000 << i64::MAX）；tick 0 亦求值（空阵营立即收束不 panic） |
| D4 run_battle | 控制流：缓存幂等 → tick 0 全灭检查 → `while tick < max_ticks { step; 全灭检查 }` → 耗尽走 hp 判定 → 存 `World.resolved` 冻结；`resolved` **不进 state_hash 不影响 step/run**（黄金锚零影响，单测 6 实证）；附 `outcome()` getter |
| D5 BattleLog | `{seed, red/blue_composition, outcome}` + Display 固定 8 行（seed/comp_red/comp_blue/winner/end_tick/alive_red/alive_blue/final_hash）；不含落库（表 6-0 口径） |
| D6 CLI | `--battle`：deploy 构战（--comp 或默认构成镜像）→ run_battle(--ticks) → BattleLog 8 行 stdout；`--dump-final-units` 可组合；×`--units`、×`--dump-formation` 互斥 exit 2；**非 battle 路径输出逐字节不变**（黄金锚证据链） |
| D7 测 1 | 1v1 必胜局：T004 时间线复用（t=181 击杀），final_hash 内聚对拍 |
| D8 测 2 | 3v3 队列歼灭（构造见 §4 上报）：结构性断言 + 幂等冻结 |
| D9 测 3~6 | 空阵营 tick0 / 上限 Draw+冻结 / hp 高者胜 / 黄金交叉（T004 锚 0x958c5938c8682529 逐位复现 + outcome() 读取） |
| D10 双灭不可达 | `Winner` 注释留痕归纳推演：末二存活者互杀要求 i_R < i_B 且 i_B < i_R 矛盾 ⇒ 同 tick 双方战斗全灭不可达，全灭 Draw 分支仅 tick 0 双空构型 |
| D11 红线 | 新增代码零浮点/挂钟/HashMap/超越函数；Display 与 CLI 全确定性内容；壁钟仅 main.rs stderr |
| D12 证据档 | 本档四文件 + README（自含命令 + 输出 + REAL_EXIT） |

## 4) 上报项（派工单位置构造笔误——主会话已裁决：接受 worker 修复）

**发现**：派工单测 2 原构造「蓝队首 x = 1000×ONE − r（远端镜像）」在 1800 ticks
内不接敌：队首相距 999 m、合闭合速度 0.05+0.10 = 0.15 m/tick，接敌
（|dx| ≤ 0.5+0.5+0.2 = 1.2 m）需 (999−1.2)/0.15 ≈ 6652 ticks > TICK_CAP_REDUCED
——对局将走上限 hp 判定而非歼灭，「必胜局多单位歼灭」被测语义不可达。

**修复（worker，已裁决接受）**：蓝队首压缩至 x = 2×ONE（2.0 m）——初距 1.5 m
出射程，t=2 移动后恰 1.2 m（闭区间）首击；队列间距式一字不改（r+r+GAP=1.5 m）。
此为连续第三卡派工单笔误（T003 裸值 / T004 倍率方向 / T005 场景几何），教训已
回流主会话记忆：派工单场景构造须附「接敌可达性算式（距离 ÷ 合闭合速度 ≤ 上限）」。

## 5) 经验值记录（非断言、防漂移参考）

- 3v3 队列歼灭战（红 3 盾兵 vs 蓝 3 长矛，蓝队首 x=2 m）：end_tick=585、
  alive=(2,0)——worker 以临时 stderr 观测取得后移除观测代码（test.txt 对应终态
  代码）；单测断言为结构性（Winner::Red、alive_blue=0、end_tick < 1800），585
  不入断言（平衡初值变动即漂移）。
- CLI `--battle` 默认构成（seed 42/43）：Draw、end_tick=1800、alive (30,30)、
  60 单位全存活满血 cd=0——「镜像不接敌」（world.rs 模块注释 D10）在 battle
  路径的实证；接触战 CLI 用例留 T009（异构构成 --comp 双方支持落地后）。
