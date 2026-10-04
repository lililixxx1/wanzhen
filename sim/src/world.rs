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
//! ## T005 扩展（主会话 D1~D12 定稿）：胜负判定与终局
//!
//! [`World::run_battle`]（D4）单局收束：全灭判定**每 tick 求值且优先于上限**
//! （一方 0 → 对方胜；双方 0 → Draw，仅 tick 0 双空构型可达——同 tick 双灭
//! 不可达推演见 [`Winner`] 注释 D10）；上限收束（tick == max_ticks 仍双方有
//! 存活）按每方存活总 hp 判定（索引序 i64 累加，5.2 固定序纪律；高者胜、
//! 同值 Draw）。tick 0（首 step 前）也求值一次——空阵营边界（0 单位一方）
//! 立即收束 end_tick=0，不 panic。终局四元组（[`BattleOutcome`]）存入
//! [`World::resolved`] 并**冻结**：再次 run_battle 幂等返回缓存、tick 不再
//! 推进。`resolved` **不进 state_hash、不影响 step/run**——哈希折叠序零变化，
//! T002/T004 黄金锚原值保持（单测 battle_golden_crosscheck_default_comp 以
//! T004 黄金 0x958c5938c8682529 实证）。final_hash 一律终局点现算
//! [`World::state_hash`]（不读 last_hash：tick 0 直构 manual world 的
//! last_hash=0，现算才语义正确；step 过的路径上两者相等）。[`BattleLog`]（D5）
//! 为战斗日志最小字段（seed / 双方构成 / 终局四元组；Display 固定 8 行确定性
//! 内容；不含落库——表 6-0 吞吐口径不含落库）。
//!
//! ## T006 扩展（主会话 D1~D12 定稿）：意图并行两阶段更新
//!
//! [`World::step_with`] 单一实现两阶段重构（D2）：**阶段 1 意图计算**——读该
//! 阶段开始时刻的快照（[`ScanInput`]，从 `&[Unit]` 一次克隆），per-i 纯函数、
//! 无共享写、可分片并行；**阶段 2 固定索引序串行应用**。threads=1 与多线程
//! 走**同一代码路径**（只有意图阶段的执行器不同：`None` = 主线程直接循环整段；
//! `Some(&pool)` = 经 [`crate::pool`] 连续均摊分片并行，片界
//! [`crate::pool::chunk_range`]，结果按片 start 升序拼装 = 索引序，分片与结果
//! 无关）。阶段序不变：tick → RNG 消耗 → move（意图→应用）→ combat
//! （意图→应用）→ retain → hash。
//!
//! 「Intent 缓冲 + 固定索引序 apply」为本模块两阶段更新模式（任务卡 D5 留痕）：
//! move 与 combat 是两个参照实例，未来士气传播按同型接入（跨单位间接影响 =
//! 先并行写标记、下一步按固定序应用）；**不抽泛型**（T009+ 再抽象）。
//!
//! ### move 两阶段（D3）与等价性
//!
//! - 意图（读 move 开始时快照，per 存活 i）：`front(i)` = 运动方向上最近存活者
//!   （`(x_j - x_i) * dir_i > 0` 中该值最小者，平局取最小索引 j）。
//! - 应用（索引序串行，读最新位置）：`d_i` = 无前方者 ? `speed_i`
//!   : `min(speed_i, max((x[f]_now - x_i)·dir_i - (r_i + r_f), 0))`；`x_i += dir_i * d_i`
//!   （差值必须乘 dir 方向感知——蓝方 dir=−1 裸差值恒负即恒停，首轮门禁实测
//!   教训见 parity-notes §5）。
//!
//! 等价性（对原 T004 串行版逐点一致，构造性论证）：
//! ① **前方身份恒定**：快照时刻严格在前者全程仍在前——每单位前进量被 clamp 于
//! 与其前方者的当前正间隙，中心不越过任何「快照时刻在其前方」单位的中心（对更
//! 远者间隙更小 ⇒ 更不越过）；严格在后者全程仍在后（对称同理）；快照时刻重合的
//! 对（`(x_j-x_i)*dir == 0`）互不为前方者、各走各的。⇒ 应用阶段用快照 front 与
//! 原「每 i 现算最近者」是**同一个单位**（这是并行意图可行的根据）。
//! ② **clamp 式与原版同构**：原版 `d_i = min(speed, max(当前 gap, 0))`，由 ①
//! 「当前 gap」即以快照 front 的**最新位置**计算之 gap ⇒ 阶段 2 逐字复现原式。
//! ③ **相向穿插防御内生于 clamp**：后结算者被先结算者已更新位置压停（原版语义
//! 正是如此；单测 two_phase_move_no_interpenetration_head_on 专查——若实现误用
//! 旧快照位置直接应用，相向单位将穿插、位置序断言即炸）。
//!
//! **对派工单 D3 应用式的一处修正（上报留痕，按 §5「算式与实测冲突按实测执行」）**：
//! D3 原文应用式为 `min(desire_i, max(x[f]_now - …, 0))`，其中
//! `desire_i = min(speed_i, max(gap_old, 0))` 由快照间隙预折算。该式在「同侧前方
//! 者先结算且本 tick 前进」时（gap_now = gap_old + d_f > gap_old，快兵追慢兵的
//! 贴身瞬态）把本 tick 前进截断在 last-tick 间隙上，与原串行
//! `min(speed, gap_now⁺)` 不一致；默认构成 deploy 为混速洗牌队列，追近瞬态
//! 必现 ⇒ T004 黄金锚 0x958c5938c8682529 失守。修正：应用式保留原版
//! `min(speed_i, gap_now⁺)`（desire 预折算不入意图缓冲；上方 ② 即修正后论证），
//! 黄金锚实测复现见 docs/evidence/t006/。
//!
//! **等价域注记**：快照时刻存在「同侧同位置重合对」时前方身份可翻转（重合对互
//! 不为前方，先行者前进后成为严格在前），该域内与原串行可分歧。生产布阵
//! （deploy 严格分离）与移动 clamp（间隙 ≥ 0、中心不越过）保持「严格分离」
//! 不变式逐 tick 传递，该域不可达；`World::new` 裸路径（N 个重合占位，T002
//! 冒烟原语）不承载任何哈希证据，留痕于此。
//!
//! ### combat 两阶段（D4）与等价性引理
//!
//! - 意图（读 combat 开始时快照 = move 应用后、retain 前；per 存活 i）：
//!   `target_old(i)` = 最近存活敌方（`|x_j - x_i|` 最小，平局取最小索引）；
//!   `in_range_old(i)` = target_old 存在时 `|dx| <= r_i + r_t + MELEE_MARGIN_Q32`
//!   （闭区间，恰边界可击）。
//! - 应用（索引序串行）：1) 先对全部存活单位 `cd = cd.saturating_sub(1)`
//!   （原样，串行侧）；2) 逐 i：墓碑 skip；target_old 仍存活 → 用之（射程用
//!   预计算 in_range_old）；已死 → **重索敌**（完整重扫当前墓碑状态，同平局
//!   规则、射程现算——保障「先手击杀剥夺后手目标选择」语义）；无目标 → skip
//!   （cd 保持）；射程不过 → skip；`cd[i] != 0` → skip；出手：`hp[t] -=
//!   damage_dealt(kind_i, kind_t)`、`cd[i] = attack_interval_ticks_i`、
//!   `hp[t] <= 0 → alive[t] = false`（墓碑即时生效）。伤害值不预计算（应用时
//!   O(1) 现算，重索敌换目标后自动正确）。
//!
//! 等价性引理（D4 留痕）：combat 阶段位置冻结（本阶段不动位置）⇒ 索敌结果仅
//! 依赖存活集；存活集只缩不增；旧集 argmin（按 (距离, 索引) 字典序）若仍存活
//! 则必为当前集 argmin（移除元素不影响存活最小值，平局序不变）；若已死则重索敌
//! ≡ 原版实时索敌同式。cd 只被本单位出手重置（全局递减对全体一致）⇒ 意图预读
//! 无竞争。
//!
//! ### 快照与不变式
//!
//! [`ScanInput`] = { x, alive, side, kind }（判别 u8），从 `&[Unit]` 一次克隆；
//! 每 tick 两次快照（move 意图前、combat 意图前——combat 快照反映 move 后位置）。
//! 意图阶段开头 `debug_assert!(units 全存活)`（retain 后不变式自检，release 零
//! 成本）。意图闭包只捕获 `Arc<ScanInput>`（owned 克隆数据）满足 'static，
//! **零 unsafe**；f 不 panic 契约见 [`crate::pool`]。
//!
//! 确定性纪律：无浮点 / 无超越函数 / 无挂钟 / 无 HashMap；
//! 遍历一律按索引序；同 seed + 同参数 + 同 tick 数 → 状态逐位一致、与线程数
//! 无关。位置为 Q32.32 定点整数（[`crate::units::ONE_Q32_32`]），纯整数运算无舍入。

