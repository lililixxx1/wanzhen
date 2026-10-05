//! T007 基准量测套件（M0-06）——`sim` 第二 bin（`src/bin/` 自动发现；零 manifest
//! 改动、零新依赖；只复用 sim lib 公共 API，**lib 零改动红线**：World 与 ThreadPool
//! 的公共接口未做任何新增/修改）。
//!
//! 浮点边界（任务卡 D5 / AGENTS.md 硬约束 4）：本文件是**量测壳**，浮点仅在此
//! 存在（统计量 / 单位换算 / OLS 拟合）；模拟态（lib）零浮点红线不受影响——所有
//! `World` 调用参数均为整数，浮点不进入任何模拟状态，也不进任何模拟态归约。
//!
//! ## 模式 1（默认，量测）
//!
//! ```text
//! bench.exe --units <usize> --threads <usize> --ticks <u64>
//!           [--warmup <usize>=1] [--repeats <usize>=5] [--seed <u64>=42]
//! ```
//!
//! 流程（D3/D4）：构成映射（per_side = units/2，六兵种 q=per_side/6、余数 r 按
//! 表序前 r 个 +1）→ threads>1 时建池一次（跨 warmup+repeats 复用；threads=1
//! 走 `run_with(ticks, None)`）→ warmup W 局（fresh deploy，同 ticks 完整 run，
//! 不计时）→ 计时 R 局（每局 fresh deploy + `Instant` 只包 `run_with(ticks)`，
//! deploy/建池排除在外）→ 全部局 final `state_hash()` 一致校验（旁证；不一致则
//! stderr 报告 + exit 4）→ stdout 单行紧凑 JSON（固定键序）+ stderr 一行人类
//! 可读摘要。
//!
//! 退出码：0 成功；2 参数非法；4 同配置多局 final_hash 不一致。
//!
//! ## 模式 2（--summarize，汇总判定）
//!
//! ```text
//! bench.exe --summarize <matrix.jsonl> [--memory-csv <mem.csv>] [--out <summary.md>]
//! ```
//!
//! 读量测模式 JSONL → 代码计算任务卡 D6 全部判定行（防手算漂移）→ 写
//! `summary.md`（--out 缺省 `summary.md`）+ 打印 stdout（内容同文件）。
//!
//! 口径常量逐字取自报告 V0.9.1 表 6-0 / 6.1（任务卡 D6 原文复制，禁止凭记忆
//! 重写）：止损 = 每单位每 tick 成本 > 2µs（单线程基线）或 12 线程实测加速比
//! < 4×（16 线程外推值 < 4× 同判）；承诺线 = 止损线 × 0.5（µs 侧 1µs）；预算
//! 常态 @60fps ≤8ms / 极限 @30fps ≤22ms；换算 = 所需加速比 = 单位数 × 单线程
//! 每单位每 tick 成本 ÷ 每帧模拟预算。加速比止损锚定 10k 采样点（万人常态）。
//!
//! 退出码：0 成功（含判定 TRIPPED——数据判定不是工具错误）；1 I/O 错误；
//! 2 输入非法（JSONL/CSV 格式、重复配置、缺失键）；3 判定不完整（拟合退化
//! 行列式≈0 或关键配置缺失，如实报失败）。
//!
//! 内存 CSV 期望格式（由 docs/evidence/t007/run_longrun.ps1 采样器生成）：
//! 表头含 `working_set` 与 `private` 列名（不区分大小写），列为
//! `i,elapsed_ms,working_set_bytes,private_memory_bytes`；判定用工作集列。

use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

use sim::pool::ThreadPool;
use sim::units::UnitKind;
use sim::world::World;

// ---------- 口径常量（报告 V0.9.1 表 6-0 / 6.1，任务卡 D6 逐字；禁止凭记忆改动） ----------

const DEFAULT_WARMUP: usize = 1;
const DEFAULT_REPEATS: usize = 5;
const DEFAULT_SEED: u64 = 42;
/// 线程档位合法域上限（与 sim CLI 同口径：1..=1024）。
const MAX_THREADS: usize = 1024;

/// 止损线：每单位每 tick 成本 > 2µs（单线程基线）。
const STOP_US_PER_UNIT_TICK: f64 = 2.0;
/// 承诺线 = 止损线 × 0.5（µs 侧 1µs）。
const PROMISE_US_PER_UNIT_TICK: f64 = 1.0;
/// 加速比止损线：12 线程实测加速比 < 4×（16 线程外推值 < 4× 同判）。
const STOP_SPEEDUP: f64 = 4.0;
/// 加速比承诺线口径 = 止损线 × 0.5（仅用于换算表对照标注）。
const PROMISE_SPEEDUP: f64 = 2.0;
/// 帧模拟预算（表 6-0）：常态 @60fps ≤ 8ms；极限 @30fps ≤ 22ms。
const BUDGET_NORMAL_MS: f64 = 8.0;
const BUDGET_EXTREME_MS: f64 = 22.0;
/// 验收⑥：稳态工作集 ≤ 2GiB。
const MEMORY_CAP_BYTES: f64 = 2.0 * 1024.0 * 1024.0 * 1024.0;
/// 验收⑥ 无单调增长（再裁决定稿，主会话 2026-10-05，任务卡 D7 留痕）：
/// 稳态段后半高水位 ≤ 前半高水位 × 1.01。泄漏的物理签名是高水位单调爬升；
/// 页修剪振荡（本卡实测 5.07~6.77 MiB 振荡、i=1 处 7090176 B 近触顶，670 s
/// 内全局 max 7102464 B 仅 +0.17%、后半高水位 7098368 B 更低——P1-1 修正：
/// 原注释单位 MB 误标 MiB 且误以后半值为全局，2026-10-05 审核轮发现）不改变
/// 高水位。1% 容差容纳采样相位。
const MEMORY_CEILING_TOLERANCE: f64 = 1.01;
/// 原 D7 口径（末 25% 均值 ≤ 稳态中位数 × 1.05）：对振荡型平稳序列误报
/// （实测 6.6 > 5.8×1.05），保留计算作为披露行、不再作判定。
const MEMORY_GROWTH_FACTOR: f64 = 1.05;

/// ①③ 固定四采样点（模拟单位总数）。
const SAMPLE_TIERS: [usize; 4] = [1_000, 5_000, 10_000, 50_000];
/// ② 固定线程档位。
const THREAD_TIERS: [usize; 4] = [1, 3, 6, 12];
/// 加速比止损锚定规模（万人常态 = 10k 采样点）。
const ANCHOR_UNITS: usize = 10_000;
/// ③ 外推目标（极限十万）。
const EXTRAPOLATE_UNITS: usize = 100_000;
/// ③ speedup16 取 50k 档（最大规模、最近外推目标）。
const SPEEDUP16_SOURCE_UNITS: usize = 50_000;

// ---------- 参数解析 ----------

#[derive(Debug)]
struct MeasureArgs {
    units: usize,
    threads: usize,
    ticks: u64,
    warmup: usize,
    repeats: usize,
    seed: u64,
}

#[derive(Debug)]
struct SummarizeArgs {
    matrix: String,
    memory_csv: Option<String>,
    out: String,
}

#[derive(Debug)]
enum Mode {
    Help,
    Measure(MeasureArgs),
    Summarize(SummarizeArgs),
}

