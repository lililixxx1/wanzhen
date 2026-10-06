//! BRP `game.*` 方法面 7 方法 + 宿主状态资源 [`HostedGame`] + [`HostRpcPlugin`]
//! 组装（T018 初始集 6，M5 席位 2；T023 +1 [`SAMPLE_OUTCOMES_METHOD`]，席位 8；
//! 依据 docs/evidence/t018/dispatch-sheet.md「实现清单」节 + taskset/t018-brp-host.md
//! 设计裁决 D1~D11 + 备忘录 §五 + taskset/t023-stats-face.md D1~D6）。
//!
//! 方法面（备忘录 §五 + T023 计数门禁 6→7，7/7）：[`DEPLOY_METHOD`] /
//! [`RUN_TO_TICK_METHOD`] / [`STATE_HASH_METHOD`] / [`OUTCOME_METHOD`] /
//! [`SAMPLE_OUTCOMES_METHOD`] / [`RUN_TESTS_METHOD`] /
//! [`SCREENSHOT_METHOD`]（headless 桩——结构化错误，不击穿进程，实装随 T019）。
//!
//! handler 形态（沿工作流仓 game/src/rpc 先例）：`fn(In(params): In<Option<Value>>,
//! world: &mut World) -> BrpResult`，由 bevy_remote 经 `world.run_system_with`
//! 独占执行（bevy_remote-0.19.1/src/lib.rs:1501）——handler 内可安全
//! `world.get_resource_mut::<HostedGame>()`。
//!
//! 命名纪律（D4）：bevy `World` 与 `sim::world::World` 同名——本文件裸名
//! `World` 一律指 bevy World（prelude 导入），sim 侧类型全限定书写、不
//! `use sim::world::World`。
//!
//! 错误码（D6）：参数错误 = JSON-RPC `INVALID_PARAMS`(-32602)（沿工作流仓
//! 先例）；域错误（协议合法、业务状态拒绝）用正码 4xxx，避开 JSON-RPC 保留段
//! -32768..-32000。
//!
//! 哈希口径（D5）：一律 `"0x" + {:016x}` hex 字符串，与 M0 CLI 口径一致。
//!
//! T021（D1/D3/D5）：`game.deploy` 新增可选 `"preset"` 底座（显式字段一律覆盖，
//! 预设数据单一来源见 [`crate::presets`]）；deploy 执行段抽取为 [`apply_deploy`]
//! （BRP handler 与 CLI auto-deploy 共用——入口无关性由构造保证）；`HostRpcPlugin`
//! 增 `port` 字段（缺省 15702 不变，地址恒回环）。
//!
//! T019（D4/D5/D8）：共享推进函数 [`advance_ticks`]（headless run_to_tick 与
//! spectate 驱动系统唯一模拟推进路径——语义零漂移）；[`HostedGame`] 增
//! `generation` 世代替换计数（表现层重建判据，D5）；`game.run_to_tick` spectate
//! 形态入队立即返回（响应多 `queued` 字段 = 模式差异留痕，headless 响应形态
//! 不变）；`game.deploy` spectate 形态下复位 autorun=false（BRP deploy 恒调用
//! 方驱动，D4）；`game.screenshot` 分形态——headless 4101 桩逐字不变、spectate
//! 实装两段式（实装体在 [`crate::spectate`]）。形态判据 = `SpectateState`
//! 资源存在性（headless 无此资源，路径零漂移——T018 冒烟回归面）。

use bevy_full as bevy;
// derive 宏（下方 Resource 等）展开为 `bevy_ecs::` 绝对路径；依赖名为别名
// bevy_full 时宏无法经清单解析命中，按 bevy_macro_utils 0.19.1 内置说明补别名
// （render-spike/src/main.rs:23-26 + T010 api-notes 先例）。
use bevy_full::ecs as bevy_ecs;
use bevy::prelude::*;
use bevy::remote::error_codes;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::{BrpError, BrpResult, RemotePlugin};
use serde_json::{json, Value};
use sim::units::{UnitKind, ONE_Q32_32};
use sim::world::{TICK_CAP_FULL, TICK_CAP_REDUCED};

use crate::presets;
use crate::spectate::SpectateState;
use crate::suite;

