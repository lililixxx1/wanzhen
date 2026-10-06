//! 观战形态核心（T019，M5 席位 1 观战/4/5 的状态与驱动面；窗口/表现层/HUD 的
//! 装配在 [`crate::main`] D3，实现体在 [`crate::present`] / [`crate::hud`]）。
//!
//! 职责（派工单 D4/D8）：
//! - [`SpectateState`]：pending 队列 + autorun 旗（Resource，仅 spectate 形态
//!   存在——**资源存在性即形态判据**，[`crate::rpc`] 的 run_to_tick/screenshot/
//!   deploy 分支据此路由，headless 路径零漂移）；
//! - [`spectate_driver`]：Update 驱动系统——Time 累积 30Hz 预算（表 6-0 tick
//!   口径），每帧推 `min(预算, pending)`（autorun 恒推预算量）；推进**只经**
//!   [`crate::rpc::advance_ticks`]（与 headless 直推同一函数——确定性红线
//!   「同参数 state_hash 跨模式逐位一致」由构造保证）；
//! - `game.screenshot` 两段式实装（D8）：受理（spawn `Screenshot` + observer）
//!   → 轮询至事件到达 → PNG 魔数核对 → captured；
//! - [`SpectatePlugin`]：资源初始化 + 系统注册（驱动 → 表现重建 → 表现更新 →
//!   HUD，`.chain()` 定序）。
//!
//! autorun 语义（D4）：CLI auto-deploy + `--spectate` → autorun=true（部署即
//! 自走至终局）；BRP deploy 恒 autorun=false（rpc::deploy_handler 复位）；
//! run_to_tick 入队时 autorun=false（手动接管优先）。终局 = `outcome()` 存在
//! （灭绝经 advance_ticks 内 run_battle_with 收束；无灭绝配置收于 max_ticks
//! 上限——autorun 且 tick ≥ max_ticks 时调 `run_battle_with(max_ticks)` 走
//! resolve_by_hp 上限判定，不再推 tick。T021 归档实测：melee-brawl seed7 无
//! 灭绝、run_to_tick 1800 收于上限 tick=1800；Lead 批复（2026-10-07）该收口
//! 属 D4「终局 = outcome 存在」语义内，任务卡执行记录留痕）。

use bevy_full as bevy;
// derive 宏（下方 Resource 等）展开为 `bevy_ecs::` 绝对路径；依赖名为别名
// bevy_full 时宏无法经清单解析命中，按 bevy_macro_utils 0.19.1 内置说明补别名
// （render-spike/src/main.rs:23-26 + T010 api-notes 先例）。
use bevy_full::ecs as bevy_ecs;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use bevy::remote::error_codes;
use bevy::remote::{BrpError, BrpResult};
use serde_json::{json, Value};
use sim::world::TICK_CAP_REDUCED;

use crate::hud;
use crate::present;
use crate::rpc::{advance_ticks, invalid_params, HostedGame};

/// 观战驱动 tick 速率（表 6-0：tick 固定 30Hz——观战节流预算基准）。
pub(crate) const TICK_RATE_HZ: f64 = 30.0;

/// 观战状态（D4）：pending 队列 + autorun 旗。仅 spectate 形态插入资源
/// （[`SpectatePlugin`] init；headless 无——资源存在性即形态判据）。
#[derive(Resource, Debug, Default)]
pub struct SpectateState {
    /// 待推进 tick 预算：`game.run_to_tick {ticks:n}` spectate 形态累加入队
    /// （驱动系统每帧消费 min(30Hz 预算, pending)）。
    pub pending: u64,
    /// 自走旗：true = 驱动系统每帧恒推预算量至终局（CLI auto-deploy 观战腿）；
    /// false = 仅按 pending 推进（BRP deploy / run_to_tick 入队时置 false）。
    pub autorun: bool,
}

/// 截图日志（D8）：受理即建 pending 行，捕获完成由 observer 翻状态。
/// 查询轮询（`{"id":n}`）按 id 定位；文件核验（PNG 魔数 + 字节数）在轮询应答
/// 时执行——最强证据在文件本身（工作流仓 screenshot.rs 同口径）。
#[derive(Resource, Default)]
pub struct ScreenshotLog {
    /// 递增 id（默认文件名序号同源）。
    next_id: u32,
    /// 全部条目（含 pending 与 captured；不清理——查证用）。
    entries: Vec<ScreenshotEntry>,
}

/// 条目状态机：`pending`（ScreenshotCaptured 未到）/ `captured`（事件已到；
/// 响应侧 captured 还须过 PNG 魔数核对——D8「PNG 魔数核对后置 captured」）。
#[derive(Debug, PartialEq, Eq)]
enum EntryStatus {
    Pending,
    Captured,
}

/// 单条截图记录。
#[derive(Debug)]
struct ScreenshotEntry {
    id: u32,
    /// 落盘目标路径（CWD 相对，D8；原样记录）。
    path: String,
    status: EntryStatus,
    /// 受理的 Screenshot 实体（observer 回填定位键）。
    entity: Entity,
}

