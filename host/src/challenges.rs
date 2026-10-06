//! host 侧挑战预设注册表（T022/D1，M5 席位 7）——**数据化挑战三件套**：构成 +
//! 参数（lane_len_m / max_ticks / seed）+ 期望终局断言（[`ExpectedOutcome`] 六字段）。
//! 与 [`crate::presets`]（布阵底座四元组，不含 seed/终局）分工：挑战另钉 seed 与
//! 期望终局；判定经 [`crate::suite`] 的 `challenges` 套件行使（D3/D4：每挑战一条
//! `challenge::<name>` 断言，判定行代码生成，期望值只在常量的单一来源）。
//!
//! ## 三挑战语义（题名 + 叙事）
//!
//! - `few-elite` 以少胜多·剑士绞肉：10 剑士 vs 30 民兵——轻甲克无甲（剑士→民兵
//!   18/击）以克制对冲人数劣势；**灭绝收束**样例（end_tick < max_ticks）。
//! - `counter-militia` 无甲克重甲·农兵拒骑：10 重骑 vs 30 民兵——克制极端例
//!   （无甲克重甲 ×1.5 / 重甲攻无甲被克 ×0.67，双向翻转）；**上限 hp 判定**。
//! - `iron-wall` 铁壁僵局·盾墙不死：20 盾兵 vs 60 民兵——低伤对撞灭绝不可达；
//!   **上限 hp 判定**（= T021 last-stand 实测 15/56 转正）。
//!
//! ## 终局语义分类（判定口径）
//!
//! - 灭绝收束：任一方存活归零即冻结于该 tick（end_tick < max_ticks，败方 alive=0）。
//! - 上限 hp 判定：至 max_ticks 未灭绝 → 按双方存活总 hp 判胜负（end_tick == max_ticks）。
//!
//! ## 锚来源（Lead 2026-10-07 实测定死；文档指针，不复制正文）
//!
//! - seed 42：`docs/evidence/t022/design/measure-candidates-run.log`
//!   （#1 L8/L11、#2 L32/L35、#3 L56/L59；行号已 `grep -n` 核对）。
//! - seed 43 敏感性：`docs/evidence/t022/design/measure-seed43-run.log`（L6/L12/L18）。
//!
//! ## seed 43 设计注（不作断言）
//!
//! 三构型 seed 43 实测 win/end_tick/存活数与 seed 42 **完全相同**（仅哈希不同：
//! #1 `0x51475d4b497f541a`、#2 `0x398a2e9baf9e4d35`、#3 `0x356d3b0a9bba9a09`）——
//! 挑战叙事跨种子稳健；断言锚只钉 seed 42（挑战语义 = 固定种子逐位复现，任务卡
//! 断言 3）。
//!
//! ## #2 敏感度旁证注
//!
//! 同构换量 ch2b（民兵 20，其余同 #2）实测**红胜**（`0xb3215593b1fbcde5`，
//! measure-candidates-run.log:43）——数量翻盘点在 20~30 之间（hp 池翻转）；#2 取
//! 30 钉蓝胜锚。机制：民兵→重骑 6×1.5 = 9/击 × 20t、重骑→民兵 14×0.67 = 9/击
//! × 45t（克制倍率表 sim/src/units.rs:190-195，倍率手臂截断）。
//!
//! ## #1 灭绝预算算式（B.2 ② 接敌可达性口径；算式仅供核对、以实测为准）
//!
//! 贴身接敌面恒 1v1 漏斗（同侧队列间距 > 射程阈值 ⇒ 后位不越前位；T021 判定注
//! 口径，move 向前夹紧不越位 sim/src/world.rs:771-775）⇒ 剑士→民兵 dmg =
//! 12 × 1.5 = 18/击（units.rs:190-195 克制表），民兵 50hp ⇒ ⌈50/18⌉ = 3 击 ×
//! 攻击间隔 25t = 75t/杀；30 杀 × 75t = 2250t + 队列推进/接敌开销 ≈ 实测 2474
//! < max_ticks 3600 ✓。

use sim::units::UnitKind;

/// 挑战定义（D1）：构成 + 参数 + 期望终局；含固定 seed（D2——挑战语义 = 固定
/// 种子逐位复现；threads 不入定义，判定路径固定串行 = suite 现状）。
pub struct ChallengeDef {
    /// 注册表 id（套件断言名后缀：`challenge::<name>`）。
    pub name: &'static str,
    /// 中文短题名（数据档文案；套件判定不读）。
    #[allow(dead_code)] // 数据档字段：证据 README / 后续观战面展示用，当前无代码读者。
    pub title_zh: &'static str,
    /// 一句话题面（挑战叙事；套件判定不读）。
    #[allow(dead_code)] // 同上（数据档字段，结构性豁免）。
    pub brief_zh: &'static str,
    /// 红方构成（kind, count 清单；顺序 = 布阵展开顺序）。
    pub red: Vec<(UnitKind, usize)>,
    /// 蓝方构成。
    pub blue: Vec<(UnitKind, usize)>,
    /// lane 全长（米，≥ 1）。
    pub lane_len_m: i64,
    /// tick 上限（1..=14400）。
    pub max_ticks: u64,
    /// 布阵/模拟种子（固定 42——见模块头注 seed 43 设计注）。
    pub seed: u64,
    /// 期望终局锚（六字段；实测来源见模块头注）。
    pub expected: ExpectedOutcome,
}

