//! 六兵种静态数据表 v0（M0 T003）：类别 / hp / 攻击力 / 攻速 / 移速 / 碰撞半径 / 克制倍率。
//!
//! **表内全部数值为平衡初值，实验场（T009）回归对象。**
//!
//! ## 克制方向留痕（主会话定稿 D1）
//!
//! 报告 V0.9.1 的 3.1 节只定义「重甲/轻甲/无甲」三类**环形克制结构**（原文：
//! 「兵种以『定位 × 克制 × 科技』三维定义：……重甲/轻甲/无甲克制环」），**未显式
//! 落字方向**；唯一语义线索是 2.1 节米拉奇原型「胜负手在重甲/轻甲克制」。按主会话
//! 拍板实现单环：**重甲 克 轻甲、轻甲 克 无甲、无甲 克 重甲**。此方向与倍率均为
//! 平衡初值，是实验场（T009）回归对象。
//!
//! ## 定点数制（主会话定稿 D2）
//!
//! 位置 / 距离 / 半径用 **Q32.32**（i64 底层，1 米 = `ONE_Q32_32`）；克制倍率用
//! **Q16.16**（i32 底层，1.0 = `ONE_Q16_16`）。彻底规避浮点确定性差异，并为 T012
//! python 独立复算提供整数级可对拍性。所有定点常量一律**纯整数表达式**定义——
//! 禁浮点、禁小数字面量、禁 `as` 浮点转换（报告 5.2 / AGENTS.md 硬约束 4）。
//! 整数除法为截断除法，表达式实际值以单测 `fixed_point_constants_selfcheck`
//! 的推导为准。

/// Q32.32 定点的 1.0（= 1 米）。
pub const ONE_Q32_32: i64 = 1 << 32;

/// Q16.16 定点的 1.0（= 倍率 1.0）。
pub const ONE_Q16_16: i32 = 1 << 16;

/// 克制类别（三类单环；环方向见模块注释 D1 留痕）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArmorClass {
    Heavy,
    Light,
    Unarmored,
}

/// 兵种 id（判别值显式固定；判别值以 1 字节进状态哈希，跨版本不得漂移）。
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnitKind {
    Shieldman = 0,   // 剑盾兵
    HeavyKnight = 1, // 重骑兵
    Pikeman = 2,     // 长矛兵
    Swordsman = 3,   // 剑士
    Archer = 4,      // 弓箭手
    Militia = 5,     // 民兵
}

/// 单兵种静态数据 v0。
///
/// `hp / attack / attack_interval_ticks` 在 T003 不参与任何运算（T004 战斗结算启用），
/// 落表即为目的；`speed_q32 / radius_q32` 为 Q32.32 纯整数表达式。
pub struct UnitSpec {
    pub name_zh: &'static str,
    pub hp: i32,
    pub attack: i32,
    /// 攻击间隔（tick/击）。
    pub attack_interval_ticks: u32,
    /// 移速（Q32.32 米/tick）。
    pub speed_q32: i64,
    /// 碰撞半径（Q32.32 米）。
    pub radius_q32: i64,
    pub armor: ArmorClass,
}

/// 六兵种数据表 v0（平衡初值，实验场 T009 回归对象）。
static SPECS: [UnitSpec; 6] = [
    // Shieldman / 剑盾兵：重甲肉盾，慢速。
    UnitSpec {
        name_zh: "剑盾兵",
        hp: 120,
        attack: 8,
        attack_interval_ticks: 30,
        speed_q32: 5 * ONE_Q32_32 / 100, // 0.05 m/tick
        radius_q32: ONE_Q32_32 / 2,      // 0.5 m
        armor: ArmorClass::Heavy,
    },
    // HeavyKnight / 重骑兵：重甲高速突击。
    UnitSpec {
        name_zh: "重骑兵",
        hp: 150,
        attack: 14,
        attack_interval_ticks: 45,
        speed_q32: 20 * ONE_Q32_32 / 100, // 0.20 m/tick
        radius_q32: 4 * ONE_Q32_32 / 5,   // 0.8 m
        armor: ArmorClass::Heavy,
    },
    // Pikeman / 长矛兵：轻甲中坚。
    UnitSpec {
        name_zh: "长矛兵",
        hp: 80,
        attack: 10,
        attack_interval_ticks: 30,
        speed_q32: 10 * ONE_Q32_32 / 100, // 0.10 m/tick
        radius_q32: ONE_Q32_32 / 2,       // 0.5 m
        armor: ArmorClass::Light,
    },
    // Swordsman / 剑士：轻甲快速近战。
    UnitSpec {
        name_zh: "剑士",
        hp: 90,
        attack: 12,
        attack_interval_ticks: 25,
        speed_q32: 12 * ONE_Q32_32 / 100, // 0.12 m/tick
        radius_q32: ONE_Q32_32 / 2,       // 0.5 m
        armor: ArmorClass::Light,
    },
    // Archer / 弓箭手：无甲远程（T003 阶段与近战同规则推进，射程 T004 定）。
    UnitSpec {
        name_zh: "弓箭手",
        hp: 60,
        attack: 9,
        attack_interval_ticks: 60,
        speed_q32: 8 * ONE_Q32_32 / 100, // 0.08 m/tick
        radius_q32: 2 * ONE_Q32_32 / 5,  // 0.4 m
        armor: ArmorClass::Unarmored,
    },
    // Militia / 民兵：无甲廉价炮灰。
    UnitSpec {
        name_zh: "民兵",
        hp: 50,
        attack: 6,
        attack_interval_ticks: 20,
        speed_q32: 9 * ONE_Q32_32 / 100, // 0.09 m/tick
        radius_q32: 2 * ONE_Q32_32 / 5,  // 0.4 m
        armor: ArmorClass::Unarmored,
    },
];

