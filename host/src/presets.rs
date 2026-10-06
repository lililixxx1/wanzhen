//! host 侧预设注册表（T021/D2）：数据化预设 = 布阵底座（red / blue / lane_len_m /
//! max_ticks），**不含 seed / threads**——种子与执行细节不属预设（派工单 D2 字面）。
//!
//! 预设是 BRP `game.deploy` 的 `"preset"` 参数与 CLI `--preset` 的单一来源；
//! 「显式字段一律覆盖预设底座」的覆盖语义分别在 [`crate::rpc::deploy_handler`]（BRP）
//! 与 host CLI 解析段实现，本模块只存数据（D2/D3/D4 分工）。
//!
//! ## 接敌可达性算式注（附录 B.2 ②口径，数值带 sim 源行号、以算式为准）
//!
//! 算式：（双方队头距离 − 射程阈值）÷ 合闭合速度 ≤ 判定窗口 tick。
//!
//! - 射程阈值（M0 贴身口径）= r_i + r_j + `MELEE_MARGIN_Q32`(0.2 m，
//!   sim/src/units.rs:34)；判定式 sim/src/world.rs:408-412（闭区间）。
//! - 布阵队头 x = radius_0、蓝方镜像（sim/src/world.rs:601-622）⇒ 双方队头
//!   初始距离 ≈ lane_len_m − (r红头 + r蓝头)；布阵洗牌（Fisher-Yates）使混排
//!   构成的队头兵种随 seed 变化 ⇒ 最坏界取最慢闭合对。
//! - move 意图向前夹紧不越位（gap clamp，sim/src/world.rs:765-776）⇒ 接敌面
//!   前沿闭合速度 = 双方队头移速之和。
//!
//! ### default（lane 1000 m）：1800 tick 内不接敌（结构性）
//! 队头初始距离 ≥ 1000 − 2×0.8 = 998.4 m（最大半径重骑 0.8 m，units.rs:97）；
//! 射程阈值 ≤ 0.8+0.8+0.2 = 1.8 m；最快闭合对剑士 0.12+0.12 = 0.24 m/tick
//! （units.rs:118）⇒ 接敌需 ≥ (998.4 − 1.8) ÷ 0.24 ≈ 4152 tick > 1800。
//! 与 T018 黄金锚②「对称未接敌 Draw@1800」一致——本预设保持与 M0 默认构成
//! 逐位同构（直接引用 sim `DEFAULT_COMPOSITION` 构造，D2）。
//!
//! ### melee-brawl（lane 100 m）：接敌 ≤ 546 tick < 1800 ✓
//! 最坏（最慢）闭合对民兵×民兵：距离 100 − (0.4+0.4) = 99.2 m（民兵半径
//! 0.4，units.rs:141）、阈值 0.4+0.4+0.2 = 1.0 m、闭合 0.09+0.09 = 0.18 m/tick
//! （units.rs:140）⇒ (99.2 − 1.0) ÷ 0.18 ≈ 545.6 → ≤ 546 tick。最快对剑士×
//! 剑士 (99.0 − 1.2) ÷ 0.24 ≈ 407.5 → ≤ 408 tick。任意 seed 接敌可达。
//!
//! ### last-stand（lane 60 m）：接敌 ≤ 415 tick < 1800 ✓；**灭绝 < 1800 不可达**
//! 接敌：盾兵(0.05 m/tick，units.rs:85)×民兵(0.09，units.rs:140)，距离
//! 60 − (0.5+0.4) = 59.1 m、阈值 0.5+0.4+0.2 = 1.1 m ⇒ (59.1 − 1.1) ÷ 0.14
//! ≈ 414.3 → ≤ 415 tick。
//! 灭绝：贴身口径 + 向前夹紧不越位 ⇒ 同侧队列（盾兵间距 0.5+0.5+0.5 =
//! 1.5 m、民兵 0.4+0.4+0.5 = 1.3 m）后位始终在前位射程（1.1 m）之外，接敌面
//! 恒 1v1 漏斗：民兵 9 dmg/20t（无甲克重甲 ×1.5，units.rs:190-195 倍率表 +
//! units.rs:312 单测锚）杀盾兵 ≈ 120/9×20 = 280 tick/个，盾兵 5 dmg/30t
//! （重甲被无甲克反 ×0.67 截断，units.rs:209-213）杀民兵 ≈ 50/5×30 = 300
//! tick/个 ⇒ 1800 tick 内双方均远未灭绝。基线 host BRP 实测（2026-10-06，
//! seed=7/lane 60/threads=1，未改任何代码）：run_to_tick 1800 → tick=1800
//! 未冻结，alive 15/56，winner=blue（走 sim run_battle_with 上限收束
//! resolve_by_hp 总 hp 判定）——见 docs/evidence/t021/README.md 上报节。

use sim::units::UnitKind;
use sim::world::{DEFAULT_COMPOSITION, TICK_CAP_REDUCED};

use crate::rpc::DEFAULT_LANE_LEN_M;

/// 预设定义（D2）：布阵底座四元组；不含 seed / threads。
pub struct PresetDef {
    /// 红方构成（kind, count 清单；顺序 = 布阵展开顺序）。
    pub red: Vec<(UnitKind, usize)>,
    /// 蓝方构成。
    pub blue: Vec<(UnitKind, usize)>,
    /// lane 全长（米，≥ 1）。
    pub lane_len_m: i64,
    /// tick 上限（1..=14400）。
    pub max_ticks: u64,
}

/// 预设名清单（注册表序；错误消息与 CLI usage 用）。
pub fn names() -> &'static [&'static str] {
    &["default", "melee-brawl", "last-stand"]
}

/// 按名取预设（未知名 → None，调用方报 INVALID_PARAMS / CLI exit 2）。
pub fn get(name: &str) -> Option<PresetDef> {
    match name {
        // default = M0 默认构成（六兵种各 5×2）+ 默认 lane + 降规模 tick 上限，
        // 与 sim CLI 缺省布阵同构（D2：直接引用 sim 常量构造）。
        "default" => Some(PresetDef {
            red: DEFAULT_COMPOSITION.to_vec(),
            blue: DEFAULT_COMPOSITION.to_vec(),
            lane_len_m: DEFAULT_LANE_LEN_M,
            max_ticks: TICK_CAP_REDUCED,
        }),
        // melee-brawl：快节奏近战混编对撞（D2 字面数值；可达性算式见模块注释）。
        "melee-brawl" => Some(PresetDef {
            red: vec![(UnitKind::Swordsman, 15), (UnitKind::Militia, 15)],
            blue: vec![(UnitKind::Swordsman, 15), (UnitKind::Militia, 15)],
            lane_len_m: 100,
            max_ticks: TICK_CAP_REDUCED,
        }),
        // last-stand：以少胜多原型（T022 地基示例，D2 字面数值）。注：当前
        // 数值表下模拟语义为蓝方（人数 + 克制双优）上限判定胜、灭绝 < 1800
        // 不可达（模块注释「灭绝」节 + 实测留痕）——「以少胜多」为设计原型名，
        // 非当前数值表的模拟结论；数值表 = T022 平衡回归对象。
        "last-stand" => Some(PresetDef {
            red: vec![(UnitKind::Shieldman, 20)],
            blue: vec![(UnitKind::Militia, 60)],
            lane_len_m: 60,
            max_ticks: TICK_CAP_REDUCED,
        }),
        _ => None,
    }
}
