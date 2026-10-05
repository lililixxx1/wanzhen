//! 指标公式（与 `docs/evidence/t010/summarize.py` 同式实现；仅用于单测 W4 的
//! 跨语言对齐——正式判定行由 summarize.py 生成，禁用本模块做人工计算）。
//!
//! 预注册公式（派工单 §5）：
//! - `avg_fps = N / (Σ delta_ns / 1e9)`
//! - `low_fps(p)`：k = ⌈N·p⌉（整数算式，k ≥ 1），取最慢 k 帧 delta 均值 → `1e9 / mean`
//! - `percentile(p)`：升序第 ⌈N·p⌉ 个（nearest-rank；整数算式）
//!
//! 整数算式避免浮点 ceil 误差：k(1/100) = (N + 99) / 100；k(1/1000) = (N + 999) / 1000；
//! rank(p%) = (N·p + 99) / 100。

/// 平均帧率（fps）。
pub fn avg_fps(deltas_ns: &[u64]) -> f64 {
    let total: u128 = deltas_ns.iter().map(|&d| d as u128).sum();
    deltas_ns.len() as f64 / (total as f64 / 1e9)
}

/// 低帧率（`num/den` 分位，如 1% → `(1, 100)`）：最慢 k 帧均值换算 fps。
pub fn low_fps(deltas_ns: &[u64], num: usize, den: usize) -> f64 {
    let n = deltas_ns.len();
    assert!(num > 0 && den > 0, "低帧率分位须为正");
    let k = ((n * num + den - 1) / den).max(1);
    let mut sorted = deltas_ns.to_vec();
    sorted.sort_unstable();
    let mean = sorted[n - k..].iter().map(|&d| d as f64).sum::<f64>() / k as f64;
    1e9 / mean
}

/// 帧时间分位（`num` 为百分位，如 p50 → `50`）：nearest-rank 升序第 ⌈N·num/100⌉ 个。
pub fn percentile(deltas_ns: &[u64], num: usize) -> u64 {
    let n = deltas_ns.len();
    let rank = ((n * num + 99) / 100).clamp(1, n);
    let mut sorted = deltas_ns.to_vec();
    sorted.sort_unstable();
    sorted[rank - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4：构造样例 `[8,8,8,16,32,64]` ms（派工单 §7 给定）。
    ///
    /// 手算（算式入注，以代码为准）：
    /// - Σ = 136 ms ⇒ avg_fps = 6 / 0.136 = 44.11764705882353…
    /// - 1% low：k = ⌈6/100⌉ = 1（整数式 (6+99)/100 = 1）⇒ mean 最慢 1 帧 = 64 ms
    ///   ⇒ 1e9 / 64_000_000 = 15.625
    /// - 0.1% low：k = ⌈6/1000⌉ = 1 ⇒ 15.625
    /// - p50：rank = (6×50+99)/100 = 3 ⇒ 8 ms；p95/p99：rank = 6 ⇒ 64 ms；max = 64 ms
    #[test]
    fn w4_sample_metrics_match_hand_calc() {
        const MS: u64 = 1_000_000;
        let sample: [u64; 6] = [8 * MS, 8 * MS, 8 * MS, 16 * MS, 32 * MS, 64 * MS];
        let avg = avg_fps(&sample);
        assert!((avg - 44.117647058823529).abs() < 1e-9, "avg={avg}");
        assert_eq!(low_fps(&sample, 1, 100), 15.625);
        assert_eq!(low_fps(&sample, 1, 1000), 15.625);
        assert_eq!(percentile(&sample, 50), 8 * MS);
        assert_eq!(percentile(&sample, 95), 64 * MS);
        assert_eq!(percentile(&sample, 99), 64 * MS);
        assert_eq!(*sample.iter().max().unwrap(), 64 * MS);
    }
}
