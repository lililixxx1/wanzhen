//! BRP `game.*` 初始集 6 方法 + 宿主状态资源 [`HostedGame`] + [`HostRpcPlugin`]
//! 组装（T018，M5 席位 2；依据 docs/evidence/t018/dispatch-sheet.md「实现清单」
//! 节 + taskset/t018-brp-host.md 设计裁决 D1~D11 + 备忘录 §五）。
//!
//! 方法面（备忘录 §五逐字，6/6）：[`DEPLOY_METHOD`] / [`RUN_TO_TICK_METHOD`] /
//! [`STATE_HASH_METHOD`] / [`OUTCOME_METHOD`] / [`RUN_TESTS_METHOD`] /
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

use bevy::prelude::*;
use bevy::remote::error_codes;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::{BrpError, BrpResult, RemotePlugin};
use serde_json::{json, Value};
use sim::units::{UnitKind, ONE_Q32_32};
use sim::world::{TICK_CAP_FULL, TICK_CAP_REDUCED};

use crate::suite;

/// `game.deploy`（config：构成/参数/种子 → 布阵快照哈希）。
pub const DEPLOY_METHOD: &str = "game.deploy";
/// `game.run_to_tick`（headless 直推 n tick；观战模式帧率节流随 T019）。
pub const RUN_TO_TICK_METHOD: &str = "game.run_to_tick";
/// `game.state_hash`（当前状态哈希——断言锚）。
pub const STATE_HASH_METHOD: &str = "game.state_hash";
/// `game.outcome`（终局四元组）。
pub const OUTCOME_METHOD: &str = "game.outcome";
/// `game.run_tests`（套件判定面；本卡最小冒烟集，全量随 T020）。
pub const RUN_TESTS_METHOD: &str = "game.run_tests";
/// `game.screenshot`（headless 桩：结构化错误，实装随 T019）。
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
const DEFAULT_LANE_LEN_M: i64 = 1000;
/// threads 上限（1..=1024，与 sim CLI `--threads` 同域）。
const MAX_THREADS: u64 = 1024;
/// 每方单位总数上限（防误配 OOM——D5）。
const MAX_UNITS_PER_SIDE: usize = 100_000;
/// 兵种六串清单（错误消息用；与 sim `kind_from_id`/`UnitKind::id` 同表——
/// 映射本体单一来源在 sim，此串仅供报错文案）。
const KIND_IDS: &str = "shieldman, heavyknight, pikeman, swordsman, archer, militia";

/// 本模块族共用的参数错误（JSON-RPC INVALID_PARAMS -32602，沿工作流仓先例）。
fn invalid_params(message: &str) -> BrpError {
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
#[derive(Resource, Default)]
pub struct HostedGame {
    pub world: Option<sim::world::World>,
    pub pool: Option<sim::pool::ThreadPool>,
    pub config: Option<GameConfig>,
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
fn alive_counts(units: &[sim::world::Unit]) -> (u32, u32) {
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

/// `game.deploy`（D5）：params `{seed, red, blue, lane_len_m=1000,
/// max_ticks=1800, threads=1}`。布阵走 `deploy_versus`——镜像情形（red == blue
/// 且 lane == `LANE_LEN_Q32`）与 `World::deploy` 逐位一致（sim 既有单测等价锚）。
/// HostedGame 整体重置；响应 `{deploy_hash, tick, units, alive_red, alive_blue}`。
pub fn deploy_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params = params.ok_or_else(|| {
        invalid_params("missing params (requires {\"seed\": u64, \"red\": [...], \"blue\": [...]})")
    })?;
    let seed = params
        .get("seed")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_params("missing/invalid seed (u64 required)"))?;
    let red = parse_composition(&params, "red")?;
    let blue = parse_composition(&params, "blue")?;
    let lane_len_m = match params.get("lane_len_m") {
        None | Some(Value::Null) => DEFAULT_LANE_LEN_M,
        Some(v) => v
            .as_i64()
            .filter(|m| *m >= 1)
            .ok_or_else(|| invalid_params("invalid lane_len_m (integer >= 1 required)"))?,
    };
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
    // 米 → Q32.32 定点乘数；checked_mul 防异常大输入的 i64 溢出（release 回绕
    // 不可接受，拒绝优于回绕）。
    let lane_q32 = lane_len_m
        .checked_mul(ONE_Q32_32)
        .ok_or_else(|| invalid_params("lane_len_m too large (Q32.32 overflow)"))?;

    let game_world = sim::world::World::deploy_versus(seed, &red, &blue, lane_q32);
    let pool = sim::pool::ThreadPool::new(threads);

    let deploy_hash = format!("0x{:016x}", game_world.last_hash);
    let units = game_world.units().len();
    let (alive_red, alive_blue) = alive_counts(game_world.units());

    let mut hosted = world
        .get_resource_mut::<HostedGame>()
        .expect("HostedGame initialized by HostRpcPlugin");
    *hosted = HostedGame {
        world: Some(game_world),
        pool: Some(pool),
        config: Some(GameConfig {
            seed,
            max_ticks,
            threads,
            lane_len_m,
        }),
    };

    Ok(json!({
        "deploy_hash": deploy_hash,
        "tick": 0,
        "units": units,
        "alive_red": alive_red,
        "alive_blue": alive_blue,
    }))
}

/// `game.run_to_tick`（D5）：params `{ticks: u64}` → `{tick, state_hash}`。
/// 语义：逐 tick 推进、每步前查双方存活——任一方归零即停于该 tick 并经
/// `run_battle_with(当前 tick)` 冻结终局（灭绝初检即收束、end_tick 真值；
/// 越过灭绝继续推会使 outcome 的 end_tick 失真——逐 tick 检查即为此语义代价）。
/// 未 deploy → [`game_error_codes::NOT_DEPLOYED`]；已冻结 →
/// [`game_error_codes::BATTLE_RESOLVED`]。
pub fn run_to_tick_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params = params
        .ok_or_else(|| invalid_params("missing params (requires {\"ticks\": u64})"))?;
    let ticks = params
        .get("ticks")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid_params("missing/invalid ticks (u64 required)"))?;

    let mut hosted = world
        .get_resource_mut::<HostedGame>()
        .expect("HostedGame initialized by HostRpcPlugin");
    let HostedGame { world: game, pool, .. } = &mut *hosted;

    let game = game.as_mut().ok_or_else(not_deployed)?;
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

    let target = game.tick.saturating_add(ticks);
    loop {
        // 每步先数双方存活（灭绝即收束）；再查 target；否则推 1 tick。
        let (alive_red, alive_blue) = alive_counts(game.units());
        if alive_red == 0 || alive_blue == 0 {
            game.run_battle_with(game.tick, pool.as_ref());
            break;
        }
        if game.tick >= target {
            break;
        }
        game.run_with(1, pool.as_ref());
    }

    Ok(json!({
        "tick": game.tick,
        "state_hash": format!("0x{:016x}", game.last_hash),
    }))
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
    let HostedGame { world: game, pool, config } = &mut *hosted;

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