/// 兵种静态数据查询（'static，无运行时借用）。
pub fn spec(kind: UnitKind) -> &'static UnitSpec {
    &SPECS[kind as usize]
}

impl UnitKind {
    /// 英文小写 id（CLI `--comp` 解析与 `--dump-formation` 输出用）。
    pub fn id(self) -> &'static str {
        match self {
            UnitKind::Shieldman => "shieldman",
            UnitKind::HeavyKnight => "heavyknight",
            UnitKind::Pikeman => "pikeman",
            UnitKind::Swordsman => "swordsman",
            UnitKind::Archer => "archer",
            UnitKind::Militia => "militia",
        }
    }

    /// 克制类别归类。
    pub fn armor(self) -> ArmorClass {
        spec(self).armor
    }
}

/// 英文 id → 兵种（未知 id 返回 None；CLI 报错 exit 2 由调用方处理）。
pub fn kind_from_id(id: &str) -> Option<UnitKind> {
    match id {
        "shieldman" => Some(UnitKind::Shieldman),
        "heavyknight" => Some(UnitKind::HeavyKnight),
        "pikeman" => Some(UnitKind::Pikeman),
        "swordsman" => Some(UnitKind::Swordsman),
        "archer" => Some(UnitKind::Archer),
        "militia" => Some(UnitKind::Militia),
        _ => None,
    }
}

/// 克制倍率 3×3 表（Q16.16；行 = 攻方类别，列 = 守方类别）。
///
/// 单环方向（D1 留痕）：重甲→轻甲、轻甲→无甲、无甲→重甲 为克制（×1.5）；
/// 反向为被克（×0.67 ≈ 2/3 定点化）；同类中性（×1.0）。
/// 倍率纯整数表达式：×1.5 = `3 * ONE_Q16_16 / 2`（整除恰为 98_304）；
/// 被克 = `2 * ONE_Q16_16 / 3`（131_072 = 3×43_690 + 2，截断 = 43_690，≈0.66669）。
static COUNTER_TABLE: [[i32; 3]; 3] = [
    // 列序：Heavy, Light, Unarmored
    [ONE_Q16_16, 3 * ONE_Q16_16 / 2, 2 * ONE_Q16_16 / 3], // 攻方 Heavy：克 Light
    [2 * ONE_Q16_16 / 3, ONE_Q16_16, 3 * ONE_Q16_16 / 2], // 攻方 Light：克 Unarmored
    [3 * ONE_Q16_16 / 2, 2 * ONE_Q16_16 / 3, ONE_Q16_16], // 攻方 Unarmored：克 Heavy
];