fn print_usage() {
    println!(
        "usage: bench.exe [measure] --units <usize> --threads <usize> --ticks <u64> \
         [--warmup <usize>=1] [--repeats <usize>=5] [--seed <u64>=42]\n\
         \x20      bench.exe --summarize <matrix.jsonl> [--memory-csv <mem.csv>] [--out <summary.md>=summary.md]\n\
         \x20      bench.exe --help\n\
         measure: units even and >=2; threads 1..=1024; ticks >=1; warmup >=0; repeats >=1;\n\
         \x20        composition per_side=units/2 split evenly over 6 kinds (remainder to table order);\n\
         \x20        fresh World::deploy per game; Instant wraps run_with(ticks) only; one pool reused;\n\
         \x20        all final state hashes must match (else exit 4); stdout = single-line JSON;\n\
         \x20        stderr = one human-readable summary line.\n\
         summarize: reads measure-mode JSONL (exactly one line per (units,threads)); computes all\n\
         \x20          judgment rows in code (report V0.9.1 表 6-0/6.1) and writes the summary.\n\
         exit: 0 ok | 1 io error | 2 bad usage/input | 3 incomplete judgments (fit degenerate) | 4 hash mismatch"
    );
}

fn parse_num<T: std::str::FromStr>(raw: &str, name: &str) -> Result<T, String> {
    raw.parse::<T>()
        .map_err(|_| format!("invalid value for {name}: '{raw}'"))
}

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

/// 手写解析（零依赖，风格同 sim CLI）：支持 `--flag value` / `--flag=value`。
/// 量测模式与汇总模式参数不可混用；--memory-csv/--out 仅汇总模式合法。
fn parse_args(argv: &[String]) -> Result<Mode, String> {
    let mut summarize: Option<String> = None;
    let mut memory_csv: Option<String> = None;
    let mut out: Option<String> = None;
    let mut units: Option<usize> = None;
    let mut threads: Option<usize> = None;
    let mut ticks: Option<u64> = None;
    let mut warmup = DEFAULT_WARMUP;
    let mut repeats = DEFAULT_REPEATS;
    let mut seed = DEFAULT_SEED;
    let mut measure_seen = false;
    let mut summarize_seen = false;
    let mut i = 0;
    while i < argv.len() {
        let raw = argv[i].as_str();
        let (name, inline) = match raw.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (raw, None),
        };
        match name {
            "--help" | "-h" => return Ok(Mode::Help),
            "--summarize" => {
                summarize = Some(take_value(argv, &mut i, inline, "--summarize")?.to_string());
                summarize_seen = true;
            }
            "--memory-csv" => {
                memory_csv = Some(take_value(argv, &mut i, inline, "--memory-csv")?.to_string());
                summarize_seen = true;
            }
            "--out" => {
                out = Some(take_value(argv, &mut i, inline, "--out")?.to_string());
                summarize_seen = true;
            }
            "--units" => {
                units = Some(parse_num(take_value(argv, &mut i, inline, "--units")?, "--units")?);
                measure_seen = true;
            }
            "--threads" => {
                threads =
                    Some(parse_num(take_value(argv, &mut i, inline, "--threads")?, "--threads")?);
                measure_seen = true;
            }
            "--ticks" => {
                ticks = Some(parse_num(take_value(argv, &mut i, inline, "--ticks")?, "--ticks")?);
                measure_seen = true;
            }
            "--warmup" => {
                warmup = parse_num(take_value(argv, &mut i, inline, "--warmup")?, "--warmup")?;
                measure_seen = true;
            }
            "--repeats" => {
                repeats = parse_num(take_value(argv, &mut i, inline, "--repeats")?, "--repeats")?;
                measure_seen = true;
            }
            "--seed" => {
                seed = parse_num(take_value(argv, &mut i, inline, "--seed")?, "--seed")?;
                measure_seen = true;
            }
            _ => return Err(format!("unknown argument: '{raw}'")),
        }
        i += 1;
    }
    if let Some(matrix) = summarize {
        if measure_seen {
            return Err(
                "--summarize cannot be mixed with measure args (--units/--threads/--ticks/--warmup/--repeats/--seed)"
                    .to_string(),
            );
        }
        return Ok(Mode::Summarize(SummarizeArgs {
            matrix,
            memory_csv,
            out: out.unwrap_or_else(|| "summary.md".to_string()),
        }));
    }
    if summarize_seen {
        return Err("--summarize requires <matrix.jsonl>; --memory-csv/--out only valid with --summarize".to_string());
    }
    // 量测模式校验（D2）：units 偶数且 ≥2；threads 1..=1024；ticks/repeats ≥1；
    // warmup ≥0 合法（0 = 仅计时局——⑥ 单局长跑口径用）。
    let units = units.ok_or("missing required argument --units")?;
    let threads = threads.ok_or("missing required argument --threads")?;
    let ticks = ticks.ok_or("missing required argument --ticks")?;
    if units < 2 || units % 2 != 0 {
        return Err(format!("--units must be even and >= 2, got {units}"));
    }
    if threads == 0 || threads > MAX_THREADS {
        return Err(format!("--threads must be in 1..={MAX_THREADS}, got {threads}"));
    }
    if ticks == 0 {
        return Err(format!("--ticks must be >= 1, got {ticks}"));
    }
    if repeats == 0 {
        return Err(format!("--repeats must be >= 1, got {repeats}"));
    }
    Ok(Mode::Measure(MeasureArgs {
        units,
        threads,
        ticks,
        warmup,
        repeats,
        seed,
    }))
}

// ---------- 量测模式 ----------

/// D3 构成映射：per_side = units/2；六兵种均分 q=per_side/6，余数 r 按表序前 r
/// 个 +1（表序 = UnitKind 判别序 Shieldman/HeavyKnight/Pikeman/Swordsman/Archer/
/// Militia）。`World::deploy(seed, comp)` 双方镜像。
///
/// 溢出复查（D3 留痕）：50k 上限 ⇒ per_side ≤ 25k，队列长度 ~35km × 2^32 ≈ 2^47.5
/// << i64 上界；x 折叠 Q32.32 无溢出。
fn composition_for(units: usize) -> (usize, Vec<(UnitKind, usize)>) {
    let per_side = units / 2;
    let q = per_side / 6;
    let r = per_side % 6;
    let kinds = [
        UnitKind::Shieldman,
        UnitKind::HeavyKnight,
        UnitKind::Pikeman,
        UnitKind::Swordsman,
        UnitKind::Archer,
        UnitKind::Militia,
    ];
    let comp = kinds
        .iter()
        .enumerate()
        .map(|(i, k)| (*k, q + usize::from(i < r)))
        .collect();
    (per_side, comp)
}

struct Stats {
    min_ns: u64,
    median_ns: f64,
    mean_ns: f64,
    max_ns: u64,
    /// 样本标准差 / 均值 × 100；R<2 时不可计算 → None（JSON 发射 null）。
    cv_pct: Option<f64>,
}

fn compute_stats(samples: &[u64]) -> Stats {
    let n = samples.len();
    debug_assert!(n >= 1);
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let min_ns = sorted[0];
    let max_ns = sorted[n - 1];
    let median_ns = if n % 2 == 1 {
        sorted[n / 2] as f64
    } else {
        (sorted[n / 2 - 1] as f64 + sorted[n / 2] as f64) / 2.0
    };
    let mean_ns = samples.iter().map(|&x| x as f64).sum::<f64>() / n as f64;
    let cv_pct = if n >= 2 {
        let var = samples
            .iter()
            .map(|&x| {
                let d = x as f64 - mean_ns;
                d * d
            })
            .sum::<f64>()
            / (n as f64 - 1.0);
        Some(var.sqrt() / mean_ns * 100.0)
    } else {
        None
    };
    Stats {
        min_ns,
        median_ns,
        mean_ns,
        max_ns,
        cv_pct,
    }
}

