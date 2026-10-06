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
//! - `game.screenshot` 两段式实装（D8 + P1 整改 2026-10-07）：受理（spawn
//!   `Screenshot` + observer，文件名 `screenshot-{pid}-{nonce}-{id}.png` 跨
//!   进程不复用——pid+毫秒双段，活进程 pid 全机唯一封死「同毫秒双进程」碰撞窗
//!   （修后复审 Important 86 整改））→ 轮询至事件到达 → **本次输出文件核验通过
//!   才置 captured**（存在
//!   + PNG 魔数 + 非零字节 + mtime ≥ 受理时刻）——事件到达不等于落盘成功
//!   （0.19.1 `save_to_disk` 写盘失败只记日志，api-notes §5）；
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

/// 截图日志（D8 + P1 整改 2026-10-07）：受理即建 pending 行；`ScreenshotCaptured`
/// 事件到达只记 `event_arrived`（**事件 ≠ 落盘**——0.19.1 `save_to_disk` 写盘
/// 失败只记日志，bevy_render-0.19.1/src/view/window/screenshot.rs:145-147，
/// api-notes §5）；**captured 状态只在轮询现场对本次输出文件核验通过后置**
/// （存在 + PNG 魔数 + 非零字节 + mtime ≥ 受理时刻，见 [`verify_captured_file`]）。
/// 查询轮询（`{"id":n}`）按 id 定位；不清理——查证用。
#[derive(Resource)]
pub struct ScreenshotLog {
    /// 进程启动 nonce 双段（pid + Unix 纪元毫秒，`init_resource` 时取一次）：
    /// 文件名 `screenshot-{pid}-{nonce}-{id}.png` 跨进程不复用——修复进程重启
    /// 后 `next_id` 从 0 起、`screenshot-0.png` 路径复用导致旧文件可能被误认为
    /// 本次产物（复审 P1）。表现层非模拟态，允许非确定值（派工单 P1 注）。
    /// 毫秒段单独不够（修后复审 Important 86：同毫秒双进程同目录理论碰撞）；
    /// pid 段补位——**活进程 pid 全机唯一**，同毫秒并存的两进程 pid 必异，
    /// 碰撞仅剩「pid 退出后被复用 + 同毫秒 + 同目录」三重叠加且 mtime ≥ 受理
    /// 时刻核验仍兜底（冒烟段 4/5 断言体例）。
    pid: u32,
    nonce: u64,
    /// 递增 id（文件名序号同源）。
    next_id: u32,
    /// 全部条目（含 pending 与 captured；不清理——查证用）。
    entries: Vec<ScreenshotEntry>,
}

impl Default for ScreenshotLog {
    fn default() -> Self {
        Self {
            pid: std::process::id(),
            nonce: u64::try_from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or_default(),
            )
            .unwrap_or_default(),
            next_id: 0,
            entries: Vec::new(),
        }
    }
}

/// 条目状态机：`pending`（ScreenshotCaptured 未到，或事件已到但本次输出文件
/// 核验未过——原因随轮询应答 `detail` 说明）/ `captured`（轮询现场核验通过，
/// 见 [`poll_screenshot`]——P1 整改：不再由 observer 直接置位）。
#[derive(Debug, PartialEq, Eq)]
enum EntryStatus {
    Pending,
    Captured,
}

/// 单条截图记录。
#[derive(Debug)]
struct ScreenshotEntry {
    id: u32,
    /// 落盘目标路径（CWD 相对，D8；`screenshot-{pid}-{nonce}-{id}.png`，P1 +
    /// 修后复审 Important 86 整改）。
    path: String,
    status: EntryStatus,
    /// 受理的 Screenshot 实体（observer 回填定位键）。
    entity: Entity,
    /// 受理时刻（P1 整改）：输出文件 mtime ≥ 此刻才认作本次产物——防同路径
    /// 预置残留文件在本次保存失败时被误认（残留文件可满足存在/魔数/非零
    /// 三校验，唯时间戳能区分「本次产物」与「残留」，冒烟段 5 物理注入实证）。
    requested_at: std::time::SystemTime,
    /// ScreenshotCaptured 事件是否已到（到达 ≠ 核验通过——状态机只进到这里，
    /// captured 由轮询核验置位）。
    event_arrived: bool,
}