/// `game.deploy`（config：构成/参数/种子 → 布阵快照哈希）。
pub const DEPLOY_METHOD: &str = "game.deploy";
/// `game.run_to_tick`（headless 直推 n tick；观战模式帧率节流随 T019）。
pub const RUN_TO_TICK_METHOD: &str = "game.run_to_tick";
/// `game.state_hash`（当前状态哈希——断言锚）。
pub const STATE_HASH_METHOD: &str = "game.state_hash";
/// `game.outcome`（终局四元组）。
pub const OUTCOME_METHOD: &str = "game.outcome";
/// `game.sample_outcomes`（T023，M5-06 席位 8）：降规模口径批量采样——n 局
/// 独立对局终局序列 + 胜率分布（纯只读玩法面，不触碰 [`HostedGame`]）。
pub const SAMPLE_OUTCOMES_METHOD: &str = "game.sample_outcomes";
/// `game.run_tests`（套件判定面；本卡最小冒烟集，全量随 T020）。
pub const RUN_TESTS_METHOD: &str = "game.run_tests";
/// `game.screenshot`（T018 headless 桩保留；T019/D8 spectate 形态两段式实装，
/// 实装体在 [`crate::spectate`]，本模块 handler 只按形态分发）。
pub const SCREENSHOT_METHOD: &str = "game.screenshot";

/// 域错误码（D6）：正码 4xxx，避开 JSON-RPC 保留段 -32768..-32000。
pub mod game_error_codes {
    /// 尚未布阵（未 `game.deploy`）即调用需要对局的方法。
    pub const NOT_DEPLOYED: i16 = 4001;
    /// 对局已收束（终局冻结），拒绝继续推进。
    pub const BATTLE_RESOLVED: i16 = 4002;
    /// 观战模式未启用（headless 宿主，T018；实装随 T019）。
    pub const SPECTATE_NOT_ENABLED: i16 = 4101;
}

/// lane_len_m 缺省值（米）= `LANE_LEN_Q32` 的米口径（默认 lane 全长 1000 m）。
/// T021/D2 起 `pub(crate)`：presets「default」预设与 rpc 缺省同源单一值。
pub(crate) const DEFAULT_LANE_LEN_M: i64 = 1000;
/// threads 上限（1..=1024，与 sim CLI `--threads` 同域）。T021/D4 起
/// `pub(crate)`：host CLI 对 `--threads` 用同域校验（防两入口漂移）。
pub(crate) const MAX_THREADS: u64 = 1024;
/// 每方单位总数上限（防误配 OOM——D5）。T021/D4 起 `pub(crate)`：host CLI
/// 对 `--comp` 用同域校验（防两入口漂移）。
pub(crate) const MAX_UNITS_PER_SIDE: usize = 100_000;
/// 批量采样局数上限（T023/D2）：`game.sample_outcomes` 的 `games` 域
/// **1..=MAX_SAMPLE_GAMES**，超域 INVALID_PARAMS 消息列域。
pub(crate) const MAX_SAMPLE_GAMES: u64 = 1000;
/// 兵种六串清单（错误消息用；与 sim `kind_from_id`/`UnitKind::id` 同表——
/// 映射本体单一来源在 sim，此串仅供报错文案）。T021/D4 起 `pub(crate)`：
/// host CLI `--comp` 报错同串（防两入口文案漂移）。
pub(crate) const KIND_IDS: &str = "shieldman, heavyknight, pikeman, swordsman, archer, militia";

/// 本模块族共用的参数错误（JSON-RPC INVALID_PARAMS -32602，沿工作流仓先例）。
/// T019 起 `pub(crate)`：spectate 截图两段式的参数错误同码同构造（D8）。
pub(crate) fn invalid_params(message: &str) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.to_string(),
        data: None,
    }
}

/// 未布阵域错误（data 附提示，D5）。
fn not_deployed() -> BrpError {
    BrpError {
        code: game_error_codes::NOT_DEPLOYED,
        message: "no deployed game".into(),
        data: Some(json!({ "hint": "call game.deploy first" })),
    }
}

/// 对局配置（D4）：`game.outcome` 驱动语义需要 max_ticks；与 world / pool 在
/// `game.deploy` 时同步整体写入 [`HostedGame`]。其余字段为 deploy 时点记录
/// （T018 内无读者——后续卡/证据面使用），结构性豁免 dead_code。
#[allow(dead_code)]
pub struct GameConfig {
    pub seed: u64,
    pub max_ticks: u64,
    pub threads: usize,
    pub lane_len_m: i64,
}

/// 宿主对局状态（D4）：`game.deploy` 整体重置（旧 World / 线程池随赋值丢弃，
/// `ThreadPool` Drop 时 join worker）。sim 类型全限定（与 bevy `World` 同名，
/// 见模块注释命名纪律）。
///
/// T019/D5 增 `generation`：布阵世代替换计数——`apply_deploy` 每次递增（首布
/// = 1），表现层据此检测重部署并重建实体池（防 10k 级实体 churn 下的新旧
/// World 错配）。`Default` 派生给出 generation=0（未布阵语义）。
#[derive(Resource, Default)]
pub struct HostedGame {
    pub world: Option<sim::world::World>,
    pub pool: Option<sim::pool::ThreadPool>,
    pub config: Option<GameConfig>,
    /// 布阵世代（T019/D5）：初始 0，`apply_deploy` 递增。
    pub generation: u64,
}

