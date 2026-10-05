//! 帧时间采集与落档（派工单 §4）。
//!
//! - Update 每帧读 `Res<Time>::delta()`（真实帧时长，无 Fixed timestep；0.19 无
//!   `delta_ns()`，以 `as_nanos()` 换算——见 api-notes.md）；
//! - warmup（默认 5 s，按累计 delta_ns 计）丢弃，跨线当帧计入 warmup；
//! - capture（默认 65 s）以「已记录帧 delta_ns 之和」计量，满即写
//!   `frames.csv`（首行 `idx,delta_ns`，逐帧原始数据）+ `meta.json`，随后发
//!   `AppExit::Success` 退出；
//! - 写档失败 / 环形缓冲溢出一律 panic（证据缺失与容量异常不可静默）。

use std::fs;
use std::io::Write;

use bevy_full as bevy;
// Resource derive 展开依赖 `bevy_ecs::` 别名（同 main.rs，理由见 api-notes.md）。
use bevy_full::ecs as bevy_ecs;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::ring::RingBuffer;
use crate::RunArgs;

/// 帧环形缓冲容量（派工单 §4）：65 s × ~3700 fps 以内均有余量；溢出 push panic。
pub const FRAME_RING_CAPACITY: usize = 240_000;

/// 采集状态机（Resource）。
#[derive(Resource)]
pub struct CaptureState {
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
    /// capture 帧数据（含完成当帧）。
    frames: RingBuffer<u64>,
    /// 是否已完成（防重复写档/重复发退出）。
    finished: bool,
}

impl CaptureState {
    pub fn new(args: &RunArgs) -> Self {
        Self {
            warmup_ns: args.warmup_sec.saturating_mul(1_000_000_000),
            capture_ns: args.capture_sec.saturating_mul(1_000_000_000),
            warmup_elapsed_ns: 0,
            capture_elapsed_ns: 0,
            warmup_done: false,
            frames: RingBuffer::with_capacity(FRAME_RING_CAPACITY),
            finished: false,
        }
    }

    /// 处理一帧 delta（ns）。返回 true 表示 capture 窗口已满。
    pub fn on_frame(&mut self, delta_ns: u64) -> bool {
        if !self.warmup_done {
            self.warmup_elapsed_ns = self.warmup_elapsed_ns.saturating_add(delta_ns);
            if self.warmup_elapsed_ns >= self.warmup_ns {
                self.warmup_done = true; // 跨线当帧计入 warmup，丢弃
            }
            return false;
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

/// Update：采集帧时间；窗口满 → 写档 + 请求退出。
pub fn capture_system(
    time: Res<Time>,
    args: Res<RunArgs>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<CaptureState>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.finished {
        return;
    }
    let delta_ns = u64::try_from(time.delta().as_nanos()).unwrap_or(u64::MAX);
    if state.on_frame(delta_ns) {
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
        let dir = args.out.join(format!("t{}", args.units));
        fs::create_dir_all(&dir).expect("create output dir failed");
        write_frames_csv(&dir.join("frames.csv"), &state.frames).expect("write frames.csv failed");
        write_meta_json(
            &dir.join("meta.json"),
            &args,
            state.frames.len(),
            &actual_res,
        )
        .expect("write meta.json failed");
        println!(
            "render-spike: capture complete units={} frames={} dir={}",
            args.units,
            state.frames.len(),
            dir.display()
        );
        exit.write(AppExit::Success);
    }
}

/// 写逐帧 CSV：首行 `idx,delta_ns`，其后逐帧原始数据。
fn write_frames_csv(path: &std::path::Path, frames: &RingBuffer<u64>) -> std::io::Result<()> {
    let mut f = std::io::BufWriter::new(fs::File::create(path)?);
    writeln!(f, "idx,delta_ns")?;
    let (a, b) = frames.as_slices();
    let mut idx = 0usize;
    for d in a.iter().chain(b.iter()) {
        writeln!(f, "{idx},{d}")?;
        idx += 1;
    }
    f.flush()
}

/// 写档元信息（JSON 手写；字段固定，零第三方依赖）。
fn write_meta_json(
    path: &std::path::Path,
    args: &RunArgs,
    captured_frames: usize,
    actual_res: &str,
) -> std::io::Result<()> {
    let json = format!(
        "{{\n  \"units\": {},\n  \"seed\": {},\n  \"resolution\": \"{}x{}\",\n  \
         \"window_resolution_actual\": \"{}\",\n  \"warmup_s\": {},\n  \"capture_s\": {},\n  \
         \"present_mode\": \"AutoNoVsync\",\n  \"captured_frames\": {}\n}}\n",
        args.units,
        args.seed,
        args.res_w,
        args.res_h,
        actual_res,
        args.warmup_sec,
        args.capture_sec,
        captured_frames
    );
    let mut f = fs::File::create(path)?;
    f.write_all(json.as_bytes())
}
