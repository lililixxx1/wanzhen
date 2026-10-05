//! 意图阶段排序序快路径（T015/WP-A）：`(x, 单位索引)` 全序建序 + O(1) 邻域 /
//! O(n) 扫掠查询，替代 [`crate::world`] 两个 O(N²) 意图阶段（move 索前方 /
//! combat 索最近敌）在生产域的实现。模拟行为逐位不变（黄金锚红线）。
//!
//! ## 全序与确定性
//!
//! 比较器 `(x[u], u)` 字典序——单位索引互异 ⇒ 无相等元素 ⇒ **全序**。因此：
//! - 排序结果唯一（与排序算法稳定性、片界、线程数均无关）；
//! - 并行建序（各片局部排序 + 自底向上两两**串行**归并）与串行
//!   `sort_unstable_by` 结果逐位一致（单测 `build_matches_serial_across_thread_tiers`）。
//!
//! ## eligibility 与回退（调用方 = world.rs 两处意图构造段）
//!
//! 「x 沿序严格递增（无平局）**且** 快照全存活」才允许走快路径；否则调用方逐字
//! 回退 T006 朴素 O(N²) 分片（`move_intent_chunk` / `combat_intent_chunk`）。
//! 退化域（`World::new` 全体重合 x=0、含同 x 手构局、含墓碑快照）一律回退，
//! 行为与优化前逐位一致。
//!
//! ## 等价性（D4 / D5，对 world.rs 朴素实现逐分支枚举）
//!
//! - **move（D4）**：x 互异 ⇒ 单位 i 运动方向上最近存活者 = x 序紧邻
//!   （红 dir=+1 取右邻 `order[p+1]`、蓝 dir=−1 取左邻 `order[p−1]`；紧邻不存在
//!   = 前方无存活者 → None）。朴素实现 `(x[j]-x[i])*dir > 0` 取最小值在互异域内
//!   该最小值唯一（同距离同侧需同 x，被互异性排除）⇒ 无平局分支，两法同值。
//! - **combat（D5）**：x 互异 ⇒ 单位 i 的最近敌候选只剩左右紧邻敌各一
//!   （prev/next 同侧扫掠值；同侧更远者距离严格更大；同距离同侧敌需同 x，被
//!   互异性排除）。`dL < dR` 取左、`dR < dL` 取右、相等取**单位索引较小者**
//!   ——与朴素实现「索引升序扫描 + 严格小于才更新 ⇒ (距离, 索引) 字典序
//!   argmin、平局取最小索引」逐位一致。双侧皆无候选 → None。in_range 同式
//!   同常量（闭区间 `dist <= r_i + r_t + MELEE_MARGIN_Q32`，与 world.rs 意图段
//!   同款；spec/UnitKind 判别自 [`crate::units`]）。
//!
//! 确定性纪律：纯整数运算、无浮点 / 无超越函数 / 无 HashMap；并行路径闭包只
//! 捕获 owned 克隆（`Arc`）满足 'static、不 panic（[`crate::pool`] 契约）；
//! 扫掠串行、per-i 查询两档共享同一纯函数——结果与线程数无关。

use std::sync::Arc;

use crate::pool::ThreadPool;
use crate::units::{spec, UnitKind, MELEE_MARGIN_Q32};

/// side 判别值（world.rs `Side` repr(u8) 显式锁定：Red=0 / Blue=1；越界不可达）。
const SIDE_RED: u8 = 0;
/// side 判别值（同上）。
const SIDE_BLUE: u8 = 1;

/// 判别 u8 → UnitKind（判别值锁定见 units.rs `UnitKind` 显式判别 0..=5；
/// 与 world.rs 私有 `kind_of` 同表镜像——本模块不得反向依赖 world 私有项）。
fn kind_of(d: u8) -> UnitKind {
    match d {
        0 => UnitKind::Shieldman,
        1 => UnitKind::HeavyKnight,
        2 => UnitKind::Pikeman,
        3 => UnitKind::Swordsman,
        4 => UnitKind::Archer,
        5 => UnitKind::Militia,
        _ => unreachable!("UnitKind discriminant out of range"),
    }
}