/// 兵种 id → [`UnitKind`]：直接用 sim 公共 API `sim::units::kind_from_id`
/// （P1-1 整改：派工单 API 事实 8 曾漏列该函数致首轮本地重复表——审核轮指出后
/// 改调公共实现，id 表单一来源在 sim）。
fn kind_from_id(id: &str) -> Option<UnitKind> {
    sim::units::kind_from_id(id)
}

/// 构成清单解析（D5）：`[{kind, count}, ...]`。字段缺失/非数组/项缺键/kind 未知
/// /count 负或非整数 → INVALID_PARAMS；每方总数累计 ≤ [`MAX_UNITS_PER_SIDE`]
/// 超限拒绝（防误配 OOM）。
fn parse_composition(params: &Value, field: &str) -> Result<Vec<(UnitKind, usize)>, BrpError> {
    let value = params.get(field).ok_or_else(|| {
        invalid_params(&format!("missing {field} (array of {{kind, count}} required)"))
    })?;
    let arr = value
        .as_array()
        .ok_or_else(|| invalid_params(&format!("{field} must be an array of {{kind, count}}")))?;
    let mut out: Vec<(UnitKind, usize)> = Vec::with_capacity(arr.len());
    let mut total: usize = 0;
    for item in arr {
        let kind_id = item
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid_params(&format!("{field} item missing \"kind\" (string)")))?;
        let kind = kind_from_id(kind_id).ok_or_else(|| {
            invalid_params(&format!("unknown kind \"{kind_id}\" (valid: {KIND_IDS})"))
        })?;
        let count = item
            .get("count")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                invalid_params(&format!(
                    "{field} item \"{kind_id}\" count must be a non-negative integer"
                ))
            })?;
        let count = usize::try_from(count)
            .map_err(|_| invalid_params(&format!("{field} count {count} exceeds usize")))?;
        total = total.saturating_add(count);
        out.push((kind, count));
    }
    if total > MAX_UNITS_PER_SIDE {
        return Err(invalid_params(&format!(
            "{field} totals {total} units, exceeding per-side cap {MAX_UNITS_PER_SIDE}"
        )));
    }
    Ok(out)
}

/// 双方存活计数（host 侧数公共 `units()` 切片——O(N) 每步一次可忽略，D5）。
/// T019 起 `pub(crate)`：HUD 终局前行（席位 5，D7）与 sim 内部 `alive_counts`
/// 同口径（按索引序单遍），单一来源在本函数。
pub(crate) fn alive_counts(units: &[sim::world::Unit]) -> (u32, u32) {
    let mut red: u32 = 0;
    let mut blue: u32 = 0;
    for u in units {
        if u.alive {
            match u.side {
                sim::world::Side::Red => red += 1,
                sim::world::Side::Blue => blue += 1,
            }
        }
    }
    (red, blue)
}

/// 单侧构成解析（T021/D3）：显式字段（非缺键且非 null）一律覆盖预设底座；
/// 未给出且 preset 存在 → 取预设底座；两者皆无 → 原缺参错误路径
/// （`parse_composition` 的 "missing ..." 报错，语义逐字保留）。
fn resolve_side(
    params: &Value,
    field: &str,
    preset: Option<&[(UnitKind, usize)]>,
) -> Result<Vec<(UnitKind, usize)>, BrpError> {
    match params.get(field) {
        None | Some(Value::Null) => match preset {
            Some(comp) => Ok(comp.to_vec()),
            None => parse_composition(params, field),
        },
        _ => parse_composition(params, field),
    }
}

/// 已决布阵请求（T021/D1）：由两入口（BRP [`deploy_handler`] / CLI auto-deploy）
/// 的校验段产出，执行统一走 [`apply_deploy`]——入口无关性由构造保证。
/// 字段 `pub(crate)`：CLI 需在 main.rs 构造（D4）。
pub(crate) struct ResolvedDeploy {
    pub(crate) seed: u64,
    pub(crate) red: Vec<(UnitKind, usize)>,
    pub(crate) blue: Vec<(UnitKind, usize)>,
    pub(crate) lane_len_m: i64,
    pub(crate) max_ticks: u64,
    pub(crate) threads: usize,
}

