//! 薄 CLI 壳：参数解析、布阵/运行、打印、计时。
//!
//! stdout 只输出确定性内容（供跨进程逐字节 diff）；
//! 壁钟计时（Instant）只存在于本外壳，不进模拟态。

use std::env;
use std::process::ExitCode;
use std::time::Instant;

use sim::units::{kind_from_id, UnitKind};
use sim::world::{World, DEFAULT_COMPOSITION};

const DEFAULT_SEED: u64 = 42;
const DEFAULT_TICKS: u64 = 1800;

struct Args {
    seed: u64,
    ticks: u64,
    /// `--units` 显式给出时的值（裸单位路径，保留 T002 语义）。
    units: Option<usize>,
    /// `--comp` 显式给出时的值（布阵路径）。
    comp: Option<Vec<(UnitKind, usize)>>,
    dump_formation: bool,
    /// `--dump-final-units`（T004/D9）：run 结束后在五行摘要之后逐单位打印。
    dump_final_units: bool,
}

fn parse_num<T: std::str::FromStr>(raw: &str, name: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    raw.parse::<T>()
        .map_err(|e| format!("invalid value for {name}: '{raw}' ({e})"))
}

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

/// 解析 `--comp` 构成清单：`kind:count` 逗号分隔（如 `shieldman:5,pikeman:10`）。
/// kind 用英文小写 id；未知 id / count=0 / 格式错 / 空清单 → Err（stderr 报错 exit 2）。
fn parse_comp(raw: &str) -> Result<Vec<(UnitKind, usize)>, String> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let Some((kind_raw, count_raw)) = part.split_once(':') else {
            return Err(format!("invalid --comp segment '{part}' (expected kind:count)"));
        };
        let Some(kind) = kind_from_id(kind_raw) else {
            return Err(format!("unknown unit kind '{kind_raw}' in --comp"));
        };
        let count: usize = count_raw
            .parse()
            .map_err(|_| format!("invalid count for kind '{kind_raw}': '{count_raw}'"))?;
        if count == 0 {
            return Err(format!("zero count for kind '{kind_raw}' in --comp"));
        }
        out.push((kind, count));
    }
    if out.is_empty() {
        return Err("empty --comp".to_string());
    }
    Ok(out)
}

/// 手写解析（零依赖）：未知参数 / 缺值 / 解析失败 → Err（stderr 报错，exit code 2）。
fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut seed = DEFAULT_SEED;
    let mut ticks = DEFAULT_TICKS;
    let mut units: Option<usize> = None;
    let mut comp: Option<Vec<(UnitKind, usize)>> = None;
    let mut dump_formation = false;
    let mut dump_final_units = false;
    let mut i = 0;
    while i < argv.len() {
        let raw = argv[i].as_str();
        let (name, inline) = match raw.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (raw, None),
        };
        match name {
            "--seed" => seed = parse_num(take_value(argv, &mut i, inline, "--seed")?, "--seed")?,
            "--ticks" => ticks = parse_num(take_value(argv, &mut i, inline, "--ticks")?, "--ticks")?,
            "--units" => {
                units = Some(parse_num(take_value(argv, &mut i, inline, "--units")?, "--units")?)
            }
            "--comp" => comp = Some(parse_comp(take_value(argv, &mut i, inline, "--comp")?)?),
            "--dump-formation" => dump_formation = true,
            "--dump-final-units" => dump_final_units = true,
            _ => return Err(format!("unknown argument: '{raw}'")),
        }
        i += 1;
    }
    if units.is_some() && comp.is_some() {
        return Err("--units and --comp are mutually exclusive".to_string());
    }
    Ok(Args {
        seed,
        ticks,
        units,
        comp,
        dump_formation,
        dump_final_units,
    })
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("sim: {msg}");
            eprintln!(
                "usage: sim.exe [--seed <u64>] [--ticks <u64>] ([--comp <kind:count,...>] | \
                 [--units <usize>]) [--dump-formation] [--dump-final-units] \
                 (--flag value or --flag=value; defaults: seed=42 ticks=1800 \
                 comp=shieldman:5,heavyknight:5,pikeman:5,swordsman:5,archer:5,militia:5; \
                 values are decimal)"
            );
            return ExitCode::from(2);
        }
    };

    let start = Instant::now();
    // 构战路径：--comp 布阵 / 缺省默认构成布阵（D7）；--units 保留 T002 裸单位语义。
    let mut world = match (&args.comp, args.units) {
        (Some(comp), None) => World::deploy(args.seed, comp),
        (None, Some(u)) => World::new(args.seed, u),
        (None, None) => World::deploy(args.seed, &DEFAULT_COMPOSITION),
        (Some(_), Some(_)) => unreachable!("parse_args rejects --comp + --units"),
    };

    if args.dump_formation {
        // 布阵快照：tick 0、不跑 tick。五行摘要（hash 为布阵快照哈希）+ 每单位一行，全走 stdout。
        println!("seed={}", args.seed);
        println!("ticks={}", args.ticks);
        println!("units={}", world.unit_count());
        println!("final_tick={}", world.tick);
        println!("hash=0x{:016x}", world.last_hash);
        for (idx, unit) in world.units().iter().enumerate() {
            println!(
                "u{} {} {} x={}",
                idx,
                unit.side.label(),
                unit.kind.id(),
                unit.x
            );
        }
        let elapsed = start.elapsed();
        eprintln!("elapsed_ms={}", elapsed.as_millis());
        return ExitCode::SUCCESS;
    }

    world.run(args.ticks);
    let elapsed = start.elapsed();

    // stdout：仅确定性五行（跨进程逐字节 diff 用）。
    println!("seed={}", args.seed);
    println!("ticks={}", args.ticks);
    println!("units={}", world.unit_count());
    println!("final_tick={}", world.tick);
    println!("hash=0x{:016x}", world.last_hash);

    // --dump-final-units（T004/D9）：逐单位终态（stdout 确定性内容，格式固定）。
    if args.dump_final_units {
        for (idx, unit) in world.units().iter().enumerate() {
            println!(
                "u{} {} {} hp={} cd={} x={}",
                idx,
                unit.side.label(),
                unit.kind.id(),
                unit.hp,
                unit.cd,
                unit.x
            );
        }
    }

    // stderr：壁钟计时（仅外壳，不进模拟态）。
    eprintln!("elapsed_ms={}", elapsed.as_millis());

    ExitCode::SUCCESS
}
