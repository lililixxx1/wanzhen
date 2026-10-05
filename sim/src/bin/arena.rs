//! T009 平衡实验场 v0（M0 验收③）——`sim` 第三 bin（`src/bin/` 自动发现；
//! 零 manifest 改动、零新依赖；stdout JSON 全部手写格式化，禁 serde；只复用
//! sim lib 公共 API，模拟核心零改动——新增 lib API 仅 [`sim::world::World::
//! deploy_versus`]）。
//!
//! 浮点边界（AGENTS.md 硬约束 4）：本文件是**实验场量测壳**，浮点仅在此存在
//! （壁钟统计换算 wall_s / games_per_hour / per_game_ms）；模拟态（lib）零浮点
//! 红线不受影响——所有 `World` 调用参数均为整数，浮点不进入任何模拟状态。
//!
//! ## 三模式（互斥；解析错误 / 冲突 → stderr + usage + exit 2；`--out <dir>`
//! 必选，不存在则创建）
//!
//! - `--matrix [--per-side <10|100>=100] [--per-cell <usize>=100] [--threads <usize>=1]`
//!   胜率矩阵：6×6 兵种对阵（cell = i*6 + j，i = 红兵种序、j = 蓝兵种序）×
//!   per_cell 局/格；lane = 50 m 缩比道；cap = [`sim::world::TICK_CAP_REDUCED`]；
//!   种子 `base + cell*K + k`（base：per_side=100 → 1_000_000 口径层；
//!   per_side=10 → 1_500_000 探针层；其余 per_side 拒绝——两层之外的种子空间
//!   未定义）。threads=1 不建池直跑；>1 经 [`sim::pool::ThreadPool::map_chunks`]
//!   按局分片（n = 36*K）。JSONL 逐局一行、顺序 = 局索引序，文件名
//!   `matrix_per100.jsonl` / `matrix_per10.jsonl`。
//! - `--throughput [--games <usize>=512] [--threads <usize>=12] [--repeats <usize>=3]`
//!   吞吐量测：每局 cell = g%36、per_side=100、lane 缩比、cap 1800、seed =
//!   2_000_000 + g；warmup 16 局（seed = 1_900_000 + w，不计时、不进数据）；
//!   计时批 R 遍，**计时窗只包对局循环 + 内存收集**（stdout / 文件 IO 一律
//!   窗外）；输出 `throughput_t{T}.json`（stdout 同内容单行）。
//!   `ai_decision_cost` 单列 0 = 报告表 6-0「AI 池扩档后须重测」钩子字段
//!   （镜像 AI v0 无独立决策模块）。
//! - `--sampling [--games <usize>=100] [--threads <usize>=12]`
//!   全规模抽样：每方 5000、lane = [`sim::world::LANE_LEN_Q32`]（1000 m）、
//!   cap = [`sim::world::TICK_CAP_FULL`]（14400）、seed = 3_000_000 + g×7919、
//!   cell = g%36；`ThreadPool::new(T)` 跨局复用、每局 `run_battle_with(14400,
//!   Some(&pool))`、局间串行；每局另记 wall_s（Instant 包单局，仅数据不进吞吐
//!   口径，随 stdout 单行 JSON 摘要落档）。JSONL `sampling.jsonl`，
//!   layer `full_per5000`。
//!
//! 退出码：0 成功；1 I/O 错误；2 用法 / 参数非法。
//!
//! 确定性纪律：种子空间四段互不重叠（matrix 1_000_000.. / probe 1_500_000.. /
//! throughput warmup 1_900_000.. / throughput 2_000_000.. / sampling
//! 3_000_000..）；并行只加速执行划分（对局间分片与局内意图阶段），模拟结果与
//! 线程数无关（W2 单测 + lib T006 两阶段等价性背书）。

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

use sim::pool::ThreadPool;
use sim::units::{UnitKind, ONE_Q32_32};
use sim::world::{World, LANE_LEN_Q32, TICK_CAP_FULL, TICK_CAP_REDUCED};

// ---------- 口径常量（派工单 §2 逐字落字；禁止凭记忆改动） ----------