/// deploy 执行段（T021/D1，单一代码路径）：建 World / Pool、HostedGame 整体
/// 重置（旧 World / 线程池随赋值丢弃，`ThreadPool` Drop 时 join worker）、响应
/// json 构造。BRP handler 与 CLI auto-deploy 共用；lane / max_ticks / threads /
/// 每方单位数等域校验已由各入口校验段完成（每方 ≤ [`MAX_UNITS_PER_SIDE`]、
/// lane ≥ 1 且 Q32.32 不溢出、max_ticks 1..=[`TICK_CAP_FULL`]、threads
/// 1..=[`MAX_THREADS`]）——本函数不重复拒绝，仅防御性复核 lane 溢出。
pub(crate) fn apply_deploy(hosted: &mut HostedGame, req: ResolvedDeploy) -> serde_json::Value {
    // 防御性复核（不可达路径）：两入口校验段均已做 checked_mul 溢出检查。
    let lane_q32 = req
        .lane_len_m
        .checked_mul(ONE_Q32_32)
        .expect("lane_len_m overflow-checked at deploy_handler/CLI validation");

    let game_world = sim::world::World::deploy_versus(req.seed, &req.red, &req.blue, lane_q32);
    let pool = sim::pool::ThreadPool::new(req.threads);

    let deploy_hash = format!("0x{:016x}", game_world.last_hash);
    let units = game_world.units().len();
    let (alive_red, alive_blue) = alive_counts(game_world.units());

    // T019/D5：世代递增（首布 = 1；wrapping 防理论溢出 panic——u64 实际不可达）。
    let generation = hosted.generation.wrapping_add(1);
    *hosted = HostedGame {
        world: Some(game_world),
        pool: Some(pool),
        config: Some(GameConfig {
            seed: req.seed,
            max_ticks: req.max_ticks,
            threads: req.threads,
            lane_len_m: req.lane_len_m,
        }),
        generation,
    };

    json!({
        "deploy_hash": deploy_hash,
        "tick": 0,
        "units": units,
        "alive_red": alive_red,
        "alive_blue": alive_blue,
    })
}

/// `game.deploy`（D5；T021/D3 增可选 preset）：params `{preset?, seed, red?,
/// blue?, lane_len_m?, max_ticks?, threads?}`。preset 给出（字符串）时以预设为
/// 底、显式字段一律覆盖（未知 preset → INVALID_PARAMS，消息列 [`presets::names`]；
/// null 与缺键同义——沿 lane/max_ticks/threads 既有约定）；无 preset 时语义与
/// T018 完全一致。布阵走 `deploy_versus`——镜像情形（red == blue 且 lane ==
/// `LANE_LEN_Q32`）与 `World::deploy` 逐位一致（sim 既有单测等价锚）。
/// HostedGame 整体重置；响应 `{deploy_hash, tick, units, alive_red, alive_blue}`。
pub fn deploy_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params = params.ok_or_else(|| {
        invalid_params("missing params (requires {\"seed\": u64, \"red\": [...], \"blue\": [...]})")
    })?;
    // T021/D3：可选 preset——先于其余字段解析（未知 preset 优先报，消息列清单）。
    let preset_base = match params.get("preset") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(presets::get(s).ok_or_else(|| {
            invalid_params(&format!(
                "unknown preset \"{s}\" (available: {})",
                presets::names().join(", ")
            ))
        })?),
        Some(_) => return Err(invalid_params("invalid preset (string required)")),
    };
    let seed = params
        .get("seed")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_params("missing/invalid seed (u64 required)"))?;
    let red = resolve_side(&params, "red", preset_base.as_ref().map(|p| &p.red[..]))?;
    let blue = resolve_side(&params, "blue", preset_base.as_ref().map(|p| &p.blue[..]))?;
    let lane_len_m = match params.get("lane_len_m") {
        None | Some(Value::Null) => match &preset_base {
            Some(p) => p.lane_len_m,
            None => DEFAULT_LANE_LEN_M,
        },
        Some(v) => v
            .as_i64()
            .filter(|m| *m >= 1)
            .ok_or_else(|| invalid_params("invalid lane_len_m (integer >= 1 required)"))?,
    };
    let max_ticks = match params.get("max_ticks") {
        None | Some(Value::Null) => match &preset_base {
            Some(p) => p.max_ticks,
            None => TICK_CAP_REDUCED,
        },
        Some(v) => v
            .as_u64()
            .filter(|t| (1..=TICK_CAP_FULL).contains(t))
            .ok_or_else(|| {
                invalid_params(&format!(
                    "invalid max_ticks (1..={TICK_CAP_FULL} required)"
                ))
            })?,
    };
    let threads = match params.get("threads") {
        None | Some(Value::Null) => 1,
        Some(v) => v
            .as_u64()
            .filter(|t| (1..=MAX_THREADS).contains(t))
            .ok_or_else(|| {
                invalid_params(&format!("invalid threads (1..={MAX_THREADS} required)"))
            })? as usize,
    };
    // 米 → Q32.32 定点乘数；checked_mul 防异常大输入的 i64 溢出（release 回绕
    // 不可接受，拒绝优于回绕）。T021/D1：校验留在本 handler（语义逐字保留），
    // 执行段（含 lane_q32 复算）统一进 [`apply_deploy`]。
    if lane_len_m.checked_mul(ONE_Q32_32).is_none() {
        return Err(invalid_params("lane_len_m too large (Q32.32 overflow)"));
    }

    let req = ResolvedDeploy {
        seed,
        red,
        blue,
        lane_len_m,
        max_ticks,
        threads,
    };
    let resp = {
        let mut hosted = world
            .get_resource_mut::<HostedGame>()
            .expect("HostedGame initialized by HostRpcPlugin");
        apply_deploy(&mut hosted, req)
    };
    // T019/D4：BRP deploy 恒 autorun=false（调用方驱动）。SpectateState 资源
    // 仅 spectate 形态存在（SpectatePlugin init）——headless 此处为 no-op，
    // 行为逐位不变。新布阵后旧 pending 作废（与新 World 无对应关系）。
    if let Some(mut st) = world.get_resource_mut::<SpectateState>() {
        st.pending = 0;
        st.autorun = false;
    }
    Ok(resp)
}

