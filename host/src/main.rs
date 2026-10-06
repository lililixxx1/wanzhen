//! M5-01（T018）BRP/观战宿主入口——headless 形态（席位 1）+ game.* 初始方法面
//! （席位 2，实装见 [`rpc`]）。观战模式（窗口/表现层/HUD/`game.screenshot`
//! 实装）随 T019，本卡为 headless 桩。
//!
//! 组装（D3/D11）：`MinimalPlugins` + `ScheduleRunnerPlugin::run_loop`(60Hz)
//! + [`rpc::HostRpcPlugin`]（内含 `RemotePlugin` 6 方法注册 + `RemoteHttpPlugin`
//! 显式回环绑定——BRP 无鉴权，禁止绑定非回环地址；显式绑定防上游默认值漂移；
//! **地址恒 127.0.0.1、端口缺省 15702，`--port` 可选覆盖（T021/D5）**）。
//!
//! T021/D4 auto-deploy：任一配置参数（`--seed/--comp/--threads/--max-ticks/
//! --lane-len-m/--preset`）出现 → 启动即布阵（解析 → 覆盖 preset 底座 →
//! [`rpc::apply_deploy`] → 构造好的 `HostedGame` 在 `add_plugins(HostRpcPlugin)`
//! **之前** `insert_resource`——`App::init_resource` 已存在不覆盖，先插者保留；
//! 依据已核验 bevy_ecs-0.19.1/src/world/mod.rs:1984（init_resource →
//! insert_resource_if_not_exists_with_caller:1947，存在即跳过）及其单测
//! `init_resource_does_not_overwrite`（同文件 :4323））；`--port` 单独出现
//! **不**触发 deploy；无任何参数 = T018 纯服务形态（行为不变）。
//!
//! CLI 解析风格对齐 `sim/src/main.rs`（std::env::args 手写、`--key value` 与
//! `--key=value` 两形态、非法值 stderr + exit 2）；`--comp` 与 sim 同语法
//! （`kind:count` 逗号分隔，双方对称构成）。CLI 值域与 BRP deploy 校验段同域
//! （threads 1..=1024 / max_ticks 1..=14400 / lane ≥ 1 且 Q32.32 不溢出 /
//! 每方 ≤ 100_000——常量直接引 [`rpc`]，防两入口漂移）。
//!
//! banner 走 `eprintln!`（stderr 元信息，零 bevy_log feature 依赖——D3），
//! 运行时留痕与 [`rpc`] 注册的方法面/绑定地址一一对应。

mod challenges;
mod presets;
mod rpc;
mod suite;

use std::env;
use std::process::ExitCode;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use sim::units::{kind_from_id, UnitKind, ONE_Q32_32};
use sim::world::TICK_CAP_FULL;

use crate::rpc::{
    apply_deploy, HostedGame, ResolvedDeploy, DEFAULT_BRP_PORT, KIND_IDS, MAX_THREADS,
    MAX_UNITS_PER_SIDE,
};

/// auto-deploy 缺省 seed（= sim CLI DEFAULT_SEED 同值 42；--seed 显式时覆盖）。
const DEFAULT_SEED: u64 = 42;

/// CLI 参数集（T021/D4）：全 Option——None = 未给出（用 preset 底座/缺省值）。
struct Args {
    seed: Option<u64>,
    /// `--comp <kind:count,...>`：双方对称构成（sim CLI 同语法）。
    comp: Option<Vec<(UnitKind, usize)>>,
    /// `--threads <N>`：线程档位（1..=1024）。
    threads: Option<u64>,
    /// `--max-ticks <N>`：tick 上限（1..=14400）。
    max_ticks: Option<u64>,
    /// `--lane-len-m <m>`：lane 全长（米，≥ 1）。
    lane_len_m: Option<i64>,
    /// `--preset <name>`：预设底座名（注册表见 [`presets`]）。
    preset: Option<String>,
    /// `--port <u16>`：BRP 监听端口（缺省 15702；单独出现不触发 auto-deploy）。
    port: Option<u16>,
}

fn parse_num<T: std::str::FromStr>(raw: &str, name: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    raw.parse::<T>()
        .map_err(|e| format!("invalid value for {name}: '{raw}' ({e})"))
}

/// 取参数值：支持 `--flag value` 与 `--flag=value` 两种写法（sim CLI 同款）。
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