/// (x, 单位索引) 升序全序（`order`）+ 逆映射（`pos`）。
///
/// - `order[p]` = x 序第 p 位的单位索引（比较器 `(x[u], u)` 字典序）；
/// - `pos[u]` = 单位 u 在 `order` 中的位置（与 `order` 互逆）。
pub(crate) struct SortedOrder {
    order: Vec<usize>,
    pos: Vec<usize>,
}

impl SortedOrder {
    /// 建序：O(n log n)。
    ///
    /// - `pool = None`：单次 `sort_unstable_by`（比较器全序 ⇒ 结果唯一）。
    /// - `pool = Some(p)`：[`crate::pool::ThreadPool::map_chunks`] 各片
    ///   （片界 = `chunk_range(n, threads, k)`）局部排序后拼接，再**串行**
    ///   自底向上两两归并成全局有序——比较器全序 ⇒ 归并结果唯一，与线程数无关。
    ///
    /// n=0/1 平凡成立（空序 / 单元素序）。
    pub(crate) fn build(x: &[i64], pool: Option<&ThreadPool>) -> SortedOrder {
        let n = x.len();
        let order: Vec<usize> = match pool {
            None => {
                let mut order: Vec<usize> = (0..n).collect();
                order.sort_unstable_by(|&a, &b| (x[a], a).cmp(&(x[b], b)));
                order
            }
            Some(p) => {
                let t = p.threads();
                // 闭包 'static 契约（pool.rs）：x 克隆进 Arc（owned），归并仍在
                // 本函数体内用原引用串行完成。
                let x_arc: Arc<Vec<i64>> = Arc::new(x.to_vec());
                let mut order: Vec<usize> = p.map_chunks(n, move |s, e| {
                    let x = &x_arc[..];
                    let mut piece: Vec<usize> = (s..e).collect();
                    piece.sort_unstable_by(|&a, &b| (x[a], a).cmp(&(x[b], b)));
                    piece
                });
                // 片界（与 map_chunks 内部 chunk_range 同式）：各片内部有序、
                // 片与片在索引空间连续拼接。自底向上两两归并（串行）。
                let mut starts: Vec<usize> = (0..=t).map(|k| k * n / t).collect();
                starts.dedup(); // 去掉空片（n < t 时存在）；n=0 时 starts=[0]。
                let mut buf: Vec<usize> = Vec::with_capacity(n);
                while starts.len() > 2 {
                    let r = starts.len() - 1; // 当前运行段数
                    let mut merged: Vec<usize> = Vec::with_capacity((r + 1) / 2 + 1);
                    let mut j = 0usize;
                    while j + 1 < r {
                        let (a0, mid, a2) = (starts[j], starts[j + 1], starts[j + 2]);
                        merge_runs(&mut order, a0, mid, a2, &mut buf, x);
                        merged.push(a0);
                        j += 2;
                    }
                    // 奇数残留段 [starts[j], n) 原样携带（其 start = 上一合并段
                    // 终点）；段边界表末项恒为 n。
                    if j < r {
                        merged.push(starts[j]);
                    }
                    merged.push(n);
                    starts = merged;
                }
                order
            }
        };
        // 逆映射一遍 O(n)。
        let mut pos: Vec<usize> = vec![0; n];
        for (p, &u) in order.iter().enumerate() {
            pos[u] = p;
        }
        SortedOrder { order, pos }
    }

    /// x 沿序严格递增（一遍检查 `x[order[p]] < x[order[p+1]]` 全体成立）。
    /// n=0/1 恒 true（无可比较相邻对）。
    pub(crate) fn x_strictly_increasing(&self, x: &[i64]) -> bool {
        self.order.windows(2).all(|w| x[w[0]] < x[w[1]])
    }

    /// 只读视图（world.rs differential 单测对拍建序结果用；字段保持私有）。
    #[cfg(test)]
    pub(crate) fn order(&self) -> &[usize] {
        &self.order
    }
}

/// 归并 `order[start..mid]` 与 `order[mid..end]` 两个有序段（原地，buf 暂存）。
/// 比较器全序（无相等元素）⇒ 结果唯一，与左右取等策略无关。
fn merge_runs(
    order: &mut [usize],
    start: usize,
    mid: usize,
    end: usize,
    buf: &mut Vec<usize>,
    x: &[i64],
) {
    buf.clear();
    let (mut p, mut q) = (start, mid);
    while p < mid && q < end {
        if (x[order[p]], order[p]) <= (x[order[q]], order[q]) {
            buf.push(order[p]);
            p += 1;
        } else {
            buf.push(order[q]);
            q += 1;
        }
    }
    buf.extend_from_slice(&order[p..mid]);
    buf.extend_from_slice(&order[q..end]);
    order[start..end].copy_from_slice(buf);
}