/// `game.run_to_tick`（D5）：params `{ticks: u64}` → `{tick, state_hash}`。
/// 语义：逐 tick 推进、每步前查双方存活——任一方归零即停于该 tick 并经
/// `run_battle_with(当前 tick)` 冻结终局（灭绝初检即收束、end_tick 真值；
/// 越过灭绝继续推会使 outcome 的 end_tick 失真——逐 tick 检查即为此语义代价）。
/// 未 deploy → [`game_error_codes::NOT_DEPLOYED`]；已冻结 →
/// [`game_error_codes::BATTLE_RESOLVED`]。
///
/// T019/D4 spectate 形态：`ticks` 入 [`SpectateState`] pending 队列**立即返回**
/// `{tick: <当前>, state_hash: <当前>, queued: n}`（响应多 `queued` 字段 = 模式
/// 差异留痕；headless 响应形态不变）；入队时 autorun=false（手动接管优先）。
/// 实际推进由 [`crate::spectate`] 驱动系统按 30Hz 预算经 [`advance_ticks`]
/// 执行——两形态唯一推进路径，确定性红线（同参数 state_hash 跨模式逐位一致）
/// 由构造保证。已冻结仍 4002（入队前拦截，两形态同域）。
pub fn run_to_tick_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params = params
        .ok_or_else(|| invalid_params("missing params (requires {\"ticks\": u64})"))?;
    let ticks = params
        .get("ticks")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_params("missing/invalid ticks (u64 required)"))?;

    // 冻结/未布阵拦截先行（读路径；T018 语义逐字保留：not_deployed 先于
    // BATTLE_RESOLVED，与原 mut 路径判序一致）。未冻结才允许入队/直推。
    {
        let hosted = world
            .get_resource::<HostedGame>()
            .expect("HostedGame initialized by HostRpcPlugin");
        let game = hosted.world.as_ref().ok_or_else(not_deployed)?;
        if let Some(outcome) = game.outcome() {
            return Err(BrpError {
                code: game_error_codes::BATTLE_RESOLVED,
                message: "battle already resolved; call game.deploy to start a new one".into(),
                data: Some(json!({
                    "winner": outcome.winner.label(),
                    "end_tick": outcome.end_tick,
                    "final_hash": format!("0x{:016x}", outcome.final_hash),
                })),
            });
        }
    }

    // T019/D4：spectate 入队（autorun=false 手动接管优先）；headless 无此资源
    // → false，走原直推路径。分支判定与改写先于 HostedGame 可变借用（两次
    // world 借用不重叠——handler 由 bevy_remote 经 run_system_with 独占执行）。
    let spectate = match world.get_resource_mut::<SpectateState>() {
        Some(mut st) => {
            st.pending = st.pending.saturating_add(ticks);
            st.autorun = false;
            true
        }
        None => false,
    };

    let mut hosted = world
        .get_resource_mut::<HostedGame>()
        .expect("HostedGame initialized by HostRpcPlugin");
    let HostedGame { world: game, pool, .. } = &mut *hosted;

    let game = game.as_mut().ok_or_else(not_deployed)?;

    if spectate {
        // 入队立即返回（模拟态未动；queued = 本次入队量）。
        return Ok(json!({
            "tick": game.tick,
            "state_hash": format!("0x{:016x}", game.last_hash),
            "queued": ticks,
        }));
    }

    advance_ticks(game, pool.as_ref(), ticks);

    Ok(json!({
        "tick": game.tick,
        "state_hash": format!("0x{:016x}", game.last_hash),
    }))
}