/// 缩比单 lane 全长：50 m（口径层 / 探针层 / 吞吐模式统一布阵道长）。
const LANE_SCALED_Q32: i64 = 50 * ONE_Q32_32;
/// 胜率矩阵口径层种子基（per_side == 100）。
const SEED_MATRIX_BASE: u64 = 1_000_000;
/// 胜率矩阵探针层种子基（per_side == 10）。
const SEED_PROBE_BASE: u64 = 1_500_000;
/// 吞吐模式种子基。
const SEED_THROUGHPUT_BASE: u64 = 2_000_000;
/// 全规模抽样种子基。
const SEED_SAMPLING_BASE: u64 = 3_000_000;
/// 全规模抽样种子步长（seed = 基 + g×7919，7919 质数防同格种子共振）。
const SAMPLING_SEED_STRIDE: u64 = 7919;
/// 吞吐 warmup 种子基（1_900_000 + w；不计时、不进数据）。
const THROUGHPUT_WARMUP_BASE: u64 = 1_900_000;
/// 吞吐 warmup 局数（派工单 §2）。
const THROUGHPUT_WARMUP_GAMES: usize = 16;
/// 全规模抽样每方单位数（派工单 §2）。
const FULL_PER_SIDE: usize = 5000;
/// 对阵格数（6 兵种 × 6 兵种）。
const CELLS: usize = 36;
/// 线程档位合法域上限（与 sim CLI 同口径 1..=1024）。
const MAX_THREADS: usize = 1024;

/// 兵种表序（= 判别值序 = units.rs SPECS 表序；cell = i*6 + j）。
const KINDS: [UnitKind; 6] = [
    UnitKind::Shieldman,
    UnitKind::HeavyKnight,
    UnitKind::Pikeman,
    UnitKind::Swordsman,
    UnitKind::Archer,
    UnitKind::Militia,
];

// ---------- 逐局记录 ----------

/// 单局 JSONL 记录（字段序固定，与派工单 §2 样例行逐键一致）。
struct GameRow {
    layer: &'static str,
    cell: usize,
    red: &'static str,
    blue: &'static str,
    per_side: usize,
    lane_m: u64,
    seed: u64,
    winner: &'static str,
    end_tick: u64,
    alive_red: u32,
    alive_blue: u32,
    resolved: bool,
    final_hash: u64,
}

/// resolved 语义（派工单 §2）：全灭收束 ⟺ 一方零存活（上限 hp-sum 收束分支
/// 双方必 >0，见 lib run_battle 控制流）。
fn is_resolved(alive_red: u32, alive_blue: u32) -> bool {
    alive_red == 0 || alive_blue == 0
}

fn write_row(out: &mut String, r: &GameRow) {
    let _ = write!(
        out,
        "{{\"layer\":\"{}\",\"cell\":{},\"red\":\"{}\",\"blue\":\"{}\",\"per_side\":{},\
         \"lane_m\":{},\"seed\":{},\"winner\":\"{}\",\"end_tick\":{},\"alive_red\":{},\
         \"alive_blue\":{},\"resolved\":{},\"final_hash\":\"0x{:016x}\"}}",
        r.layer,
        r.cell,
        r.red,
        r.blue,
        r.per_side,
        r.lane_m,
        r.seed,
        r.winner,
        r.end_tick,
        r.alive_red,
        r.alive_blue,
        r.resolved,
        r.final_hash
    );
}

// ---------- 局清单参数（纯函数，供模式实现与 W3 单测共用） ----------

/// cell →（红兵种序 i，蓝兵种序 j）；cell = i*6 + j。
fn cell_kinds(cell: usize) -> (usize, usize) {
    (cell / 6, cell % 6)
}

/// matrix 局清单闭式（派工单 §2）：cell = g/K、k = g%K、seed = base + cell*K + k
/// （恒等于 base + g；以闭式落字与 W3 断言同源）。
fn matrix_seed(base: u64, per_cell: usize, game: usize) -> u64 {
    let cell = game / per_cell;
    let k = game % per_cell;
    base + (cell * per_cell + k) as u64
}

/// sampling 局清单闭式（派工单 §2）：cell = g%36、seed = 3_000_000 + g*7919。
fn sampling_params(game: usize) -> (usize, u64) {
    (
        game % CELLS,
        SEED_SAMPLING_BASE + game as u64 * SAMPLING_SEED_STRIDE,
    )
}

// ---------- 逐局执行 ----------

