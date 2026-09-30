# T003 证据档——六兵种数据与单 lane 移动（M0 W1）

## 各文件用途

| 文件 | 内容 |
| --- | --- |
| `check.txt` | `cargo check --workspace -j 3` 完整输出 + 真实退出码（0 警告门禁，REAL_EXIT=0） |
| `test.txt` | `cargo test -p sim -j 3` 完整输出 + 真实退出码（17/17 全绿，REAL_EXIT=0） |
| `build-release.txt` | `cargo build -p sim --release -j 3` 构建记录（跨进程对拍所用二进制来源，REAL_EXIT=0） |
| `deploy-snap1.txt` / `deploy-snap2.txt` | 两次独立进程 `./target/release/sim.exe --seed 42 --dump-formation` 全记录（stdout 原始五行摘要 + 60 单位位置表、stderr 分离附后、REAL_EXIT=0；不跑 tick，final_tick=0） |
| `cross-check.txt` | 两快照 stdout 逐字节对拍（cmp + diff 均 0 差异）+ seed 43 快照对照（不同）+ T002 回归锚 CLI 口径核验（`--units 0 --seed 42 --ticks 1800` 仍为 0xd3b6408fd46c2008） |
| `golden.txt` | 新黄金值实测固化全过程（PIT-M-002：占位 0 → 测试路径实测 17489830120905256261 → release CLI 独立复测 0xf2b85bd4727c2d45 逐位一致 → 回填 → 复跑全绿） |
| `redline-grep.txt` | 模拟态红线复核（剥离注释后代码 token 扫描：无浮点 / 无超越函数 / 无浮点 as / 无 HashMap；Instant 仅 main.rs 外壳） |

## 执行摘要

T003 交付六兵种静态数据表 v0（`sim/src/units.rs`，Q32.32 定点移速/半径 + Q16.16 克制倍率，
全部纯整数表达式）+ 双方对称确定性布阵（`World::deploy`，Fisher-Yates 独立 RNG `seed ^ DEPLOY_SALT`，
不消耗 tick 级 RNG）+ 单 lane 一维移动（按索引序顺序结算、前方查询用当前最新位置、
最近者朴素 O(N²)、贴身停 + 友军排队堵停）。状态哈希折叠扩展为
tick → rng 4 状态字 → 每单位 [alive 1B + kind 1B + side 1B + x 8B LE]（索引序）；
T002 黄金锚（units=0, 1800 ticks = 15255451774252490760）原值保持。

门禁：`cargo check --workspace -j 3` 0 警告；`cargo test -p sim -j 3` 17/17 全绿
（含克制 3×3 穷举、定点常量自检、全速推进精确等式、贴身停稳定、友军不穿插、
布阵镜像对称、布阵确定性、T002 黄金回归、新黄金值、布阵算式手推对照）。
跨进程确定性：`--seed 42 --dump-formation` 两次独立进程 stdout 逐字节一致
（布阵快照哈希 `0x22bce4da752220de`，60 单位）；`--seed 43` 快照 `0x70f65c2f7f585cad`
（不同，洗牌生效）。新黄金值：默认构成（六兵种各 5 × 双方 = 60 单位）、seed 42、
1800 ticks 终局哈希 `0xf2b85bd4727c2d45`（17489830120905256261）。

## D1 · 克制方向拍板留痕（主会话定稿，实现原样落码）

报告 V0.9.1 的 **3.1 节只定义三类环形克制的结构、未显式落字方向**。原文：

> 兵种以「定位 × 克制 × 科技」三维定义：近战/远程/飞行/攻城等定位，**重甲/轻甲/无甲克制环**，
> 以及 2–3 级科技升级。（3.1 战前：构筑即玩法）

唯一语义线索在 2.1 节米拉奇原型：

> ……两军自动交锋，打空对方血条获胜；**胜负手在重甲/轻甲克制**、攻击速度差异与出兵时机的
> 滚雪球博弈。（2.1 米拉奇战记：类型原点）

据此主会话拍板单环方向：**重甲 克 轻甲、轻甲 克 无甲、无甲 克 重甲**（`sim/src/units.rs`
模块注释与 `COUNTER_TABLE` 行注释同步留痕）。倍率：克制 ×1.5（`3 * ONE_Q16_16 / 2` =
98304）、中性 ×1.0（65536）、被克 ×2/3 定点化（`2 * ONE_Q16_16 / 3`，整数除法截断 =
43690，≈0.66669）。**方向与倍率均为平衡初值，实验场（T009）回归对象。**

### 数值口径说明（与派工单 D2/单测 9 示例的差异，实现以 D2 表达式为准）

- 派工单 D2 括注「`2 * ONE_Q16_16 / 3`（43691）」、单测 9 示例「`5 * ONE_Q32_32 / 100 ==
  214_748_365`」为**四舍五入口径**；Rust 整数除法为截断：131_072 = 3×43_690 + 2 →
  **43_690**；21_474_836_480 = 100×214_748_364 + 80 → **214_748_364**。
- 实现按 D2 给定的**表达式**落码（表达式为定义、括注值为误估），实际值在
  `units::tests::fixed_point_constants_selfcheck` 以完整推导断言，并在
  `COUNTER_TABLE` 注释注明。差 1/65536 量级，不影响确定性；若主会话裁定应取
  四舍五入值（43691），仅需改表达式并同步单测，属一行级返工。

## CLI 语义（D7 扩展后的完整口径）

- `--comp <kind:count,...>`：英文 id（shieldman/heavyknight/pikeman/swordsman/archer/militia），
  未知 id / count=0 / 格式错 / 空 → stderr 报错 exit 2；与 `--units` 互斥（同时给出 →
  exit 2，主会话未定夺处的保守处理，见上报）。
- 缺省（无 `--comp` 无 `--units`）= 默认构成布阵（六兵种各 5，双方对称，60 单位）——
  即 D7 默认；因此缺省 stdout 的 `units=` 自 T003 起为 60（T002 时代为 0）。
- `--units <n>` 保留 T002 裸单位语义（`World::new` 缺省态：Shieldman/Red/x=0 退化占位，
  供冒烟与兼容；真实交战一律 `deploy`）。
- `--dump-formation`：布阵后（tick 0、不跑 tick）输出五行摘要（`final_tick=0`、`hash` 为
  布阵快照哈希；`ticks=` 行打印**配置值**而非 0，配合 `final_tick=0` 读作「配置 N tick、
  快照取于 tick 0」）+ 每单位一行 `u<idx> <red|blue> <kind> x=<i64 十进制>`（索引序），
  全走 stdout；`elapsed_ms` 仍走 stderr（不参与 diff）。
- 布阵位置可为负 x：D5 布阵式红方队尾向 -x 排队可越过 lane 原点（快照中 u29 x≈-43.4 m），
  T003 不设 lane 边界约束，移动模型下无影响（镜像对称保持）。

## 结论

验收 1（check 0 警告 + 同 seed 跨进程布阵快照逐位一致 + seed 43 不同）、验收 2（全速推进
精确等式 + 贴身停稳定，单测 2/3）、验收 3（克制 3×3 穷举，单测 1）全部留证于本档。
