//! T010 渲染 spike 入口（M0-09，验收⑤）：胶囊体灰盒万单位同屏帧率验证。
//!
//! 口径（任务卡 D2~D4 / 派工单 §3~§4）：
//! - 确定性布阵：crate 本地 SplitMix64（不依赖 sim），单位 i 位置只依赖 i 与 seed
//!   ⇒ 四档同 seed 为前缀布局；
//! - 窗口化 1920×1080（可 `--res` 覆写）、PresentMode::AutoNoVsync（vsync 关）、
//!   固定相机 (0,180,180) looking_at 原点；
//! - Update 每帧取 `Res<Time>::delta()` 入帧环形缓冲：warmup（默认 5 s）丢弃 +
//!   capture（默认 65 s）满 → 写 frames.csv/meta.json 后发 AppExit::Success 退出。
//!
//! 0.19 版本差异（api-notes.md）：无 `delta_ns()`（用 `delta().as_nanos()`）；
//! AppExit 是 Message（`MessageWriter<AppExit>::write`）；全局环境光为
//! `GlobalAmbientLight`（`AmbientLight` 已是相机组件）。
//!
//! 用法：
//!   render-spike --units <N> [--seed <u64>] [--warmup-sec <u64>] [--capture-sec <u64>]
//!                [--res <WxH>] --out <dir>
//! 解析错误 → stderr + exit 2（`--out` 必选）。

mod capture;
mod grid;
mod ring;
mod rng;
mod scene;
#[cfg(test)]
mod stats;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use bevy_full as bevy;
// derive 宏（Resource 等）展开为 `bevy_ecs::...` 绝对路径；依赖名为别名 bevy_full 时
// 宏无法经清单解析命中，按 bevy_macro_utils 0.19.1 内置说明补别名（见 api-notes.md）。
use bevy_full::ecs as bevy_ecs;
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

pub const DEFAULT_UNITS: usize = 10_000;
pub const DEFAULT_SEED: u64 = 42;
pub const DEFAULT_WARMUP_SEC: u64 = 5;
pub const DEFAULT_CAPTURE_SEC: u64 = 65;
pub const DEFAULT_RES: (u32, u32) = (1920, 1080);

/// 运行参数（CLI 解析结果；Resource 供场景/采集系统读取）。
#[derive(Resource, Clone, Debug)]
pub struct RunArgs {
    pub units: usize,
    pub seed: u64,
    pub warmup_sec: u64,
    pub capture_sec: u64,
    pub res_w: u32,
    pub res_h: u32,
    pub out: PathBuf,
}

const USAGE: &str = "usage: render-spike.exe [--units <N>=10000] [--seed <u64>=42] \
[--warmup-sec <u64>=5] [--capture-sec <u64>=65] [--res <WxH>=1920x1080] --out <dir> \
(--flag value or --flag=value; parse errors exit 2)";

/// 取参数值：支持 `--flag value` 与 `--flag=value` 两种写法。
fn take_value<'a>(
    argv: &'a [String],
    i: &mut usize,
    inline: Option<&'a str>,
    name: &str,
) -> Result<&'a str, String> {
    if let Some(v) = inline {
        return Ok(v);
    }
    match argv.get(*i + 1) {
        Some(v) => {
            *i += 1;
            Ok(v)
        }
        None => Err(format!("missing value for {name}")),
    }
}

fn parse_num<T: std::str::FromStr>(raw: &str, name: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    raw.parse::<T>()
        .map_err(|e| format!("invalid value for {name}: '{raw}' ({e})"))
}

/// 解析 `--res WxH`（接受大写 X；两侧必须为正整数）。
fn parse_res(raw: &str) -> Result<(u32, u32), String> {
    let (w, h) = raw
        .split_once(['x', 'X'])
        .ok_or_else(|| format!("invalid --res '{raw}' (expected WxH)"))?;
    let w: u32 = parse_num(w, "--res width")?;
    let h: u32 = parse_num(h, "--res height")?;
    if w == 0 || h == 0 {
        return Err(format!("--res dimensions must be >= 1, got '{raw}'"));
    }
    Ok((w, h))
}

/// 手写解析（零依赖）；未知参数 / 缺值 / 解析失败 → Err（stderr 报错，exit 2）。
fn parse_args(argv: &[String]) -> Result<RunArgs, String> {
    let mut units = DEFAULT_UNITS;
    let mut seed = DEFAULT_SEED;
    let mut warmup_sec = DEFAULT_WARMUP_SEC;
    let mut capture_sec = DEFAULT_CAPTURE_SEC;
    let mut res = DEFAULT_RES;
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < argv.len() {
        let raw = argv[i].as_str();
        let (name, inline) = match raw.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (raw, None),
        };
        match name {
            "--units" => {
                units = parse_num(take_value(argv, &mut i, inline, "--units")?, "--units")?
            }
            "--seed" => seed = parse_num(take_value(argv, &mut i, inline, "--seed")?, "--seed")?,
            "--warmup-sec" => {
                warmup_sec = parse_num(take_value(argv, &mut i, inline, "--warmup-sec")?, "--warmup-sec")?
            }
            "--capture-sec" => {
                capture_sec =
                    parse_num(take_value(argv, &mut i, inline, "--capture-sec")?, "--capture-sec")?
            }
            "--res" => res = parse_res(take_value(argv, &mut i, inline, "--res")?)?,
            "--out" => out = Some(PathBuf::from(take_value(argv, &mut i, inline, "--out")?)),
            _ => return Err(format!("unknown argument: '{raw}'")),
        }
        i += 1;
    }
    if units == 0 {
        return Err("--units must be >= 1".to_string());
    }
    if capture_sec == 0 {
        return Err("--capture-sec must be >= 1".to_string());
    }
    let out = out.ok_or_else(|| "missing required --out <dir>".to_string())?;
    Ok(RunArgs {
        units,
        seed,
        warmup_sec,
        capture_sec,
        res_w: res.0,
        res_h: res.1,
        out,
    })
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("render-spike: {msg}");
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    println!(
        "render-spike: start units={} seed={} res={}x{} warmup_s={} capture_s={} out={}",
        args.units,
        args.seed,
        args.res_w,
        args.res_h,
        args.warmup_sec,
        args.capture_sec,
        args.out.display()
    );

    let capture_state = capture::CaptureState::new(&args);
    let window = Window {
        title: "render-spike (T010)".to_string(),
        resolution: WindowResolution::new(args.res_w, args.res_h),
        present_mode: PresentMode::AutoNoVsync,
        ..default()
    };

    let exit = App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .insert_resource(args)
        // 0.19：场景级环境光资源名为 GlobalAmbientLight（AmbientLight 已改为相机组件）。
        .insert_resource(GlobalAmbientLight::default())
        .insert_resource(capture_state)
        .add_systems(Startup, scene::setup_scene)
        .add_systems(Update, capture::capture_system)
        .run();

    match exit {
        AppExit::Success => ExitCode::SUCCESS,
        AppExit::Error(code) => ExitCode::from(code.get()),
    }
}