/// matrix 单局：`deploy_versus(seed, [(红兵种, N)], [(蓝兵种, N)], 50m 缩比道)`
/// → `run_battle(1800)`（串行 None 路径；局间并行只按局分片，不改变单局串行语义）。
fn matrix_row(game: usize, per_side: usize, per_cell: usize, base: u64) -> GameRow {
    let cell = game / per_cell;
    let (i, j) = cell_kinds(cell);
    let seed = matrix_seed(base, per_cell, game);
    let mut world = World::deploy_versus(
        seed,
        &[(KINDS[i], per_side)],
        &[(KINDS[j], per_side)],
        LANE_SCALED_Q32,
    );
    let o = world.run_battle(TICK_CAP_REDUCED);
    GameRow {
        layer: if per_side == 100 {
            "matrix_per100"
        } else {
            "matrix_per10"
        },
        cell,
        red: KINDS[i].id(),
        blue: KINDS[j].id(),
        per_side,
        lane_m: 50,
        seed,
        winner: o.winner.label(),
        end_tick: o.end_tick,
        alive_red: o.alive_red,
        alive_blue: o.alive_blue,
        resolved: is_resolved(o.alive_red, o.alive_blue),
        final_hash: o.final_hash,
    }
}

/// throughput 单局（计时批与 warmup 同构路径）：返回 (final_hash, end_tick)。
/// warmup 与计时批以 `seed_base` 参数区分（warmup 传 1_900_000，不计时、不进数据）。
fn throughput_game(seed_base: u64, game: usize) -> (u64, u64) {
    let cell = game % CELLS;
    let (i, j) = cell_kinds(cell);
    let seed = seed_base + game as u64;
    let mut world = World::deploy_versus(
        seed,
        &[(KINDS[i], 100)],
        &[(KINDS[j], 100)],
        LANE_SCALED_Q32,
    );
    let o = world.run_battle(TICK_CAP_REDUCED);
    (o.final_hash, o.end_tick)
}

/// sampling 单局：每方 5000、lane 1000 m、cap 14400、`run_battle_with(Some(pool))`；
/// 返回（记录行，单局 wall_s——Instant 包单局，仅数据不进吞吐口径）。
fn sampling_row(game: usize, pool: &ThreadPool) -> (GameRow, f64) {
    let (cell, seed) = sampling_params(game);
    let (i, j) = cell_kinds(cell);
    let mut world = World::deploy_versus(
        seed,
        &[(KINDS[i], FULL_PER_SIDE)],
        &[(KINDS[j], FULL_PER_SIDE)],
        LANE_LEN_Q32,
    );
    let t0 = Instant::now();
    let o = world.run_battle_with(TICK_CAP_FULL, Some(pool));
    let wall = t0.elapsed().as_secs_f64();
    (
        GameRow {
            layer: "full_per5000",
            cell,
            red: KINDS[i].id(),
            blue: KINDS[j].id(),
            per_side: FULL_PER_SIDE,
            lane_m: 1000,
            seed,
            winner: o.winner.label(),
            end_tick: o.end_tick,
            alive_red: o.alive_red,
            alive_blue: o.alive_blue,
            resolved: is_resolved(o.alive_red, o.alive_blue),
            final_hash: o.final_hash,
        },
        wall,
    )
}

// ---------- 小工具 ----------

fn fnum(x: f64) -> String {
    debug_assert!(x.is_finite());
    format!("{x}")
}

fn median_f64(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("wall_s must be finite"));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn write_out_file(out_dir: &str, file_name: &str, content: &str) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|e| format!("cannot create dir {out_dir}: {e}"))?;
    let path = format!("{out_dir}/{file_name}");
    fs::write(&path, content).map_err(|e| format!("cannot write {path}: {e}"))
}

fn rows_to_jsonl(rows: &[GameRow]) -> String {
    let mut buf = String::new();
    for r in rows {
        write_row(&mut buf, r);
        buf.push('\n');
    }
    buf
}

// ---------- 参数解析（手写零依赖；风格同 sim main.rs / bench.rs） ----------

#[derive(Debug)]
enum Mode {
    Matrix {
        per_side: usize,
        per_cell: usize,
        threads: usize,
        out: String,
    },
    Throughput {
        games: usize,
        threads: usize,
        repeats: usize,
        out: String,
    },
    Sampling {
        games: usize,
        threads: usize,
        out: String,
    },
}

