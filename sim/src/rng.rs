//! 确定性 RNG：SplitMix64（仅作 seed 扩展）+ Xoshiro256StarStar（主发生器）。
//!
//! 确定性纪律：零第三方依赖、零系统熵源、无浮点、无挂钟；同 seed 必产生同序列。
//! 算法为 Blackman & Vigna 公开参考实现的手写移植；黄金序列按 PIT-M-002 教训
//! 以实测固化（见 docs/evidence/t002/rng-golden.txt），不凭记忆引用参考向量。

/// SplitMix64——仅用于把 u64 seed 扩展为 Xoshiro256StarStar 的 4 个初始状态字。
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Xoshiro256\*\* 主发生器（模拟态唯一随机源入口）。
pub struct Xoshiro256StarStar {
    s: [u64; 4],
}

impl Xoshiro256StarStar {
    /// 从 u64 seed 经 SplitMix64 连续取 4 个输出作为初始状态（标准扩展法）。
    pub fn from_seed(seed: u64) -> Self {
        let mut sm = SplitMix64::new(seed);
        Self {
            s: [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()],
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// 当前内部状态（供状态哈希折叠；模拟态可见路径）。
    pub fn state_words(&self) -> [u64; 4] {
        self.s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_sequence_seed42_first8() {
        // 黄金值由实测固化（PIT-M-002 纪律）：占位 0 运行取得实测输出后回填；
        // 并经独立 python 重实现对拍一致（docs/evidence/t002/rng-golden.txt）。
        const GOLDEN_SEED42_FIRST8: [u64; 8] = [
            1546998764402558742,
            6990951692964543102,
            12544586762248559009,
            17057574109182124193,
            18295552978065317476,
            14199186830065750584,
            13267978908934200754,
            15679888225317814407,
        ];
        let mut rng = Xoshiro256StarStar::from_seed(42);
        let mut got = [0u64; 8];
        for slot in &mut got {
            *slot = rng.next_u64();
        }
        assert_eq!(
            got, GOLDEN_SEED42_FIRST8,
            "golden sequence mismatch (seed 42, first 8 next_u64)"
        );
    }

    #[test]
    fn stats_high4_bits_16_buckets_160k() {
        // 固定 seed → 结果确定，不 flaky：每桶期望 10,000，阈值 ±5%。
        const N: u64 = 160_000;
        let mut rng = Xoshiro256StarStar::from_seed(42);
        let mut buckets = [0u64; 16];
        for _ in 0..N {
            let x = rng.next_u64();
            buckets[(x >> 60) as usize] += 1;
        }
        let expected = N / 16; // 10_000
        let tol = expected / 20; // 5% = 500
        for (i, &b) in buckets.iter().enumerate() {
            let diff = b.abs_diff(expected);
            assert!(diff <= tol, "bucket {i}: {b} (expected {expected} +/-5% = +/-{tol})");
        }
    }

    #[test]
    fn stats_bit32_ratio_50pct() {
        // 固定 seed → 结果确定：bit 32 上 1 的比例 50% ± 0.5%（[79_200, 80_800]）。
        const N: u64 = 160_000;
        let mut rng = Xoshiro256StarStar::from_seed(42);
        let mut ones = 0u64;
        for _ in 0..N {
            let x = rng.next_u64();
            ones += (x >> 32) & 1;
        }
        assert!(
            (79_200..=80_800).contains(&ones),
            "bit32 ones = {ones} (expected 80_000 +/-0.5% = [79_200, 80_800])"
        );
    }
}