impl ScreenshotLog {
    /// 记一条受理（id 取自 `next_id` 并自增；受理时刻同点记录——mtime 界桩）。
    fn request(&mut self, path: &str, entity: Entity) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(ScreenshotEntry {
            id,
            path: path.to_string(),
            status: EntryStatus::Pending,
            entity,
            requested_at: std::time::SystemTime::now(),
            event_arrived: false,
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
            // T024/D1：帧采集状态（缺省 = 未启用零开销；`--frame-capture` 时
            // main 在插件装配后 insert_resource 覆盖为启用态）。
            .init_resource::<crate::frame_capture::FrameCaptureState>()
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
            )
            // T024/D1：帧采集系统（独立于表现链——只读 Time/HostedGame，
            // 未启用时立即返回零开销）。
            .add_systems(Update, crate::frame_capture::frame_capture_system);
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
/// 契约（D8 + P1 整改）：
/// - `{}` / null → 受理：`{status:"requested", id}`；
/// - `{"id":n}` → 轮询：`{status:"pending"}`（事件未到，或事件已到但本次输出
///   文件核验未过——后者附 `detail` 说明，**不得报 captured**）/ `{status:
///   "captured", path, bytes}`（本次输出文件核验通过后应答并置 captured）/
///   未知 id → -32602 / 已 captured 但文件事后损坏 → -32603。
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
/// 同步写 PNG；[`on_captured`] 只记事件到达，captured 归轮询核验置位——P1
/// 整改），返回 `{status:"requested", id}`。
///
/// 依据（0.19.1 registry，逐条行号另见 docs/evidence/t019/api-notes.md）：
/// `Screenshot(pub RenderTarget)`/`primary_window()`（bevy_render-0.19.1/src/
/// view/window/screenshot.rs:80/98）、`ScreenshotCaptured { entity, image }`
/// （:49-52）、`save_to_disk`（:134，写盘失败只记日志 :145-147）、
/// `EntityWorldMut::observe`（工作流仓先例 game/src/rpc/screenshot.rs 同款两
/// observer 形态）。
fn request_screenshot(world: &mut World) -> BrpResult {
    // 默认文件名需要 pid + nonce + id：handler 单线程独占执行，读后 spawn 前
    // 无人改写（工作流仓先例同款注释口径）。pid+毫秒双段语义见
    // [`ScreenshotLog`] 字段注（P1 + 修后复审 Important 86）。
    let (pid, nonce, id_for_name) = {
        let log = world.resource::<ScreenshotLog>();
        (log.pid, log.nonce, log.next_id)
    };
    let path = format!("screenshot-{pid}-{nonce}-{id_for_name}.png");
    let entity = world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(std::path::PathBuf::from(path.clone())))
        .observe(on_captured)
        .id();
    let id = world
        .resource_mut::<ScreenshotLog>()
        .request(&path, entity);
    info!("[spectate] screenshot #{id} requested -> {path} (entity {entity})");
    Ok(json!({ "status": "requested", "id": id }))
}

/// 轮询：`{"id":n}` → 状态应答（P1 整改语义）。
/// - 事件未到 → `{status:"pending"}`；
/// - 事件已到 → 现场核验本次输出文件（存在 + PNG 魔数 \x89PNG\r\n\x1a\n +
///   非零字节 + mtime ≥ 受理时刻）：通过 → 置 captured 并应答
///   `{status:"captured", path, bytes}`；未过 → **保持 pending** 并附 `detail`
///   （0.19.1 save_to_disk 写盘失败只留 error 日志——不误报 captured，也
///   不静默 pending）；
/// - 已 captured 条目每次应答前复验（既有 poll 校验保留），文件事后损坏 →
///   -32603 内部错误。
fn poll_screenshot(world: &mut World, id: u32) -> BrpResult {
    let (path, verified, event_arrived, requested_at) = {
        let log = world.resource::<ScreenshotLog>();
        let entry = log
            .entries
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| invalid_params(&format!("unknown screenshot id {id}")))?;
        (
            entry.path.clone(),
            entry.status == EntryStatus::Captured,
            entry.event_arrived,
            entry.requested_at,
        )
    };
    if verified {
        let bytes = verify_captured_file(&path, requested_at).map_err(|reason| {
            internal_error(format!(
                "screenshot #{id} captured but verification failed: {reason}"
            ))
        })?;
        return Ok(json!({ "status": "captured", "path": path, "bytes": bytes }));
    }
    if !event_arrived {
        return Ok(json!({ "status": "pending" }));
    }
    match verify_captured_file(&path, requested_at) {
        Ok(bytes) => {
            if let Some(entry) = world
                .resource_mut::<ScreenshotLog>()
                .entries
                .iter_mut()
                .find(|e| e.id == id)
            {
                entry.status = EntryStatus::Captured;
            }
            Ok(json!({ "status": "captured", "path": path, "bytes": bytes }))
        }
        Err(reason) => Ok(json!({
            "detail": format!(
                "screenshot #{id} capture event arrived but output file check failed: {reason}; not reporting captured (P1)"
            ),
            "status": "pending",
        })),
    }
}