fn print_usage() {
    eprintln!(
        "usage: arena.exe --matrix [--per-side <10|100>=100] [--per-cell <usize>=100] \
         [--threads <usize>=1] --out <dir>\n\
         \x20      arena.exe --throughput [--games <usize>=512] [--threads <usize>=12] \
         [--repeats <usize>=3] --out <dir>\n\
         \x20      arena.exe --sampling [--games <usize>=100] [--threads <usize>=12] --out <dir>\n\
         modes are mutually exclusive; --out <dir> is required (created if missing);\n\
         matrix: 6x6 kind matchups (cell=i*6+j) x per_cell games/cell, lane 50 m scaled,\n\
         \x20        cap 1800 ticks, seed = base + cell*K + k (base 1000000 for per-side 100,\n\
         \x20        1500000 for per-side 10; other per-side values rejected); writes\n\
         \x20        matrix_per100.jsonl / matrix_per10.jsonl (one row per game, game order);\n\
         throughput: games cell=g%36 per-side 100 lane 50m cap 1800, seed=2000000+g,\n\
         \x20        warmup 16 games (seeds 1900000+w, untimed), timing window wraps the\n\
         \x20        game loop + row collection only; writes throughput_t{{T}}.json;\n\
         sampling: 5000 per side, lane 1000 m, cap 14400 ticks, seed=3000000+g*7919,\n\
         \x20        games sequential on one shared pool, per-game wall_s recorded (data\n\
         \x20        only, not a throughput figure); writes sampling.jsonl;\n\
         exit: 0 ok | 1 io error | 2 bad usage (values are decimal)"
    );
}

fn parse_num<T: std::str::FromStr>(raw: &str, name: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    raw.parse::<T>()
        .map_err(|e| format!("invalid value for {name}: '{raw}' ({e})"))
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

fn parse_args(argv: &[String]) -> Result<Mode, String> {
    let mut mode_flags = 0usize; // bit0 matrix / bit1 throughput / bit2 sampling
    let mut per_side: Option<usize> = None;
    let mut per_cell: Option<usize> = None;
    let mut threads: Option<usize> = None;
    let mut games: Option<usize> = None;
    let mut repeats: Option<usize> = None;
    let mut out: Option<String> = None;
    let mut i = 0;
    while i < argv.len() {
        let raw = argv[i].as_str();
        let (name, inline) = match raw.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (raw, None),
        };
        match name {
            "--matrix" => mode_flags |= 1,
            "--throughput" => mode_flags |= 2,
            "--sampling" => mode_flags |= 4,
            "--per-side" => {
                per_side =
                    Some(parse_num(take_value(argv, &mut i, inline, "--per-side")?, "--per-side")?)
            }
            "--per-cell" => {
                per_cell =
                    Some(parse_num(take_value(argv, &mut i, inline, "--per-cell")?, "--per-cell")?)
            }
            "--threads" => {
                threads =
                    Some(parse_num(take_value(argv, &mut i, inline, "--threads")?, "--threads")?)
            }
            "--games" => {
                games = Some(parse_num(take_value(argv, &mut i, inline, "--games")?, "--games")?)
            }
            "--repeats" => {
                repeats =
                    Some(parse_num(take_value(argv, &mut i, inline, "--repeats")?, "--repeats")?)
            }
            "--out" => {
                out = Some(take_value(argv, &mut i, inline, "--out")?.to_string());
            }
            _ => return Err(format!("unknown argument: '{raw}'")),
        }
        i += 1;
    }
    if mode_flags == 0 {
        return Err("one of --matrix / --throughput / --sampling is required".to_string());
    }
    if mode_flags.count_ones() > 1 {
        return Err("--matrix / --throughput / --sampling are mutually exclusive".to_string());
    }
    let out = out.ok_or("--out <dir> is required")?;
    // threads 校验统一在此：显式给出时必须 1..=1024（0 拒绝，与 sim CLI 同口径）；
    // 未给出时按模式缺省（matrix 1 / throughput 12 / sampling 12）。
    if let Some(t) = threads {
        if t == 0 || t > MAX_THREADS {
            return Err(format!("--threads must be in 1..={MAX_THREADS}, got {t}"));
        }
    }
    let threads_of = |default: usize| threads.unwrap_or(default);
    match mode_flags {
        1 => {
            if per_side.is_some() && !matches!(per_side, Some(10) | Some(100)) {
                return Err(format!(
                    "--per-side must be 10 or 100 (matrix seed base is only defined for the two layers), got {:?}",
                    per_side
                ));
            }
            let per_side = per_side.unwrap_or(100);
            let per_cell = per_cell.unwrap_or(100);
            if per_cell == 0 {
                return Err("--per-cell must be >= 1".to_string());
            }
            Ok(Mode::Matrix {
                per_side,
                per_cell,
                threads: threads_of(1),
                out,
            })
        }
        2 => {
            let games = games.unwrap_or(512);
            let repeats = repeats.unwrap_or(3);
            if games == 0 {
                return Err("--games must be >= 1".to_string());
            }
            if repeats == 0 {
                return Err("--repeats must be >= 1".to_string());
            }
            Ok(Mode::Throughput {
                games,
                threads: threads_of(12),
                repeats,
                out,
            })
        }
        4 => {
            let games = games.unwrap_or(100);
            if games == 0 {
                return Err("--games must be >= 1".to_string());
            }
            if per_side.is_some() || per_cell.is_some() || repeats.is_some() {
                return Err(
                    "--per-side/--per-cell/--repeats are only valid with --matrix/--throughput"
                        .to_string(),
                );
            }
            Ok(Mode::Sampling {
                games,
                threads: threads_of(12),
                out,
            })
        }
        _ => unreachable!("mode_flags validated above"),
    }
}

