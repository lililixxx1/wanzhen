//! 薄 CLI 壳：参数解析、布阵/运行、打印、计时。
//!
//! stdout 只输出确定性内容（供跨进程逐字节 diff）；
//! 壁钟计时（Instant）只存在于本外壳，不进模拟态。
//!
//! T005 扩展（D6）：`--battle` 对局路径——deploy 构战（--comp 或默认构成，
//! 双方镜像同清单）→ `run_battle(--ticks)` 收束 → stdout 打印 BattleLog 固定
//! 8 行；`--dump-final-units` 可组合。与 `--units` / `--dump-formation` 互斥
//! （stderr + usage + exit 2）。非 battle 路径输出逐字节不变（黄金锚证据链依赖）。
//!
//! T006 扩展（D6/D7）：
//! - `--threads <usize>`（默认 1）：线程档位；`0` 或 `>1024` → stderr + usage +
//!   exit 2。>1 时建 [`sim::pool::ThreadPool`] 传 `Some`，作用域包住整个模拟段
//!   后 drop。与 `--comp/--units/--battle/--dump-*` 均可组合（无互斥）。
//!   **stdout 五行摘要 / BattleLog 8 行输出格式零变化**（黄金锚 diff 链依赖）；
//!   `threads=N` 打 stderr（与 elapsed_ms 同侧）。
//! - `--hash-samples <t1,t2,...>`：逗号分隔 u64，**必须严格升序**（重复/非升序/
//!   解析失败 → exit 2）；含 0 合法（= 布阵快照哈希）。**与 `--battle` 互斥**
//!   （exit 2；BattleLog 已含终局 final_hash）。仅作用非 battle run 路径：分段
//!   推进（段边界 = 采样点，每段一次 run_with），每达点向 stdout 打一行
//!   `sample=<tick> hash=0x{:016x}`（打印在五行摘要之前，时序自然）。不用该
//!   flag 时输出零变化。采样点 > `--ticks` → exit 2（严格解析口径）。

use std::env;
use std::process::ExitCode;
use std::time::Instant;

use sim::pool::ThreadPool;
use sim::units::{kind_from_id, UnitKind};
use sim::world::{BattleLog, World, DEFAULT_COMPOSITION};

const DEFAULT_SEED: u64 = 42;
const DEFAULT_TICKS: u64 = 1800;
/// 线程档位合法域上限（D6：>1024 → exit 2）。
const MAX_THREADS: usize = 1024;