impl ChallengeDef {
    /// suite 判定行名（D4）：`challenge::<name>`——字面量直书（`AssertionResult.name`
    /// 为 `&'static str`，不支持运行时拼接；注册表封闭，`_` 臂不可达）。
    pub fn assertion_name(&self) -> &'static str {
        match self.name {
            "few-elite" => "challenge::few-elite",
            "counter-militia" => "challenge::counter-militia",
            "iron-wall" => "challenge::iron-wall",
            _ => "challenge::<unknown>",
        }
    }
}

/// 期望终局锚（D1）：布阵快照哈希 + 终局五字段（与 BRP deploy/outcome 响应同域）。
pub struct ExpectedOutcome {
    /// 胜方标签（与 `sim::world::Winner::label()`、BRP outcome `winner` 同串）。
    pub winner: &'static str,
    /// 结束 tick（灭绝收束 < max_ticks；上限判定 == max_ticks）。
    pub end_tick: u64,
    /// 红方存活数。
    pub alive_red: u32,
    /// 蓝方存活数。
    pub alive_blue: u32,
    /// 终局状态哈希（BRP `final_hash`）。
    pub final_hash: u64,
    /// 布阵快照哈希（BRP `deploy_hash`，即 tick 0 的 last_hash）。
    pub deploy_hash: u64,
}

/// 挑战名清单（注册表序；套件判定序与错误消息用）。
pub fn names() -> &'static [&'static str] {
    &["few-elite", "counter-militia", "iron-wall"]
}

/// 按名取挑战（未知名 → None）。
pub fn get(name: &str) -> Option<ChallengeDef> {
    match name {
        // #1 以少胜多·剑士绞肉（灭绝收束）。
        // 锚：measure-candidates-run.log L8（deploy 0x583ddfef1fc446cf）/
        // L11（outcome red 2474 7/0 0xa48ce79b1e83e105）——十六进制字面量逐字
        // 使用勿换算。
        "few-elite" => Some(ChallengeDef {
            name: "few-elite",
            title_zh: "以少胜多·剑士绞肉",
            brief_zh: "10 剑士 vs 30 民兵：轻甲克无甲换人数劣势，灭绝收束红胜。",
            red: vec![(UnitKind::Swordsman, 10)],
            blue: vec![(UnitKind::Militia, 30)],
            lane_len_m: 60,
            max_ticks: 3600,
            seed: 42,
            expected: ExpectedOutcome {
                winner: "red",
                end_tick: 2474,
                alive_red: 7,
                alive_blue: 0,
                final_hash: 0xa48ce79b1e83e105,
                deploy_hash: 0x583ddfef1fc446cf,
            },
        }),
        // #2 无甲克重甲·农兵拒骑（上限 hp 判定；敏感度旁证见模块头注）。
        // 锚：measure-candidates-run.log L32（deploy 0x6a96df6f3c94e741）/
        // L35（outcome blue 1800 6/24 0x3247aae383d5d5bd）。
        "counter-militia" => Some(ChallengeDef {
            name: "counter-militia",
            title_zh: "无甲克重甲·农兵拒骑",
            brief_zh: "10 重骑 vs 30 民兵：克制双向翻转的极端例（数量翻盘点 20~30 之间）。",
            red: vec![(UnitKind::HeavyKnight, 10)],
            blue: vec![(UnitKind::Militia, 30)],
            lane_len_m: 60,
            max_ticks: 1800,
            seed: 42,
            expected: ExpectedOutcome {
                winner: "blue",
                end_tick: 1800,
                alive_red: 6,
                alive_blue: 24,
                final_hash: 0x3247aae383d5d5bd,
                deploy_hash: 0x6a96df6f3c94e741,
            },
        }),
        // #3 铁壁僵局·盾墙不死（上限 hp 判定；= T021 last-stand seed7 实测
        // 15/56 转正——本挑战改钉 seed 42，锚见 measure-candidates-run.log）。
        // 锚：measure-candidates-run.log L56（deploy 0x9a926533cbf1d445）/
        // L59（outcome blue 1800 15/56 0x81166458a5437951）。
        "iron-wall" => Some(ChallengeDef {
            name: "iron-wall",
            title_zh: "铁壁僵局·盾墙不死",
            brief_zh: "20 盾兵 vs 60 民兵：低伤对撞灭绝不可达，上限 hp 判定。",
            red: vec![(UnitKind::Shieldman, 20)],
            blue: vec![(UnitKind::Militia, 60)],
            lane_len_m: 60,
            max_ticks: 1800,
            seed: 42,
            expected: ExpectedOutcome {
                winner: "blue",
                end_tick: 1800,
                alive_red: 15,
                alive_blue: 56,
                final_hash: 0x81166458a5437951,
                deploy_hash: 0x9a926533cbf1d445,
            },
        }),
        _ => None,
    }
}
