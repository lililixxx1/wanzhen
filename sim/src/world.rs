//! 确定性 World：tick 纯计数推进 + 索引数组单位容器 + 每 tick 末状态哈希。
//!
//! T003 扩展：六兵种挂载（[`crate::units`]）、双方对称布阵（[`World::deploy`]）、
//! 单 lane 一维移动与间距碰撞（贴身停 + 友军排队堵停）。
//!
//! 确定性纪律：无浮点 / 无超越函数 / 无挂钟 / 无 HashMap；
//! 遍历一律按索引序；同 seed + 同参数 + 同 tick 数 → 状态逐位一致。
//! 位置为 Q32.32 定点整数（[`crate::units::ONE_Q32_32`]），纯整数运算无舍入。

use crate::hash::Fnv1a;
use crate::rng::Xoshiro256StarStar;
use crate::units::{spec, UnitKind, ONE_Q32_32};

/// 单 lane 全长（Q32.32）：1000 米（纯整数表达式）。
pub const LANE_LEN_Q32: i64 = 1000 * ONE_Q32_32;

/// 布阵相邻单位间隙（Q32.32）：0.5 米（纯整数表达式，整除精确）。
const GAP_Q32: i64 = ONE_Q32_32 / 2;

/// 布阵洗牌 RNG 盐值（主会话 D5 定稿）：`deploy` 用 `seed ^ DEPLOY_SALT` 派生
/// **独立** RNG 实例，不消耗 World 的 tick 级 RNG——保证 T002 黄金锚
/// （units=0 哈希）不受布阵路径影响。
const DEPLOY_SALT: u64 = 0x6465_706c_6f79_0001;

/// 默认交战构成（CLI `--comp` 缺省值，主会话 D7 定稿）：六兵种各 5，
/// 双方同清单对称布阵 → 每方 30、共 60 单位。
pub const DEFAULT_COMPOSITION: [(UnitKind, usize); 6] = [
    (UnitKind::Shieldman, 5),
    (UnitKind::HeavyKnight, 5),
    (UnitKind::Pikeman, 5),
    (UnitKind::Swordsman, 5),
    (UnitKind::Archer, 5),
    (UnitKind::Militia, 5),
];

/// 交战方（判别值显式固定；判别值以 1 字节进状态哈希）。
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Red = 0,
    Blue = 1,
}

impl Side {
    /// 运动方向（主会话 D4 定稿）：红方向 +1、蓝方向 -1（单 lane 正向为 +x）。
    pub fn dir(self) -> i64 {
        match self {
            Side::Red => 1,
            Side::Blue => -1,
        }
    }

    /// CLI 输出用小写标签。
    pub fn label(self) -> &'static str {
        match self {
            Side::Red => "red",
            Side::Blue => "blue",
        }
    }
}

/// 单位（T003 起：兵种 + 阵营 + 一维位置；攻击/伤害/死亡属 T004，`alive` 先行占位）。
pub struct Unit {
    pub alive: bool,
    pub kind: UnitKind,
    pub side: Side,
    /// 一维位置（Q32.32 定点整数）。
    pub x: i64,
}

pub struct World {
    /// 已推进的 tick 数（纯计数，不依赖任何时间源）。
    pub tick: u64,
    units: Vec<Unit>,
    rng: Xoshiro256StarStar,
    /// 最近一次 step 末的状态哈希；`deploy` 后为布阵快照哈希（tick 0 语义）；
    /// `World::new` 直构（未 step / 未 deploy）时为 0。
    pub last_hash: u64,
}