impl ScreenshotLog {
    /// 记一条受理（id 取自 `next_id` 并自增）。
    fn request(&mut self, path: String, entity: Entity) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(ScreenshotEntry {
            id,
            path,
            status: EntryStatus::Pending,
            entity,
        });
        id
    }
}

/// 观战插件（T019/D3）：资源初始化 + 系统注册。headless 形态不 add 本插件。
pub struct SpectatePlugin;

impl Plugin for SpectatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpectateState>()
            .init_resource::<ScreenshotLog>()
            // 表现层资源（P1 修复：PresentationRoot 此前漏 init——
            // rebuild_on_deploy 首帧 ResMut 校验失败 panic，宿主退出；
            // PresentationAssets 由 setup_assets（Startup）经 Commands 插入）。
            .init_resource::<crate::present::PresentationRoot>()
            // Startup：表现层共享资产（6 mesh × 2 材质）+ HUD 三行文本。
            .add_systems(Startup, (present::setup_assets, hud::spawn_hud))
            // Update 定序（.chain()）：驱动（模拟推进）→ 表现重建（generation
            // 世代替换检测）→ 表现逐帧映射（只读 HostedGame）→ HUD 刷新。
            .add_systems(
                Update,
                (
                    spectate_driver,
                    present::rebuild_on_deploy,
                    present::update_presentation,
                    hud::update_hud,
                )
                    .chain(),
            );
    }
}

/// 驱动系统（D4）：30Hz 预算节流的模拟推进。签名取 `ResMut<HostedGame>`——
/// 模拟态推进的**唯一**系统入口（表现层/HUD 全部只读 `Res<HostedGame>`，
/// 单向只读红线不受影响）。
fn spectate_driver(
    mut st: ResMut<SpectateState>,
    mut hosted: ResMut<HostedGame>,
    time: Res<Time>,
    mut acc: Local<f64>,
) {
    let HostedGame {
        world: game,
        pool,
        config,
        ..
    } = &mut *hosted;
    let Some(game) = game.as_mut() else {
        // 未布阵：无推进对象；状态清零（autorun 误开也无副作用）。
        st.pending = 0;
        st.autorun = false;
        return;
    };
    // 冻结即清（D4：pending=0、autorun 停）——幂等。
    if game.outcome().is_some() {
        st.pending = 0;
        st.autorun = false;
        return;
    }

    // 30Hz 预算：Time 累积（首帧起 accumulator），整 tick 消费、余量留存。
    // f64 as u64 截断（acc 恒 ≥ 0：只加正量、减整量；Rust 语义负值饱和为 0
    // 双保险）。
    *acc += time.delta().as_secs_f64() * TICK_RATE_HZ;
    let whole = *acc as u64;
    if whole == 0 {
        return; // 预算不足 1 tick——模拟态不动
    }
    *acc -= whole as f64;

    // autorun 恒推预算量至终局；手动模式推 min(预算, pending)。
    let n = if st.autorun {
        whole
    } else {
        whole.min(st.pending)
    };
    if n == 0 {
        return; // 非 autorun 且无 pending——预算作废（挂钟口径，不留存）
    }
    st.pending = st.pending.saturating_sub(n);

    // 推进走共享函数（与 headless 直推同路径；run_with(n) = n 次 step_with，
    // 分帧切块与单次直推终态逐位一致——确定性红线）。
    advance_ticks(game, pool.as_ref(), n);

    // autorun 上限收束（终局 = outcome 存在，D4；语义见模块注释——Lead 批复
    // 2026-10-07）。手动模式不收口：target 语义与 headless run_to_tick 一致。
    if st.autorun && game.outcome().is_none() {
        let max_ticks = config
            .as_ref()
            .map(|c| c.max_ticks)
            .unwrap_or(TICK_CAP_REDUCED);
        if game.tick >= max_ticks {
            game.run_battle_with(max_ticks, pool.as_ref());
        }
    }

    // 冻结（灭绝收束或上限收束）→ 状态清零。
    if game.outcome().is_some() {
        st.pending = 0;
        st.autorun = false;
    }
}

// ── game.screenshot 两段式实装（T019/D8）──────────────────────────────────