/// 克制倍率查询（Q16.16；T004 战斗结算使用，T003 仅落表 + 单测穷举）。
pub fn counter_multiplier(attacker: ArmorClass, defender: ArmorClass) -> i32 {
    COUNTER_TABLE[attacker as usize][defender as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 单测 1：克制表 3×3 穷举 + 6 兵种归类。
    #[test]
    fn counter_table_3x3_exhaustive_and_armor_classification() {
        // 期望值全部写明推导（Q16.16：ONE = 65_536）：
        //   中性 ×1.0 = 65_536；
        //   克制 ×1.5 = 3*65_536/2 = 196_608/2 = 98_304（整除精确）；
        //   被克 ×0.67≈2/3 = 2*65_536/3 = 131_072/3，商 43_690 余 2 → 截断 43_690。
        const NEUTRAL: i32 = 65_536;
        const COUNTER: i32 = 98_304; // 3 * ONE / 2
        const COUNTERED: i32 = 43_690; // 2 * ONE / 3（截断）
        let expected = [
            // 行 = 攻方，列 = 守方：Heavy, Light, Unarmored
            [NEUTRAL, COUNTER, COUNTERED],
            [COUNTERED, NEUTRAL, COUNTER],
            [COUNTER, COUNTERED, NEUTRAL],
        ];
        let classes = [ArmorClass::Heavy, ArmorClass::Light, ArmorClass::Unarmored];
        for (ai, &a) in classes.iter().enumerate() {
            for (di, &d) in classes.iter().enumerate() {
                assert_eq!(
                    counter_multiplier(a, d),
                    expected[ai][di],
                    "counter_multiplier({a:?}, {d:?})"
                );
            }
        }
        // 单环方向语义复核：Heavy 克 Light、Light 克 Unarmored、Unarmored 克 Heavy；
        // 同类 9 格中的 3 个对角格 = 中性。
        assert_eq!(counter_multiplier(ArmorClass::Heavy, ArmorClass::Light), COUNTER);
        assert_eq!(counter_multiplier(ArmorClass::Light, ArmorClass::Unarmored), COUNTER);
        assert_eq!(counter_multiplier(ArmorClass::Unarmored, ArmorClass::Heavy), COUNTER);
        for c in classes {
            assert_eq!(counter_multiplier(c, c), NEUTRAL);
        }
        // 6 兵种归类各正确。
        assert_eq!(UnitKind::Shieldman.armor(), ArmorClass::Heavy);
        assert_eq!(UnitKind::HeavyKnight.armor(), ArmorClass::Heavy);
        assert_eq!(UnitKind::Pikeman.armor(), ArmorClass::Light);
        assert_eq!(UnitKind::Swordsman.armor(), ArmorClass::Light);
        assert_eq!(UnitKind::Archer.armor(), ArmorClass::Unarmored);
        assert_eq!(UnitKind::Militia.armor(), ArmorClass::Unarmored);
    }

    /// 单测 9：定点常量自检（纯整数算术，推导写明；非凭记忆引用）。
    #[test]
    fn fixed_point_constants_selfcheck() {
        // Q32.32：ONE = 2^32 = 4_294_967_296。
        assert_eq!(ONE_Q32_32, 4_294_967_296);
        // 5 m 的 Q32.32 = 5 * 2^32 = 21_474_836_480；÷100：商 214_748_364 余 80 → 截断 214_748_364。
        assert_eq!(5 * ONE_Q32_32 / 100, 214_748_364);
        // 0.5 m = ONE/2：2_147_483_648（整除精确）。
        assert_eq!(ONE_Q32_32 / 2, 2_147_483_648);
        // Q16.16：ONE = 2^16 = 65_536。
        assert_eq!(ONE_Q16_16, 65_536);
        // ×1.5 = 3 * 65_536 / 2 = 196_608 / 2 = 98_304（整除精确）。
        assert_eq!(3 * ONE_Q16_16 / 2, 98_304);
        // ×2/3 = 131_072 / 3：商 43_690 余 2 → 截断 43_690。
        assert_eq!(2 * ONE_Q16_16 / 3, 43_690);
        // 六兵种移速/半径表达式落值（推导同上，均为 N * ONE / D 截断除法）：
        // Shieldman 0.05 m/tick、HeavyKnight 0.20、Pikeman 0.10、Swordsman 0.12、
        // Archer 0.08、Militia 0.09；半径 0.5 / 0.8 / 0.4 m。
        assert_eq!(spec(UnitKind::Shieldman).speed_q32, 214_748_364); // 5/100
        assert_eq!(spec(UnitKind::HeavyKnight).speed_q32, 858_993_459); // 20/100：20*2^32=85_899_345_920/100 商 858_993_459 余 20
        assert_eq!(spec(UnitKind::Pikeman).speed_q32, 429_496_729); // 10/100：42_949_672_960/100 商 429_496_729 余 60
        assert_eq!(spec(UnitKind::Swordsman).speed_q32, 515_396_075); // 12/100：51_539_607_552/100 商 515_396_075 余 52
        assert_eq!(spec(UnitKind::Archer).speed_q32, 343_597_383); // 8/100：34_359_738_368/100 商 343_597_383 余 68
        assert_eq!(spec(UnitKind::Militia).speed_q32, 386_547_056); // 9/100：38_654_705_664/100 商 386_547_056 余 64
        assert_eq!(spec(UnitKind::Shieldman).radius_q32, 2_147_483_648); // ONE/2
        assert_eq!(spec(UnitKind::HeavyKnight).radius_q32, 3_435_973_836); // 4/5：17_179_869_184/5 商 3_435_973_836 余 4
        assert_eq!(spec(UnitKind::Pikeman).radius_q32, 2_147_483_648); // ONE/2
        assert_eq!(spec(UnitKind::Swordsman).radius_q32, 2_147_483_648); // ONE/2
        assert_eq!(spec(UnitKind::Archer).radius_q32, 1_717_986_918); // 2/5：8_589_934_592/5 商 1_717_986_918 余 2
        assert_eq!(spec(UnitKind::Militia).radius_q32, 1_717_986_918); // 2/5
        // id 往返：kind_from_id(kind.id()) 恒等（CLI 解析一致性）。
        for kind in [
            UnitKind::Shieldman,
            UnitKind::HeavyKnight,
            UnitKind::Pikeman,
            UnitKind::Swordsman,
            UnitKind::Archer,
            UnitKind::Militia,
        ] {
            assert_eq!(kind_from_id(kind.id()), Some(kind));
        }
        // 判别值显式固定（状态哈希 kind 1 字节的跨版本稳定性）。
        assert_eq!(UnitKind::Shieldman as u8, 0);
        assert_eq!(UnitKind::HeavyKnight as u8, 1);
        assert_eq!(UnitKind::Pikeman as u8, 2);
        assert_eq!(UnitKind::Swordsman as u8, 3);
        assert_eq!(UnitKind::Archer as u8, 4);
        assert_eq!(UnitKind::Militia as u8, 5);
    }
}