use std::sync::Arc;

use crate::hash::Fnv1a;
use crate::pool::ThreadPool;
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

/// 降规模对局 tick 上限（报告表 6-0 直接落字：单局 ≤ 60s（≤ 1,800 ticks @30Hz））。
pub const TICK_CAP_REDUCED: u64 = 1800;
/// 全规模对局 tick 上限（表 6-0 无全规模 tick 上限显式字段；按产品口径
/// 「单局时长 3–8 分钟」上限 8 分钟 @30Hz 换算钉死：8 × 60 × 30 = 14,400。
/// V1.0 收官修订可回写。留痕于此。）
pub const TICK_CAP_FULL: u64 = 8 * 60 * 30;

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

/// 胜负（Draw = 平局：tick 0 双方皆空 / 上限判定存活总 hp 同值。
/// 「双方同 tick 战斗全灭」经归纳不可达——末二存活者必分先后手，
/// 先死者不再出手，注释留痕此推演）。
///
/// 同 tick 双灭不可达推演（主会话 D10 定稿，留痕）：设某 tick 战斗阶段内红的
/// 最后一死者死于蓝方某单位的攻击（该蓝的行动时刻 = 其索引位 i_B）、蓝的
/// 最后一死者死于红方某单位（索引位 i_R）。行凶者出手时必仍存活，即行凶发生
/// 于己方全灭之前：若 i_R < i_B，则 i_R 时刻蓝方最后一人已死 → i_B 处再无蓝方
/// 可出手——矛盾；若 i_B < i_R，对称矛盾。故「双方同 tick 战斗全灭」不可达，
/// 全灭 Draw 分支仅 tick 0 双空（或直构双空）构型可达。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Winner {
    Red,
    Blue,
    Draw,
}

impl Winner {
    /// CLI 输出用小写标签。
    pub fn label(self) -> &'static str {
        match self {
            Winner::Red => "red",
            Winner::Blue => "blue",
            Winner::Draw => "draw",
        }
    }
}

/// 终局四元组（任务卡口径：winner / 结束 tick / 双方存活计数 / 最终哈希）。
/// final_hash = 终局点 state_hash()（T002 起折叠 rng 4 状态字——已含 rng_state）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BattleOutcome {
    pub winner: Winner,
    pub end_tick: u64,
    pub alive_red: u32,
    pub alive_blue: u32,
    pub final_hash: u64,
}

/// 战斗日志最小字段（任务卡：对局 seed、双方兵种构成、终局四元组；
/// 不含落库——表 6-0 吞吐口径不含落库）。Display 输出固定 8 行（顺序固定、
/// 确定性内容），供 CLI stdout 与 T009 胜率表采集。
#[derive(Clone, Debug)]
pub struct BattleLog {
    pub seed: u64,
    pub red_composition: Vec<(UnitKind, usize)>,
    pub blue_composition: Vec<(UnitKind, usize)>,
    pub outcome: BattleOutcome,
}

/// 构成清单格式化（D5）：`kind:count` 逗号分隔无空格；kind 用 [`UnitKind::id`]，
/// 清单顺序 = 构成清单顺序。
fn fmt_composition(f: &mut std::fmt::Formatter<'_>, comp: &[(UnitKind, usize)]) -> std::fmt::Result {
    for (i, (kind, count)) in comp.iter().enumerate() {
        if i > 0 {
            f.write_str(",")?;
        }
        write!(f, "{}:{}", kind.id(), count)?;
    }
    Ok(())
}

impl std::fmt::Display for BattleLog {
    /// 固定 8 行、无尾随换行（CLI 以 println! 输出即得恰好 8 行）：
    /// seed / comp_red / comp_blue / winner / end_tick / alive_red / alive_blue / final_hash。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "seed={}", self.seed)?;
        f.write_str("\ncomp_red=")?;
        fmt_composition(f, &self.red_composition)?;
        f.write_str("\ncomp_blue=")?;
        fmt_composition(f, &self.blue_composition)?;
        write!(
            f,
            "\nwinner={}\nend_tick={}\nalive_red={}\nalive_blue={}\nfinal_hash=0x{:016x}",
            self.outcome.winner.label(),
            self.outcome.end_tick,
            self.outcome.alive_red,
            self.outcome.alive_blue,
            self.outcome.final_hash
        )
    }
}

/// 意图阶段输入快照（T006/D5）：从 `&[Unit]` 一次克隆构建；per-i 意图纯函数
/// 只读此结构（并行分片经 `Arc<ScanInput>` 共享，满足 'static，零 unsafe）。
/// side / kind 存判别 u8（克隆廉价；解释经 [`side_of`] / [`kind_of`]）。
struct ScanInput {
    x: Vec<i64>,
    alive: Vec<bool>,
    side: Vec<u8>,
    kind: Vec<u8>,
}

impl ScanInput {
    fn from_units(units: &[Unit]) -> Self {
        ScanInput {
            x: units.iter().map(|u| u.x).collect(),
            alive: units.iter().map(|u| u.alive).collect(),
            side: units.iter().map(|u| u.side as u8).collect(),
            kind: units.iter().map(|u| u.kind as u8).collect(),
        }
    }
}

/// 判别 u8 → Side（快照值来自本枚举自身，越界不可达；unreachable 兜底）。
fn side_of(d: u8) -> Side {
    match d {
        0 => Side::Red,
        1 => Side::Blue,
        _ => unreachable!("Side discriminant out of range"),
    }
}

