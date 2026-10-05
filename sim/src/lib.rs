//! 万阵 M0 确定性模拟核心（headless，不调用任何 bevy API）。
//!
//! 模块：[`rng`] 确定性随机数（SplitMix64 seed 扩展 + Xoshiro256\*\*）；
//! [`hash`] FNV-1a 64-bit 状态哈希；[`units`] 六兵种静态数据表 v0（定点常量 +
//! 克制表 + 克制倍率伤害）；[`world`] tick 纯计数推进、索引数组单位容器、
//! 双方对称布阵、单 lane 一维移动与最近邻索敌战斗结算（T004：cd 冷却 +
//! 克制倍率伤害 + 死亡移除）、胜负判定与终局收束 run_battle（T005：全灭 /
//! tick 上限双路径、BattleOutcome 终局冻结、BattleLog 最小日志）、
//! 意图并行两阶段更新（T006：step_with / run_with / run_battle_with）；
//! [`pool`] 手写持久线程池执行器（T006/D5：std::thread + mpsc，零新依赖，
//! 连续均摊分块 map_chunks，结果与线程数无关）；[`spatial`] 意图阶段排序序
//! 快路径（T015：`(x, 索引)` 全序建序 + O(1) 邻域 / O(n) 扫掠查询，eligibility
//! 不满足即逐字回退朴素 O(N²) 分片——等价加速，模拟行为逐位不变）。
//!
//! 确定性纪律（报告 5.2 / AGENTS.md 硬约束 4）：模拟态禁 HashMap 迭代序、浮点、
//! 超越函数、挂钟时间源；tick 为纯计数；位置 / 距离 / 倍率一律定点整数；
//! 同 seed + 同参数 + 同 tick 数 → 状态逐位一致、与线程数无关
//! （T006 起 step_with 意图阶段并行只加速执行划分，不改变串行语义）。

pub mod hash;
pub mod pool;
pub mod rng;
pub mod spatial;
pub mod units;
pub mod world;
