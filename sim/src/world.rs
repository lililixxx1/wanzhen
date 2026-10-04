//! 确定性 World：tick 纯计数推进 + 索引数组单位容器 + 每 tick 末状态哈希。
//!
//! T003 扩展：六兵种挂载（[`crate::units`]）、双方对称布阵（[`World::deploy`]）、
//! 单 lane 一维移动与间距碰撞（贴身停 + 友军排队堵停）。
//!
//! T004 扩展（主会话 D1~D10 定稿）：最近邻索敌 + 攻击计时（cd 冷却制）+ 克制倍率
//! 伤害 + 死亡移除——单 lane 自动交战闭合。[`World::step`] 阶段序固定（不得倒置）：
//! tick → RNG 固定消耗 → move_units → combat → tick 末 retain 清墓碑 → state_hash。
//!
//! ## 死亡移除与索引重排规则（主会话 D6 定稿）
//!
//! 战斗阶段内死亡 = 墓碑标记（`alive = false`，立即生效：同 tick 内排在后面的单位
//! 不再选它为目标、它自己也不再行动）；tick 末 `units.retain(|u| u.alive)` 按索引序
//! 稳定保序物理移除——存活者相对序不变、整体前移。**跨 tick 单位索引不稳定**：
//! 凡有死亡的 tick 末都会重排索引；T009 战斗日志如需稳定单位标识须另配不回收 uid
//! （本卡不做，留痕）。
//!
//! ## 默认构成不接敌观察（主会话 D10 定稿，移交 T005）
//!
//! 默认构成 deploy 后双方队首相距约 998 m，合闭合速度 ≤ 0.4 m/tick，
//! 1800 ticks 内不接敌（≈999/0.4 ≈ 2498 ticks 才贴身）；本卡不调初值（最小变更，
//! 平衡初值留 T009 回归），T004 全部战斗验证用测试内手动构造近距对局。
//!
//! 确定性纪律：无浮点 / 无超越函数 / 无挂钟 / 无 HashMap；
//! 遍历一律按索引序；同 seed + 同参数 + 同 tick 数 → 状态逐位一致。
//! 位置为 Q32.32 定点整数（[`crate::units::ONE_Q32_32`]），纯整数运算无舍入。

use crate::hash::Fnv1a;
use crate::rng::Xoshiro256StarStar;
use crate::units::{damage_dealt, spec, UnitKind, MELEE_MARGIN_Q32, ONE_Q32_32};

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