/// 判别 u8 → UnitKind（同上；判别值锁定见 units::UnitKind）。
fn kind_of(d: u8) -> UnitKind {
    match d {
        0 => UnitKind::Shieldman,
        1 => UnitKind::HeavyKnight,
        2 => UnitKind::Pikeman,
        3 => UnitKind::Swordsman,
        4 => UnitKind::Archer,
        5 => UnitKind::Militia,
        _ => unreachable!("UnitKind discriminant out of range"),
    }
}

/// move 意图（T006/D3）：per-i 快照前方者索引（None = 运动方向上无存活者）。
/// 注：派工单 D3 的 desire（gap_old 预折算）**不入缓冲**——应用阶段按模块注释
/// 「修正」小节以 `min(speed_i, max(gap_now, 0))` 现算（黄金锚等价性要求）。
struct MoveIntent {
    front: Option<usize>,
}

/// combat 意图（T006/D4）：per-i 快照目标与射程预判定（二者都被应用侧消费）。
struct CombatIntent {
    target: Option<usize>,
    in_range: bool,
}

/// move 意图分片计算（D3：per-i 纯函数，读快照，无共享写 ⇒ 分片与结果无关）。
/// 片 [start, end) 输出与索引一一对应的意图向量。
fn move_intent_chunk(input: &ScanInput, start: usize, end: usize) -> Vec<MoveIntent> {
    let mut out = Vec::with_capacity(end - start);
    for i in start..end {
        // 快照全存活不变式由意图阶段开头 debug_assert 兜底；alive 过滤保留为
        // 语义防御（与原版同口径）。
        if !input.alive[i] {
            out.push(MoveIntent { front: None });
            continue;
        }
        let dir = side_of(input.side[i]).dir();
        let x_i = input.x[i];
        // 运动方向上最近存活者：扫全索引、取最小正 ahead；严格小于才更新
        // → 平局取最小 j（与原版同款模式）。
        let mut front: Option<(usize, i64)> = None;
        for j in 0..input.x.len() {
            if j == i || !input.alive[j] {
                continue;
            }
            // 溢出安全：x ∈ [-2^35, 2^42] 量级（原版留痕），差与 *dir 均远不及 i64 上界。
            let d = (input.x[j] - x_i) * dir;
            if d > 0 && front.map_or(true, |(_, b)| d < b) {
                front = Some((j, d));
            }
        }
        out.push(MoveIntent {
            front: front.map(|(j, _)| j),
        });
    }
    out
}