/// 解析 `--comp` 构成清单：`kind:count` 逗号分隔（如 `swordsman:15,militia:15`）。
/// 未知 id / count=0 / 格式错 / 空清单 → Err（stderr + exit 2）。总量（= 单方
/// 数量，双方对称）超过 [`MAX_UNITS_PER_SIDE`] 拒绝——与 BRP `parse_composition`
/// 每方上限同域（防误配 OOM）。
fn parse_comp(raw: &str) -> Result<Vec<(UnitKind, usize)>, String> {
    let mut out = Vec::new();
    let mut total: usize = 0;
    for part in raw.split(',') {
        let Some((kind_raw, count_raw)) = part.split_once(':') else {
            return Err(format!("invalid --comp segment '{part}' (expected kind:count)"));
        };
        let Some(kind) = kind_from_id(kind_raw) else {
            return Err(format!(
                "unknown unit kind '{kind_raw}' in --comp (valid: {KIND_IDS})"
            ));
        };
        let count: usize = count_raw
            .parse()
            .map_err(|_| format!("invalid count for kind '{kind_raw}': '{count_raw}'"))?;
        if count == 0 {
            return Err(format!("zero count for kind '{kind_raw}' in --comp"));
        }
        total = total.saturating_add(count);
        out.push((kind, count));
    }
    if out.is_empty() {
        return Err("empty --comp".to_string());
    }
    if total > MAX_UNITS_PER_SIDE {
        return Err(format!(
            "--comp totals {total} units per side, exceeding per-side cap {MAX_UNITS_PER_SIDE}"
        ));
    }
    Ok(out)
}

/// 手写解析（零依赖）：未知参数 / 缺值 / 解析失败 / 值出域 → Err（stderr 报错，
/// exit code 2）。值域校验与 BRP [`rpc::deploy_handler`] 校验段同域（D4）。
fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut args = Args {
        seed: None,
        comp: None,
        threads: None,
        max_ticks: None,
        lane_len_m: None,
        preset: None,
        port: None,
    };
    let mut i = 0;
    while i < argv.len() {
        let raw = argv[i].as_str();
        let (name, inline) = match raw.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (raw, None),
        };
        match name {
            "--seed" => {
                args.seed =
                    Some(parse_num(take_value(argv, &mut i, inline, "--seed")?, "--seed")?)
            }
            "--comp" => args.comp = Some(parse_comp(take_value(argv, &mut i, inline, "--comp")?)?),
            "--threads" => {
                args.threads =
                    Some(parse_num(take_value(argv, &mut i, inline, "--threads")?, "--threads")?)
            }
            "--max-ticks" => {
                args.max_ticks = Some(parse_num(
                    take_value(argv, &mut i, inline, "--max-ticks")?,
                    "--max-ticks",
                )?)
            }
            "--lane-len-m" => {
                args.lane_len_m = Some(parse_num(
                    take_value(argv, &mut i, inline, "--lane-len-m")?,
                    "--lane-len-m",
                )?)
            }
            "--preset" => {
                args.preset =
                    Some(take_value(argv, &mut i, inline, "--preset")?.to_string())
            }
            "--port" => args.port = Some(parse_num(take_value(argv, &mut i, inline, "--port")?, "--port")?),
            _ => return Err(format!("unknown argument: '{raw}'")),
        }
        i += 1;
    }
    // 值域校验（与 BRP deploy 校验段同域；D4：非法值 stderr + exit 2）。
    if let Some(t) = args.threads {
        if t == 0 || t > MAX_THREADS {
            return Err(format!("--threads must be in 1..={MAX_THREADS}, got {t}"));
        }
    }
    if let Some(m) = args.max_ticks {
        if m == 0 || m > TICK_CAP_FULL {
            return Err(format!("--max-ticks must be in 1..={TICK_CAP_FULL}, got {m}"));
        }
    }
    if let Some(l) = args.lane_len_m {
        if l < 1 {
            return Err(format!("--lane-len-m must be >= 1, got {l}"));
        }
        // Q32.32 溢出域与 BRP 校验段同口径（release 回绕不可接受，拒绝优于回绕）。
        if l.checked_mul(ONE_Q32_32).is_none() {
            return Err(format!("--lane-len-m too large (Q32.32 overflow): {l}"));
        }
    }
    if let Some(p) = &args.preset {
        if presets::get(p).is_none() {
            return Err(format!(
                "unknown preset '{p}' (available: {})",
                presets::names().join(", ")
            ));
        }
    }
    Ok(args)
}

