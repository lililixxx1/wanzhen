//! T024 观战帧采集插桩（M5-07 席位 9）：`--frame-capture <dir>` 下的帧时间落档。
//!
//! 口径**逐字移植** render-spike T010 `capture_system` 体例（派工单 D1；
//! `render-spike/src/capture.rs:59-119` + `render-spike/src/ring.rs`）：
//! - Update 每帧读 `Res<Time>::delta()`（真实帧时长；0.19 无 `delta_ns()`，以
//!   `as_nanos()` 换算——T010 先例）入容量缓冲（append-only，满容 panic 拒绝
//!   覆盖——采集完整性纪律，`ring.rs:26-33` 同语义）；
//! - warmup（`WARMUP_SEC` = 5 s，按累计 delta_ns 计）丢弃，跨线当帧计入 warmup
//!   （`capture.rs:59-66` 逐字）；
//! - capture（`CAPTURE_SEC` = 65 s，以「已记录帧 delta_ns 之和」计量）满 →
//!   写 `frames.csv`（首行 `idx,delta_ns`，逐帧原始数据）+ `meta.json` →
//!   发 `AppExit::Success` 自然退出（`capture.rs:78-119`）；
//! - 写档失败 / 缓冲溢出一律 panic（证据缺失与容量异常不可静默——T010 注）。
//!
//! 挂载红线（D1）：本系统与资源**仅**由 [`crate::spectate::SpectatePlugin`]
//! 注册（spectate 形态插件集）——headless 形态零行为变化；`--frame-capture`
//! 未给时资源为缺省态（`out == None`），系统立即返回零开销。sim/ 零改动。
//!
//! meta.json 字段适配说明（D1 语义不变、字段面按宿主语境）：T010 的
//! `units/seed` 替换为布阵时点读数 `units_total/tick_at_write/seed/max_ticks`
//! （取自 [`crate::rpc::HostedGame`]；未布阵 → null）；`resolution` 为固定
//! 1920×1080（D3 范围预裁剪「多分辨率不做」），`window_resolution_actual`
//! 为实际物理窗口查询值（125% DPI 缩放下与逻辑尺寸差异——T010 披露先例）。

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use bevy_full as bevy;
// derive 宏（下方 Resource）展开为 `bevy_ecs::` 绝对路径；依赖名为别名
// bevy_full 时宏无法经清单解析命中，按 bevy_macro_utils 0.19.1 内置说明补
// 别名（render-spike/src/main.rs:23-26 + T010 api-notes 先例）。
use bevy_full::ecs as bevy_ecs;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::rpc::HostedGame;

/// warmup 窗口（秒）：沿 T010 缺省（`render-spike/src/main.rs:41`
/// `DEFAULT_WARMUP_SEC = 5`）。
pub const WARMUP_SEC: u64 = 5;
/// capture 窗口（秒）：沿 T010 缺省（`render-spike/src/main.rs:42`
/// `DEFAULT_CAPTURE_SEC = 65`）。
pub const CAPTURE_SEC: u64 = 65;
/// 帧缓冲容量：沿 T010 `render-spike/src/capture.rs:24` `FRAME_RING_CAPACITY`
/// = 240_000（65 s × ~3700 fps 以内余量；溢出 panic）。
pub const FRAME_RING_CAPACITY: usize = 240_000;

/// 采集状态机（Resource）。`out == None` = 未启用（缺省态，系统零开销返回）。
#[derive(Resource, Default)]
pub struct FrameCaptureState {
    /// 落档目录（`--frame-capture <dir>`；None = 未启用）。
    out: Option<PathBuf>,
    /// warmup 窗口（ns）。
    warmup_ns: u64,
    /// capture 窗口（ns）。
    capture_ns: u64,
    /// warmup 已累计 delta（ns）。
    warmup_elapsed_ns: u64,
    /// capture 已累计 delta（ns）。
    capture_elapsed_ns: u64,
    /// warmup 是否结束（跨线帧丢弃）。
    warmup_done: bool,
    /// capture 帧数据（含完成当帧）。预分配满容量——采集窗口内零 realloc
    /// （分配尖峰不进帧时间样本）。
    frames: Vec<u64>,
    /// 是否已完成（防重复写档/重复发退出）。
    finished: bool,
}