impl World {
    /// 裸单位容器（非布阵路径）：全部单位为确定缺省态
    /// `alive: true, kind: Shieldman, side: Red, x: 0`（彼此重合的退化占位，
    /// 供 `--units` 冒烟路径与 T002 兼容；真实交战一律走 [`World::deploy`]）。
    /// 初始不消耗 RNG（RNG 仅经 step 的固定消耗点演化）。
    pub fn new(seed: u64, unit_count: usize) -> Self {
        let mut units = Vec::with_capacity(unit_count);
        for _ in 0..unit_count {
            units.push(Unit {
                alive: true,
                kind: UnitKind::Shieldman,
                side: Side::Red,
                x: 0,
            });
        }
        Self {
            tick: 0,
            units,
            rng: Xoshiro256StarStar::from_seed(seed),
            last_hash: 0,
        }
    }

    /// 双方对称布阵（主会话 D5 定稿，关联函数、全新 World）：
    ///
    /// 1. 构成清单按顺序展开成序列（清单顺序 = 兵种块顺序）；
    /// 2. Fisher-Yates 全洗：`for i in (1..m).rev() { j = rng.next_u64() % (i+1); swap(i, j) }`。
    ///    `%` 取模偏差（2^64 非整除 i+1 时 j 非严格均匀）对布阵用途可接受，留痕于此；
    ///    洗牌 RNG 为独立实例 `Xoshiro256StarStar::from_seed(seed ^ DEPLOY_SALT)`，
    ///    **不消耗 World 的 tick 级 RNG**；
    /// 3. 洗后同一序列用于双方（完美对称）：红方索引 0..m、蓝方 m..2m。
    ///    红方队头（序列第 0 个）`x = radius_0`，其后
    ///    `x_k = x_{k-1} - (radius_{k-1} + radius_k + GAP)`（向 -x 排队）；
    ///    蓝方镜像 `x'_k = LANE_LEN_Q32 - x_k`；
    /// 4. 布阵后 tick=0、alive 全 true、`last_hash` = 布阵快照哈希（tick 0 状态哈希）。
    pub fn deploy(seed: u64, composition: &[(UnitKind, usize)]) -> Self {
        let mut seq: Vec<UnitKind> = Vec::new();
        for (kind, count) in composition {
            for _ in 0..*count {
                seq.push(*kind);
            }
        }
        let m = seq.len();

        // Fisher-Yates 全洗（独立 RNG，seed ^ DEPLOY_SALT；% 取模偏差可接受，见上）。
        let mut shuffler = Xoshiro256StarStar::from_seed(seed ^ DEPLOY_SALT);
        for i in (1..m).rev() {
            let j = (shuffler.next_u64() % (i as u64 + 1)) as usize;
            seq.swap(i, j);
        }

        // 红方队列位置：队头 x = radius_0，向 -x 方向排队（间隙 GAP）。
        let mut x_red: Vec<i64> = Vec::with_capacity(m);
        for k in 0..m {
            let r_k = spec(seq[k]).radius_q32;
            let x_k = if k == 0 {
                r_k
            } else {
                x_red[k - 1] - (spec(seq[k - 1]).radius_q32 + r_k + GAP_Q32)
            };
            x_red.push(x_k);
        }

        let mut units: Vec<Unit> = Vec::with_capacity(2 * m);
        for k in 0..m {
            units.push(Unit {
                alive: true,
                kind: seq[k],
                side: Side::Red,
                x: x_red[k],
            });
        }
        for k in 0..m {
            units.push(Unit {
                alive: true,
                kind: seq[k],
                side: Side::Blue,
                x: LANE_LEN_Q32 - x_red[k],
            });
        }

        let mut world = Self {
            tick: 0,
            units,
            rng: Xoshiro256StarStar::from_seed(seed),
            last_hash: 0,
        };
        world.last_hash = world.state_hash();
        world
    }

    pub fn unit_count(&self) -> usize {
        self.units.len()
    }