/// 单位（T004 起：+ hp / 攻击冷却 cd；死亡为墓碑标记，tick 末物理移除）。
pub struct Unit {
    pub alive: bool,
    pub kind: UnitKind,
    pub side: Side,
    /// 当前生命值（`<= 0` 即死亡墓碑；进状态哈希时存活者恒正——墓碑在
    /// retain 后不参与折叠）。
    pub hp: i32,
    /// 距下次可攻击的剩余 tick 数（0 = 可击；仅实际出手时重置为攻击间隔，
    /// 每 tick 战斗阶段开头统一 `saturating_sub(1)`——主会话 D4 定稿）。
    pub cd: u32,
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
                hp: spec(UnitKind::Shieldman).hp,
                cd: 0,
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
                hp: spec(seq[k]).hp,
                cd: 0,
                x: x_red[k],
            });
        }
        for k in 0..m {
            units.push(Unit {
                alive: true,
                kind: seq[k],
                side: Side::Blue,
                hp: spec(seq[k]).hp,
                cd: 0,
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
    /// hp 4B LE + cd 4B LE + x 的 8B LE]（按索引序）。
    /// （T002 的仅 alive 1B 折叠自 T003 起扩展 kind/side/x，T004 起再扩展 hp/cd
    /// 各 4B——主会话 D7 定稿；units=0 时折叠序列不变，T002 黄金锚不受影响。
    /// hp 以 u32 位型折叠：state_hash 的调用路径上存活者恒正，墓碑已被 retain
    /// 物理移除、不参与折叠。）
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
            h.write_u32(unit.hp as u32);
            h.write_u32(unit.cd);
            h.write_u64(unit.x as u64);
        }
        h.finish()
    }

    /// 推进一个 tick（主会话 D5 定稿阶段序，固定不得倒置）：
    /// 1) tick 计数 +1；
    /// 2) tick 级 RNG 固定消耗点——取一个 u64 丢弃（T002 设计保留；无单位级
    ///    RNG 消耗，战斗结算确定性不依赖随机）；
    /// 3) 单 lane 移动（按索引序顺序结算）；
    /// 4) 战斗结算（按索引序：cd 推进 → 索敌 → 射程 → 伤害 → 墓碑；combat 在
    ///    move 之后——同 tick 内先动后打）；
    /// 5) tick 末 `units.retain(|u| u.alive)` 稳定保序清除墓碑（存活者相对序
    ///    不变前移，见模块注释 D6）；
    /// 6) 末尾刷新状态哈希。
    pub fn step(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        let _ = self.rng.next_u64();
        self.move_units();
        self.combat();
        self.units.retain(|u| u.alive);
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
    /// T004 起墓碑在 tick 末 retain 物理移除，移动阶段（下一 tick 起）自然只见
    /// 存活者；`alive` 过滤保留为语义防御。
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

    /// 战斗结算（主会话 D1/D2/D3/D4 定稿；索引序，5.2 纪律）：
    ///
    /// 1) **先**对全部存活单位 `cd = cd.saturating_sub(1)`（cd 冷却制：出手后置
    ///    间隔值，此后每 tick 战斗阶段开头统一递减；未接敌者自然停在 0——
    ///    首次接敌即击。interval=30 语义校验：击于 t、t+30、t+60…）；
    /// 2) **再**按索引序逐单位判定：存活 && 索敌到目标 && 在射程 && `cd == 0`
    ///    → 伤害立即写目标 hp（目标 hp <= 0 → 墓碑，同 tick 内排在后面的单位
    ///    不再选它、它也不再行动），攻击者 `cd = attack_interval_ticks`。
    ///
    /// 索敌（D2）：存活敌方中 |x_j - x_i| 最小者；平局取最小索引（j 自 0 向上扫、
    /// 严格小于才更新，与 move_units 同款模式）；每 tick 重算，O(n²) 朴素可接受。
    /// 射程（D3，M0 全近战口径）：`|dx| <= r_i + r_j + MELEE_MARGIN_Q32`
    /// （闭区间，恰边界可击；1 维无绕后、不区分方向）。
    /// 伤害（D1）：`attack * counter_multiplier / ONE_Q16_16` 截断除法，
    /// 溢出安全见 [`crate::units::damage_dealt`]。
    /// `UnitSpec::range_q32`（D8）本阶段**不读**——远程行为留 T009+ 启用。
    ///
    /// 阶段序（D5）：combat 在 move 之后——同 tick 内先动后打；本阶段产生的
    /// 墓碑由 step 的 retain 物理移除，不出现在本 tick 末哈希与下一 tick 移动中。
    fn combat(&mut self) {
        let n = self.units.len();
        // (1) cd 推进：先于个体判定，对全部存活单位统一 -1（饱和减，0 不下穿）。
        for u in self.units.iter_mut() {
            if u.alive {
                u.cd = u.cd.saturating_sub(1);
            }
        }
        // (2) 个体判定：按索引序。
        for i in 0..n {
            if !self.units[i].alive {
                continue; // 墓碑：不攻击、不可被选
            }
            let x_i = self.units[i].x;
            let side_i = self.units[i].side;
            let kind_i = self.units[i].kind;
            // 索敌：最近存活敌方；平局取最小索引（严格小于才更新）。
            let mut target: Option<(usize, i64)> = None;
            for j in 0..n {
                if j == i || !self.units[j].alive || self.units[j].side == side_i {
                    continue;
                }
                let d = (self.units[j].x - x_i).abs();
                let closer = match target {
                    Some((_, best)) => d < best,
                    None => true,
                };
                if closer {
                    target = Some((j, d));
                }
            }
            let Some((j, target_dist)) = target else {
                continue; // 无存活敌方：cd 保持现值（D4：未接敌不重置）
            };
            // 射程判定（闭区间；溢出安全：距离与半径和均远不及 i64 上界）。
            let radius_sum = spec(kind_i).radius_q32 + spec(self.units[j].kind).radius_q32;
            if target_dist > radius_sum + MELEE_MARGIN_Q32 {
                continue; // 出射程：未出手，cd 不重置（保持 0 或继续衰减）
            }
            if self.units[i].cd != 0 {
                continue; // 冷却中
            }
            // 伤害立即写入目标 hp；目标 hp <= 0 → 墓碑。
            let dmg = damage_dealt(kind_i, self.units[j].kind);
            self.units[j].hp -= dmg;
            self.units[i].cd = spec(kind_i).attack_interval_ticks;
            if self.units[j].hp <= 0 {
                self.units[j].alive = false;
            }
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
            hp: spec(kind).hp,
            cd: 0,
            x,
        }
    }

    /// 带 hp 覆盖的构造（T004 战斗手算例用：一击死 / 打不死等场景直填）。
    fn unit_with_hp(kind: UnitKind, side: Side, x: i64, hp: i32) -> Unit {
        Unit {
            hp,
            ..unit(kind, side, x)
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

    /// 单测 3（验收 2 后半，T004 战斗化改造 G）：贴身停稳定 + 战斗互击回归——
    /// 红蓝各 1 单位相距 3m 相向，贴身后按克制表互击；位置稳定性窗口收窄为
    /// 贴身后的 20 tick（双方均未死亡的窗口内逐 tick 不变）。
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
        w.run(100); // 相向闭合 0.05+0.09=0.14 m/tick，3m 贴身（含 0.2m 余量约 14 tick 可击）
        // (a) 末态间距恰 = 半径和 0.9 m（整数精确，T003 原断言保留）。
        assert_eq!(
            w.units[1].x - w.units[0].x,
            r_r + r_b,
            "末态间距应恰为半径和（贴身停）"
        );
        // (b) t=101..120 仅 20 tick 双方 x 逐 tick 不变（派工单 G：原 100 tick 窗口
        //     收窄）。战斗已进行但双方均未死亡——按锁定克制表：盾兵(Heavy)→民兵
        //     (Unarmored) = 8×43690/65536 = 5/击（interval 30）、民兵→盾兵 =
        //     6×98304/65536 = 9/击（interval 20），50 hp 需 10 击、120 hp 需 14 击，
        //     首个死亡远在窗口外（派工单原估 t≈135 系倍率方向笔误；按锁定表逐击
        //     推演首亡 = 红盾兵于 t=274，见上报）。窗口收窄保留以防御未来数值调整。
        let prev_r = w.units[0].x;
        let prev_b = w.units[1].x;
        for t in 101..=120 {
            w.step();
            assert_eq!(w.units[0].x, prev_r, "tick {t} 红方位置应不变");
            assert_eq!(w.units[1].x, prev_b, "tick {t} 蓝方位置应不变");
        }
        // (c)(d) t=200：双方仍存活（200 tick 内无死亡——首亡实测推演 t≈275），
        // hp 按锁定克制表逐击推演：T0=14（合闭 0.14 m/tick，|dx| 首次落入
        // 0.9+0.2=1.1 m 射程），盾兵击点 14+30k、民兵击点 14+20k。
        //   蓝民兵：被击 14,44,...,194 共 7 击 × 5 = 35 → hp = 50-35 = 15；
        //   红盾兵：被击 14,34,...,194 共 10 击 × 9 = 90 → hp = 120-90 = 30。
        //   【派工单 G(d) 原断言 102 = 120-6×3 系民兵对重甲倍率方向笔误（3 为
        //   Unarmored→Light 方向），按锁定表 9/击修正为 30，已上报。】
        w.run(80); // t=200
        assert_eq!(w.tick, 200);
        assert_eq!(w.unit_count(), 2, "200 tick 内无死亡（双方 hp 均未耗尽）");
        assert_eq!(w.units[0].side, Side::Red);
        assert_eq!(w.units[0].hp, 120 - 10 * 9, "红盾兵 120 - 10击×9");
        assert_eq!(w.units[1].hp, 50 - 7 * 5, "蓝民兵 50 - 7击×5");
        assert_eq!(w.units[0].x, prev_r, "战斗期贴身停位置不变");
        assert_eq!(w.units[1].x, prev_b, "战斗期贴身停位置不变");
    }

    /// 单测 4（T004 战斗化改造 H）：友军不穿插——红方两异速单位前后排 + 远处蓝方
    /// 使前排停住：全程友军间距 >= 半径和（异速不穿插），且后排在前排停后被堵停
    /// （末态间距恰 = 半径和、恒定）。战斗化后双方 hp 手动垫高（6000），使本测的
    /// 被试对象保持为「移动/排队」而非伤亡时间线。
    #[test]
    fn friendly_units_never_interpenetrate_and_queue_behind_stopped_front() {
        // 前排 = 重骑兵（快，0.20 m/tick，r=0.8），后排 = 剑盾兵（慢，0.05 m/tick，r=0.5），
        // 蓝方民兵在 20 m 处（r=0.4）：重骑兵 ~65 tick 后与敌贴身停住。
        // hp 垫高（派工单 H 只垫民兵；骑士必须同垫——按锁定克制表民兵→重甲 =
        // 6×98304/65536 = 9/击 × interval 20，骑士 hp 150 将于 t=385 死亡、落在
        // 400 tick 循环内并破坏索引断言；派工单原存活推演「3/击」系 D1 例 4 同源
        // 倍率方向笔误，已上报。垫 6000：民兵 9/击×20 需 667 击、骑士 9/击×45
        // 需 667 击，均 >> 450 tick，双方全窗口存活）。
        let mut w = manual_world(
            vec![
                unit_with_hp(UnitKind::HeavyKnight, Side::Red, 0, 6000),
                unit(UnitKind::Shieldman, Side::Red, -2 * ONE_Q32_32),
                unit_with_hp(UnitKind::Militia, Side::Blue, 20 * ONE_Q32_32, 6000),
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
    /// PIT-M-002 流程：占位 0 → 实测 → 回填，见 docs/evidence/t004/golden.txt）。
    #[test]
    fn golden_deploy_default_comp_seed42_1800ticks() {
        // T003 旧黄金 0xf2b85bd4727c2d45（= 17489830120905256261）因 T004 战斗上线
        // 失效：D7 哈希折叠扩展（每单位新增 hp 4B + cd 4B）+ 战斗行为改变状态演化。
        // 旧值失效留痕见 docs/evidence/t004/golden.txt；本轮按 PIT-M-002 重走：
        // 占位 0 → 实测 → 回填 → 复跑全绿。
        // 实测回填（PIT-M-002）：占位 0 运行得 10776086108806063401，与 release CLI
        //（--seed 42 默认构成 1800 ticks）输出 0x958c5938c8682529 逐位一致后固化。
        const GOLDEN_HASH: u64 = 10776086108806063401; // 0x958c5938c8682529
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

    /// 单测 B（T004 手算例 1，验收 2）：克制互殴到击杀——红盾兵 vs 蓝长矛兵恰贴身。
    /// 逐 tick 推演表见 docs/evidence/t004/handcalc.md 例 1。
    #[test]
    fn handcalc_pair_counter_fight_to_kill() {
        // 红盾兵 idx0 x=0.5m（r=0.5）、蓝长矛兵 idx1 x=1.5m（r=0.5）：
        // |dx| = 1.0m = 半径和 → 双方 gap=0 全程 0 移动。
        // 伤害：盾兵(Heavy)→长矛(Light) = 8×98304/65536 = 12；长矛(Light)→盾兵(Heavy)
        // = 10×43690/65536 = 6；双方 interval 均 30 → 击点 t = 1, 31, 61, 91, 121, 151。
        let mut w = manual_world(
            vec![
                unit(UnitKind::Shieldman, Side::Red, ONE_Q32_32 / 2),
                unit(UnitKind::Pikeman, Side::Blue, 3 * ONE_Q32_32 / 2),
            ],
            7,
        );
        w.run(151);
        assert_eq!(w.tick, 151);
        // t=151 快照：各 6 击。
        assert_eq!(w.units[1].hp, 80 - 6 * 12, "蓝长矛 hp = 80 - 6击×12 = 8");
        assert_eq!(w.units[0].hp, 120 - 6 * 6, "红盾兵 hp = 120 - 6击×6 = 84");
        assert_eq!(w.units[0].cd, 30, "击后 cd = interval");
        assert_eq!(w.units[1].cd, 30);
        assert_eq!(w.units[0].x, ONE_Q32_32 / 2, "贴身 gap=0，红 x 全程不变");
        assert_eq!(w.units[1].x, 3 * ONE_Q32_32 / 2, "贴身 gap=0，蓝 x 全程不变");
        // t=181：cd 衰减到 0 恰为 181；索引序红先击（蓝 8-12 = -4 → 墓碑），
        // 蓝已死不反击；tick 末 retain 清除 → count=1。
        w.run(30);
        assert_eq!(w.tick, 181);
        assert_eq!(w.unit_count(), 1, "蓝长矛 t=181 死亡并于 tick 末清除");
        assert_eq!(w.units[0].side, Side::Red);
        assert_eq!(w.units[0].hp, 84, "红 hp 不再变化（蓝未反击）");
        assert_eq!(w.units[0].x, ONE_Q32_32 / 2, "t=181 先动后打：蓝尚在，红仍被堵停");
        // t=185：红恢复全速 4 tick（x += 4×0.05；cd 30 → 30-4）。
        w.run(4);
        assert_eq!(w.tick, 185);
        assert_eq!(
            w.units[0].x,
            ONE_Q32_32 / 2 + 4 * (5 * ONE_Q32_32 / 100),
            "红恢复全速 4 tick"
        );
        assert_eq!(w.units[0].cd, 26, "cd = 30 - 4（衰减不出手）");
        assert_eq!(w.units[0].hp, 84);
    }

    /// 单测 C（T004 手算例 2，验收 2）：等距目标决胜——B 不被选中、贴身 A 被击。
    /// 逐 tick 推演表见 docs/evidence/t004/handcalc.md 例 2。
    #[test]
    fn handcalc_equal_distance_target_tiebreak_by_index() {
        // 红民兵 idx0 x=0；蓝民兵 A idx1 x=0.8m（恰贴身）；蓝民兵 B idx2 x=-0.8m。
        let mut w = manual_world(
            vec![
                unit(UnitKind::Militia, Side::Red, 0),
                unit(UnitKind::Militia, Side::Blue, 4 * ONE_Q32_32 / 5),
                unit(UnitKind::Militia, Side::Blue, -(4 * ONE_Q32_32 / 5)),
            ],
            7,
        );
        // t=1 移动：B 前方（-x 向）无人 → 全速退 0.09；红/A 互相贴身 gap=0 不动。
        // t=1 战斗：红索敌最近敌方（战斗时 A 距 0.8、B 已退至 0.89——A 严格最近；
        // 按布阵位口径 0.8=0.8 亦由 idx1 决胜）→ 击 A；A 击红；B 距 0.89 ≤
        // 0.4+0.4+0.2=1.0 仍在射程 → 也击红。三方 interval 均 20。
        w.run(1);
        assert_eq!(w.tick, 1);
        assert_eq!(w.units[0].hp, 50 - 6 - 6, "红被 A、B 各击一次 = 38");
        assert_eq!(w.units[1].hp, 50 - 6, "A 被红击一次 = 44");
        assert_eq!(w.units[2].hp, 50, "B 未被红选中（决胜证据）");
        assert_eq!(w.units[0].cd, 20);
        assert_eq!(w.units[1].cd, 20);
        assert_eq!(w.units[2].cd, 20);
        assert_eq!(
            w.units[2].x,
            -(4 * ONE_Q32_32 / 5) - (9 * ONE_Q32_32 / 100),
            "B 全速后撤一格"
        );
        // t=21：红/A 第二击（cd 20 → 0 恰 t=21）；B 距 0.8+21×0.09 = 2.69 出射程
        // 不击且 cd 自然衰减到 0 后保持 0（未出手不重置）。
        w.run(20);
        assert_eq!(w.tick, 21);
        assert_eq!(w.units[0].hp, 50 - 6 - 6 - 6, "红 38 - 6 = 32（仅 A 击）");
        assert_eq!(w.units[1].hp, 50 - 6 - 6, "A 44 - 6 = 38");
        assert_eq!(w.units[2].hp, 50, "B 仍未被击");
        assert_eq!(
            w.units[2].x,
            -(4 * ONE_Q32_32 / 5) - 21 * (9 * ONE_Q32_32 / 100),
            "B 已后撤 21 格"
        );
        assert_eq!(w.units[2].cd, 0, "B 出射程未出手，cd 保持 0");
    }

    /// 单测 C2（T004 复核补充，主会话裁决）：真等距索敌决胜——worker-2 双盲复算
    /// 上报单测 C 的等距分支实际未触达（D5 先动后打，B 在战斗结算前已移开，
    /// 红方按严格最近命中 A）；本例用链式贴身 5 单位构造使 t=1 移动阶段
    /// **全静止**，红方 idx0 对 A/B 严格等距（0.8m/0.8m）→ 平局取最小索引 A。
    /// python 双盲对拍：docs/evidence/t004/py_recalc/case4.txt。
    #[test]
    fn handcalc_true_equal_distance_tiebreak_chain_contact() {
        // 全民兵（r=0.4m，半径和 0.8m；同类中性 6/击；interval 20）。
        // 索引序：红0 x=0、蓝A1 x=+0.8、蓝B2 x=-0.8、蓝C3 x=-1.6、红D4 x=-2.4。
        let q4_5 = 4 * ONE_Q32_32 / 5; // 0.8m
        let mut w = manual_world(
            vec![
                unit(UnitKind::Militia, Side::Red, 0),
                unit(UnitKind::Militia, Side::Blue, q4_5),
                unit(UnitKind::Militia, Side::Blue, -q4_5),
                unit(UnitKind::Militia, Side::Blue, -2 * q4_5),
                unit(UnitKind::Militia, Side::Red, -3 * q4_5),
            ],
            7,
        );
        // t=1 移动全静止：红0↔A 相向互堵（gap=0）、B 前方（-x 侧）最近者 C gap=0、
        // C↔D 相向互堵——五者皆 gap=0，链式贴身无一人可动。
        // t=1 战斗（索引序，索敌目标）：红0→A（|A|=|B|=0.8 严格等距，平局取
        // idx1——本单测的被测分支）；A→红0（0.8）；B→红0（0.8 < D 的 1.6）；
        // C→D（0.8 < 红0 的 1.6）；D→C（0.8 < B 的 1.6）。
        w.run(1);
        assert_eq!(w.tick, 1);
        assert_eq!(w.unit_count(), 5);
        assert_eq!(w.units[0].hp, 50 - 6 - 6, "红0 被 A、B 各击一次 = 38");
        assert_eq!(w.units[1].hp, 50 - 6, "A 被红0 击 = 44（等距决胜选中 idx1）");
        assert_eq!(w.units[2].hp, 50, "B 未被击——真等距平局取最小索引的证据");
        assert_eq!(w.units[3].hp, 50 - 6, "C 被 D 击 = 44");
        assert_eq!(w.units[4].hp, 50 - 6, "D 被 C 击 = 44");
        for i in 0..5 {
            assert_eq!(w.units[i].cd, 20, "u{i} 出手后 cd = interval");
        }
        assert_eq!(w.units[0].x, 0, "链式贴身全静止");
        assert_eq!(w.units[1].x, q4_5);
        assert_eq!(w.units[2].x, -q4_5);
        assert_eq!(w.units[3].x, -2 * q4_5);
        assert_eq!(w.units[4].x, -3 * q4_5);
        // 哈希黄金（双盲验证后固化）：与 python 独立第二实现 recalc.py
        // `--case 4 --seed 7 --ticks 1` 输出逐位一致（docs/evidence/t004/py_recalc/case4.txt）。
        assert_eq!(w.state_hash(), 0xdcd5_69af_50a2_8656, "case4 t=1 双盲哈希锚");
        // t=21（第二击轮，cd 20 → 0 恰 t=21）：全静止格局不变，目标同 t=1。
        w.run(20);
        assert_eq!(w.tick, 21);
        assert_eq!(w.units[0].hp, 50 - 4 * 6, "红0 26（A、B 各两击）");
        assert_eq!(w.units[1].hp, 50 - 2 * 6, "A 38");
        assert_eq!(w.units[2].hp, 50, "B 始终未被击");
        assert_eq!(w.units[3].hp, 50 - 2 * 6, "C 38");
        assert_eq!(w.units[4].hp, 50 - 2 * 6, "D 38");
        assert_eq!(w.units[0].x, 0, "静止格局维持");
        // 同上 t=21 双盲哈希锚（python `--case 4 --seed 7 --ticks 21` 逐位一致）。
        assert_eq!(w.state_hash(), 0x7769_5db8_b679_c474, "case4 t=21 双盲哈希锚");
    }

    /// 单测 D（T004 手算例 3，验收 2 + 死亡移除）：一击死 → 墓碑 → tick 末清除
    /// 保序压缩（原 idx1→0、idx2→1）。逐 tick 推演表见 docs/evidence/t004/handcalc.md 例 3。
    #[test]
    fn handcalc_death_removal_compacts_indices() {
        // 蓝民兵 idx0 x=1.8m（hp 直填 6 = 一击死）、红民兵 idx1 x=1.0m（恰贴身 0.8）、
        // 红民兵 idx2 x=0（后排距 1.0）。
        let mut w = manual_world(
            vec![
                unit_with_hp(UnitKind::Militia, Side::Blue, 9 * ONE_Q32_32 / 5, 6),
                unit(UnitKind::Militia, Side::Red, ONE_Q32_32),
                unit(UnitKind::Militia, Side::Red, 0),
            ],
            7,
        );
        w.run(1);
        assert_eq!(w.tick, 1);
        // t=1 移动：蓝前 gap=0 停、红1 前 gap=0 停、红2 距红1 1.0m gap=0.2 > 0.09 前进 0.09。
        // t=1 战斗（索引序）：蓝击红1（6 中性）→ 红1 击蓝（6-6=0 墓碑）→ 红2 距蓝
        // 1.71 出射程不击；tick 末清除 → [红1, 红2]，索引前移。
        assert_eq!(w.unit_count(), 2, "蓝民兵死亡并于 tick 末清除");
        assert_eq!(w.units[0].side, Side::Red, "原 idx1 前移为 idx0");
        assert_eq!(w.units[0].hp, 50 - 6, "红1 被蓝先击一次 = 44");
        assert_eq!(w.units[0].cd, 20, "红1 出手后 cd = 20");
        assert_eq!(w.units[0].x, ONE_Q32_32, "红1 gap=0 未动");
        assert_eq!(w.units[1].hp, 50);
        assert_eq!(w.units[1].cd, 0, "红2 出射程未击，cd 保持 0");
        assert_eq!(w.units[1].x, 9 * ONE_Q32_32 / 100, "红2 前进 0.09");
        // t=10：蓝死后红方全速（0.09 m/tick），队形保持、间距恒 0.91m。
        w.run(9);
        assert_eq!(w.tick, 10);
        assert_eq!(
            w.units[0].x,
            ONE_Q32_32 + 9 * (9 * ONE_Q32_32 / 100),
            "红1 t=2..10 共 9 tick 全速"
        );
        assert_eq!(
            w.units[1].x,
            10 * (9 * ONE_Q32_32 / 100),
            "红2 t=1..10 共 10 tick 全速"
        );
        assert_eq!(w.units[0].cd, 11, "20 - 9 tick 衰减");
        assert_eq!(w.units[1].cd, 0);
        assert_eq!(
            w.units[0].x - w.units[1].x,
            ONE_Q32_32 - 9 * ONE_Q32_32 / 100,
            "同速队形间距恒 0.91m"
        );
    }

    /// 单测 E（T004 验收 3 后半）：死亡移除后哈希仍稳定——手算例 1 型构造两个
    /// 独立同参 World，逐 tick step 对拍 state_hash 到 t=200（覆盖死亡 t=181 与
    /// 清除后区间；清除顺序确定性由索引序 retain 保证）。
    #[test]
    fn death_tick_hash_replay_pairwise() {
        let make = || {
            manual_world(
                vec![
                    unit(UnitKind::Shieldman, Side::Red, ONE_Q32_32 / 2),
                    unit(UnitKind::Pikeman, Side::Blue, 3 * ONE_Q32_32 / 2),
                ],
                7,
            )
        };
        let mut a = make();
        let mut b = make();
        for t in 1..=200u64 {
            a.step();
            b.step();
            assert_eq!(a.tick, t, "tick 计数一致");
            assert_eq!(a.state_hash(), b.state_hash(), "tick {t} 状态哈希分歧");
            assert_eq!(a.last_hash, b.last_hash, "tick {t} last_hash 分歧");
        }
        // 末态自证：t=200 已越过死亡（t=181）与清除后区间，蓝已物理移除。
        assert_eq!(a.unit_count(), 1, "蓝长矛已于 t=181 死亡清除");
        assert_eq!(a.units[0].hp, 84, "红盾兵存活 hp");
    }

    /// 单测 F（T004）：近战射程余量常量自检 + 六兵种 range_q32 落值（D8）。
    #[test]
    fn melee_margin_and_range_constants_selfcheck() {
        // MELEE_MARGIN_Q32 = 20 * ONE / 100：20×4294967296 = 85899345920；
        // ÷100 → 商 858993459 余 20 → 截断 858993459。
        assert_eq!(MELEE_MARGIN_Q32, 858_993_459);
        // range_q32 落值（D8）：近战五兵种 0（贴身语义占位），弓箭手 30m。
        // M0 战斗判定不读此字段——本断言仅为字段落值防漂移。
        assert_eq!(spec(UnitKind::Shieldman).range_q32, 0);
        assert_eq!(spec(UnitKind::HeavyKnight).range_q32, 0);
        assert_eq!(spec(UnitKind::Pikeman).range_q32, 0);
        assert_eq!(spec(UnitKind::Swordsman).range_q32, 0);
        assert_eq!(spec(UnitKind::Archer).range_q32, 30 * ONE_Q32_32);
        assert_eq!(spec(UnitKind::Militia).range_q32, 0);
    }
}