/// final_hash 一致校验（D4 自由旁证）：期望值首发后固定；任一局不一致即
/// Err（调用方 stderr 报告 + exit 4，不输出 JSON）。
fn register_hash(expected: &mut Option<u64>, h: u64, tag: &str) -> Result<(), String> {
    match *expected {
        Some(e) if e != h => Err(format!(
            "hash mismatch: {tag} final_hash=0x{h:016x}, expected 0x{e:016x} \
             (same seed/composition/ticks must be bit-identical)"
        )),
        Some(_) => Ok(()),
        None => {
            *expected = Some(h);
            Ok(())
        }
    }
}

/// f64 → JSON 数值串（Rust Display 最短往返；整值打印为整数字面量，无 NaN/Inf
/// 路径——调用处均为有限值）。
fn fnum(x: f64) -> String {
    debug_assert!(x.is_finite());
    format!("{x}")
}

fn run_measure(args: &MeasureArgs) -> ExitCode {
    let (per_side, comp) = composition_for(args.units);
    // D4：threads>1 建池一次，跨 warmup+repeats 复用；threads=1 走 None 串行路径
    // （与 sim CLI threads=1 同路径）。
    let pool = (args.threads > 1).then(|| ThreadPool::new(args.threads));
    let pool_ref = pool.as_ref();

    let mut expected_hash: Option<u64> = None;
    // warmup：W 局完整 run（同 ticks 全程），不计时；final_hash 一并参与一致校验
    // （同配置各局状态必须逐位相同——warmup 与计时局无差别）。
    for w in 0..args.warmup {
        let mut world = World::deploy(args.seed, &comp);
        world.run_with(args.ticks, pool_ref);
        let h = world.state_hash();
        if let Err(msg) = register_hash(&mut expected_hash, h, &format!("warmup#{w}")) {
            eprintln!("bench: {msg}");
            return ExitCode::from(4u8);
        }
    }

    let mut samples_ns: Vec<u64> = Vec::with_capacity(args.repeats);
    let mut final_hash = 0u64;
    for r in 0..args.repeats {
        // 每局 fresh deploy（同 seed 同构）；deploy 在计时窗外。
        let mut world = World::deploy(args.seed, &comp);
        let t0 = Instant::now();
        world.run_with(args.ticks, pool_ref);
        // 单局上限 ~10^3 s 量级 ⇒ ns < 1.8e19（u64 上界 ≈ 584 年），转换安全。
        let ns = t0.elapsed().as_nanos() as u64;
        let h = world.state_hash();
        if let Err(msg) = register_hash(&mut expected_hash, h, &format!("repeat#{r}")) {
            eprintln!("bench: {msg}");
            return ExitCode::from(4u8);
        }
        samples_ns.push(ns);
        final_hash = h;
    }

    let stats = compute_stats(&samples_ns);
    let ns_per_tick_median = stats.median_ns / args.ticks as f64;
    // D5：us_per_unit_tick = median_ns / ticks / units / 1000.0（左结合序照抄公式）。
    let us_per_unit_tick = stats.median_ns / args.ticks as f64 / args.units as f64 / 1000.0;

    // stdout：单行紧凑 JSON（固定键序，供 runner 追加成 matrix.jsonl）。
    let mut json = String::new();
    let _ = write!(
        json,
        "{{\"config\":{{\"units\":{},\"per_side\":{},\"threads\":{},\"ticks\":{},\"warmup\":{},\"repeats\":{},\"seed\":{}}},\"samples_ns\":[",
        args.units, per_side, args.threads, args.ticks, args.warmup, args.repeats, args.seed
    );
    for (idx, v) in samples_ns.iter().enumerate() {
        if idx > 0 {
            json.push(',');
        }
        let _ = write!(json, "{v}");
    }
    let _ = write!(
        json,
        "],\"min_ns\":{},\"median_ns\":{},\"mean_ns\":{},\"max_ns\":{},\"cv_pct\":",
        stats.min_ns,
        fnum(stats.median_ns),
        fnum(stats.mean_ns),
        stats.max_ns
    );
    match stats.cv_pct {
        Some(cv) => {
            let _ = write!(json, "{}", fnum(cv));
        }
        None => json.push_str("null"),
    }
    let _ = write!(
        json,
        ",\"ns_per_tick_median\":{},\"us_per_unit_tick\":{},\"final_hash\":\"0x{:016x}\",\"hash_consistent\":true}}",
        fnum(ns_per_tick_median),
        fnum(us_per_unit_tick),
        final_hash
    );
    println!("{json}");

    // stderr：一行人类可读摘要（config + median + µs）。
    eprintln!(
        "bench: units={} per_side={} threads={} ticks={} seed={} warmup={} repeats={} median_ms={:.3} ns_per_tick={:.1} us_per_unit_tick={:.6} final_hash=0x{:016x} hash_consistent=true",
        args.units,
        per_side,
        args.threads,
        args.ticks,
        args.seed,
        args.warmup,
        args.repeats,
        stats.median_ns / 1e6,
        ns_per_tick_median,
        us_per_unit_tick,
        final_hash
    );
    ExitCode::SUCCESS
}

// ---------- 最小 JSON 解析（汇总模式用；schema 为量测模式自产固定键序） ----------

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(map) => map.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }

    fn as_usize(&self) -> Option<usize> {
        match self {
            Json::Num(n) if n.is_finite() && *n >= 0.0 && n.fract() == 0.0 && *n <= usize::MAX as f64 => {
                Some(*n as usize)
            }
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            Json::Num(n) if n.is_finite() && *n >= 0.0 && n.fract() == 0.0 && *n <= u64::MAX as f64 => {
                Some(*n as u64)
            }
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(v) => Some(v),
            _ => None,
        }
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            bytes: s.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => Ok(Json::Str(self.parse_string()?)),
            Some(b't') => self.parse_lit("true", Json::Bool(true)),
            Some(b'f') => self.parse_lit("false", Json::Bool(false)),
            Some(b'n') => self.parse_lit("null", Json::Null),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(format!("unexpected byte '{}' at offset {}", c as char, self.pos)),
            None => Err("unexpected end of input".to_string()),
        }
    }

    fn parse_lit(&mut self, lit: &str, v: Json) -> Result<Json, String> {
        if self.bytes[self.pos..].starts_with(lit.as_bytes()) {
            self.pos += lit.len();
            Ok(v)
        } else {
            Err(format!("expected literal '{lit}' at offset {}", self.pos))
        }
    }

    fn parse_number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, b'-' | b'+' | b'.' | b'e' | b'E') {
                self.pos += 1;
            } else {
                break;
            }
        }
        let raw = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| "non-utf8 number token".to_string())?;
        raw.parse::<f64>()
            .map(Json::Num)
            .map_err(|e| format!("invalid number '{raw}': {e}"))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.pos += 1;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err("unterminated string".to_string()),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        Some(b'"') => out.push('"'),
                        Some(b'\\') => out.push('\\'),
                        Some(b'/') => out.push('/'),
                        Some(b'n') => out.push('\n'),
                        Some(b't') => out.push('\t'),
                        Some(b'r') => out.push('\r'),
                        Some(b'b') => out.push('\u{0008}'),
                        Some(b'f') => out.push('\u{000c}'),
                        Some(b'u') => {
                            let hex = self
                                .bytes
                                .get(self.pos + 1..self.pos + 5)
                                .ok_or("bad \\u escape")?;
                            let hex = std::str::from_utf8(hex).map_err(|_| "bad \\u escape")?;
                            let code = u16::from_str_radix(hex, 16)
                                .map_err(|_| format!("bad \\u escape '{hex}'"))?;
                            let ch = char::from_u32(code as u32).ok_or("invalid \\u code point")?;
                            out.push(ch);
                            self.pos += 4;
                        }
                        _ => return Err("bad escape sequence".to_string()),
                    }
                    self.pos += 1;
                }
                Some(c) if c < 0x20 => return Err("raw control byte in string".to_string()),
                Some(c) => {
                    // ASCII 快速路径 + UTF-8 多字节透传（本工具自产内容为纯 ASCII）。
                    out.push(c as char);
                    self.pos += 1;
                }
            }
        }
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.pos += 1; // '['
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(format!("expected ',' or ']' at offset {}", self.pos)),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.pos += 1; // '{'
        let mut map = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Json::Obj(map));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(format!("expected ':' at offset {}", self.pos));
            }
            self.pos += 1;
            let value = self.parse_value()?;
            map.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Json::Obj(map));
                }
                _ => return Err(format!("expected ',' or '}}' at offset {}", self.pos)),
            }
        }
    }
}