/// move 快路径并行上下文（owned 克隆，`Arc` 共享满足 'static）。
struct MoveFastCtx {
    x: Vec<i64>,
    side: Vec<u8>,
    order: Vec<usize>,
    pos: Vec<usize>,
}

/// combat 快路径并行上下文（owned 克隆 + 扫掠产物与 kind；per-i 查询只用
/// pos 与扫掠数组，order 不入上下文）。
struct CombatFastCtx {
    x: Vec<i64>,
    side: Vec<u8>,
    kind: Vec<u8>,
    pos: Vec<usize>,
    prev_blue: Vec<Option<usize>>,
    prev_red: Vec<Option<usize>>,
    next_blue: Vec<Option<usize>>,
    next_red: Vec<Option<usize>>,
}

/// per-i 纯函数（两执行档共享）：单位 i 的运动方向前方者（序紧邻，None = 前方
/// 无存活者）。前置：eligibility 已由调用方保证（x 沿序严格递增 + 全存活）。
fn move_front_at(i: usize, x: &[i64], side: &[u8], order: &[usize], pos: &[usize]) -> Option<usize> {
    let n = x.len();
    let p = pos[i];
    if side[i] == SIDE_RED {
        // 红 dir=+1：前方 = 更大 x 侧 = 序右邻。
        if p + 1 < n {
            Some(order[p + 1])
        } else {
            None
        }
    } else {
        // 蓝 dir=-1：前方 = 更小 x 侧 = 序左邻。
        if p > 0 {
            Some(order[p - 1])
        } else {
            None
        }
    }
}

/// per-i 纯函数（两执行档共享）：单位 i 的最近敌（单位索引）+ in_range 闭区间
/// 预判定。前置：eligibility 已由调用方保证。等价性推导见模块注释 D5。
#[allow(clippy::too_many_arguments)]
fn combat_target_at(
    i: usize,
    x: &[i64],
    side: &[u8],
    kind: &[u8],
    pos: &[usize],
    prev_blue: &[Option<usize>],
    prev_red: &[Option<usize>],
    next_blue: &[Option<usize>],
    next_red: &[Option<usize>],
) -> (Option<usize>, bool) {
    let p = pos[i];
    // 左右候选 = 敌方（异侧）紧邻：本侧红 → 敌蓝取 prev_blue/next_blue；反之亦然。
    let (left, right) = if side[i] == SIDE_BLUE {
        (prev_red[p], next_red[p])
    } else {
        (prev_blue[p], next_blue[p])
    };
    // x 互异 ⇒ dL / dR 均为正（左候选 x 更小、右候选 x 更大）。
    let d_l = left.map(|l| x[i] - x[l]);
    let d_r = right.map(|r| x[r] - x[i]);
    let target = match (d_l, d_r) {
        (None, None) => None,
        (Some(_), None) => left,
        (None, Some(_)) => right,
        (Some(dl), Some(dr)) => {
            if dl < dr {
                left
            } else if dr < dl {
                right
            } else if left.unwrap() < right.unwrap() {
                // 真等距：取单位索引较小者（= 朴素实现「索引升序扫描 + 严格
                // 小于才更新」的平局决胜结果）。
                left
            } else {
                right
            }
        }
    };
    match target {
        None => (None, false),
        Some(t) => {
            let dist = (x[t] - x[i]).abs();
            // 闭区间，与 world.rs 意图段同式同常量（溢出安全：距离与半径和均
            // 远不及 i64 上界，world.rs 同款留痕）。
            let radius_sum = spec(kind_of(kind[i])).radius_q32 + spec(kind_of(kind[t])).radius_q32;
            (Some(t), dist <= radius_sum + MELEE_MARGIN_Q32)
        }
    }
}