/// 由 CLI 参数组装 [`ResolvedDeploy`]（T021/D4；值域校验已在 [`parse_args`]
/// 完成——本函数不可失败）。preset 底座：`--preset` 显式给值；未给时以
/// 「default」预设为底座（其构成/lane/max_ticks = M0 默认构成 + 默认 lane +
/// 降规模 tick 上限，与 sim CLI 缺省布阵同构）；`--comp` 双方对称覆盖 red/blue；
/// seed 缺省 [`DEFAULT_SEED`]、threads 缺省 1（= BRP deploy 缺省）。
fn build_resolved(args: &Args) -> ResolvedDeploy {
    let base = presets::get(args.preset.as_deref().unwrap_or("default"))
        .expect("preset name validated in parse_args");
    ResolvedDeploy {
        seed: args.seed.unwrap_or(DEFAULT_SEED),
        red: match &args.comp {
            Some(c) => c.clone(),
            None => base.red,
        },
        blue: match &args.comp {
            Some(c) => c.clone(),
            None => base.blue,
        },
        lane_len_m: args.lane_len_m.unwrap_or(base.lane_len_m),
        max_ticks: args.max_ticks.unwrap_or(base.max_ticks),
        threads: args.threads.unwrap_or(1) as usize,
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("host: {msg}");
            eprintln!(
                "usage: host.exe [--seed <u64>] [--comp <kind:count,...>] \
                 [--threads <1..=1024>] [--max-ticks <1..=14400>] [--lane-len-m <m>] \
                 [--preset <{}>] [--port <u16>] \
                 (--flag value or --flag=value; any config flag other than --port \
                 triggers auto-deploy at startup, overriding the preset base; --comp \
                 deploys symmetrically to both sides; --port alone only changes the BRP \
                 listen port; no flags = pure BRP service, deploy via game.deploy; \
                 values are decimal)",
                presets::names().join("|")
            );
            return ExitCode::from(2);
        }
    };

    // 三行运行时留痕（stderr）：headless 形态 / 回环绑定（实际端口，D5）/
    // 方法面计数（6/6）。
    let port = args.port.unwrap_or(DEFAULT_BRP_PORT);
    eprintln!("[host] mode=headless (spectate arrives with T019)");
    eprintln!("[host] BRP listening on 127.0.0.1:{port} (explicit loopback bind; non-loopback forbidden)");
    eprintln!("[host] methods: game.deploy, game.run_to_tick, game.state_hash, game.outcome, game.run_tests, game.screenshot(stub->T019)");

    // T021/D4：任一配置参数（除 --port）出现 → 启动即 auto-deploy。
    let auto = args.seed.is_some()
        || args.comp.is_some()
        || args.threads.is_some()
        || args.max_ticks.is_some()
        || args.lane_len_m.is_some()
        || args.preset.is_some();

    let mut app = App::new();
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
        // headless 常驻循环（官方文档例原样）：BRP 请求经调度排空。
        Duration::from_secs_f64(1.0 / 60.0),
    )));

    if auto {
        let req = build_resolved(&args);
        let seed = req.seed;
        // banner 预设名（D4：preset 名或 "custom"）：--comp 出现 = 构成用户自定义
        // → "custom"；否则 --preset 显式给名 → 该名；否则（隐式 default 底座）
        // → "default"。
        let deploy_name = if args.comp.is_some() {
            "custom".to_string()
        } else {
            args.preset.clone().unwrap_or_else(|| "default".to_string())
        };
        let mut hosted = HostedGame::default();
        let resp = apply_deploy(&mut hosted, req);
        let units = resp["units"].as_u64().unwrap_or(0);
        // 先 insert_resource，HostRpcPlugin 的 init_resource 已存在不覆盖
        // （bevy_ecs-0.19.1 语义，见模块注释）——auto-deploy 生效的关键顺序。
        app.insert_resource(hosted);
        eprintln!("[host] auto-deployed {deploy_name} seed={seed} units={units} tick=0");
    }

    app.add_plugins(rpc::HostRpcPlugin { port });
    app.run();
    ExitCode::SUCCESS
}