fn parse_json(s: &str) -> Result<Json, String> {
    let mut p = Parser::new(s);
    let v = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.bytes.len() {
        return Err(format!("trailing bytes at offset {}", p.pos));
    }
    Ok(v)
}

// ---------- 汇总模式 ----------

/// 一条量测记录（JSONL 行）。派生值（ns/tick、µs/单位/tick）由 median 复算并
/// 与文件内记录值交叉校验（相对容差 1e-6），防手改/漂移。
struct Rec {
    units: usize,
    threads: usize,
    ticks: u64,
    median_ns: f64,
    cv_pct: Option<f64>,
    ns_per_tick: f64,
    us_per_unit_tick: f64,
    final_hash: String,
}

fn close_enough(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6 * a.abs().max(b.abs()).max(1.0)
}

impl Rec {
    fn from_json(j: &Json) -> Result<Rec, String> {
        let cfg = j.get("config").ok_or("missing 'config'")?;
        let units = cfg.get("units").and_then(Json::as_usize).ok_or("config.units")?;
        let per_side = cfg
            .get("per_side")
            .and_then(Json::as_usize)
            .ok_or("config.per_side")?;
        let threads = cfg
            .get("threads")
            .and_then(Json::as_usize)
            .ok_or("config.threads")?;
        let ticks = cfg.get("ticks").and_then(Json::as_u64).ok_or("config.ticks")?;
        // warmup/seed 仅做存在性校验（本汇总判定不读）。
        cfg.get("warmup").ok_or("config.warmup")?;
        cfg.get("seed").ok_or("config.seed")?;
        let repeats = cfg
            .get("repeats")
            .and_then(Json::as_usize)
            .ok_or("config.repeats")?;
        if units < 2 || units % 2 != 0 {
            return Err(format!("units must be even and >=2, got {units}"));
        }
        if per_side != units / 2 {
            return Err(format!(
                "per_side mismatch: units={units} implies {}, got {per_side}",
                units / 2
            ));
        }
        if threads == 0 || threads > MAX_THREADS {
            return Err(format!("threads out of 1..={MAX_THREADS}: {threads}"));
        }
        if ticks == 0 || repeats == 0 {
            return Err(format!("ticks/repeats must be >=1: ticks={ticks} repeats={repeats}"));
        }
        let consistent = j
            .get("hash_consistent")
            .and_then(Json::as_bool)
            .ok_or("hash_consistent")?;
        if !consistent {
            return Err("hash_consistent=false (measurement is invalid; do not summarize)".to_string());
        }
        let samples = j.get("samples_ns").and_then(Json::as_arr).ok_or("samples_ns")?;
        if samples.len() != repeats {
            return Err(format!(
                "samples_ns length {} != repeats {repeats}",
                samples.len()
            ));
        }
        let median_ns = j.get("median_ns").and_then(Json::as_f64).ok_or("median_ns")?;
        let stored_ns_per_tick = j
            .get("ns_per_tick_median")
            .and_then(Json::as_f64)
            .ok_or("ns_per_tick_median")?;
        let stored_us = j
            .get("us_per_unit_tick")
            .and_then(Json::as_f64)
            .ok_or("us_per_unit_tick")?;
        let cv_pct = match j.get("cv_pct") {
            Some(Json::Null) | None => None,
            Some(v) => Some(v.as_f64().ok_or("cv_pct")?),
        };
        let final_hash = j
            .get("final_hash")
            .and_then(Json::as_str)
            .ok_or("final_hash")?
            .to_string();
        // 复算（D5 公式照抄）并与记录值交叉校验。
        let ns_per_tick = median_ns / ticks as f64;
        let us_per_unit_tick = median_ns / ticks as f64 / units as f64 / 1000.0;
        if !close_enough(stored_ns_per_tick, ns_per_tick) {
            return Err(format!(
                "ns_per_tick_median mismatch: stored {stored_ns_per_tick}, recomputed {ns_per_tick}"
            ));
        }
        if !close_enough(stored_us, us_per_unit_tick) {
            return Err(format!(
                "us_per_unit_tick mismatch: stored {stored_us}, recomputed {us_per_unit_tick}"
            ));
        }
        Ok(Rec {
            units,
            threads,
            ticks,
            median_ns,
            cv_pct,
            ns_per_tick,
            us_per_unit_tick,
            final_hash,
        })
    }
}

/// Amdahl 时间模型 OLS：elapsed_per_tick(T) = c1 + c2/T，基变量 [1, 1/T]，
/// 2×2 正规方程手写解（D6）。返回 (c1, c2, R², 最大残差)；行列式≈0 或
/// 预测耗时 ≤0 时 Err（如实报失败）。
struct AmdahlFit {
    c1: f64,
    c2: f64,
    r2: Option<f64>,
    max_res: f64,
}

fn fit_amdahl(points: &[(f64, f64)]) -> Result<AmdahlFit, String> {
    // points = [(T as f64, ns_per_tick)]，T ∈ {1,3,6,12}
    let n = points.len() as f64;
    let xs: Vec<f64> = points.iter().map(|(t, _)| 1.0 / t).collect();
    let ys: Vec<f64> = points.iter().map(|(_, y)| *y).collect();
    let sx: f64 = xs.iter().sum();
    let sx2: f64 = xs.iter().map(|x| x * x).sum();
    let sy: f64 = ys.iter().sum();
    let sxy: f64 = xs.iter().zip(&ys).map(|(x, y)| x * y).sum();
    let det = n * sx2 - sx * sx;
    if det.abs() <= 1e-12 * (n * sx2).max(1.0) {
        return Err(format!("Amdahl OLS 行列式≈0（det={det:e}），拟合退化"));
    }
    let c1 = (sy * sx2 - sx * sxy) / det;
    let c2 = (n * sxy - sx * sy) / det;
    let pred16 = c1 + c2 / 16.0;
    if !(pred16 > 0.0) || !pred16.is_finite() {
        return Err(format!("Amdahl 拟合预测 16 线程每 tick 耗时非正/非有限（c1+c2/16={pred16}）"));
    }
    let ybar = sy / n;
    let ss_tot: f64 = ys.iter().map(|y| (y - ybar) * (y - ybar)).sum();
    let mut ss_res = 0.0;
    let mut max_res = 0.0f64;
    for (x, y) in xs.iter().zip(&ys) {
        let pred = c1 + c2 * x;
        let res = (y - pred).abs();
        ss_res += res * res;
        if res > max_res {
            max_res = res;
        }
    }
    let r2 = if ss_tot > 0.0 {
        Some(1.0 - ss_res / ss_tot)
    } else {
        None
    };
    Ok(AmdahlFit {
        c1,
        c2,
        r2,
        max_res,
    })
}

/// 二次拟合 C(N) = a·N + b·N²（D6 ③；基变量 [N, N²]，2×2 正规方程手写解）。
struct QuadFit {
    a: f64,
    b: f64,
    r2: Option<f64>,
    max_res: f64,
}