/// `game.screenshot` spectate 实装（由 [`crate::rpc::screenshot_handler`] 分发；
/// headless 4101 桩留在 rpc.rs 逐字不变）。直接函数调用（非 BRP 直注册），
/// 故 params 为已解包的 `Option<Value>`（`In` 包装由 rpc 侧 handler 承担）。
/// 契约（D8）：
/// - `{}` / null → 受理：`{status:"requested", id}`；
/// - `{"id":n}` → 轮询：`{status:"pending"}` / `{status:"captured", path, bytes}`
///   （事件到达后过 PNG 魔数核对才应答 captured）/ 未知 id → -32602。
pub(crate) fn screenshot_handler(params: Option<Value>, world: &mut World) -> BrpResult {
    match params {
        None | Some(Value::Null) => request_screenshot(world),
        Some(Value::Object(ref map)) if map.is_empty() => request_screenshot(world),
        Some(Value::Object(ref map)) => {
            let raw = map
                .get("id")
                .ok_or_else(|| invalid_params("missing id (poll requires {\"id\": u64})"))?;
            let id = raw
                .as_u64()
                .ok_or_else(|| invalid_params("invalid id (u64 required)"))?;
            let id = u32::try_from(id)
                .map_err(|_| invalid_params(&format!("id {id} exceeds u32 range")))?;
            poll_screenshot(world, id)
        }
        Some(other) => Err(invalid_params(&format!(
            "params must be empty (request) or {{\"id\": u64}} (poll); got {other}"
        ))),
    }
}

/// 受理：spawn 携带 [`Screenshot`] 的实体 + 双 observer（官方 `save_to_disk`
/// 同步写 PNG；[`on_captured`] 翻日志状态），返回 `{status:"requested", id}`。
///
/// 依据（0.19.1 registry，逐条行号另见 docs/evidence/t019/api-notes.md）：
/// `Screenshot(pub RenderTarget)`/`primary_window()`（bevy_render-0.19.1/src/
/// view/window/screenshot.rs:80/98）、`ScreenshotCaptured { entity, image }`
/// （:49-52）、`save_to_disk`（:134）、`EntityWorldMut::observe`（工作流仓先例
/// game/src/rpc/screenshot.rs 同款两 observer 形态）。
fn request_screenshot(world: &mut World) -> BrpResult {
    // 默认文件名需要 id：handler 单线程独占执行，读后 spawn 前无人改写
    // （工作流仓先例同款注释口径）。
    let id_for_name = world.resource::<ScreenshotLog>().next_id;
    let path = format!("screenshot-{id_for_name}.png");
    let entity = world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(std::path::PathBuf::from(path.clone())))
        .observe(on_captured)
        .id();
    let id = world
        .resource_mut::<ScreenshotLog>()
        .request(path, entity);
    info!("[spectate] screenshot #{id} requested -> screenshot-{id}.png (entity {entity})");
    Ok(json!({ "status": "requested", "id": id }))
}

/// 轮询：`{"id":n}` → 状态应答。captured 分支现场核验 PNG 魔数（\x89PNG\r\n\x1a\n）
/// 与字节数——文件缺失/魔数不符 → -32603 内部错误（不静默 pending，防轮询死等）。
fn poll_screenshot(world: &World, id: u32) -> BrpResult {
    let log = world.resource::<ScreenshotLog>();
    let entry = log
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| invalid_params(&format!("unknown screenshot id {id}")))?;
    match entry.status {
        EntryStatus::Pending => Ok(json!({ "status": "pending" })),
        EntryStatus::Captured => {
            let path = entry.path.clone();
            let mut file = std::fs::File::open(&path).map_err(|e| {
                internal_error(format!("screenshot #{id} captured but file unreadable: {e}"))
            })?;
            let mut magic = [0u8; 8];
            std::io::Read::read_exact(&mut file, &mut magic).map_err(|e| {
                internal_error(format!("screenshot #{id} file too small for PNG magic: {e}"))
            })?;
            if magic != PNG_MAGIC {
                return Err(internal_error(format!(
                    "screenshot #{id} PNG magic mismatch: {:02x?}",
                    magic
                )));
            }
            let bytes = std::fs::metadata(&path)
                .map_err(|e| internal_error(format!("screenshot #{id} metadata unreadable: {e}")))?
                .len();
            Ok(json!({ "status": "captured", "path": path, "bytes": bytes }))
        }
    }
}

/// PNG 文件魔数（\x89PNG\r\n\x1a\n，8 字节）。
const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// 内部错误（-32603，bevy_remote-0.19.1/src/lib.rs:1404）。
fn internal_error(message: String) -> BrpError {
    BrpError {
        code: error_codes::INTERNAL_ERROR,
        message,
        data: None,
    }
}

/// observer：捕获完成（`ScreenshotCaptured` 实体事件）→ 翻日志状态。
/// 与官方 `save_to_disk` 并挂同一实体（两 observer 相对次序无契约——工作流仓
/// 先例口径），故本条目只记「事件已到」，文件核验归轮询应答（见
/// [`poll_screenshot`]）。
fn on_captured(event: On<ScreenshotCaptured>, mut log: ResMut<ScreenshotLog>) {
    let entity = event.entity;
    if let Some(entry) = log
        .entries
        .iter_mut()
        .rev()
        .find(|e| e.entity == entity && e.status == EntryStatus::Pending)
    {
        entry.status = EntryStatus::Captured;
        info!("[spectate] screenshot #{} captured event arrived", entry.id);
    } else {
        warn!("[spectate] ScreenshotCaptured matched no pending log entry (entity {entity})");
    }
}