// ---------- 三模式实现 ----------

fn run_matrix(per_side: usize, per_cell: usize, threads: usize, out: &str) -> Result<(), String> {
    let base = if per_side == 100 {
        SEED_MATRIX_BASE
    } else {
        SEED_PROBE_BASE
    };
    let layer = if per_side == 100 {
        "matrix_per100"
    } else {
        "matrix_per10"
    };
    let total = CELLS * per_cell;
    let t0 = Instant::now();
    // threads=1 不建池直跑（派工单 §2）；>1 按局分片（map_chunks 结果按片 start
    // 升序拼装 = 局索引序）。
    let rows: Vec<GameRow> = if threads == 1 {
        (0..total)
            .map(|g| matrix_row(g, per_side, per_cell, base))
            .collect()
    } else {
        let pool = ThreadPool::new(threads);
        pool.map_chunks(total, move |s, e| {
            (s..e)
                .map(|g| matrix_row(g, per_side, per_cell, base))
                .collect()
        })
    };
    let wall = t0.elapsed().as_secs_f64();
    let file = format!("{layer}.jsonl");
    write_out_file(out, &file, &rows_to_jsonl(&rows))?;
    let resolved_games = rows.iter().filter(|r| r.resolved).count();
    println!(
        "{{\"mode\":\"matrix\",\"layer\":\"{layer}\",\"per_side\":{per_side},\
         \"per_cell\":{per_cell},\"threads\":{threads},\"games\":{total},\
         \"resolved_games\":{resolved_games},\"wall_s\":{}}}",
        fnum(wall)
    );
    Ok(())
}

fn run_throughput(games: usize, threads: usize, repeats: usize, out: &str) -> Result<(), String> {
    let pool = ThreadPool::new(threads);
    // warmup：16 局完整对局（seed = 1_900_000 + w），不计时、不进数据；
    // 与计时批共用同一池（页缓存 / 池预热口径与 bench 一致）。
    let warm_rows: Vec<(u64, u64)> = pool.map_chunks(THROUGHPUT_WARMUP_GAMES, |s, e| {
        (s..e)
            .map(|g| throughput_game(THROUGHPUT_WARMUP_BASE, g))
            .collect()
    });
    debug_assert_eq!(warm_rows.len(), THROUGHPUT_WARMUP_GAMES);

    // 计时批：Instant 只包 map_chunks（对局循环 + rows 内存收集）；
    // hash_xor 折叠 / 统计 / stdout / 文件 IO 一律窗外。
    let mut walls: Vec<f64> = Vec::with_capacity(repeats);
    let mut hash_xor: u64 = 0;
    for _r in 0..repeats {
        let t0 = Instant::now();
        let rows: Vec<(u64, u64)> =
            pool.map_chunks(games, |s, e| (s..e).map(|g| throughput_game(SEED_THROUGHPUT_BASE, g)).collect());
        let dt = t0.elapsed();
        hash_xor = rows.iter().fold(0u64, |acc, (h, _)| acc ^ h);
        walls.push(dt.as_secs_f64());
    }
    let median = median_f64(walls.clone());
    let games_per_hour = games as f64 * 3600.0 / median;
    let per_game_ms = median * 1000.0 / games as f64;
    let mut wall_arr = String::from("[");
    for (idx, w) in walls.iter().enumerate() {
        if idx > 0 {
            wall_arr.push(',');
        }
        let _ = write!(wall_arr, "{}", fnum(*w));
    }
    wall_arr.push(']');
    let json = format!(
        "{{\"games\":{games},\"threads\":{threads},\"warmup\":{THROUGHPUT_WARMUP_GAMES},\
         \"wall_s\":{wall_arr},\"wall_s_median\":{},\"games_per_hour\":{},\
         \"per_game_ms\":{},\"ai_decision_cost\":0,\"final_hash_xor\":\"0x{hash_xor:016x}\"}}",
        fnum(median),
        fnum(games_per_hour),
        fnum(per_game_ms)
    );
    write_out_file(out, &format!("throughput_t{threads}.json"), &json)?;
    println!("{json}");
    Ok(())
}

