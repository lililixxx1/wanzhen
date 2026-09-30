//! 薄 CLI 壳：参数解析、运行、打印、计时。
//!
//! stdout 只输出确定性内容（供跨进程逐字节 diff）；
//! 壁钟计时（Instant）只存在于本外壳，不进模拟态。

use std::env;
use std::process::ExitCode;
use std::time::Instant;

use sim::world::World;

const DEFAULT_SEED: u64 = 42;
const DEFAULT_TICKS: u64 = 1800;
const DEFAULT_UNITS: usize = 0;

struct Args {
    seed: u64,
    ticks: u64,
    units: usize,
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

/// 手写解析（零依赖）：未知参数 / 缺值 / 解析失败 → Err（stderr 报错，exit code 2）。
fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut seed = DEFAULT_SEED;
    let mut ticks = DEFAULT_TICKS;
    let mut units = DEFAULT_UNITS;
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
            "--units" => units = parse_num(take_value(argv, &mut i, inline, "--units")?, "--units")?,
            _ => return Err(format!("unknown argument: '{raw}'")),
        }
        i += 1;
    }
    Ok(Args { seed, ticks, units })
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("sim: {msg}");
            eprintln!(
                "usage: sim.exe [--seed <u64>] [--ticks <u64>] [--units <usize>] \
                 (--flag value or --flag=value; defaults: seed=42 ticks=1800 units=0; values are decimal)"
            );
            return ExitCode::from(2);
        }
    };

    let start = Instant::now();
    let mut world = World::new(args.seed, args.units);
    world.run(args.ticks);
    let elapsed = start.elapsed();

    // stdout：仅确定性五行（跨进程逐字节 diff 用）。
    println!("seed={}", args.seed);
    println!("ticks={}", args.ticks);
    println!("units={}", args.units);
    println!("final_tick={}", world.tick);
    println!("hash=0x{:016x}", world.last_hash);

    // stderr：壁钟计时（仅外壳，不进模拟态）。
    eprintln!("elapsed_ms={}", elapsed.as_millis());

    ExitCode::SUCCESS
}
