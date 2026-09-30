//! 万阵 M0 确定性模拟核心（headless，不调用任何 bevy API）。
//!
//! 模块：[`rng`] 确定性随机数（SplitMix64 seed 扩展 + Xoshiro256\*\*）；
//! [`hash`] FNV-1a 64-bit 状态哈希；[`world`] tick 纯计数推进与索引数组单位容器。
//!
//! 确定性纪律（报告 5.2 / AGENTS.md 硬约束 4）：模拟态禁 HashMap 迭代序、浮点、
//! 超越函数、挂钟时间源；tick 为纯计数，同 seed + 同参数 + 同 tick 数
//! → 状态逐位一致、与线程数无关。

pub mod hash;
pub mod rng;
pub mod world;
