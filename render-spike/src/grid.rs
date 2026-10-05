//! 网格布阵（确定性，派工单 §3）。
//!
//! 单位 i：`gx = i % 100`、`gz = i / 100`；
//! `x = (gx - 49.5) * 1.6 + jx`、`z = (gz - 49.5) * 1.6 + jz`。
//!
//! 几何算式（入档 docs/evidence/t010/README.md）：
//! - 格距 1.6 m、jitter 各 ±0.25 m（严格开区间）⇒ 相邻中心距下限 = 1.6 - 0.5 = 1.1 m
//!   > 胶囊直径 2×0.5 = 1.0 m ⇒ 圆柱足迹不重叠；
//! - 场地跨度 99 × 1.6 = 158.4 m ∈ ±79.2 m；
//! - 前缀性质：单位 i 位置只依赖 i 与 seed ⇒ 四档同 seed 为前缀布局（D6）。

/// 网格列数（行宽）。
pub const GRID_COLS: usize = 100;
/// 格距（米）。
pub const PITCH_M: f32 = 1.6;
/// jitter 半幅（米）：jx/jz ∈ (-0.25, +0.25)。
pub const JITTER_HALF_M: f32 = 0.25;
/// 胶囊半径（米）。
pub const CAPSULE_RADIUS_M: f32 = 0.5;
/// 胶囊圆柱段全长（米）：`Capsule3d::new(radius, length)` 的 length（half_length = 0.5）。
pub const CAPSULE_LENGTH_M: f32 = 1.0;
/// 落地高度：总高 = length + 2×radius = 2.0 ⇒ 中心 y = 1.0（底部贴地）。
pub const REST_Y_M: f32 = CAPSULE_RADIUS_M + CAPSULE_LENGTH_M / 2.0;

/// 由 SplitMix64 一次输出映射到 (-0.25, +0.25) 的抖动值（严格开区间）。
///
/// 映射：取高 23 位 `v23 ∈ [0, 2^23)`，`unit = (v23 + 0.5) / 2^23 ∈ (0, 1)`（严格；
/// 23 位内 `v23 + 0.5` 可被 f32 精确表示），`jitter = (unit - 0.5) * 2 * JITTER_HALF_M`。
/// 极端输入 v=0 / u64::MAX 亦不越界（见单测）。
pub fn jitter_from_u64(v: u64) -> f32 {
    let v23 = (v >> 41) as f32; // 64 - 41 = 23 位
    let unit = (v23 + 0.5) / 8_388_608.0; // 8_388_608 = 2^23
    (unit - 0.5) * (2.0 * JITTER_HALF_M)
}

/// 单位 i 的平面坐标 (x, z)；y 恒为 [`REST_Y_M`]（落地）。
pub fn unit_xz(i: usize, jx: f32, jz: f32) -> (f32, f32) {
    let gx = i % GRID_COLS;
    let gz = i / GRID_COLS;
    let x = (gx as f32 - 49.5) * PITCH_M + jx;
    let z = (gz as f32 - 49.5) * PITCH_M + jz;
    (x, z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::SplitMix64;

    /// W2：i=0 → x 基值 -79.2（±jitter），即场地最小角。
    #[test]
    fn w2_unit_zero_at_min_corner() {
        let mut rng = SplitMix64::new(42);
        let jx = jitter_from_u64(rng.next_u64());
        let jz = jitter_from_u64(rng.next_u64());
        let (x, z) = unit_xz(0, jx, jz);
        assert!((x - (-79.2)).abs() < JITTER_HALF_M, "x={x}");
        assert!((z - (-79.2)).abs() < JITTER_HALF_M, "z={z}");
    }

    /// W2：i=99 → x 基值 +79.2 边界（第 0 行行尾）；i=100 → gz=1 行首（x 回 -79.2，z=-77.6）。
    #[test]
    fn w2_edge_99_and_row_start_100() {
        let mut rng = SplitMix64::new(42);
        let jx = jitter_from_u64(rng.next_u64());
        let jz = jitter_from_u64(rng.next_u64());
        let (x99, z99) = unit_xz(99, jx, jz);
        assert!((x99 - 79.2).abs() < JITTER_HALF_M, "x99={x99}");
        assert!((z99 - (-79.2)).abs() < JITTER_HALF_M, "z99={z99}");
        let (x100, z100) = unit_xz(100, jx, jz);
        assert!((x100 - (-79.2)).abs() < JITTER_HALF_M, "x100={x100}");
        assert!((z100 - (-77.6)).abs() < JITTER_HALF_M, "z100={z100}");
    }

    /// W2：jitter 严格开区间——1200 对（2400 样本）逐点断言 + 极端输入。
    #[test]
    fn w2_jitter_strictly_inside_open_interval() {
        let mut rng = SplitMix64::new(7);
        for _ in 0..1200 {
            for v in [rng.next_u64(), rng.next_u64()] {
                let j = jitter_from_u64(v);
                assert!(
                    j > -JITTER_HALF_M && j < JITTER_HALF_M,
                    "jitter 越界: {j} (v={v})"
                );
            }
        }
        // 极端输入：0 → 略大于 -0.25；u64::MAX → 略小于 +0.25。
        let lo = jitter_from_u64(0);
        let hi = jitter_from_u64(u64::MAX);
        assert!(lo > -JITTER_HALF_M && lo < 0.0, "lo={lo}");
        assert!(hi < JITTER_HALF_M && hi > 0.0, "hi={hi}");
    }

    /// W2：不重叠算式——相邻格（含对角）中心距下限 > 胶囊直径。
    #[test]
    fn w2_min_spacing_exceeds_capsule_diameter() {
        let min_axis_gap = PITCH_M - 2.0 * JITTER_HALF_M; // 1.6 - 0.5 = 1.1
        assert!(min_axis_gap > 2.0 * CAPSULE_RADIUS_M, "min_axis_gap={min_axis_gap}");
        let min_diag_gap = (min_axis_gap * min_axis_gap * 2.0).sqrt();
        assert!(min_diag_gap > 2.0 * CAPSULE_RADIUS_M, "min_diag_gap={min_diag_gap}");
    }
}