impl FrameCaptureState {
    /// 启用态构造（`--frame-capture <dir>` 时由 main 经 `insert_resource` 覆盖
    /// 缺省态；`SpectatePlugin` 侧 `init_resource` 先建缺省态——T019 autorun
    /// 先例同款「后插覆盖」语义）。
    pub fn new(out: PathBuf) -> Self {
        Self {
            out: Some(out),
            warmup_ns: WARMUP_SEC.saturating_mul(1_000_000_000),
            capture_ns: CAPTURE_SEC.saturating_mul(1_000_000_000),
            frames: Vec::with_capacity(FRAME_RING_CAPACITY),
            ..Default::default()
        }
    }

    /// 处理一帧 delta（ns）。返回 true 表示 capture 窗口已满。
    /// （`render-spike/src/capture.rs:59-74` 逐字移植；溢出检查替代 RingBuffer
    /// 的 push-panic——语义相同：append-only、拒绝覆盖。）
    fn on_frame(&mut self, delta_ns: u64) -> bool {
        if !self.warmup_done {
            self.warmup_elapsed_ns = self.warmup_elapsed_ns.saturating_add(delta_ns);
            if self.warmup_elapsed_ns >= self.warmup_ns {
                self.warmup_done = true; // 跨线当帧计入 warmup，丢弃
            }
            return false;
        }
        if self.frames.len() >= FRAME_RING_CAPACITY {
            panic!(
                "frame capture buffer overflow: capacity {} reached, refusing to overwrite",
                FRAME_RING_CAPACITY
            );
        }
        self.frames.push(delta_ns);
        self.capture_elapsed_ns = self.capture_elapsed_ns.saturating_add(delta_ns);
        if self.capture_elapsed_ns >= self.capture_ns {
            self.finished = true;
            return true;
        }
        false
    }
}

/// Update：采集帧时间；窗口满 → 写档 + 请求退出（`render-spike/src/capture.rs:
/// 78-119` 体例）。未启用（`out == None`）或已完成 → 立即返回。
pub(crate) fn frame_capture_system(
    time: Res<Time>,
    hosted: Res<HostedGame>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<FrameCaptureState>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.finished || state.out.is_none() {
        return;
    }
    let delta_ns = u64::try_from(time.delta().as_nanos()).unwrap_or(u64::MAX);
    if state.on_frame(delta_ns) {
        let dir = state
            .out
            .clone()
            .expect("frame capture enabled checked above");
        let actual_res = windows
            .iter()
            .next()
            .map(|w| {
                format!(
                    "{}x{}",
                    w.resolution.physical_width(),
                    w.resolution.physical_height()
                )
            })
            .unwrap_or_else(|| "unknown".to_string());
        fs::create_dir_all(&dir).expect("create frame-capture output dir failed");
        write_frames_csv(&dir.join("frames.csv"), &state.frames)
            .expect("write frames.csv failed");
        write_meta_json(&dir.join("meta.json"), &state, &hosted, &actual_res)
            .expect("write meta.json failed");
        println!(
            "host: frame-capture complete frames={} dir={}",
            state.frames.len(),
            dir.display()
        );
        exit.write(AppExit::Success);
    }
}

/// 写逐帧 CSV：首行 `idx,delta_ns`，其后逐帧原始数据（T010 同格式——判定
/// 脚本 `summarize_t024.py` 表头逐字校验）。
fn write_frames_csv(path: &std::path::Path, frames: &[u64]) -> std::io::Result<()> {
    let mut f = std::io::BufWriter::new(fs::File::create(path)?);
    writeln!(f, "idx,delta_ns")?;
    for (idx, d) in frames.iter().enumerate() {
        writeln!(f, "{idx},{d}")?;
    }
    f.flush()
}

/// 写档元信息（JSON；serde_json 手构——字段面见模块注释「meta.json 字段适配
/// 说明」；未布阵字段为 null）。
fn write_meta_json(
    path: &std::path::Path,
    state: &FrameCaptureState,
    hosted: &HostedGame,
    actual_res: &str,
) -> std::io::Result<()> {
    let game = hosted.world.as_ref();
    let config = hosted.config.as_ref();
    let json = serde_json::json!({
        "mode": "spectate",
        "resolution": "1920x1080",
        "window_resolution_actual": actual_res,
        "present_mode": "AutoNoVsync",
        "warmup_s": WARMUP_SEC,
        "capture_s": CAPTURE_SEC,
        "captured_frames": state.frames.len(),
        "units_total": game.map(|g| g.units().len()),
        "tick_at_write": game.map(|g| g.tick),
        "seed": config.map(|c| c.seed),
        "max_ticks": config.map(|c| c.max_ticks),
    });
    let text = serde_json::to_string_pretty(&json)
        .expect("meta.json serialization cannot fail for this value");
    let mut f = fs::File::create(path)?;
    f.write_all(text.as_bytes())?;
    f.write_all(b"\n")
}