/// 共享推进函数（T019/D4）：headless `game.run_to_tick` 直推与 spectate 驱动
/// 系统的**唯一**模拟推进路径。循环体自 `run_to_tick_handler` 逐字迁移
/// （T018 原实现，语义零漂移）：逐 tick 灭绝检查——任一方归零即以
/// `run_battle_with(当前 tick)` 收束（end_tick 真值），否则推 1 tick 至 target。
/// 确定性注：`run_with(n)` 内部即 n 次 `step_with`（sim/src/world.rs:895-899），
/// 故分帧切块推进（spectate 30Hz 预算）与单次直推（headless）终态逐位一致。
pub(crate) fn advance_ticks(
    game: &mut sim::world::World,
    pool: Option<&sim::pool::ThreadPool>,
    ticks: u64,
) {
    let target = game.tick.saturating_add(ticks);
    loop {
        // 每步先数双方存活（灭绝即收束）；再查 target；否则推 1 tick。
        let (alive_red, alive_blue) = alive_counts(game.units());
        if alive_red == 0 || alive_blue == 0 {
            game.run_battle_with(game.tick, pool);
            break;
        }
        if game.tick >= target {
            break;
        }
        game.run_with(1, pool);
    }
}

/// `game.state_hash`（D5）：无 params → `{tick, state_hash}`（读 last_hash）。
/// 未 deploy → [`game_error_codes::NOT_DEPLOYED`]。
pub fn state_hash_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    // 纯读路径用共享引用（S-3 采纳；outcome/run_to_tick 因驱动模拟仍需 mut）。
    let hosted = world
        .get_resource::<HostedGame>()
        .expect("HostedGame initialized by HostRpcPlugin");
    let game = hosted.world.as_ref().ok_or_else(not_deployed)?;
    Ok(json!({
        "tick": game.tick,
        "state_hash": format!("0x{:016x}", game.last_hash),
    }))
}

/// `game.outcome`（D5）：无 params → 终局四元组。`run_battle_with` 幂等冻结：
/// 已收束直接返回缓存、未收束驱动至终局（灭绝或上限 hp 判定）——同一入口
/// 覆盖两语义。未 deploy → [`game_error_codes::NOT_DEPLOYED`]。
pub fn outcome_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let mut hosted = world
        .get_resource_mut::<HostedGame>()
        .expect("HostedGame initialized by HostRpcPlugin");
    let HostedGame { world: game, pool, config, .. } = &mut *hosted;

    let game = game.as_mut().ok_or_else(not_deployed)?;
    // config 与 world 在 deploy 时同步写入；config 为 None 仅当未 deploy
    // （上一行已拒）——兜底取降规模上限仅为类型完备，不可达路径。
    let max_ticks = config
        .as_ref()
        .map(|c| c.max_ticks)
        .unwrap_or(TICK_CAP_REDUCED);
    let outcome = game.run_battle_with(max_ticks, pool.as_ref());

    Ok(json!({
        "winner": outcome.winner.label(),
        "end_tick": outcome.end_tick,
        "alive_red": outcome.alive_red,
        "alive_blue": outcome.alive_blue,
        "final_hash": format!("0x{:016x}", outcome.final_hash),
    }))
}