fn fit_quadratic(points: &[(f64, f64)]) -> Result<QuadFit, String> {
    // points = [(N as f64, ns_per_tick)]
    let n = points.len() as f64;
    let ns: Vec<f64> = points.iter().map(|(x, _)| *x).collect();
    let ys: Vec<f64> = points.iter().map(|(_, y)| *y).collect();
    let s2: f64 = ns.iter().map(|x| x * x).sum();
    let s3: f64 = ns.iter().map(|x| x * x * x).sum();
    let s4: f64 = ns.iter().map(|x| x * x * x * x).sum();
    let sy1: f64 = ns.iter().zip(&ys).map(|(x, y)| x * y).sum();
    let sy2: f64 = ns.iter().zip(&ys).map(|(x, y)| x * x * y).sum();
    let det = s2 * s4 - s3 * s3;
    if det.abs() <= 1e-12 * (s2 * s4).max(1.0) {
        return Err(format!("二次拟合 OLS 行列式≈0（det={det:e}），拟合退化"));
    }
    let a = (sy1 * s4 - s3 * sy2) / det;
    let b = (s2 * sy2 - s3 * sy1) / det;
    let ybar = ys.iter().sum::<f64>() / n;
    let ss_tot: f64 = ys.iter().map(|y| (y - ybar) * (y - ybar)).sum();
    let mut ss_res = 0.0;
    let mut max_res = 0.0f64;
    for (x, y) in ns.iter().zip(&ys) {
        let pred = a * x + b * x * x;
        let res = (y - pred).abs();
        ss_res += res * res;
        if res > max_res {
            max_res = res;
        }
    }
    let r2 = if ss_tot > 0.0 {
        Some(1.0 - ss_res / ss_tot)
    } else {
        None
    };
    Ok(QuadFit { a, b, r2, max_res })
}

fn median_f64(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    debug_assert!(n >= 1);
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

struct MemStats {
    n: usize,
    dropped: usize,
    steady_max_bytes: u64,
    steady_median_bytes: f64,
    /// 稳态段前半高水位（判定口径基线）。
    first_half_max_bytes: u64,
    /// 稳态段后半高水位（判定对象）。
    second_half_max_bytes: u64,
    /// 无单调增长判定：后半高水位 ≤ 前半 × 1.01（再裁决口径，任务卡 D7）。
    ceiling_ok: bool,
    /// 原 D7 口径数值（披露行，不作判定）：末 25% 均值。
    tail_n: usize,
    tail_mean_bytes: f64,
    /// 原 D7 口径布尔（振荡敏感，仅披露）。
    tail_ok: bool,
    cap_ok: bool,
    private_max_bytes: u64,
    window_ms: f64,
}

/// 验收⑥ 判定（D7 再裁决定稿，2026-10-05，方法入档）：
/// - 稳态 = 弃首 10% 样本（floor：n/10）；
/// - 工作集 max ≤ 2GiB；
/// - 无单调增长 = 稳态后半高水位 ≤ 前半高水位 × 1.01（泄漏使高水位单调爬升；
///   页修剪振荡不改变高水位。原「末 25% 均值 ≤ 中位数 ×1.05」口径对振荡型
///   平稳序列误报，保留为披露行）。
/// 稳态样本 < 4 时判定不可得 → Err（如实报失败）。
fn memory_judge(ws: &[u64], private: &[u64], window_ms: f64) -> Result<MemStats, String> {
    let n = ws.len();
    let dropped = n / 10;
    let steady = &ws[dropped..];
    if steady.len() < 4 {
        return Err(format!(
            "steady samples too few ({}) for ceiling-stability judgment",
            steady.len()
        ));
    }
    let mut sorted: Vec<f64> = steady.iter().map(|&x| x as f64).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let steady_median_bytes = median_f64(&sorted);
    let steady_max_bytes = *steady.iter().max().expect("steady non-empty");
    let half = steady.len() / 2;
    let first_half_max_bytes = *steady[..half].iter().max().expect("half >= 2");
    let second_half_max_bytes = *steady[half..].iter().max().expect("half >= 2");
    let tail_n = (steady.len() / 4).max(1);
    let tail = &steady[steady.len() - tail_n..];
    let tail_mean_bytes = tail.iter().map(|&x| x as f64).sum::<f64>() / tail_n as f64;
    Ok(MemStats {
        n,
        dropped,
        steady_max_bytes,
        steady_median_bytes,
        first_half_max_bytes,
        second_half_max_bytes,
        ceiling_ok: second_half_max_bytes as f64
            <= first_half_max_bytes as f64 * MEMORY_CEILING_TOLERANCE,
        tail_n,
        tail_mean_bytes,
        tail_ok: tail_mean_bytes <= steady_median_bytes * MEMORY_GROWTH_FACTOR,
        cap_ok: steady_max_bytes as f64 <= MEMORY_CAP_BYTES,
        private_max_bytes: private.iter().copied().max().unwrap_or(0),
        window_ms,
    })
}

fn read_memory_csv(path: &str) -> Result<(Vec<u64>, Vec<u64>, f64), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or(format!("{path}: empty CSV"))?;
    let cols: Vec<String> = header
        .split(',')
        .map(|c| c.trim().trim_matches('"').to_ascii_lowercase())
        .collect();
    let ws_idx = cols
        .iter()
        .position(|c| c.contains("working_set") || c.contains("workingset"))
        .ok_or(format!("{path}: header has no working_set column: '{header}'"))?;
    let pm_idx = cols
        .iter()
        .position(|c| c.contains("private"))
        .ok_or(format!("{path}: header has no private column: '{header}'"))?;
    let time_idx = cols.iter().position(|c| c.contains("elapsed"));
    let mut ws = Vec::new();
    let mut pm = Vec::new();
    let mut window_ms = 0.0f64;
    for (lineno, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        let parse = |idx: usize, what: &str| -> Result<u64, String> {
            fields
                .get(idx)
                .ok_or(format!("{path}: line {}: missing {what}", lineno + 2))?
                .trim_matches('"')
                .parse::<u64>()
                .map_err(|e| format!("{path}: line {}: bad {what}: {e}", lineno + 2))
        };
        ws.push(parse(ws_idx, "working_set_bytes")?);
        pm.push(parse(pm_idx, "private_memory_bytes")?);
        if let Some(ti) = time_idx {
            if let Some(raw) = fields.get(ti) {
                if let Ok(t) = raw.trim_matches('"').parse::<f64>() {
                    window_ms = t;
                }
            }
        }
    }
    if ws.is_empty() {
        return Err(format!("{path}: no data rows"));
    }
    Ok((ws, pm, window_ms))
}