/// move 意图快路径（前置：调用方已保证 eligibility——x 沿序严格递增 + 全存活）。
/// 输出按单位索引 i 对齐：`out[i]` = i 的前方者（None = 前方无存活者）。
///
/// pool=Some：x/side/order/pos 克隆进 `Arc` 后 [`crate::pool::ThreadPool::map_chunks`]
/// 并行；pool=None：主线程直循环。两档共享同一 per-i 纯函数 [`move_front_at`]。
pub(crate) fn move_fronts_fast(
    x: &[i64],
    side: &[u8],
    so: &SortedOrder,
    pool: Option<&ThreadPool>,
) -> Vec<Option<usize>> {
    let n = x.len();
    match pool {
        None => (0..n)
            .map(|i| move_front_at(i, x, side, &so.order, &so.pos))
            .collect(),
        Some(p) => {
            let ctx = Arc::new(MoveFastCtx {
                x: x.to_vec(),
                side: side.to_vec(),
                order: so.order.clone(),
                pos: so.pos.clone(),
            });
            p.map_chunks(n, move |start, end| {
                (start..end)
                    .map(|i| move_front_at(i, &ctx.x, &ctx.side, &ctx.order, &ctx.pos))
                    .collect::<Vec<Option<usize>>>()
            })
        }
    }
}