/// `game.sample_outcomes`（T023，M5-06 席位 8；派工单 §2 语义定案）：params
/// `{red, blue, seed_base, games, lane_len_m?, max_ticks?, threads?}`。
/// red/blue 必填（复用 [`parse_composition`]，与 `game.deploy` 同解析同报错）；
/// lane/max_ticks/threads 缺省与域同 `game.deploy`（[`DEFAULT_LANE_LEN_M`] /
/// [`TICK_CAP_REDUCED`]（= 降规模 1800 ticks 字面）/ 1；threads 1..=[`MAX_THREADS`]）；
/// `games` 域 1..=[`MAX_SAMPLE_GAMES`]（超域 INVALID_PARAMS，消息列域）。
///
/// 批语义（确定性红线）：局 i ∈ 0..games-1，种子 seed_i =
/// `seed_base.wrapping_add(i)`（u64 模 2^64 回绕**显式留痕**——序列公开可复现）；
/// 每局全新 `World::deploy_versus`（沿 suite 净副作用纪律：每局独立 World，
/// 批与批之间零状态残留；**本 handler 不读写 [`HostedGame`]**——纯只读玩法面，
/// 不影响已布阵对局），`run_battle_with(max_ticks, pool)` 收束（局间串行、
/// 池跨局复用——执行资源；T006 跨线程纪律：结果与线程数无关）。
///
/// response：`{games, seed_base, red_wins, blue_wins, draws, win_rate_red_pp,
/// outcomes}`——`win_rate_red_pp = red_wins × 10000 / games`（**万分比整数，
/// 禁浮点统计**，判定与展示同源）；outcomes 全量（1000 局上限），每局六字段
/// `{seed, winner, end_tick, alive_red, alive_blue, final_hash}`（final_hash
/// `0x%016x` 与 `game.outcome` 同格式）。错误面：仅参数错 INVALID_PARAMS
/// （-32602）——本方法无 NOT_DEPLOYED/BATTLE_RESOLVED 路径（不触对局状态）。
pub fn sample_outcomes_handler(In(params): In<Option<Value>>, _world: &mut World) -> BrpResult {
    let params = params.ok_or_else(|| {
        invalid_params(
            "missing params (requires {\"red\": [...], \"blue\": [...], \"seed_base\": u64, \"games\": u64})",
        )
    })?;
    let red = parse_composition(&params, "red")?;
    let blue = parse_composition(&params, "blue")?;
    let seed_base = params
        .get("seed_base")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_params("missing/invalid seed_base (u64 required)"))?;
    let games = params
        .get("games")
        .and_then(Value::as_u64)
        .filter(|g| (1..=MAX_SAMPLE_GAMES).contains(g))
        .ok_or_else(|| {
            invalid_params(&format!("invalid games (1..={MAX_SAMPLE_GAMES} required)"))
        })?;
    let lane_len_m = match params.get("lane_len_m") {
        None | Some(Value::Null) => DEFAULT_LANE_LEN_M,
        Some(v) => v
            .as_i64()
            .filter(|m| *m >= 1)
            .ok_or_else(|| invalid_params("invalid lane_len_m (integer >= 1 required)"))?,
    };
    // 米 → Q32.32；checked_mul 防 i64 溢出（与 deploy_handler 同报错、同理由：
    // release 回绕不可接受，拒绝优于回绕）。
    let lane_q32 = lane_len_m
        .checked_mul(ONE_Q32_32)
        .ok_or_else(|| invalid_params("lane_len_m too large (Q32.32 overflow)"))?;
    let max_ticks = match params.get("max_ticks") {
        None | Some(Value::Null) => TICK_CAP_REDUCED,
        Some(v) => v
            .as_u64()
            .filter(|t| (1..=TICK_CAP_FULL).contains(t))
            .ok_or_else(|| {
                invalid_params(&format!(
                    "invalid max_ticks (1..={TICK_CAP_FULL} required)"
                ))
            })?,
    };
    let threads = match params.get("threads") {
        None | Some(Value::Null) => 1,
        Some(v) => v
            .as_u64()
            .filter(|t| (1..=MAX_THREADS).contains(t))
            .ok_or_else(|| {
                invalid_params(&format!("invalid threads (1..={MAX_THREADS} required)"))
            })? as usize,
    };

    let pool = sim::pool::ThreadPool::new(threads);
    let mut red_wins: u64 = 0;
    let mut blue_wins: u64 = 0;
    let mut draws: u64 = 0;
    let mut outcomes: Vec<Value> = Vec::with_capacity(games as usize);
    for i in 0..games {
        let seed = seed_base.wrapping_add(i);
        let mut game = sim::world::World::deploy_versus(seed, &red, &blue, lane_q32);
        let o = game.run_battle_with(max_ticks, Some(&pool));
        match o.winner {
            sim::world::Winner::Red => red_wins += 1,
            sim::world::Winner::Blue => blue_wins += 1,
            sim::world::Winner::Draw => draws += 1,
        }
        outcomes.push(json!({
            "seed": seed,
            "winner": o.winner.label(),
            "end_tick": o.end_tick,
            "alive_red": o.alive_red,
            "alive_blue": o.alive_blue,
            "final_hash": format!("0x{:016x}", o.final_hash),
        }));
    }
    // 万分比整数（red_wins ≤ games ≤ 1000 ⇒ 无溢出面）；games ≥ 1 已保证除零安全。
    let win_rate_red_pp = red_wins * 10_000 / games;

    Ok(json!({
        "games": games,
        "seed_base": seed_base,
        "red_wins": red_wins,
        "blue_wins": blue_wins,
        "draws": draws,
        "win_rate_red_pp": win_rate_red_pp,
        "outcomes": outcomes,
    }))
}