/// combat 意图分片计算（D4：per-i 纯函数，读快照 = move 应用后位置）。
fn combat_intent_chunk(input: &ScanInput, start: usize, end: usize) -> Vec<CombatIntent> {
    let mut out = Vec::with_capacity(end - start);
    for i in start..end {
        if !input.alive[i] {
            out.push(CombatIntent {
                target: None,
                in_range: false,
            });
            continue;
        }
        let x_i = input.x[i];
        let side_i = input.side[i];
        // 索敌：最近存活敌方；平局取最小索引（严格小于才更新，与原版同款）。
        let mut target: Option<(usize, i64)> = None;
        for j in 0..input.x.len() {
            if j == i || !input.alive[j] || input.side[j] == side_i {
                continue;
            }
            let d = (input.x[j] - x_i).abs();
            let closer = match target {
                Some((_, best)) => d < best,
                None => true,
            };
            if closer {
                target = Some((j, d));
            }
        }
        let intent = match target {
            None => CombatIntent {
                target: None,
                in_range: false,
            },
            Some((j, dist)) => {
                // 射程预判定（闭区间；溢出安全：距离与半径和均远不及 i64 上界）。
                let radius_sum = spec(kind_of(input.kind[i])).radius_q32
                    + spec(kind_of(input.kind[j])).radius_q32;
                CombatIntent {
                    target: Some(j),
                    in_range: dist <= radius_sum + MELEE_MARGIN_Q32,
                }
            }
        };
        out.push(intent);
    }
    out
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
    /// 终局缓存（T005/D4）：`run_battle` 收束后写入并冻结（幂等返回）；
    /// `None` = 未收束。**不进 state_hash、不影响 step/run**——哈希折叠序
    /// 零变化，T002/T004 黄金锚原值保持。
    resolved: Option<BattleOutcome>,
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
            resolved: None,
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
            resolved: None,
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

    /// 推进一个 tick（串行路径；T006 起委托 [`World::step_with(None)`]，
    /// 阶段序与语义同 T005 定稿，签名与语义不变——D6）。
    pub fn step(&mut self) {
        self.step_with(None);
    }

    /// 推进一个 tick（T006/D2/D6 两阶段版，主会话 D5 定稿阶段序固定不得倒置）：
    /// 1) tick 计数 +1；
    /// 2) tick 级 RNG 固定消耗点——取一个 u64 丢弃（T002 设计保留；无单位级
    ///    RNG 消耗，战斗结算确定性不依赖随机）；
    /// 3) 单 lane 移动：意图（快照，可并行）→ 索引序串行应用；
    /// 4) 战斗结算：意图（move 后快照，可并行）→ 索引序串行应用（cd 推进 →
    ///    目标选择/重索敌 → 射程 → 伤害 → 墓碑；combat 在 move 之后——同 tick
    ///    内先动后打）；
    /// 5) tick 末 `units.retain(|u| u.alive)` 稳定保序清除墓碑（存活者相对序
    ///    不变前移，见模块注释 D6）；
    /// 6) 末尾刷新状态哈希。
    ///
    /// `pool`：`None` = 意图阶段主线程直跑（threads=1 语义）；`Some(&pool)` =
    /// 经线程池分片并行。两档共享同一意图纯函数与应用循环——结果与线程数无关
    /// （D2 单一实现）。
    pub fn step_with(&mut self, pool: Option<&ThreadPool>) {
        self.tick = self.tick.wrapping_add(1);
        let _ = self.rng.next_u64();
        self.move_units_with(pool);
        self.combat_with(pool);
        self.units.retain(|u| u.alive);
        self.last_hash = self.state_hash();
    }

    /// 单 lane 一维移动两阶段（T006/D3，替代原单阶段 move_units；等价性论证与
    /// 对派工单应用式的修正见模块注释「move 两阶段」小节）：
    ///
    /// 阶段 1（意图，可并行）：per 存活 i 在快照上求 `front(i)`（运动方向最近
    /// 存活者，平局取最小索引）。
    /// 阶段 2（应用，索引序串行，读最新位置——与原版循环同构）：
    /// - 无前方者：前进 `speed`；
    /// - 有前方者：前进 `min(speed, max(x[f]_now - x_i - (r_i + r_f), 0))`——
    ///   f 若已先结算则读到其已更新位置（友军排队堵停 + 相向穿插防御，原版
    ///   语义）；f 未结算则读到旧值（未应用）。
    ///
    /// 该模型同时覆盖「与敌方贴身停（间距=半径和）」与「友军排队堵停」（异速
    /// 友军不穿插）。O(N²) 朴素查找；空间划分 / 数据布局留 T007。
    fn move_units_with(&mut self, pool: Option<&ThreadPool>) {
        // retain 后不变式自检（意图阶段开头，release 零成本）。
        debug_assert!(self.units.iter().all(|u| u.alive));
        let n = self.units.len();
        if n == 0 {
            return;
        }
        // 每 tick 两次快照之第一次（move 意图前）。
        let input = ScanInput::from_units(&self.units);
        let intents: Vec<MoveIntent> = match pool {
            None => move_intent_chunk(&input, 0, n),
            Some(p) => {
                let input = Arc::new(input);
                p.map_chunks(n, move |start, end| move_intent_chunk(&input, start, end))
            }
        };
        // 阶段 2：索引序串行应用（本阶段无死亡 ⇒ 快照索引 = 当前索引）。
        for i in 0..n {
            let dir = self.units[i].side.dir();
            let speed = spec(self.units[i].kind).speed_q32;
            let advance = match intents[i].front {
                None => speed,
                Some(f) => {
                    // clamp 读 f 的最新位置（修正后原版式，见模块注释 ②③）；
                    // 差值必须乘 dir（原版 ahead = (x_j - x_i) * dir 方向感知）——
                    // 蓝方 dir = -1，裸差值恒负即恒停（首轮门禁实测教训，留痕）。
                    let gap = (self.units[f].x - self.units[i].x) * dir
                        - (spec(self.units[i].kind).radius_q32
                            + spec(self.units[f].kind).radius_q32);
                    let gap = if gap > 0 { gap } else { 0 };
                    if speed < gap { speed } else { gap }
                }
            };
            // 溢出安全：x ∈ [-2^35, 2^42] 量级（原版留痕），advance ≥ 0 有界。
            self.units[i].x += dir * advance;
        }
    }

    /// 战斗结算两阶段（T006/D4，替代原单阶段 combat；等价性引理见模块注释
    /// 「combat 两阶段」小节。索引序，5.2 纪律）：
    ///
    /// 阶段 1（意图，可并行；快照 = move 应用后、retain 前的全存活态）：
    /// per i 求 target_old 与 in_range_old。
    /// 阶段 2（应用，索引序串行）：
    /// 1) 先对全部存活单位 `cd = cd.saturating_sub(1)`（cd 冷却制：出手后置
    ///    间隔值，此后每 tick 战斗阶段开头统一递减；未接敌者自然停在 0——
    ///    首次接敌即击）；
    /// 2) 逐单位判定：目标选择（快照目标仍存活 → 直接用；已死 → 重索敌，保障
    ///    「先手击杀剥夺后手目标选择」语义）→ 射程 → `cd == 0` → 伤害立即写
    ///    目标 hp（hp <= 0 → 墓碑，同 tick 内排在后面的单位不再选它、它也不再
    ///    行动），攻击者 cd = attack_interval_ticks。
    ///
    /// 索敌/射程/伤害口径与原版逐项一致（|dx| 最小平局最小索引、闭区间
    /// MELEE_MARGIN、damage_dealt 截断除法、range_q32 本阶段不读）；每 tick 重算。
    fn combat_with(&mut self, pool: Option<&ThreadPool>) {
        // 不变式：move 阶段无死亡（本卡口径），此刻仍全存活。
        debug_assert!(self.units.iter().all(|u| u.alive));
        let n = self.units.len();
        if n == 0 {
            return;
        }
        // 每 tick 两次快照之第二次（combat 意图前——已反映 move 后位置）。
        let input = ScanInput::from_units(&self.units);
        let intents: Vec<CombatIntent> = match pool {
            None => combat_intent_chunk(&input, 0, n),
            Some(p) => {
                let input = Arc::new(input);
                p.map_chunks(n, move |start, end| combat_intent_chunk(&input, start, end))
            }
        };
        // (1) cd 推进：先于个体判定，对全部存活单位统一 -1（饱和减，0 不下穿；
        //     原样放串行侧）。
        for u in self.units.iter_mut() {
            if u.alive {
                u.cd = u.cd.saturating_sub(1);
            }
        }
        // (2) 个体判定：按索引序；墓碑即时生效。
        for i in 0..n {
            if !self.units[i].alive {
                continue; // 墓碑：不攻击、不可被选
            }
            // 目标选择（D4）：快照目标仍存活 → 直接用（引理：仍为当前最近）；
            // 已被先手击杀 → 重索敌（完整重扫当前墓碑状态，同平局规则）。
            let (target, in_range) = match intents[i].target {
                // 快照时已无敌方；存活集只缩不增 ⇒ 现在也无：cd 保持现值跳过
                // （D4：未接敌不重置）。
                None => continue,
                Some(t0) if self.units[t0].alive => (t0, intents[i].in_range),
                Some(_) => {
                    let x_i = self.units[i].x;
                    let side_i = self.units[i].side;
                    // 重索敌：完整重扫当前墓碑状态；|dx| 最小、平局取最小索引
                    // （严格小于才更新，与意图阶段同式——原版实时索敌即此式）。
                    let mut found: Option<(usize, i64)> = None;
                    for j in 0..n {
                        if j == i || !self.units[j].alive || self.units[j].side == side_i {
                            continue;
                        }
                        let d = (self.units[j].x - x_i).abs();
                        if found.map_or(true, |(_, best)| d < best) {
                            found = Some((j, d));
                        }
                    }
                    match found {
                        None => continue, // 无存活敌方：cd 保持现值
                        Some((j, dist)) => {
                            // 射程现算（重索敌换目标后自动正确；闭区间）。
                            let radius_sum = spec(self.units[i].kind).radius_q32
                                + spec(self.units[j].kind).radius_q32;
                            (j, dist <= radius_sum + MELEE_MARGIN_Q32)
                        }
                    }
                }
            };
            if !in_range {
                continue; // 出射程：未出手，cd 不重置（保持 0 或继续衰减）
            }
            if self.units[i].cd != 0 {
                continue; // 冷却中
            }
            // 伤害立即写入目标 hp（O(1) 现算，不预计算——D4）；hp <= 0 → 墓碑。
            let dmg = damage_dealt(self.units[i].kind, self.units[target].kind);
            self.units[target].hp -= dmg;
            self.units[i].cd = spec(self.units[i].kind).attack_interval_ticks;
            if self.units[target].hp <= 0 {
                self.units[target].alive = false;
            }
        }
    }

    /// 连续推进 `ticks` 个 tick（串行路径；T006 起委托 run_with(None)）。
    pub fn run(&mut self, ticks: u64) {
        self.run_with(ticks, None);
    }

    /// 连续推进 `ticks` 个 tick（T006/D6 两阶段版：每 tick 一次 step_with）。
    pub fn run_with(&mut self, ticks: u64, pool: Option<&ThreadPool>) {
        for _ in 0..ticks {
            self.step_with(pool);
        }
    }

    /// 单局收束（串行路径；T006 起委托 run_battle_with(None)——控制流与语义
    /// 与 T005 定稿完全一致）。
    pub fn run_battle(&mut self, max_ticks: u64) -> BattleOutcome {
        self.run_battle_with(max_ticks, None)
    }

    /// 单局收束（T006/D6 两阶段版）：推进至全灭或 max_ticks（先到者），返回
    /// 终局四元组并冻结（幂等：已收束则直接返回缓存，tick 不再推进——终局
    /// 冻结机制）。控制流与 run_battle 完全一致（D6），仅逐 tick 推进换用
    /// step_with：全灭判定每 tick 求值且优先于上限（D3）；上限收束按存活总
    /// hp 判定（D3）。step/run 保持 T002/T004 纯原语语义不变；本方法只是
    /// 控制流 + 判定。
    pub fn run_battle_with(&mut self, max_ticks: u64, pool: Option<&ThreadPool>) -> BattleOutcome {
        if let Some(cached) = &self.resolved {
            return cached.clone();
        }
        // tick 0 全灭检查（D3：空阵营边界——0 单位一方——立即收束 end_tick=0，
        // 不 panic；空阵营不进入循环）。
        let (alive_red, alive_blue) = self.alive_counts();
        let outcome = if alive_red == 0 || alive_blue == 0 {
            self.resolve_extinction(alive_red, alive_blue)
        } else {
            let mut extinction: Option<BattleOutcome> = None;
            while self.tick < max_ticks {
                self.step_with(pool);
                let (ar, ab) = self.alive_counts();
                if ar == 0 || ab == 0 {
                    // 恰在 tick == max_ticks 发生全灭亦走此分支——全灭优先于上限（D3）。
                    extinction = Some(self.resolve_extinction(ar, ab));
                    break;
                }
            }
            match extinction {
                Some(o) => o,
                None => self.resolve_by_hp(),
            }
        };
        self.resolved = Some(outcome.clone());
        outcome
    }

    /// 已收束的终局（未收束为 `None`）。
    pub fn outcome(&self) -> Option<&BattleOutcome> {
        self.resolved.as_ref()
    }

    /// 双方存活计数（按索引序单遍；retain 后 units 全为存活者，alive 过滤为
    /// 语义防御，与 move_units 同口径）。
    fn alive_counts(&self) -> (u32, u32) {
        let mut red: u32 = 0;
        let mut blue: u32 = 0;
        for u in &self.units {
            if u.alive {
                match u.side {
                    Side::Red => red += 1,
                    Side::Blue => blue += 1,
                }
            }
        }
        (red, blue)
    }

    /// 全灭收束（私有，D3）：一方 0 且对方 >0 → 对方胜；双方 0 → Draw（仅
    /// tick 0 双空构型可达——同 tick 双灭不可达，见 [`Winner`] 注释 D10 推演）。
    /// final_hash 一律终局点现算 [`World::state_hash`]（不读 last_hash——tick 0
    /// 直构 manual world 的 last_hash=0，现算才语义正确；step 过的路径两者相等）。
    fn resolve_extinction(&self, alive_red: u32, alive_blue: u32) -> BattleOutcome {
        let winner = match (alive_red, alive_blue) {
            (0, 0) => Winner::Draw,
            (_, 0) => Winner::Red,
            (0, _) => Winner::Blue,
            _ => unreachable!("resolve_extinction requires at least one extinct side"),
        };
        BattleOutcome {
            winner,
            end_tick: self.tick,
            alive_red,
            alive_blue,
            final_hash: self.state_hash(),
        }
    }

    /// 上限收束（私有，D3）：每方存活单位 hp 按索引序 i64 累加求和（5.2 纪律
    /// 固定序——归约序与线程数无关的要求从现在钉死，本阶段虽单线程亦不豁免，
    /// 并行求和结果不进模拟态）；高者胜，同值 Draw。
    /// 溢出安全：存活者 hp ≥ 1、降规模满编 200 单位 × max hp 150 = 30,000
    /// << i64::MAX（全规模万人量级 20,000 × 150 = 3×10^6 亦安全）。
    fn resolve_by_hp(&self) -> BattleOutcome {
        let mut hp_red: i64 = 0;
        let mut hp_blue: i64 = 0;
        let mut alive_red: u32 = 0;
        let mut alive_blue: u32 = 0;
        for u in &self.units {
            if !u.alive {
                continue;
            }
            match u.side {
                Side::Red => {
                    hp_red += u.hp as i64;
                    alive_red += 1;
                }
                Side::Blue => {
                    hp_blue += u.hp as i64;
                    alive_blue += 1;
                }
            }
        }
        let winner = if hp_red > hp_blue {
            Winner::Red
        } else if hp_blue > hp_red {
            Winner::Blue
        } else {
            Winner::Draw
        };
        BattleOutcome {
            winner,
            end_tick: self.tick,
            alive_red,
            alive_blue,
            final_hash: self.state_hash(),
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
            resolved: None,
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

    // ---- T005 胜负判定与终局（主会话 D7~D9 定稿）----

    /// T005 测 1（验收 1 必胜局，1v1）：复用 T004 单测 B 已锁定时间线
    /// （docs/evidence/t004/handcalc.md 例 1）——红盾兵 x=0.5m / 蓝长矛 x=1.5m
    /// 恰贴身（|dx| = 半径和，gap=0 全程 0 移动）；盾兵(Heavy)→长矛(Light)
    /// 12/击、长矛→盾兵 6/击，interval 均 30 → 击点 t=1,31,…,151；蓝 hp 80
    /// 耗尽于第 7 击 t=181（80 − 6×12 = 8；8 − 12 = −4 → 墓碑），红存活
    /// （120 − 6×6 = 84）。全部引用 T004 已证事实，不重新推演。
    /// final_hash 与另一个独立同构 World 手动 run(181) 后 state_hash() 相等
    /// （内聚对拍，不固新黄金值）。
    #[test]
    fn battle_decisive_1v1_counter_kill_at_181() {
        let mut w = manual_world(
            vec![
                unit(UnitKind::Shieldman, Side::Red, ONE_Q32_32 / 2),
                unit(UnitKind::Pikeman, Side::Blue, 3 * ONE_Q32_32 / 2),
            ],
            7,
        );
        let outcome = w.run_battle(TICK_CAP_REDUCED);
        assert_eq!(outcome.winner, Winner::Red);
        assert_eq!(outcome.end_tick, 181, "第 7 击致死 t=181（T004 单测 B 锁定）");
        assert_eq!(outcome.alive_red, 1);
        assert_eq!(outcome.alive_blue, 0);
        let mut w2 = manual_world(
            vec![
                unit(UnitKind::Shieldman, Side::Red, ONE_Q32_32 / 2),
                unit(UnitKind::Pikeman, Side::Blue, 3 * ONE_Q32_32 / 2),
            ],
            7,
        );
        w2.run(181);
        assert_eq!(
            outcome.final_hash,
            w2.state_hash(),
            "final_hash = 终局点 state_hash 现算（内聚对拍）"
        );
    }

    /// T005 测 2（验收 1 多单位形态）：3 盾兵 vs 3 长矛队列歼灭战。
    /// 队列构造（同 deploy 口径）：红队首 x = r（=ONE/2），后续
    /// x_k = x_{k-1} − (r+r+GAP)，GAP 用 ONE/2 → 间距 1.5m；蓝方镜像侧同式
    /// 相向（x_k = x_{k-1} + (r+r+ONE/2)）。
    /// 【派工单原蓝队首 x = 1000×ONE − r 不满足本测断言，已按算式上报主会话：
    /// 队首相距 999m、合闭合 0.05+0.10 = 0.15 m/tick，接敌（|dx| ≤ 0.5+0.5+0.2
    /// = 1.2m）需 (999−1.2)/0.15 ≈ 6652 ticks > TICK_CAP_REDUCED——只会走上限
    /// hp 判定而非歼灭。为保住本测被测语义（任务卡验收 1 必胜局的多单位歼灭
    /// 形态），蓝队首压缩至 x = 2×ONE（2.0m）：初距 1.5m > 1.2m 出射程，t=2
    /// 移动后恰 1.2m（闭区间）首击；队列间距式一字不改。】
    /// 断言：Red 胜（盾兵克长矛：12/击 vs 6/击、120 hp vs 80 hp）、alive_blue=0、
    /// alive_red ≥ 1、end_tick < TICK_CAP_REDUCED；end_tick 具体值不固化
    /// （实测记入证据档 README）；幂等（终局冻结：同 outcome、tick 不变）。
    #[test]
    fn battle_decisive_3v3_counter_queue() {
        let r = ONE_Q32_32 / 2; // 盾兵 / 长矛半径均 0.5m
        let gap = ONE_Q32_32 / 2; // GAP = ONE/2（派工单式）
        let spacing = r + r + gap; // 相邻同侧单位间距 1.5m
        let mut w = manual_world(
            vec![
                unit(UnitKind::Shieldman, Side::Red, r),
                unit(UnitKind::Shieldman, Side::Red, r - spacing),
                unit(UnitKind::Shieldman, Side::Red, r - 2 * spacing),
                unit(UnitKind::Pikeman, Side::Blue, 2 * ONE_Q32_32),
                unit(UnitKind::Pikeman, Side::Blue, 2 * ONE_Q32_32 + spacing),
                unit(UnitKind::Pikeman, Side::Blue, 2 * ONE_Q32_32 + 2 * spacing),
            ],
            7,
        );
        let outcome = w.run_battle(TICK_CAP_REDUCED);
        assert_eq!(outcome.winner, Winner::Red);
        assert_eq!(outcome.alive_blue, 0);
        assert!(outcome.alive_red >= 1, "alive_red={}", outcome.alive_red);
        assert!(
            outcome.end_tick < TICK_CAP_REDUCED,
            "end_tick={} 应在上限前歼灭收束",
            outcome.end_tick
        );
        // 幂等：再次 run_battle 返回相同 outcome 且 world.tick 不变。
        let tick_at_resolve = w.tick;
        let again = w.run_battle(TICK_CAP_REDUCED);
        assert_eq!(again, outcome, "终局冻结：幂等返回");
        assert_eq!(w.tick, tick_at_resolve, "终局冻结：tick 不再推进");
    }

    /// T005 测 3（验收 1 空阵营边界）：仅红 2 民兵（无蓝）与完全空场两构型。
    /// tick 0（首 step 前）即收束：end_tick=0、不 panic、二次调用幂等。
    #[test]
    fn battle_empty_side_tick0_no_panic() {
        let mut w = manual_world(
            vec![
                unit(UnitKind::Militia, Side::Red, 0),
                unit(UnitKind::Militia, Side::Red, ONE_Q32_32),
            ],
            7,
        );
        let outcome = w.run_battle(1800);
        assert_eq!(outcome.winner, Winner::Red);
        assert_eq!(outcome.end_tick, 0);
        assert_eq!(outcome.alive_red, 2);
        assert_eq!(outcome.alive_blue, 0);
        assert_eq!(w.run_battle(1800), outcome, "幂等：同 outcome");
        assert_eq!(w.tick, 0, "tick 0 收束不再推进");

        let mut empty = manual_world(vec![], 7);
        let outcome = empty.run_battle(1800);
        assert_eq!(outcome.winner, Winner::Draw, "双空 → Draw");
        assert_eq!(outcome.end_tick, 0);
        assert_eq!(outcome.alive_red, 0);
        assert_eq!(outcome.alive_blue, 0);
        assert_eq!(empty.run_battle(1800), outcome, "幂等：同 outcome");
        assert_eq!(empty.tick, 0);
    }

    /// T005 测 4（验收 2 僵局到上限 + 终局冻结）：镜像民兵（x=1m / x=999m，
    /// 1+999=1000=LANE，同速同 hp）。合闭合速度 2×0.09 = 0.18 m/tick，接敌
    /// （|dx| ≤ 0.4+0.4+0.2 = 1.0m）需 (998−1.0)/0.18 ≈ 5539 ticks > 1800
    /// ——上限前不接敌。→ Winner::Draw（上限等 hp 平局分支）、end_tick=1800、
    /// alive (1,1)；终局冻结：再次 run_battle 同 outcome、tick 仍 1800、
    /// state_hash 不变。
    #[test]
    fn battle_cap_mirror_stalemate_draw_and_freeze() {
        let mut w = manual_world(
            vec![
                unit(UnitKind::Militia, Side::Red, ONE_Q32_32),
                unit(UnitKind::Militia, Side::Blue, 999 * ONE_Q32_32),
            ],
            7,
        );
        let outcome = w.run_battle(TICK_CAP_REDUCED);
        assert_eq!(outcome.winner, Winner::Draw);
        assert_eq!(outcome.end_tick, TICK_CAP_REDUCED);
        assert_eq!(outcome.alive_red, 1);
        assert_eq!(outcome.alive_blue, 1);
        let hash_at_resolve = w.state_hash();
        let again = w.run_battle(TICK_CAP_REDUCED);
        assert_eq!(again, outcome, "终局冻结：幂等返回");
        assert_eq!(w.tick, TICK_CAP_REDUCED, "tick 不再推进");
        assert_eq!(w.state_hash(), hash_at_resolve, "state_hash 不变");
    }

    /// T005 测 5（验收 2 判定规则分支）：上限 hp 判定高者胜——红民兵 hp 覆盖
    /// 100 @ x=1m vs 蓝民兵 hp 50 @ x=999m（镜像位同测 4，不接敌）→ cap →
    /// Winner::Red（hp 100 > 50 高者胜分支）。等 hp 对照组 → Draw 已由测 4
    /// 覆盖（互证，不重复构造）。
    #[test]
    fn battle_cap_hp_judgment_asymmetric() {
        let mut w = manual_world(
            vec![
                unit_with_hp(UnitKind::Militia, Side::Red, ONE_Q32_32, 100),
                unit(UnitKind::Militia, Side::Blue, 999 * ONE_Q32_32),
            ],
            7,
        );
        let outcome = w.run_battle(TICK_CAP_REDUCED);
        assert_eq!(outcome.winner, Winner::Red, "hp 100 > 50 高者胜分支");
        assert_eq!(outcome.end_tick, TICK_CAP_REDUCED);
        assert_eq!(outcome.alive_red, 1);
        assert_eq!(outcome.alive_blue, 1);
    }

    /// T005 测 6（验收 3 内聚 + 黄金锚零成本复用）：默认构成 seed 42 走
    /// run_battle——镜像不接敌（模块注释 D10：队首相距约 998m、合闭合 ≤0.4
    /// m/tick ≈ 2498 ticks 才贴身）→ 上限收束、双方 hp 和对称相等 → Draw、
    /// alive (30,30)；final_hash 必须逐位复现 T004 黄金锚 0x958c5938c8682529
    /// ——run_battle 只逐步 step 1800 次 + 判定不动状态，此断言同时证明
    /// resolved 字段零哈希影响。
    #[test]
    fn battle_golden_crosscheck_default_comp() {
        let mut w = World::deploy(42, &DEFAULT_COMPOSITION);
        let outcome = w.run_battle(TICK_CAP_REDUCED);
        assert_eq!(outcome.winner, Winner::Draw);
        assert_eq!(outcome.end_tick, TICK_CAP_REDUCED);
        assert_eq!(outcome.alive_red, 30);
        assert_eq!(outcome.alive_blue, 30);
        assert_eq!(
            outcome.final_hash, 10_776_086_108_806_063_401, // 0x958c5938c8682529
            "T004 黄金锚原值必须逐位复现（resolved 零哈希影响实证）"
        );
        assert_eq!(w.outcome(), Some(&outcome), "outcome() 读到冻结终局");
    }

    // ---- T006 并行化 v0 与两阶段更新（主会话 D10 定稿；手算依据见派工单 §3）----

    /// §3.1 构型（代码生成，非 deploy）：每方 250 单位、kind 按 KINDS[i % 6] 轮转、
    /// 双方前排 3 个单位 hp 直填 2（一击死层——锁定表最小单击伤害 3（民兵→轻甲
    /// 6×43690/65536 = 3）> 2，任何一击必死）。红队列贴身链 GAP = ONE/2；
    /// 蓝队首 = 红队首 + 11*ONE/10（首对 |dx| = 1.1 ≤ r+r+margin，恰入射程即开打）。
    /// 索引序：红 0..250、蓝 250..500（同侧排队链结算次序 = 索引序）。
    fn build_nearfield_world(seed: u64) -> World {
        const PER_SIDE: usize = 250;
        const KINDS: [UnitKind; 6] = [
            UnitKind::Shieldman,
            UnitKind::HeavyKnight,
            UnitKind::Pikeman,
            UnitKind::Swordsman,
            UnitKind::Archer,
            UnitKind::Militia,
        ];
        let gap = ONE_Q32_32 / 2;
        let mut units: Vec<Unit> = Vec::with_capacity(2 * PER_SIDE);
        // 红队列：x_red(0) = r(kind_0)，向 -x 排队：x_red(k) = x_red(k-1) - (r_{k-1} + r_k + GAP)。
        let mut x = spec(KINDS[0]).radius_q32;
        for k in 0..PER_SIDE {
            let kind = KINDS[k % 6];
            let hp = if k < 3 { 2 } else { spec(kind).hp };
            units.push(Unit {
                alive: true,
                kind,
                side: Side::Red,
                hp,
                cd: 0,
                x,
            });
            x -= spec(kind).radius_q32 + spec(KINDS[(k + 1) % 6]).radius_q32 + gap;
        }
        // 蓝队列：x_blue(0) = 红队首 + 11*ONE/10，向 +x 同式排队。
        let mut x = spec(KINDS[0]).radius_q32 + 11 * ONE_Q32_32 / 10;
        for k in 0..PER_SIDE {
            let kind = KINDS[k % 6];
            let hp = if k < 3 { 2 } else { spec(kind).hp };
            units.push(Unit {
                alive: true,
                kind,
                side: Side::Blue,
                hp,
                cd: 0,
                x,
            });
            x += spec(kind).radius_q32 + spec(KINDS[(k + 1) % 6]).radius_q32 + gap;
        }
        manual_world(units, seed)
    }

    /// T006 单测 1（D10-1 / §3.1）：近距战斗局 threads{1,3,6,12} 逐采样点哈希全等。
    /// 覆盖：近战互击 / 一击死 / 墓碑剥夺 / 重索敌 / retain 压缩 / 混速排队 clamp 链。
    /// 布阵用同一构型代码构造四份（同构造即同初态，tick0 哈希四档相等为首断言）。
    #[test]
    fn parallel_parity_nearfield_battle() {
        let tiers = [1usize, 3, 6, 12];
        let mut worlds: Vec<World> = tiers.iter().map(|_| build_nearfield_world(7)).collect();
        let h0 = worlds[0].state_hash();
        for w in &worlds {
            assert_eq!(w.state_hash(), h0, "tick0 四档构型必须逐位一致");
        }
        // 每 20 ticks 采样 ×6 轮（t=20..120），连同 t0 共 7 个采样点。
        let mut samples = [[0u64; 7]; 4];
        for (tier, &t) in tiers.iter().enumerate() {
            samples[tier][0] = h0;
            let mut pool = ThreadPool::new(t); // 每档独立建池、跑完 drop（D10-1）
            for round in 1..=6 {
                worlds[tier].run_with(20, Some(&pool));
                samples[tier][round] = worlds[tier].state_hash();
            }
        }
        for round in 0..7 {
            for tier in 1..4 {
                assert_eq!(
                    samples[tier][round],
                    samples[0][round],
                    "采样点 {}（t={}）threads={} 与 threads=1 哈希分歧",
                    round,
                    round * 20,
                    tiers[tier]
                );
            }
        }
    }

    /// T006 单测 2（D10-2 / §3.2）：重索敌正路径手算（seed 7，manual_world）。
    /// idx1 的快照目标 idx2（民兵 A，hp 直填 6）被 idx0 先手一击死 → 应用时
    /// 重索敌切至 idx3——「先手击杀剥夺后手目标选择」语义的并行等价实现专测。
    /// 伤害算式：剑士→民兵 = 12×98304/65536 = 18（整除精确）；
    /// 民兵→剑士 = 6×43690/65536 = 3（商 3 余 65532，截断）。
    #[test]
    fn combat_rescan_after_first_blood_switches_target() {
        let build = || {
            manual_world(
                vec![
                    unit(UnitKind::Swordsman, Side::Red, 0),
                    unit(UnitKind::Swordsman, Side::Red, 9 * ONE_Q32_32 / 10),
                    unit_with_hp(UnitKind::Militia, Side::Blue, 11 * ONE_Q32_32 / 10, 6),
                    unit(UnitKind::Militia, Side::Blue, 3 * ONE_Q32_32 / 2),
                ],
                7,
            )
        };
        let mut w = build();
        w.run(1);
        // t=1 移动：四者前方 gap 全负 → 全部原位（逐单位核过：idx0 前方 idx1
        // gap = 0.9-1.0 = -0.1；idx1 前方 idx2 gap = 0.2-0.9 = -0.7；idx2 前方
        // idx1 gap = -0.7；idx3 前方 idx2 gap = 0.4-0.8 = -0.4）。
        // t=1 战斗（索引序）：idx0 击 idx2（|dx| = 1.1 = 0.5+0.4+0.2 恰闭区间
        // 边界可击）18 → hp 6-18 = -12 墓碑、cd 25；idx1 快照目标 = idx2 已死
        // → 重索敌 → idx3（0.6 ≤ 1.1 ✓）出手 18 → B hp 32、cd 25；idx3 索敌
        // idx1（0.6 < 1.5）出手 3 → idx1 hp 87、cd 20；idx2 墓碑 skip。
        assert_eq!(w.tick, 1);
        assert_eq!(w.unit_count(), 3, "民兵 A（hp 6）一击死并于 tick 末清除");
        assert_eq!(w.units[0].hp, 90, "红 idx0 未被反击（A 死、B 目标为 idx1）");
        assert_eq!(w.units[0].cd, 25, "剑士出手后 cd = interval 25");
        assert_eq!(w.units[0].x, 0, "四者移动全停");
        assert_eq!(w.units[1].hp, 87, "idx1 hp = 90 - 3（B 反击）");
        assert_eq!(w.units[1].cd, 25);
        assert_eq!(w.units[1].x, 9 * ONE_Q32_32 / 10);
        assert_eq!(
            w.units[2].hp,
            32,
            "民兵 B = 50 - 18（idx1 重索敌后命中 B——被测分支）"
        );
        assert_eq!(w.units[2].cd, 20, "民兵 interval 20");
        assert_eq!(w.units[2].x, 3 * ONE_Q32_32 / 2);
        // 同构型 threads=4 走 step_with(Some) 跑一遍 t=1：与串行逐字段相等。
        let mut wp = build();
        let mut pool = ThreadPool::new(4);
        wp.step_with(Some(&pool));
        assert_eq!(
            wp.state_hash(),
            w.state_hash(),
            "threads=4 与串行 t=1 状态逐位一致"
        );
        assert_eq!(wp.unit_count(), w.unit_count());
        for (p, s) in wp.units().iter().zip(w.units().iter()) {
            assert_eq!(p.alive, s.alive);
            assert_eq!(p.kind, s.kind);
            assert_eq!(p.side, s.side);
            assert_eq!(p.hp, s.hp);
            assert_eq!(p.cd, s.cd);
            assert_eq!(p.x, s.x);
        }
    }

    /// T006 单测 3（D10-3 / §3.3）：相向穿插防御——阶段 2 clamp 缺失即炸：
    /// 红 idx2 结算前进 min(0.12, gap 0.05) = 0.05 后，蓝 idx3（索引序在后，
    /// 读红 idx2 已更新位置）gap = 0 → 停；若误用旧快照直接应用，蓝 idx3 亦
    /// 前进 0.05 → 合前进 0.10 > gap 0.05 → 穿插 → 位置序断言炸。
    /// 【派工单 §3.3 构造笔误修正（上报留痕）：原红队间距 0.9 < 剑士半径和
    /// 1.0（ONE/2 ×2），初始即自穿插、与断言②「相邻间距 ≥ 半径和（红红 1.0）」
    /// 直接矛盾——按该测自身的断言口径把红队间距修为恰 = 半径和 1.0·ONE，
    /// 蓝队首相应 +0.2·ONE（首对间距 0.95 / gap 0.05 不变，被测动力学不变）】。
    #[test]
    fn two_phase_move_no_interpenetration_head_on() {
        let r_sword = spec(UnitKind::Swordsman).radius_q32;
        let r_mil = spec(UnitKind::Militia).radius_q32;
        // 红 3 剑士 x = 0 / ONE / 2*ONE（同队贴身链，间距恰 = 半径和 1.0）；
        // 蓝 3 民兵 x = 2*ONE + 19*ONE/20 起、间距 9*ONE/10（= 半径和 0.8 + 间隙 0.1）。
        // 首对（红 idx2 @2.0 vs 蓝 idx3 @2.95）间距 0.95，gap = 0.05 > 0。
        let x_blue0 = 2 * ONE_Q32_32 + 19 * ONE_Q32_32 / 20;
        let x_blue1 = x_blue0 + 9 * ONE_Q32_32 / 10;
        let x_blue2 = x_blue1 + 9 * ONE_Q32_32 / 10;
        let mut w = manual_world(
            vec![
                unit(UnitKind::Swordsman, Side::Red, 0),
                unit(UnitKind::Swordsman, Side::Red, ONE_Q32_32),
                unit(UnitKind::Swordsman, Side::Red, 2 * ONE_Q32_32),
                unit(UnitKind::Militia, Side::Blue, x_blue0),
                unit(UnitKind::Militia, Side::Blue, x_blue1),
                unit(UnitKind::Militia, Side::Blue, x_blue2),
            ],
            7,
        );
        // 相邻对半径和：(0,1)(1,2) 红红 = 2×0.5；(2,3) 红蓝 = 0.5+0.4；
        // (3,4)(4,5) 蓝蓝 = 2×0.4——spec 现算断言（防漂移）。
        let sums = [
            r_sword + r_sword,
            r_sword + r_sword,
            r_sword + r_mil,
            r_mil + r_mil,
            r_mil + r_mil,
        ];
        for t in 1..=40u64 {
            w.step();
            assert_eq!(
                w.unit_count(),
                6,
                "tick {t}: 40 ticks 内无死亡（击点核过：首对互击起 t=1，蓝 idx3 第三击 t=51 > 40）"
            );
            let xs: Vec<i64> = w.units().iter().map(|u| u.x).collect();
            for pair in 0..5 {
                assert!(
                    xs[pair + 1] > xs[pair],
                    "tick {t}: 对 ({pair},{}) 位置序破坏（穿插）",
                    pair + 1
                );
                let spacing = xs[pair + 1] - xs[pair];
                assert!(
                    spacing >= sums[pair],
                    "tick {t}: 对 ({pair},{}) 间距 {spacing} < 半径和 {}（穿插）",
                    pair + 1,
                    sums[pair]
                );
            }
        }
        // 末态补充（结构性口径——Q32.32 截断传播使精确终值脆断）：修正后几何
        // （贴身 gap=0，非原派工单重叠 -0.1）下队列出现一步波传播松弛：idx2 于
        // t=1 前进 214748365（=19·ONE/20 − 半径和）后，idx1（t=2）、idx0（t=3）
        // 依次跟进同量恢复贴身，t≥4 全静止——故「恒停未动」不成立，改为断言
        // 队列重新完全贴身（间距恰 = 半径和）。hp 按锁定表逐击推演：
        // 红 idx2 被击 t=1,21 两击 ×3 = 84；蓝 idx3 被击 t=1,26 两击 ×18 = 14。
        assert_eq!(
            w.units[1].x - w.units[0].x,
            r_sword + r_sword,
            "红队贴身链波传播后恢复贴身（间距恰 = 半径和）"
        );
        assert_eq!(w.units[2].x - w.units[1].x, r_sword + r_sword);
        assert_eq!(w.units[3].x - w.units[2].x, r_sword + r_mil);
        assert_eq!(
            w.units[2].hp,
            90 - 2 * 3,
            "民兵→剑士 = 6×43690/65536 = 3/击，击点 t=1,21"
        );
        assert_eq!(
            w.units[3].hp,
            50 - 2 * 18,
            "剑士→民兵 = 12×98304/65536 = 18/击，击点 t=1,26"
        );
    }
}

