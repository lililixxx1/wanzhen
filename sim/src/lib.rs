//! 万阵 M0 确定性模拟核心（headless，不调用任何 bevy API）。
//!
//! 模块：[`rng`] 确定性随机数（SplitMix64 seed 扩展 + Xoshiro256\*\*）；
//! [`hash`] FNV-1a 64-bit 状态哈希；[`units`] 六兵种静态数据表 v0（定点常量 +
//! 克制表 + 克制倍率伤害）；[`world`] tick 纯计数推进、索引数组单位容器、
//! 双方对称布阵、单 lane 一维移动与最近邻索敌战斗结算（T004：cd 冷却 +
//! 克制倍率伤害 + 死亡移除）。
//!
//! 确定性纪律（报告 5.2 / AGENTS.md 硬约束 4）：模拟态禁 HashMap 迭代序、浮点、
//! 超越函数、挂钟时间源；tick 为纯计数；位置 / 距离 / 倍率一律定点整数；
//! 同 seed + 同参数 + 同 tick 数 → 状态逐位一致、与线程数无关。

pub mod hash;
pub mod rng;
pub mod units;
pub mod world;