struct Args {
    seed: u64,
    ticks: u64,
    /// `--threads`（T006/D6）：线程档位，默认 1（= 串行 None 路径）。
    threads: usize,
    /// `--hash-samples`（T006/D7）：严格升序采样 tick 列表（None = 不采样）。
    hash_samples: Option<Vec<u64>>,
    /// `--units` 显式给出时的值（裸单位路径，保留 T002 语义）。
    units: Option<usize>,
    /// `--comp` 显式给出时的值（布阵路径）。
    comp: Option<Vec<(UnitKind, usize)>>,
    /// `--battle`（T005/D6）：对局路径（与 --units / --dump-formation 互斥）。
    battle: bool,
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

/// 解析 `--hash-samples`（D7）：逗号分隔 u64，**必须严格升序**（重复/非升序/
/// 解析失败 → Err，exit 2）；含 0 合法。
fn parse_hash_samples(raw: &str) -> Result<Vec<u64>, String> {
    let mut out: Vec<u64> = Vec::new();
    for part in raw.split(',') {
        let t: u64 = part
            .parse()
            .map_err(|_| format!("invalid --hash-samples tick: '{part}'"))?;
        if let Some(&last) = out.last() {
            if t <= last {
                return Err(format!(
                    "--hash-samples must be strictly ascending: {t} follows {last}"
                ));
            }
        }
        out.push(t);
    }
    Ok(out)
}

/// 手写解析（零依赖）：未知参数 / 缺值 / 解析失败 → Err（stderr 报错，exit code 2）。
fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut seed = DEFAULT_SEED;
    let mut ticks = DEFAULT_TICKS;
    let mut threads = 1usize;
    let mut hash_samples: Option<Vec<u64>> = None;
    let mut units: Option<usize> = None;
    let mut comp: Option<Vec<(UnitKind, usize)>> = None;
    let mut battle = false;
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
            "--threads" => {
                threads = parse_num(take_value(argv, &mut i, inline, "--threads")?, "--threads")?
            }
            "--hash-samples" => {
                hash_samples =
                    Some(parse_hash_samples(take_value(argv, &mut i, inline, "--hash-samples")?)?)
            }
            "--units" => {
                units = Some(parse_num(take_value(argv, &mut i, inline, "--units")?, "--units")?)
            }
            "--comp" => comp = Some(parse_comp(take_value(argv, &mut i, inline, "--comp")?)?),
            "--battle" => battle = true,
            "--dump-formation" => dump_formation = true,
            "--dump-final-units" => dump_final_units = true,
            _ => return Err(format!("unknown argument: '{raw}'")),
        }
        i += 1;
    }
    // T006/D6 线程档位合法域：1..=1024（0 或 >1024 → exit 2）。
    if threads == 0 || threads > MAX_THREADS {
        return Err(format!("--threads must be in 1..={MAX_THREADS}, got {threads}"));
    }
    if units.is_some() && comp.is_some() {
        return Err("--units and --comp are mutually exclusive".to_string());
    }
    // T005/D6 互斥：--battle 是对局收束路径，--units 是 T002 冒烟原语、
    // --dump-formation 是 tick 0 快照——语义冲突，一律拒绝。
    if battle && units.is_some() {
        return Err("--battle and --units are mutually exclusive".to_string());
    }
    if battle && dump_formation {
        return Err("--battle and --dump-formation are mutually exclusive".to_string());
    }
    // T006/D7 互斥：--hash-samples × --battle（BattleLog 已含终局 final_hash）。
    if battle && hash_samples.is_some() {
        return Err("--hash-samples and --battle are mutually exclusive".to_string());
    }
    // T006/D7 严格解析口径：采样点超出 --ticks 无段可跑 → exit 2。
    if let Some(samples) = &hash_samples {
        if let Some(&last) = samples.last() {
            if last > ticks {
                return Err(format!(
                    "--hash-samples tick {last} exceeds --ticks {ticks}"
                ));
            }
        }
    }
    Ok(Args {
        seed,
        ticks,
        threads,
        hash_samples,
        units,
        comp,
        battle,
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
                "usage: sim.exe [--seed <u64>] [--ticks <u64>] [--threads <usize>] \
                 [--hash-samples <u64,u64,...>] ([--comp <kind:count,...>] | \
                 [--units <usize>]) [--battle] [--dump-formation] [--dump-final-units] \
                 (--flag value or --flag=value; defaults: seed=42 ticks=1800 threads=1 \
                 comp=shieldman:5,heavyknight:5,pikeman:5,swordsman:5,archer:5,militia:5; \
                 --threads: worker thread tier 1..=1024, 1 = serial (stdout identical); \
                 --hash-samples: print sample=<tick> hash per strictly-ascending tick \
                 (0 = formation snapshot) before the summary, run-path only, mutually \
                 exclusive with --battle; \
                 --battle: run a battle to extinction or tick cap and print BattleLog (8 lines), \
                 mutually exclusive with --units and --dump-formation; values are decimal)"
            );
            return ExitCode::from(2);
        }
    };

    // T006/D6：threads > 1 时建持久线程池，作用域包住整个模拟段后 drop
    // （默认 1 = None 串行路径；两档共享同一两阶段代码路径，stdout 逐字节一致）。
    let _pool = (args.threads > 1).then(|| ThreadPool::new(args.threads));
    let pool = _pool.as_ref();

    let start = Instant::now();
    // 对局路径（T005/D6）：deploy 构战（--comp 或默认构成，双方镜像同清单）
    // → run_battle 收束（cap = --ticks，默认 1800 = TICK_CAP_REDUCED 口径）
    // → stdout 打印 BattleLog 固定 8 行（Display，确定性内容）；
    // --dump-final-units 可组合（终局后按现有格式逐单位打印）。
    // --units / --dump-formation / --hash-samples 已被 parse_args 互斥拒绝。
    if args.battle {
        let comp: Vec<(UnitKind, usize)> = match &args.comp {
            Some(c) => c.clone(),
            None => DEFAULT_COMPOSITION.to_vec(),
        };
        let mut world = World::deploy(args.seed, &comp);
        let outcome = world.run_battle_with(args.ticks, pool);
        let log = BattleLog {
            seed: args.seed,
            red_composition: comp.clone(),
            blue_composition: comp,
            outcome,
        };
        println!("{}", log);
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
        eprintln!("threads={}", args.threads);
        eprintln!("elapsed_ms={}", start.elapsed().as_millis());
        return ExitCode::SUCCESS;
    }
    // 构战路径：--comp 布阵 / 缺省默认构成布阵（D7）；--units 保留 T002 裸单位语义。
    let mut world = match (&args.comp, args.units) {
        (Some(comp), None) => World::deploy(args.seed, comp),
        (None, Some(u)) => World::new(args.seed, u),
        (None, None) => World::deploy(args.seed, &DEFAULT_COMPOSITION),
        (Some(_), Some(_)) => unreachable!("parse_args rejects --comp + --units"),
    };

    if args.dump_formation {
        // 布阵快照：tick 0、不跑 tick。五行摘要（hash 为布阵快照哈希）+ 每单位一行，全走 stdout。
        // （--hash-samples 仅作用 run 路径，此路径不生效——D7。）
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
        eprintln!("threads={}", args.threads);
        let elapsed = start.elapsed();
        eprintln!("elapsed_ms={}", elapsed.as_millis());
        return ExitCode::SUCCESS;
    }

    // T006/D7：分段推进（段边界 = 采样点，每段一次 run_with），每达点打一行
    // sample=<tick> hash=...（stdout，在五行摘要之前）。不用 flag 时零变化。
    match &args.hash_samples {
        None => world.run_with(args.ticks, pool),
        Some(samples) => {
            let mut current: u64 = 0;
            for &s in samples {
                if s > current {
                    world.run_with(s - current, pool);
                    current = s;
                }
                // s == 0（或与当前 tick 重合的起点）：布阵快照哈希，无需推进。
                println!("sample={} hash=0x{:016x}", s, world.state_hash());
            }
            if current < args.ticks {
                world.run_with(args.ticks - current, pool);
            }
        }
    }
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

    // stderr：线程档位（D6）+ 壁钟计时（仅外壳，不进模拟态）。
    eprintln!("threads={}", args.threads);
    eprintln!("elapsed_ms={}", elapsed.as_millis());

    ExitCode::SUCCESS
}