/// 本次输出文件核验（P1 整改）：存在 + PNG 魔数 + 非零字节 + mtime ≥ 受理
/// 时刻。Ok(字节数) / Err(ASCII 原因，进轮询 `detail` 或 -32603 消息)。
/// mtime 界桩是前三者的必要补充：预置残留文件可满足前三者，唯时间戳能区分
/// 「本次产物」与「同路径残留」（冒烟段 5 物理注入实证）。
fn verify_captured_file(path: &str, requested_at: std::time::SystemTime) -> Result<u64, String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("file open failed: {e}"))?;
    let mut magic = [0u8; 8];
    std::io::Read::read_exact(&mut file, &mut magic)
        .map_err(|e| format!("file too small for PNG magic: {e}"))?;
    if magic != PNG_MAGIC {
        return Err(format!("PNG magic mismatch: {:02x?}", magic));
    }
    let meta = std::fs::metadata(path).map_err(|e| format!("metadata unreadable: {e}"))?;
    let bytes = meta.len();
    if bytes == 0 {
        return Err("file is zero bytes".to_string());
    }
    let modified = meta.modified().map_err(|e| format!("mtime unreadable: {e}"))?;
    if modified < requested_at {
        return Err(format!(
            "file mtime {modified:?} predates request acceptance {requested_at:?} (stale leftover suspected)"
        ));
    }
    Ok(bytes)
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

/// observer：捕获完成（`ScreenshotCaptured` 实体事件）→ 只记「事件已到」
/// （`event_arrived`）。与官方 `save_to_disk` 并挂同一实体（两 observer 相对
/// 次序无契约——工作流仓先例口径），且事件到达 ≠ 落盘成功（save_to_disk 写盘
/// 失败只记日志，screenshot.rs:145-147）——故本 observer **不置 captured**，
/// 文件核验与状态翻转归轮询应答（见 [`poll_screenshot`]，P1 整改）。
fn on_captured(event: On<ScreenshotCaptured>, mut log: ResMut<ScreenshotLog>) {
    let entity = event.entity;
    if let Some(entry) = log
        .entries
        .iter_mut()
        .rev()
        .find(|e| e.entity == entity && !e.event_arrived)
    {
        entry.event_arrived = true;
        info!("[spectate] screenshot #{} captured event arrived", entry.id);
    } else {
        warn!("[spectate] ScreenshotCaptured matched no pending log entry (entity {entity})");
    }
}