fn run_sampling(games: usize, threads: usize, out: &str) -> Result<(), String> {
    let pool = ThreadPool::new(threads);
    let mut rows: Vec<GameRow> = Vec::with_capacity(games);
    let mut walls: Vec<f64> = Vec::with_capacity(games);
    let t_all = Instant::now();
    // 局间串行（派工单 §2）；每局 wall_s = Instant 包单局（仅数据不进吞吐口径）。
    for g in 0..games {
        let (row, wall) = sampling_row(g, &pool);
        rows.push(row);
        walls.push(wall);
    }
    let total = t_all.elapsed().as_secs_f64();
    write_out_file(out, "sampling.jsonl", &rows_to_jsonl(&rows))?;
    let median = median_f64(walls.clone());
    let min = walls.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = walls.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut wall_arr = String::from("[");
    for (idx, w) in walls.iter().enumerate() {
        if idx > 0 {
            wall_arr.push(',');
        }
        let _ = write!(wall_arr, "{}", fnum(*w));
    }
    wall_arr.push(']');
    println!(
        "{{\"mode\":\"sampling\",\"games\":{games},\"threads\":{threads},\
         \"file\":\"sampling.jsonl\",\"wall_total_s\":{},\"wall_s_median\":{},\
         \"wall_s_min\":{},\"wall_s_max\":{},\"per_game_wall_s\":{wall_arr}}}",
        fnum(total),
        fnum(median),
        fnum(min),
        fnum(max)
    );
    Ok(())
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mode = match parse_args(&argv) {
        Ok(m) => m,
        Err(msg) => {
            eprintln!("arena: {msg}");
            print_usage();
            return ExitCode::from(2u8);
        }
    };
    let result = match mode {
        Mode::Matrix {
            per_side,
            per_cell,
            threads,
            out,
        } => run_matrix(per_side, per_cell, threads, &out),
        Mode::Throughput {
            games,
            threads,
            repeats,
            out,
        } => run_throughput(games, threads, repeats, &out),
        Mode::Sampling { games, threads, out } => run_sampling(games, threads, &out),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("arena: {msg}");
            ExitCode::from(1u8)
        }
    }
}

// ---------- T009 单测（派工单 §5 W2/W3/W4） ----------

#[cfg(test)]
mod tests {
    use super::*;
    use sim::world::Winner;

    /// T009 W2：确定性 + 跨线程——(HeavyKnight,3) vs (Militia,3)、lane 50m 缩比、
    /// cap 1800：同参双跑 state_hash 相等；threads {1,4} run_battle final_hash 相等。
    #[test]
    fn w2_determinism_same_params_and_cross_thread_final_hash() {
        let mk = || {
            World::deploy_versus(
                20_261_005,
                &[(UnitKind::HeavyKnight, 3)],
                &[(UnitKind::Militia, 3)],
                LANE_SCALED_Q32,
            )
        };
        let a = mk();
        let b = mk();
        assert_eq!(
            a.state_hash(),
            b.state_hash(),
            "同参双跑 tick0 state_hash 必须逐位相等"
        );
        assert_eq!(a.last_hash, b.last_hash, "同参双跑布阵快照哈希必须逐位相等");
        let mut w1 = mk();
        let o1 = w1.run_battle(TICK_CAP_REDUCED);
        let mut w4 = mk();
        let pool = ThreadPool::new(4);
        let o4 = w4.run_battle_with(TICK_CAP_REDUCED, Some(&pool));
        assert_eq!(
            o1.final_hash, o4.final_hash,
            "threads {{1,4}} final_hash 必须逐位相等"
        );
        assert_eq!(o1.end_tick, o4.end_tick, "threads {{1,4}} end_tick 必须相等");
        assert_eq!(
            (o1.alive_red, o1.alive_blue),
            (o4.alive_red, o4.alive_blue),
            "threads {{1,4}} 存活计数必须相等"
        );
    }