/// `game.run_tests`（D5/D8）：params `{suite: "t018-smoke"}`（可选，缺省即
/// 冒烟集）→ `{suite, total, passed, failed, results}`。断言失败 ≠ 协议错误
/// （pass=false 正常返回——判定主体语义）；套件无净副作用（全新 World，不碰
/// HostedGame）。非冒烟套件名 → INVALID_PARAMS（全量断言面随 T020）。
pub fn run_tests_handler(In(params): In<Option<Value>>, _world: &mut World) -> BrpResult {
    // P2-1 整改：suite 键存在但非字符串 → 显式 INVALID_PARAMS（缺键/null 才走缺省，
    // 不再静默回落默认套件）。
    let suite_name = match params.as_ref().and_then(|p| p.get("suite")) {
        None | Some(Value::Null) => suite::SMOKE_SUITE,
        Some(Value::String(s)) => s.as_str(),
        Some(_) => {
            return Err(invalid_params(
                "invalid suite (string required; omit for default \"t018-smoke\")",
            ))
        }
    };
    if suite_name != suite::SMOKE_SUITE {
        return Err(invalid_params(&format!(
            "unknown suite \"{suite_name}\" (only \"{}\" exists in T018; full suite arrives with T020)",
            suite::SMOKE_SUITE
        )));
    }

    let results = suite::run_smoke();
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

/// `game.screenshot`（D5/D6 headless 桩）：恒返回结构化错误
/// [`game_error_codes::SPECTATE_NOT_ENABLED`]，不击穿进程；实装随 T019。
pub fn screenshot_handler(In(_params): In<Option<Value>>, _world: &mut World) -> BrpResult {
    Err(BrpError {
        code: game_error_codes::SPECTATE_NOT_ENABLED,
        message: "spectate mode not enabled (headless host, T018); arrives with T019".into(),
        data: Some(json!({ "mode": "headless", "planned_task": "T019" })),
    })
}

/// 宿主 BRP 插件（D1/D3/D11）：资源初始化 + 6 方法注册 + 显式回环 HTTP 绑定。
/// 端口/地址写死 127.0.0.1:15702（= bevy_remote DEFAULT_ADDR/DEFAULT_PORT 常量
/// 值），无 CLI 参数（沿工作流仓「写死不暴露」先例）；回环约束代码侧留痕。
pub struct HostRpcPlugin;

impl Plugin for HostRpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HostedGame>();
        app.add_plugins(
            RemotePlugin::default()
                .with_method_main(DEPLOY_METHOD, deploy_handler)
                .with_method_main(RUN_TO_TICK_METHOD, run_to_tick_handler)
                .with_method_main(STATE_HASH_METHOD, state_hash_handler)
                .with_method_main(OUTCOME_METHOD, outcome_handler)
                .with_method_main(RUN_TESTS_METHOD, run_tests_handler)
                .with_method_main(SCREENSHOT_METHOD, screenshot_handler),
        );
        app.add_plugins(
            RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST),
        );
    }
}