    /// 单位切片（按索引序；CLI 快照输出与测试用）。
    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    /// 状态哈希：FNV-1a 64 逐字段固定序折叠——
    /// tick → rng 的 4 个状态字 → 每单位 [alive 1B + kind 判别 1B + side 判别 1B +
    /// x 的 8B LE]（按索引序）。（T002 的仅 alive 1B 折叠自 T003 起扩展；
    /// units=0 时折叠序列不变，T002 黄金锚不受影响。）
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv1a::new();
        h.write_u64(self.tick);
        for w in self.rng.state_words() {
            h.write_u64(w);
        }
        for unit in &self.units {
            h.write_u8(u8::from(unit.alive));
            h.write_u8(unit.kind as u8);
            h.write_u8(unit.side as u8);
            h.write_u64(unit.x as u64);
        }
        h.finish()
    }

    /// 推进一个 tick：
    /// 1) tick 计数 +1；
    /// 2) tick 级 RNG 固定消耗点——取一个 u64 丢弃（T002 设计保留；T003 无单位级
    ///    RNG 消耗，战斗结算的消耗点留 T004）；
    /// 3) 单 lane 移动（按索引序顺序结算）；
    /// 4) 末尾刷新状态哈希。
    pub fn step(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        let _ = self.rng.next_u64();
        self.move_units();
        self.last_hash = self.state_hash();
    }

    /// 单 lane 一维移动（主会话 D6 定稿）：
    ///
    /// 对每个存活单位 i（**按索引序顺序结算**——5.2 纪律，固定序与线程数无关；
    /// 前方查询使用**当前最新位置**，即本 tick 内先结算者的已更新位置）：
    /// - `dir = ±1`（红 +1 / 蓝 -1）；在所有存活单位 j≠i 中找运动方向上最近者：
    ///   `ahead = (x_j - x_i) * dir > 0` 中 ahead 最小者（平局取最小索引 j——
    ///   从 0 向上扫、严格小于才更新，自然实现）；
    /// - 存在最近者：`gap = ahead - (radius_i + radius_j)`，本 tick 前进
    ///   `min(speed, max(gap, 0))`；否则前进 `speed`；
    /// - `x_i += dir * 前进`。
    ///
    /// 该模型同时覆盖「与敌方贴身停（间距=半径和）」与「友军排队堵停」（异速
    /// 友军不穿插——无友军堵则高速兵可穿过低速兵，实验场失真）。
    /// 最近者查找为朴素 O(N²)；空间划分 / 并行属 T006/T007，本阶段禁止引入。
    /// 死亡属 T004；本阶段全存活，`alive` 过滤仅为语义完整。
    ///
    /// 溢出安全：x ∈ [-2^35, 2^42] 量级（lane 1000m = 2^42），(x_j - x_i) 与
    /// `* dir` 均远不及 i64 上界。
    fn move_units(&mut self) {
        let n = self.units.len();
        for i in 0..n {
            if !self.units[i].alive {
                continue;
            }
            let dir = self.units[i].side.dir();
            let x_i = self.units[i].x;
            let spec_i = spec(self.units[i].kind);
            // 运动方向上最近者：扫全索引、取最小正 ahead；严格小于才更新 → 平局取最小 j。
            let mut best_ahead: Option<i64> = None;
            let mut best_radius_sum: i64 = 0;
            for j in 0..n {
                if j == i || !self.units[j].alive {
                    continue;
                }
                let d = (self.units[j].x - x_i) * dir;
                if d > 0 && best_ahead.map_or(true, |b| d < b) {
                    best_ahead = Some(d);
                    best_radius_sum = spec_i.radius_q32 + spec(self.units[j].kind).radius_q32;
                }
            }
            let speed = spec_i.speed_q32;
            let advance = match best_ahead {
                Some(ahead) => {
                    let gap = ahead - best_radius_sum;
                    if gap > 0 {
                        if speed < gap {
                            speed
                        } else {
                            gap
                        }
                    } else {
                        0
                    }
                }
                None => speed,
            };
            self.units[i].x += dir * advance;
        }
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
    use crate::units::ArmorClass;

    /// 手动构造 World（不经 deploy / new 的缺省路径，测试专用直填）。
    fn manual_world(units: Vec<Unit>, seed: u64) -> World {
        World {
            tick: 0,
            units,
            rng: Xoshiro256StarStar::from_seed(seed),
            last_hash: 0,
        }
    }

    fn unit(kind: UnitKind, side: Side, x: i64) -> Unit {
        Unit {
            alive: true,
            kind,
            side,
            x,
        }
    }

    #[test]
    fn golden_hash_seed42_units0_1800ticks() {
        // 黄金值由实测固化（PIT-M-002 纪律）：占位 0 运行取得实测输出后回填；
        // 并经独立 python 重实现对拍一致（docs/evidence/t002/rng-golden.txt）。
        // T003 回归锚：哈希折叠扩展（D4）后 units=0 折叠序列不变，本值必须原样保持。
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
        // 两个独立 World，同 seed 同参数：起（tick 0）/ 中（tick 900）/ 末（tick 1800）逐点对拍。
        let mut a = World::deploy(42, &DEFAULT_COMPOSITION);
        let mut b = World::deploy(42, &DEFAULT_COMPOSITION);
        assert_eq!(a.state_hash(), b.state_hash(), "checkpoint start (tick 0)");
        assert_eq!(a.last_hash, b.last_hash, "deploy snapshot hash consistent");
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
        // 同 seed 同 ticks：units=0 与布阵 60 单位终局哈希不同（单位段参与折叠）。
        let mut a = World::new(42, 0);
        let mut b = World::deploy(42, &DEFAULT_COMPOSITION);
        a.run(1800);
        b.run(1800);
        assert_ne!(a.last_hash, b.last_hash, "unit segment must enter the hash");
    }

    /// 单测 2（验收 2 前半）：无敌方时全速推进 N tick 的位置 = 初值 + 速度×N。
    /// 口径：Q32.32 定点纯整数运算，无舍入——等式为整数精确断言。
    #[test]
    fn full_speed_advance_exact_linear() {
        // 手动构造仅 1 个红方单位（不 deploy）：Swordsman 0.12 m/tick，初值 x = 0。
        let mut w = manual_world(vec![unit(UnitKind::Swordsman, Side::Red, 0)], 7);
        let speed = spec(UnitKind::Swordsman).speed_q32;
        assert_eq!(w.units[0].x, 0);
        w.run(100);
        assert_eq!(w.tick, 100);
        assert_eq!(
            w.units[0].x,
            0 + speed * 100,
            "无阻挡 → 每 tick 精确累加 speed；x == 初值 + speed*N（整数精确）"
        );
    }

    /// 单测 3（验收 2 后半）：贴身停稳定——红蓝各 1 单位相距 3m 相向，
    /// 末 100 tick 双方 x 逐 tick 不变，且末态间距恰 = 半径和（Q32.32 整数精确）。
    #[test]
    fn contact_stop_stable_at_radius_sum() {
        let r_r = spec(UnitKind::Shieldman).radius_q32; // 0.5 m
        let r_b = spec(UnitKind::Militia).radius_q32; // 0.4 m
        // 红在 x=0 向 +x，蓝在 x=3m 向 -x（相向）。
        let mut w = manual_world(
            vec![
                unit(UnitKind::Shieldman, Side::Red, 0),
                unit(UnitKind::Militia, Side::Blue, 3 * ONE_Q32_32),
            ],
            7,
        );
        w.run(100); // 相向闭合 0.05+0.09=0.14 m/tick，3m → 1m 需 ~15 tick，100 tick 内早已贴身
        // 末 100 tick：双方 x 逐 tick 不变（贴身停稳定）。
        let mut prev_r = w.units[0].x;
        let mut prev_b = w.units[1].x;
        for t in 0..100 {
            w.step();
            assert_eq!(w.units[0].x, prev_r, "tick {} 红方位置应不变", 101 + t);
            assert_eq!(w.units[1].x, prev_b, "tick {} 蓝方位置应不变", 101 + t);
            prev_r = w.units[0].x;
            prev_b = w.units[1].x;
        }
        // 恰贴身：|x_r - x_b| == 半径和（整数精确）。
        assert_eq!(
            w.units[1].x - w.units[0].x,
            r_r + r_b,
            "末态间距应恰为半径和（贴身停）"
        );
    }

    /// 单测 4：友军不穿插——红方两异速单位前后排 + 远处蓝方使前排停住：
    /// 全程友军间距 >= 半径和（异速不穿插），且后排在前排停后被堵停（末态间距恰 = 半径和、恒定）。
    #[test]
    fn friendly_units_never_interpenetrate_and_queue_behind_stopped_front() {
        // 前排 = 重骑兵（快，0.20 m/tick，r=0.8），后排 = 剑盾兵（慢，0.05 m/tick，r=0.5），
        // 蓝方民兵在 20 m 处（r=0.4）：重骑兵 ~65 tick 后与敌贴身停住。
        let mut w = manual_world(
            vec![
                unit(UnitKind::HeavyKnight, Side::Red, 0),
                unit(UnitKind::Shieldman, Side::Red, -2 * ONE_Q32_32),
                unit(UnitKind::Militia, Side::Blue, 20 * ONE_Q32_32),
            ],
            7,
        );
        let r_front = spec(UnitKind::HeavyKnight).radius_q32;
        let r_back = spec(UnitKind::Shieldman).radius_q32;
        let radius_sum = r_front + r_back;
        // 全程（400 tick，足够后排追上停住的前排）间距 >= 半径和。
        for t in 0..400 {
            w.step();
            let gap = w.units[0].x - w.units[1].x;
            assert!(
                gap >= radius_sum,
                "tick {t}: 友军间距 {gap} < 半径和 {radius_sum}（发生穿插）"
            );
        }
        // 末态：后排恰被停住的前排堵停（间距 = 半径和），且此后恒定。
        assert_eq!(w.units[0].x - w.units[1].x, radius_sum, "末态应恰贴身排队");
        let gap_end = w.units[0].x - w.units[1].x;
        w.run(50);
        assert_eq!(w.units[0].x - w.units[1].x, gap_end, "堵停后间距必须恒定");
        // 前排与敌亦应恰贴身（前排停住的原因成立）。
        let r_enemy = spec(UnitKind::Militia).radius_q32;
        assert_eq!(w.units[2].x - w.units[0].x, r_front + r_enemy, "前排应与敌恰贴身");
    }

    /// 单测 5：布阵镜像对称——默认清单 deploy 后，对每 k：x_red[k] + x_blue[k] == LANE_LEN_Q32。
    #[test]
    fn deploy_mirror_symmetry_default_comp() {
        let w = World::deploy(42, &DEFAULT_COMPOSITION);
        assert_eq!(w.tick, 0);
        assert_eq!(w.unit_count(), 60, "默认构成：六兵种各 5 × 双方 = 60");
        assert_ne!(w.last_hash, 0, "deploy 后 last_hash = 布阵快照哈希");
        let m = w.unit_count() / 2;
        for k in 0..m {
            assert_eq!(w.units[k].side, Side::Red);
            assert_eq!(w.units[m + k].side, Side::Blue);
            assert_eq!(w.units[k].kind, w.units[m + k].kind, "k={k}: 双方同一洗后序列");
            assert!(w.units[k].alive && w.units[m + k].alive);
            assert_eq!(
                w.units[k].x + w.units[m + k].x,
                LANE_LEN_Q32,
                "k={k}: 镜像对称 x_red[k] + x_blue[k] == LANE_LEN_Q32"
            );
        }
        // 红方队头（k=0）x = 半径（D5 布阵式起点）。
        assert_eq!(w.units[0].x, spec(w.units[0].kind).radius_q32);
    }

    /// 单测 6：布阵确定性——同 seed 两个独立 World 位置逐位一致；
    /// 同清单异 seed 至少一个位置不同（洗牌生效）。
    #[test]
    fn deploy_deterministic_same_seed_differs_across_seeds() {
        let a = World::deploy(42, &DEFAULT_COMPOSITION);
        let b = World::deploy(42, &DEFAULT_COMPOSITION);
        assert_eq!(a.unit_count(), b.unit_count());
        for i in 0..a.unit_count() {
            assert_eq!(a.units[i].kind, b.units[i].kind, "u{i} kind");
            assert_eq!(a.units[i].side, b.units[i].side, "u{i} side");
            assert_eq!(a.units[i].x, b.units[i].x, "u{i} x");
        }
        assert_eq!(a.last_hash, b.last_hash, "同 seed 布阵快照哈希一致");
        let c = World::deploy(43, &DEFAULT_COMPOSITION);
        let diff = a
            .units
            .iter()
            .zip(c.units.iter())
            .filter(|(p, q)| p.x != q.x)
            .count();
        assert!(diff > 0, "异 seed 洗牌应至少改变一个位置");
        assert_ne!(a.last_hash, c.last_hash);
    }

    /// 单测 8：新黄金值——默认 comp、seed 42、1800 ticks 终局哈希（实测产出后固化，
    /// PIT-M-002 流程：占位 0 → 实测 → 回填，见 docs/evidence/t003/golden.txt）。
    #[test]
    fn golden_deploy_default_comp_seed42_1800ticks() {
        // 实测回填（PIT-M-002）：占位 0 运行得 17489830120905256261，
        // 与 release CLI（--seed 42 默认构成 1800 ticks）输出 0xf2b85bd4727c2d45 逐位一致
        // 后固化；过程见 docs/evidence/t003/golden.txt。
        const GOLDEN_HASH: u64 = 17489830120905256261; // 0xf2b85bd4727c2d45
        let mut w = World::deploy(42, &DEFAULT_COMPOSITION);
        w.run(1800);
        assert_eq!(w.tick, 1800);
        assert_eq!(
            w.last_hash, GOLDEN_HASH,
            "golden deploy hash mismatch (default comp, seed 42, 1800 ticks)"
        );
    }

    /// 布阵位置构造自检（辅助单测 5/6 的算式正确性）：两人清单手推对照。
    #[test]
    fn deploy_two_unit_positions_match_formula() {
        // 构成 [Militia(1), HeavyKnight(1)]：洗牌 m=2 仅一次（i=1: j = rng % 2）。
        // 用独立同参 RNG 复现该次抽取，得到洗后序列，再对照布阵式。
        let comp = [(UnitKind::Militia, 1), (UnitKind::HeavyKnight, 1)];
        let seed = 1234;
        let w = World::deploy(seed, &comp);
        let mut shuffler = Xoshiro256StarStar::from_seed(seed ^ DEPLOY_SALT);
        let j = (shuffler.next_u64() % 2) as usize;
        // swap(1, j)：j=1 → 恒等（原序）；j=0 → 交换。统一式：seq = [comp[1-j], comp[j]]。
        let seq = [comp[1 - j].0, comp[j].0];
        // 红方：x_0 = r_0；x_1 = x_0 - (r_0 + r_1 + GAP)。
        let r0 = spec(seq[0]).radius_q32;
        let r1 = spec(seq[1]).radius_q32;
        let gap = ONE_Q32_32 / 2;
        let x1 = r0 - (r0 + r1 + gap);
        assert_eq!(w.units[0].x, r0);
        assert_eq!(w.units[0].kind, seq[0]);
        assert_eq!(w.units[1].x, x1);
        assert_eq!(w.units[1].kind, seq[1]);
        // 蓝方镜像。
        assert_eq!(w.units[2].x, LANE_LEN_Q32 - r0);
        assert_eq!(w.units[3].x, LANE_LEN_Q32 - x1);
        // 类别快照防漂移自检。
        assert_eq!(spec(UnitKind::Shieldman).armor, ArmorClass::Heavy);
    }
}