    /// T009 W3：种子算式自检——matrix 局清单前 5 局 seed == 闭式 base + cell*K + k
    /// （两基各查）；sampling 前 3 局 seed == 3_000_000 + g*7919。
    #[test]
    fn w3_seed_formulas_matrix_and_sampling() {
        for &(per_side, base) in &[(100usize, SEED_MATRIX_BASE), (10usize, SEED_PROBE_BASE)] {
            let per_cell = 100usize;
            for g in 0..5 {
                let cell = g / per_cell;
                let k = g % per_cell;
                assert_eq!(
                    matrix_seed(base, per_cell, g),
                    base + (cell * per_cell + k) as u64,
                    "per_side={per_side} g={g}: seed 闭式不符"
                );
                let row = matrix_row(g, per_side, per_cell, base);
                assert_eq!(row.cell, cell, "per_side={per_side} g={g}: cell 应为 g/K");
                assert_eq!(
                    row.seed,
                    base + (cell * per_cell + k) as u64,
                    "per_side={per_side} g={g}: 局清单 seed 与闭式不符"
                );
                assert_eq!(row.per_side, per_side);
                assert_eq!(row.lane_m, 50);
            }
        }
        for g in 0..3 {
            let (cell, seed) = sampling_params(g);
            assert_eq!(
                seed,
                SEED_SAMPLING_BASE + g as u64 * SAMPLING_SEED_STRIDE,
                "sampling g={g}: seed 闭式不符"
            );
            assert_eq!(cell, g % CELLS, "sampling g={g}: cell 应为 g%36");
        }
    }

    /// T009 W4：resolved 语义两构型（断言以实测为准；派工单 §5 参考值仅供人眼
    /// 核对）。① 必截断：mirror (Shieldman,2)、lane 1000m、cap 1800——不接敌
    /// （t_e ≈ (1000−2.0)/0.10 = 9980 > 1800）→ 上限 hp-sum 收束：镜像同和 →
    /// Draw、end_tick=1800、双方全存活 → resolved=false。② 必接敌：mirror
    /// (Militia,2)、lane 50m——t_e ≈ (50−1.6)/0.18 ≈ 269 ≪ 1800，全歼远早于
    /// 上限 → 一方零存活 → resolved=true。
    #[test]
    fn w4_resolved_semantics_truncation_and_engagement() {
        // ① 必截断。
        let mut a = World::deploy_versus(
            4242,
            &[(UnitKind::Shieldman, 2)],
            &[(UnitKind::Shieldman, 2)],
            LANE_LEN_Q32,
        );
        let oa = a.run_battle(TICK_CAP_REDUCED);
        assert_eq!(oa.winner, Winner::Draw, "① 镜像同 hp-sum 上限收束应为 Draw");
        assert_eq!(oa.end_tick, TICK_CAP_REDUCED, "① 应顶到 cap 1800");
        assert_eq!(oa.alive_red, 2, "① 双方全存活（不接敌）");
        assert_eq!(oa.alive_blue, 2, "① 双方全存活（不接敌）");
        assert!(
            !is_resolved(oa.alive_red, oa.alive_blue),
            "① resolved 必须 = false"
        );
        // ② 必接敌（参考链：t_e≈269、首杀 ≈269+8×20=429（派工单速算 409 系
        // (ceil(50/6)−1)*20 应为 160 非 140——以实测为准，披露见证据档）。
        let mut b = World::deploy_versus(
            4242,
            &[(UnitKind::Militia, 2)],
            &[(UnitKind::Militia, 2)],
            LANE_SCALED_Q32,
        );
        let ob = b.run_battle(TICK_CAP_REDUCED);
        assert!(
            is_resolved(ob.alive_red, ob.alive_blue),
            "② 必接敌构型应一方全歼（实测 alive {}/{} @ end_tick {}）",
            ob.alive_red,
            ob.alive_blue,
            ob.end_tick
        );
        assert!(
            ob.end_tick < TICK_CAP_REDUCED,
            "② 全歼应早于 cap（实测 end_tick={}）",
            ob.end_tick
        );
        assert_ne!(ob.winner, Winner::Draw, "② 全灭收束不应为 Draw");
    }
}