/// combat 意图快路径（前置同上）。输出按单位索引 i 对齐：
/// `out[i]` = (最近敌单位索引, in_range)；无敌方 → (None, false)。
///
/// 先两遍 O(n) 串行扫掠得 per-位置最近异侧邻居（L→R 得严格左侧、R→L 得严格
/// 右侧；Option 存**单位索引**；写 out[p] 用进入 p 前的携带值，写完再按
/// `order[p]` 的 side 更新携带值），再 per-i 查询（pool=Some 经 `Arc` +
/// map_chunks 并行 / None 直循环，两档共享同一纯函数 [`combat_target_at`]）。
pub(crate) fn combat_targets_fast(
    x: &[i64],
    side: &[u8],
    kind: &[u8],
    so: &SortedOrder,
    pool: Option<&ThreadPool>,
) -> Vec<(Option<usize>, bool)> {
    let n = x.len();
    // L→R 一遍：prev_blue[p] / prev_red[p] = 位置 p 严格左侧最近的蓝 / 红单位。
    let mut prev_blue: Vec<Option<usize>> = vec![None; n];
    let mut prev_red: Vec<Option<usize>> = vec![None; n];
    let (mut carry_blue, mut carry_red) = (None, None);
    for p in 0..n {
        prev_blue[p] = carry_blue;
        prev_red[p] = carry_red;
        let u = so.order[p];
        if side[u] == SIDE_BLUE {
            carry_blue = Some(u);
        } else {
            carry_red = Some(u);
        }
    }
    // R→L 一遍：next_blue[p] / next_red[p] = 位置 p 严格右侧最近的蓝 / 红单位。
    let mut next_blue: Vec<Option<usize>> = vec![None; n];
    let mut next_red: Vec<Option<usize>> = vec![None; n];
    let (mut carry_blue, mut carry_red) = (None, None);
    for p in (0..n).rev() {
        next_blue[p] = carry_blue;
        next_red[p] = carry_red;
        let u = so.order[p];
        if side[u] == SIDE_BLUE {
            carry_blue = Some(u);
        } else {
            carry_red = Some(u);
        }
    }
    match pool {
        None => (0..n)
            .map(|i| {
                combat_target_at(
                    i, x, side, kind, &so.pos, &prev_blue, &prev_red, &next_blue, &next_red,
                )
            })
            .collect(),
        Some(p) => {
            let ctx = Arc::new(CombatFastCtx {
                x: x.to_vec(),
                side: side.to_vec(),
                kind: kind.to_vec(),
                pos: so.pos.clone(),
                prev_blue,
                prev_red,
                next_blue,
                next_red,
            });
            p.map_chunks(n, move |start, end| {
                (start..end)
                    .map(|i| {
                        combat_target_at(
                            i,
                            &ctx.x,
                            &ctx.side,
                            &ctx.kind,
                            &ctx.pos,
                            &ctx.prev_blue,
                            &ctx.prev_red,
                            &ctx.next_blue,
                            &ctx.next_red,
                        )
                    })
                    .collect::<Vec<(Option<usize>, bool)>>()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 快路径基础（S5-1）：order 按 (x, idx) 全序、pos 互逆；
    /// `x_strictly_increasing` 对互异序列 = true / 含相等 = false / n=0,1 = true。
    #[test]
    fn sorted_order_basic_total_order_and_inverse() {
        // 手构例：x = [10, 5, 10, 0]——idx0/idx2 同 x=10，由索引决胜（idx0 在前）。
        let x = [10i64, 5, 10, 0];
        let so = SortedOrder::build(&x, None);
        assert_eq!(so.order, vec![3, 1, 0, 2], "order 应按 (x, idx) 字典序");
        // pos 互逆：pos[order[p]] == p 全体成立。
        for (p, &u) in so.order.iter().enumerate() {
            assert_eq!(so.pos[u], p, "pos[order[{p}]] 应为 {p}");
        }
        // 互异序列 → true。
        let xd = [3i64, 1, 4, 9, 5];
        let sod = SortedOrder::build(&xd, None);
        assert!(sod.x_strictly_increasing(&xd), "互异序列沿序必严格递增");
        // 含相等 → false（含同 x 手构对）。
        assert!(!so.x_strictly_increasing(&x), "重合 x 对必须破坏严格递增");
        // n=0 / n=1 → true（平凡）。
        let s0 = SortedOrder::build(&[], None);
        assert!(s0.x_strictly_increasing(&[]), "n=0 平凡成立");
        let s1 = SortedOrder::build(&[7], None);
        assert!(s1.x_strictly_increasing(&[7]), "n=1 平凡成立");
    }

    /// 建序确定性（S5-1 补强）：并行建序（threads=3/7，含空片 n<t 路径）与串行
    /// `sort_unstable_by` 在互异与含重合 x 两种域内逐位一致（全序 ⇒ 唯一结果）。
    #[test]
    fn build_matches_serial_across_thread_tiers() {
        let distinct: Vec<i64> = (0..37usize)
            .map(|i| ((i * 13 + 5) % 41) as i64 * ONE_TEST_METER)
            .collect();
        // 人为保证互异：(i*13+5)%41 对 i<37 无重复（13 与 41 互素，前 37 项不回绕）。
        let mut sorted = distinct.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 37, "测试自检：构造应互异");
        let with_dup: Vec<i64> = vec![5 * ONE_TEST_METER, 1, 5 * ONE_TEST_METER, 0, 5 * ONE_TEST_METER];
        for (name, x) in [("distinct", &distinct), ("with_dup", &with_dup)] {
            let serial = SortedOrder::build(x, None);
            for t in [3usize, 7] {
                let pool = ThreadPool::new(t);
                let parallel = SortedOrder::build(x, Some(&pool));
                assert_eq!(
                    parallel.order, serial.order,
                    "name={name} threads={t}: 并行建序应与串行逐位一致"
                );
            }
        }
    }

    /// move / combat 快路径手构小例（S5-1 补强）：序紧邻语义与候选侧选择。
    #[test]
    fn fast_queries_small_handbuilt_examples() {
        // x = [0, 10, 20] m；side = [红, 蓝, 红]。
        let m = ONE_TEST_METER;
        let x = [0i64, 10 * m, 20 * m];
        let side = [SIDE_RED, SIDE_BLUE, SIDE_RED];
        let so = SortedOrder::build(&x, None);
        assert!(so.x_strictly_increasing(&x));
        // move：红0 前方 = 右邻 idx1；蓝1 前方 = 左邻 idx0；红2 右邻不存在 → None。
        let fronts = move_fronts_fast(&x, &side, &so, None);
        assert_eq!(fronts, vec![Some(1), Some(0), None]);
        // combat：红0 唯一敌 = 蓝1；蓝1 最近敌 = 红0（0 距 10 < 红2 的 20）；红2 最近敌 = 蓝1。
        // 民兵半径 0.4m×2 + 余量 0.2 = 1.0m：0↔10m 距离 10m 出射程 → in_range=false。
        let kind = [5u8, 5, 5]; // 全民兵
        let targets = combat_targets_fast(&x, &side, &kind, &so, None);
        assert_eq!(
            targets,
            vec![(Some(1), false), (Some(0), false), (Some(1), false)]
        );
    }
}

/// 测试辅助常量：Q32.32 的 1 米（避免对 units 常量的额外引依赖，值同源 2^32）。
#[cfg(test)]
const ONE_TEST_METER: i64 = 1i64 << 32;
