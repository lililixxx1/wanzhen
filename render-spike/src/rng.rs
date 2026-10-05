//! 本地确定性伪随机源：SplitMix64（步进式）。
//!
//! T010 场景只需「同 seed 可复现」；不依赖 sim（任务卡红线），常数与步进形态按
//! 派工单 §3 给定：`state = seed`，每取数 `state += 0x9E3779B97F4A7C15` 后混合。
//! 黄金序列按 PIT-M-002 纪律双盲对拍后固化：python 独立重实现见
//! `docs/evidence/t010/rng_double_blind.py`，对拍档见 `docs/evidence/t010/runs/w1_*`。

/// SplitMix64 步进发生器（u64 状态）。
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// 以 `seed` 为初始状态。
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// 取下一个 64 位输出（先步进、后两轮 xor-shift-multiply 混合）。
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W1：seed=42 前 8 输出黄金序列。
    /// 固化流程（双盲，PIT-M-002）：先以占位期望值运行本测试取得实测输出
    /// （runs/w1_rust_first8/attempt_failing），与 python 独立重实现输出
    /// （runs/w1_python_first8）逐位比对 8/8 一致后，才把期望值回填此处并转绿
    /// （runs/w1_rust_first8/comparison_note.txt + runs/r0_gate3_test_rs）。
    /// 禁凭记忆引用参考向量——占位值禁止直接凑数。
    #[test]
    fn golden_sequence_seed42_first8() {
        let mut rng = SplitMix64::new(42);
        let mut got = [0u64; 8];
        for slot in &mut got {
            *slot = rng.next_u64();
        }
        const GOLDEN_SEED42_FIRST8: [u64; 8] = [
            13679457532755275413,
            2949826092126892291,
            5139283748462763858,
            6349198060258255764,
            701532786141963250,
            16015981125662989062,
            4028864712777624925,
            14769051326987775908,
        ];
        assert_eq!(got, GOLDEN_SEED42_FIRST8, "W1 黄金值不一致：got={got:?}");
    }

    /// 步进语义自证：同一 seed 两次实例序列一致（前缀可复现）。
    #[test]
    fn same_seed_same_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
