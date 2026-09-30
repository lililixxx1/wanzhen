//! 确定性 World：tick 纯计数推进 + 索引数组单位容器 + 每 tick 末状态哈希。
//!
//! 确定性纪律：无浮点 / 无超越函数 / 无挂钟 / 无 HashMap；
//! 遍历一律按索引序；同 seed + 同参数 + 同 tick 数 → 状态逐位一致。

use crate::hash::Fnv1a;
use crate::rng::Xoshiro256StarStar;

/// 单位空壳（T003 填充具体字段与行为）。
pub struct Unit {
    pub alive: bool,
}

pub struct World {
    /// 已推进的 tick 数（纯计数，不依赖任何时间源）。
    pub tick: u64,
    units: Vec<Unit>,
    rng: Xoshiro256StarStar,
    /// 最近一次 step 末的状态哈希（未 step 前为 0）。
    pub last_hash: u64,
}

impl World {
    /// 单位容器以 with_capacity 预留，全部 `alive: true`；
    /// 初始态不消耗 RNG（RNG 仅经 step 的固定消耗点演化）。
    pub fn new(seed: u64, unit_count: usize) -> Self {
        let mut units = Vec::with_capacity(unit_count);
        for _ in 0..unit_count {
            units.push(Unit { alive: true });
        }
        Self {
            tick: 0,
            units,
            rng: Xoshiro256StarStar::from_seed(seed),
            last_hash: 0,
        }
    }

    pub fn unit_count(&self) -> usize {
        self.units.len()
    }

    /// 状态哈希：FNV-1a 64 逐字段固定序折叠——
    /// tick → rng 的 4 个状态字 → 每单位 alive（按索引序，1 字节 0/1）。
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv1a::new();
        h.write_u64(self.tick);
        for w in self.rng.state_words() {
            h.write_u64(w);
        }
        for unit in &self.units {
            h.write_u8(u8::from(unit.alive));
        }
        h.finish()
    }

    /// 推进一个 tick：
    /// 1) tick 计数 +1；
    /// 2) tick 级 RNG 固定消耗点——取一个 u64 丢弃（为未来全局随机源预留的固定消耗，
    ///    保证 rng_state 随 tick 演化、seed 进入哈希可见路径；单位级消耗留 T003+）；
    /// 3) 单位按索引序遍历（本阶段空壳无操作，保留遍历路径）；
    /// 4) 末尾刷新状态哈希（O(U)/tick，真实成本计入 smoke bench）。
    pub fn step(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        let _ = self.rng.next_u64();
        for i in 0..self.units.len() {
            let _ = self.units[i].alive;
        }
        self.last_hash = self.state_hash();
    }

    /// 连续推进 `ticks` 个 tick。
    pub fn run(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_hash_seed42_units0_1800ticks() {
        // 黄金值由实测固化（PIT-M-002 纪律）：占位 0 运行取得实测输出后回填；
        // 并经独立 python 重实现对拍一致（docs/evidence/t002/rng-golden.txt）。
        const GOLDEN_HASH: u64 = 15255451774252490760; // 0xd3b6408fd46c2008
        let mut w = World::new(42, 0);
        w.run(1800);
        assert_eq!(w.tick, 1800);
        assert_eq!(w.last_hash, GOLDEN_HASH, "golden world hash mismatch (seed 42, 0 units, 1800 ticks)");
    }

    #[test]
    fn seed_sensitivity_42_vs_43() {
        let mut a = World::new(42, 0);
        let mut b = World::new(43, 0);
        a.run(1800);
        b.run(1800);
        assert_ne!(a.last_hash, b.last_hash, "different seeds must diverge");
    }

    #[test]
    fn replay_consistency_checkpoints_start_mid_end() {
        // 两个独立 World，同 seed 同 units：起（tick 0）/ 中（tick 900）/ 末（tick 1800）逐点对拍。
        let mut a = World::new(42, 100);
        let mut b = World::new(42, 100);
        assert_eq!(a.state_hash(), b.state_hash(), "checkpoint start (tick 0)");
        a.run(900);
        b.run(900);
        assert_eq!(a.tick, 900);
        assert_eq!(a.state_hash(), b.state_hash(), "checkpoint mid (tick 900)");
        assert_eq!(a.last_hash, b.last_hash, "checkpoint mid (tick 900, last_hash)");
        a.run(900);
        b.run(900);
        assert_eq!(a.tick, 1800);
        assert_eq!(a.state_hash(), b.state_hash(), "checkpoint end (tick 1800)");
        assert_eq!(a.last_hash, b.last_hash, "checkpoint end (tick 1800, last_hash)");
    }

    #[test]
    fn units_participate_in_hash() {
        // 同 seed 同 ticks：units=0 与 units=100 终局哈希不同（单位参与折叠）。
        let mut a = World::new(42, 0);
        let mut b = World::new(42, 100);
        a.run(1800);
        b.run(1800);
        assert_ne!(a.last_hash, b.last_hash, "unit segment must enter the hash");
    }
}