/// `game.run_tests`（D5/D8）：params `{suite: str}`（可选，缺省 = 默认套件）→
/// `{suite, total, passed, failed, results}`。断言失败 ≠ 协议错误（pass=false
/// 正常返回——判定主体语义）；套件无净副作用（全新 World，不碰 HostedGame）。
/// 套件注册与判定**全部委托 [`suite`] 模块**（T020 预备重构：本 handler 只做
/// 参数形状检查与响应组装，套件清单的单一来源在 suite.rs——后续卡扩充断言面
/// 不再触碰本文件）。
pub fn run_tests_handler(In(params): In<Option<Value>>, _world: &mut World) -> BrpResult {
    // P2-1 整改：suite 键存在但非字符串 → 显式 INVALID_PARAMS（缺键/null 才走缺省，
    // 不再静默回落默认套件）。
    let suite_name = match params.as_ref().and_then(|p| p.get("suite")) {
        None | Some(Value::Null) => suite::DEFAULT_SUITE,
        Some(Value::String(s)) => s.as_str(),
        Some(_) => {
            return Err(invalid_params(&format!(
                "invalid suite (string required; omit for default \"{}\")",
                suite::DEFAULT_SUITE
            )))
        }
    };
    let results = suite::run(suite_name).ok_or_else(|| {
        invalid_params(&format!(
            "unknown suite \"{suite_name}\" (available: {})",
            suite::names().join(", ")
        ))
    })?;
    let total = results.len();
    let passed = results.iter().filter(|r| r.pass).count();
    let failed = total - passed;
    let results: Vec<Value> = results
        .into_iter()
        .map(|r| json!({ "name": r.name, "pass": r.pass, "detail": r.detail }))
        .collect();

    Ok(json!({
        "suite": suite_name,
        "total": total,
        "passed": passed,
        "failed": failed,
        "results": results,
    }))
}

/// `game.screenshot`（D8 分形态）：headless（无 `SpectateState` 资源）→ 4101 桩
/// **逐字不变**（语义分阶段留痕沿 T018，t018 冒烟 CHK-12 回归面）；spectate →
/// 两段式实装（受理/轮询，见 [`crate::spectate::screenshot_handler`]）。
pub fn screenshot_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    if !world.contains_resource::<SpectateState>() {
        return Err(BrpError {
            code: game_error_codes::SPECTATE_NOT_ENABLED,
            message: "spectate mode not enabled (headless host, T018); arrives with T019".into(),
            data: Some(json!({ "mode": "headless", "planned_task": "T019" })),
        });
    }
    crate::spectate::screenshot_handler(params, world)
}

/// 缺省 BRP 监听端口（T021/D5：15702 不变；= bevy_remote DEFAULT_PORT 常量值，
/// bevy_remote-0.19.1/src/http.rs:52——本仓自持字面量，不随上游漂移）。
pub const DEFAULT_BRP_PORT: u16 = 15702;

/// 宿主 BRP 插件（D1/D3/D11；T021/D5 增 `port` 字段；T023 注册 +1 方法）：资源
/// 初始化 + 7 方法注册 + 显式回环 HTTP 绑定。**地址恒为 127.0.0.1**（回环硬约束
/// 不变——BRP 无鉴权，禁止绑定非回环地址；显式绑定防上游默认值漂移）；端口缺省
/// [`DEFAULT_BRP_PORT`]、经 `port` 字段可覆盖（CLI `--port` 落地）。留痕
/// （D5）：并行波次验证隔离（多树多实例同机）与后续多实例实验需要；
/// T018「写死」精确化为「地址写死回环、端口默认 15702 可选覆盖」。
pub struct HostRpcPlugin {
    /// 监听端口（T021/D5）。
    pub port: u16,
}

impl Plugin for HostRpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HostedGame>();
        app.add_plugins(
            RemotePlugin::default()
                .with_method_main(DEPLOY_METHOD, deploy_handler)
                .with_method_main(RUN_TO_TICK_METHOD, run_to_tick_handler)
                .with_method_main(STATE_HASH_METHOD, state_hash_handler)
                .with_method_main(OUTCOME_METHOD, outcome_handler)
                .with_method_main(SAMPLE_OUTCOMES_METHOD, sample_outcomes_handler)
                .with_method_main(RUN_TESTS_METHOD, run_tests_handler)
                .with_method_main(SCREENSHOT_METHOD, screenshot_handler),
        );
        app.add_plugins(
            RemoteHttpPlugin::default()
                .with_address(std::net::Ipv4Addr::LOCALHOST)
                .with_port(self.port),
        );
    }
}