fn run_summarize(args: &SummarizeArgs) -> ExitCode {
    let text = match fs::read_to_string(&args.matrix) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("bench: cannot read {}: {e}", args.matrix);
            return ExitCode::from(1u8);
        }
    };
    let mut records: Vec<Rec> = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let json = match parse_json(line) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("bench: {}:{}: invalid JSON: {e}", args.matrix, lineno + 1);
                return ExitCode::from(2u8);
            }
        };
        match Rec::from_json(&json) {
            Ok(r) => records.push(r),
            Err(e) => {
                eprintln!("bench: {}:{}: {e}", args.matrix, lineno + 1);
                return ExitCode::from(2u8);
            }
        }
    }
    if records.is_empty() {
        eprintln!("bench: {} has no records", args.matrix);
        return ExitCode::from(2u8);
    }
    // 重复配置拒绝（matrix 语义 = 每 (units,threads) 恰一行；复测档另存不并入）。
    let mut seen: BTreeMap<(usize, usize), ()> = BTreeMap::new();
    for r in &records {
        if seen.insert((r.units, r.threads), ()).is_some() {
            eprintln!(
                "bench: duplicate config (units={}, threads={}) in {}",
                r.units, r.threads, args.matrix
            );
            return ExitCode::from(2u8);
        }
    }
    let index: BTreeMap<(usize, usize), usize> = records
        .iter()
        .enumerate()
        .map(|(i, r)| ((r.units, r.threads), i))
        .collect();
    let get = |units: usize, threads: usize| index.get(&(units, threads)).map(|&i| &records[i]);

    // 同规模跨线程档 ticks 必须一致：加速比 = median_ns 相除的前提（D8 矩阵按
    // 规模统一 ticks）；不一致 = 数据混档，拒绝判定（主会话复核补丁，2026-10-05，
    // 仅影响 summarize 模式——量测路径与 matrix.jsonl 产出零改动）。
    {
        let mut by_units: BTreeMap<usize, Vec<u64>> = BTreeMap::new();
        for r in &records {
            by_units.entry(r.units).or_default().push(r.ticks);
        }
        for (u, ts) in &by_units {
            let mut uniq = ts.clone();
            uniq.sort_unstable();
            uniq.dedup();
            if uniq.len() > 1 {
                eprintln!(
                    "bench: units={u} has mixed ticks across thread tiers ({uniq:?}) - speedup ratios would be invalid"
                );
                return ExitCode::from(2u8);
            }
        }
    }

    let mut incomplete = false;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# T007 基准量测汇总（M0-06 验收①②⑥ · `bench --summarize` 自动生成，禁止手改）\n"
    );
    let _ = writeln!(out, "- 数据源：{}（{} 条配置记录）", args.matrix, records.len());
    match &args.memory_csv {
        Some(p) => {
            let _ = writeln!(out, "- 内存采样：{p}");
        }
        None => {
            let _ = writeln!(out, "- 内存采样：（未提供 --memory-csv，验收⑥ 判定跳过）");
        }
    }
    let _ = writeln!(
        out,
        "- 口径（报告 V0.9.1 表 6-0 / 6.1，任务卡 D6 逐字）：止损 = 每单位每 tick 成本 > 2µs（单线程基线）或 12 线程实测加速比 < 4×（16 线程外推值 < 4× 同判）；承诺线 = 止损线 × 0.5（µs 侧 1µs）；预算 常态 @60fps ≤ 8ms / 极限 @30fps ≤ 22ms；所需加速比 = 单位数 × 单线程每单位每 tick 成本 ÷ 每帧模拟预算；加速比止损锚定 10k 采样点（万人常态）"
    );
    let _ = writeln!(
        out,
        "- 复算声明：下列全部数值由本程序从各配置 median_ns 复算（防手算漂移）；四采样点精确匹配 1k/5k/10k/50k、线程档 1/3/6/12 的记录才参与对应判定\n"
    );

    // ---------- ① 四采样点单线程 µs ----------
    let _ = writeln!(out, "## ① 四采样点单线程微秒成本（止损 2µs / 承诺 1µs）\n");
    let _ = writeln!(
        out,
        "| units | median_ns | ns/tick | us/单位/tick | 止损 2µs | 承诺 1µs |"
    );
    let _ = writeln!(out, "|---:|---:|---:|---:|---|---|");
    let mut point_rows: Vec<(usize, String)> = Vec::new();
    for &u in &SAMPLE_TIERS {
        match get(u, 1) {
            Some(r) => {
                let stop = if r.us_per_unit_tick > STOP_US_PER_UNIT_TICK {
                    "TRIPPED"
                } else {
                    "OK"
                };
                let promise = if r.us_per_unit_tick > PROMISE_US_PER_UNIT_TICK {
                    "未达"
                } else {
                    "达标"
                };
                let _ = writeln!(
                    out,
                    "| {} | {} | {:.1} | {:.6} | {} | {} |",
                    u,
                    fnum(r.median_ns),
                    r.ns_per_tick,
                    r.us_per_unit_tick,
                    stop,
                    promise
                );
                point_rows.push((u, format!(
                    "- [①] units={u}: us_per_unit_tick={:.6} µs | 止损线 2µs: {}（µs {} 2） | 承诺线 1µs: {}（µs {} 1）",
                    r.us_per_unit_tick,
                    stop,
                    if r.us_per_unit_tick > STOP_US_PER_UNIT_TICK { ">" } else { "<=" },
                    promise,
                    if r.us_per_unit_tick > PROMISE_US_PER_UNIT_TICK { ">" } else { "<=" },
                )));
            }
            None => {
                incomplete = true;
                let _ = writeln!(out, "| {} | （数据缺失） | - | - | - | - |", u);
                point_rows.push((u, format!("- [①] units={u}: 数据缺失（矩阵无 units={u}, threads=1 记录）")));
            }
        }
    }
    let _ = writeln!(out, "\n判定行：");
    for (_, line) in &point_rows {
        let _ = writeln!(out, "{line}");
    }

    // ---------- ② 加速比表 + Amdahl OLS + 16 线程外推 ----------
    let _ = writeln!(out, "\n## ② 加速比（每规模；elapsed = median_ns，全用 median）\n");
    let mut tiers: Vec<usize> = records.iter().map(|r| r.units).collect();
    tiers.sort_unstable();
    tiers.dedup();
    let _ = writeln!(
        out,
        "| units | t1 ns/tick | t3 ns/tick | t6 ns/tick | t12 ns/tick | ×3 | ×6 | ×12 | ×16（OLS 外推） |"
    );
    let _ = writeln!(out, "|---:|---:|---:|---:|---:|---:|---:|---:|---:|");
    struct TierFit {
        units: usize,
        c1: f64,
        c2: f64,
        r2: Option<f64>,
        max_res: f64,
        speedup16: f64,
        measured12: f64,
    }
    let mut fits: Vec<TierFit> = Vec::new();
    for &u in &tiers {
        let r1 = get(u, 1);
        let mut cells: Vec<String> = Vec::new();
        let mut ratios: Vec<String> = Vec::new();
        for &t in &THREAD_TIERS {
            match get(u, t) {
                Some(r) => cells.push(format!("{:.1}", r.ns_per_tick)),
                None => {
                    cells.push("-".to_string());
                    incomplete = true;
                }
            }
        }
        let mut measured12 = f64::NAN;
        match r1 {
            Some(base) => {
                for &t in &THREAD_TIERS[1..] {
                    match get(u, t) {
                        Some(r) => {
                            let sp = base.median_ns / r.median_ns;
                            if t == 12 {
                                measured12 = sp;
                            }
                            ratios.push(format!("{sp:.6}"));
                        }
                        None => ratios.push("-".to_string()),
                    }
                }
                let all_present = THREAD_TIERS.iter().all(|&t| get(u, t).is_some());
                if all_present {
                    let points: Vec<(f64, f64)> = THREAD_TIERS
                        .iter()
                        .map(|&t| (t as f64, get(u, t).unwrap().ns_per_tick))
                        .collect();
                    match fit_amdahl(&points) {
                        Ok(fit) => {
                            let speedup16 = base.ns_per_tick / (fit.c1 + fit.c2 / 16.0);
                            let _ = writeln!(
                                out,
                                "| {} | {} | {} | {} | {} | {} | {} | {} | {:.6} |",
                                u,
                                cells[0],
                                cells[1],
                                cells[2],
                                cells[3],
                                ratios[0],
                                ratios[1],
                                ratios[2],
                                speedup16
                            );
                            fits.push(TierFit {
                                units: u,
                                c1: fit.c1,
                                c2: fit.c2,
                                r2: fit.r2,
                                max_res: fit.max_res,
                                speedup16,
                                measured12,
                            });
                        }
                        Err(e) => {
                            incomplete = true;
                            let _ = writeln!(
                                out,
                                "| {} | {} | {} | {} | {} | {} | {} | {} | 拟合失败 |",
                                u,
                                cells[0],
                                cells[1],
                                cells[2],
                                cells[3],
                                ratios[0],
                                ratios[1],
                                ratios[2]
                            );
                            let _ = writeln!(out, "- [②] units={u}: Amdahl 拟合失败——{e}");
                        }
                    }
                } else {
                    incomplete = true;
                    let _ = writeln!(
                        out,
                        "| {} | {} | {} | {} | {} | {} | {} | {} | （缺线程档，拟合跳过） |",
                        u,
                        cells[0],
                        cells[1],
                        cells[2],
                        cells[3],
                        ratios[0],
                        ratios[1],
                        ratios[2]
                    );
                }
            }
            None => {
                incomplete = true;
                let _ = writeln!(
                    out,
                    "| {} | - | {} | {} | {} | - | - | - | （缺 t1，跳过） |",
                    u, cells[1], cells[2], cells[3]
                );
            }
        }
    }

    let _ = writeln!(
        out,
        "\nAmdahl 拟合（elapsed_per_tick(T) = c1 + c2/T；OLS 基变量 [1, 1/T]，T∈{{1,3,6,12}}；每秒 tick 耗时单位 ns）：\n"
    );
    let _ = writeln!(out, "| units | c1 (ns) | c2 (ns) | R² | 最大残差 (ns) | ×16 |");
    let _ = writeln!(out, "|---:|---:|---:|---:|---:|---:|");
    for f in &fits {
        let r2 = match f.r2 {
            Some(v) => format!("{v:.6}"),
            None => "null".to_string(),
        };
        let _ = writeln!(
            out,
            "| {} | {:.3} | {:.3} | {} | {:.3} | {:.6} |",
            f.units, f.c1, f.c2, r2, f.max_res, f.speedup16
        );
    }

    let mut anchor_tripped: Option<bool> = None;
    let _ = writeln!(out, "\n判定行（锚定 {} 万人常态）：", ANCHOR_UNITS);
    match fits.iter().find(|f| f.units == ANCHOR_UNITS) {
        Some(f) => {
            let m12_ok = f.measured12 >= STOP_SPEEDUP;
            let s16_ok = f.speedup16 >= STOP_SPEEDUP;
            anchor_tripped = Some(!(m12_ok && s16_ok));
            let _ = writeln!(
                out,
                "- [②] units={}: 12 线程实测加速比={:.6} | 止损判定（<4×）: {} | 16 线程外推加速比={:.6} | 同判（<4×）: {}",
                ANCHOR_UNITS,
                f.measured12,
                if m12_ok { "OK" } else { "TRIPPED" },
                f.speedup16,
                if s16_ok { "OK" } else { "TRIPPED" }
            );
        }
        None => {
            incomplete = true;
            let _ = writeln!(
                out,
                "- [②] units={ANCHOR_UNITS}: 数据缺失（缺 10k 某线程档或拟合失败），无法判定"
            );
        }
    }

    // ---------- ③ 外推十万 ----------
    let _ = writeln!(
        out,
        "\n## ③ 外推十万（C(N) = a·N + b·N²；四采样点单线程 ns/tick OLS；极限预算 22ms）\n"
    );
    let quad_points: Vec<(f64, f64)> = SAMPLE_TIERS
        .iter()
        .filter_map(|&u| get(u, 1).map(|r| (u as f64, r.ns_per_tick)))
        .collect();
    let mut quad_ok = false;
    let mut predicted_ms = f64::NAN;
    if quad_points.len() == SAMPLE_TIERS.len() {
        match fit_quadratic(&quad_points) {
            Ok(q) => {
                let c100k = q.a * EXTRAPOLATE_UNITS as f64
                    + q.b * (EXTRAPOLATE_UNITS as f64) * (EXTRAPOLATE_UNITS as f64);
                let r2 = match q.r2 {
                    Some(v) => format!("{v:.6}"),
                    None => "null".to_string(),
                };
                let _ = writeln!(
                    out,
                    "- 拟合：a={:.9}，b={:.9}，R²={}，最大残差={:.3} ns（拟合区间 1k~50k；R²/残差为附加留痕）",
                    q.a, q.b, r2, q.max_res
                );
                let _ = writeln!(out, "- C(100000)={c100k:.1} ns/tick（单线程外推）");
                match fits.iter().find(|f| f.units == SPEEDUP16_SOURCE_UNITS) {
                    Some(f) => {
                        let speedup16_50k = f.speedup16;
                        predicted_ms = c100k / speedup16_50k / 1e6;
                        let req = c100k / (BUDGET_EXTREME_MS * 1e6);
                        let keep = predicted_ms <= BUDGET_EXTREME_MS;
                        quad_ok = true;
                        let _ = writeln!(
                            out,
                            "- speedup16（{SPEEDUP16_SOURCE_UNITS} 档外推）={speedup16_50k:.6}"
                        );
                        let _ = writeln!(
                            out,
                            "- C(100000) ÷ speedup16 = {predicted_ms:.6} ms vs 22 ms 极限预算 → {}",
                            if keep { "[保留]" } else { "[超界：极限十万目标不保留]" }
                        );
                        let _ = writeln!(
                            out,
                            "- 加速比缺口对照：极限十万 @22ms 所需加速比={req:.6}（对照 16 线程外推 {speedup16_50k:.6}）"
                        );
                        // 「超界」是完整的数据判定（目标不保留），不是工具失败——
                        // 不置 incomplete、exit 仍为 0（与 TRIPPED 同理）。
                    }
                    None => {
                        incomplete = true;
                        let _ = writeln!(
                            out,
                            "- [③] {SPEEDUP16_SOURCE_UNITS} 档 Amdahl 拟合缺失，无法计算 speedup16——判定不可得"
                        );
                    }
                }
            }
            Err(e) => {
                incomplete = true;
                let _ = writeln!(out, "- [③] 二次拟合失败——{e}；判定不可得");
            }
        }
    } else {
        incomplete = true;
        let missing: Vec<String> = SAMPLE_TIERS
            .iter()
            .filter(|&&u| get(u, 1).is_none())
            .map(|u| format!("units={u},threads=1"))
            .collect();
        let _ = writeln!(out, "- [③] 四采样点单线程记录缺失（{}），拟合不可得", missing.join(" / "));
    }
    let _ = writeln!(
        out,
        "- 不确定度留痕：50k→100k 为 2× 超界外推（拟合样本区间 1k~50k，无 100k 实测）"
    );

    // ---------- ④ 6.1 换算对照 + 止损判定如实汇总 ----------
    let _ = writeln!(out, "\n## ④ 6.1 换算对照与止损判定（如实汇总）\n");
    let _ = writeln!(out, "换算表（所需加速比 = 单位数 × 单线程每单位每 tick 成本 ÷ 每帧模拟预算）：\n");
    let _ = writeln!(
        out,
        "| units | 单线程 us/单位/tick | 所需@8ms（常态） | 对照 | 所需@22ms（极限） | 对照 |"
    );
    let _ = writeln!(out, "|---:|---:|---:|---|---:|---|");
    for &u in &SAMPLE_TIERS {
        match get(u, 1) {
            Some(r) => {
                let req8 = u as f64 * r.us_per_unit_tick / (BUDGET_NORMAL_MS * 1000.0);
                let req22 = u as f64 * r.us_per_unit_tick / (BUDGET_EXTREME_MS * 1000.0);
                let label = |req: f64| -> &'static str {
                    if req <= PROMISE_SPEEDUP {
                        "承诺线内（≤2×）"
                    } else if req <= STOP_SPEEDUP {
                        "未超止损线（≤4×）"
                    } else {
                        "超止损线（>4×）"
                    }
                };
                let _ = writeln!(
                    out,
                    "| {} | {:.6} | {:.6} | {} | {:.6} | {} |",
                    u,
                    r.us_per_unit_tick,
                    req8,
                    label(req8),
                    req22,
                    label(req22)
                );
            }
            None => {
                let _ = writeln!(out, "| {} | （缺失） | - | - | - | - |", u);
            }
        }
    }

    let _ = writeln!(out, "\n止损判定（任一线触发即 TRIPPED，不粉饰）：");
    let mut triggers: Vec<String> = Vec::new();
    let mut us_tripped: Vec<usize> = Vec::new();
    for &u in &SAMPLE_TIERS {
        if let Some(r) = get(u, 1) {
            if r.us_per_unit_tick > STOP_US_PER_UNIT_TICK {
                us_tripped.push(u);
            }
        }
    }
    if us_tripped.is_empty() {
        let _ = writeln!(out, "- µs 侧（①）：四采样点全部 OK（均 ≤ 2µs 止损线）");
    } else {
        let list: Vec<String> = us_tripped.iter().map(|u| u.to_string()).collect();
        let _ = writeln!(
            out,
            "- µs 侧（①）：{}/4 采样点 TRIPPED（units={}，单线程 us/单位/tick > 2µs）",
            us_tripped.len(),
            list.join("/")
        );
        triggers.push(format!("① µs>2：units={}", list.join("/")));
    }
    if let Some(t) = anchor_tripped {
        if t {
            let _ = writeln!(out, "- 加速比侧（②锚定 {ANCHOR_UNITS}）：TRIPPED（12 线程实测 或 16 线程外推 < 4×）");
            triggers.push(format!("② 加速比<4×（锚定 {ANCHOR_UNITS}）"));
        } else {
            let _ = writeln!(out, "- 加速比侧（②锚定 {ANCHOR_UNITS}）：OK（实测与外推均 ≥ 4×）");
        }
    }
    if quad_ok {
        if predicted_ms <= BUDGET_EXTREME_MS {
            let _ = writeln!(
                out,
                "- 极限十万（③）：保留（外推 {predicted_ms:.6} ms ≤ 22ms）"
            );
        } else {
            let _ = writeln!(
                out,
                "- 极限十万（③）：超界（外推 {predicted_ms:.6} ms > 22ms，目标不保留；非止损阈值，独立于 ①②）"
            );
        }
    } else {
        let _ = writeln!(out, "- 极限十万（③）：判定不可得（见 ③ 节）");
    }
    let _ = writeln!(
        out,
        "- 总判定：{}",
        if triggers.is_empty() {
            "OK（①② 止损阈值均未触发）".to_string()
        } else {
            format!("TRIPPED（触发项：{}）；按 6.1/R2 由此进入后续优化决策（本卡不优化）", triggers.join("；"))
        }
    );

    // ---------- ⑥ 内存长跑判定 ----------
    if let Some(csv) = &args.memory_csv {
        let _ = writeln!(
            out,
            "\n## ⑤ 内存长跑判定（验收⑥；再裁决定稿口径见任务卡 D7：泄漏检测 = 高水位稳定性）\n"
        );
        match read_memory_csv(csv).and_then(|(ws, pm, window_ms)| {
            memory_judge(&ws, &pm, window_ms).map(|m| (m, ws.len()))
        }) {
            Ok((m, _ws_n)) => {
                let _ = writeln!(
                    out,
                    "- 样本：n={} 条（采样间隔 5s；每 5s 读 Get-Process WorkingSet64 + PrivateMemorySize64；判定用工作集）",
                    m.n
                );
                let _ = writeln!(out, "- 窗口时长（末样本 elapsed_ms）：{:.1} s", m.window_ms / 1000.0);
                let _ = writeln!(
                    out,
                    "- 稳态（弃首 10%，floor {}/10={} 条）：工作集 max={:.1} MiB ≤ 2048 MiB（2GiB）？ {}",
                    m.n,
                    m.dropped,
                    m.steady_max_bytes as f64 / (1024.0 * 1024.0),
                    if m.cap_ok { "OK" } else { "不达标" }
                );
                let _ = writeln!(
                    out,
                    "- 无单调增长（判定口径：高水位稳定性——泄漏使高水位单调爬升，页修剪振荡不改变高水位）：稳态后半 max={:.1} MiB ≤ 前半 max={:.1} MiB × 1.01 = {:.1} MiB？ {}",
                    m.second_half_max_bytes as f64 / (1024.0 * 1024.0),
                    m.first_half_max_bytes as f64 / (1024.0 * 1024.0),
                    m.first_half_max_bytes as f64 * MEMORY_CEILING_TOLERANCE / (1024.0 * 1024.0),
                    if m.ceiling_ok { "OK" } else { "不达标" }
                );
                let _ = writeln!(
                    out,
                    "- 原 D7 口径（披露行，振荡敏感、不作判定）：末 25%（{} 条）均值={:.1} MiB vs 稳态中位数 {:.1} MiB × 1.05 = {:.1} MiB → {}（振荡型平稳序列下该口径会误报——再裁决因果链见任务卡 D7 与证据档 README）",
                    m.tail_n,
                    m.tail_mean_bytes / (1024.0 * 1024.0),
                    m.steady_median_bytes / (1024.0 * 1024.0),
                    m.steady_median_bytes * MEMORY_GROWTH_FACTOR / (1024.0 * 1024.0),
                    if m.tail_ok { "满足" } else { "不满足" }
                );
                let _ = writeln!(
                    out,
                    "- [⑥] 判定：{}（附：PrivateMemorySize64 max={:.1} MiB）",
                    if m.cap_ok && m.ceiling_ok { "达标" } else { "不达标" },
                    m.private_max_bytes as f64 / (1024.0 * 1024.0)
                );
            }
            Err(e) => {
                eprintln!("bench: {e}");
                return ExitCode::from(2u8);
            }
        }
    }

    // ---------- 附录：全配置原始数据 ----------
    let _ = writeln!(out, "\n## 附录 A 全配置中位数与样本 CV（原始 median 复算）\n");
    let _ = writeln!(
        out,
        "| units | threads | ticks | median_ns | median_ms | us/单位/tick | cv_pct | final_hash |"
    );
    let _ = writeln!(out, "|---:|---:|---:|---:|---:|---:|---:|---|");
    for r in &records {
        let cv = match r.cv_pct {
            Some(v) => format!("{v:.6}"),
            None => "null".to_string(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {:.3} | {:.6} | {} | {} |",
            r.units,
            r.threads,
            r.ticks,
            fnum(r.median_ns),
            r.median_ns / 1e6,
            r.us_per_unit_tick,
            cv,
            r.final_hash
        );
    }
    let _ = writeln!(
        out,
        "\n（end；本文件由 bench --summarize 从 {matrix} 复算生成）",
        matrix = args.matrix
    );

    if let Err(e) = fs::write(&args.out, &out) {
        eprintln!("bench: cannot write {}: {e}", args.out);
        return ExitCode::from(1u8);
    }
    print!("{out}");
    if incomplete {
        eprintln!("bench: WARNING: 判定不完整（拟合退化或关键配置缺失），exit 3");
        ExitCode::from(3u8)
    } else {
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mode = match parse_args(&argv) {
        Ok(m) => m,
        Err(msg) => {
            eprintln!("bench: {msg}");
            print_usage();
            return ExitCode::from(2u8);
        }
    };
    match mode {
        Mode::Help => {
            print_usage();
            ExitCode::SUCCESS
        }
        Mode::Measure(a) => run_measure(&a),
        Mode::Summarize(s) => run_summarize(&s),
    }
}
