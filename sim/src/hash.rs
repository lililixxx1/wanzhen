//! FNV-1a 64-bit 状态哈希（自实现，零依赖；逐字节固定序折叠）。

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64 累加器：调用方按固定顺序写字段，`finish` 取终值。
pub struct Fnv1a {
    hash: u64,
}

impl Fnv1a {
    pub fn new() -> Self {
        Self {
            hash: FNV_OFFSET_BASIS,
        }
    }

    /// 折叠一个 u64 字段：小端 8 字节序（固定字节序，跨平台一致）。
    pub fn write_u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    /// 折叠一个 u32 字段：小端 4 字节序（与 [`Fnv1a::write_u64`] 同风格；
    /// T004 起单位 hp / cd 各以 4 字节进状态哈希，主会话 D7 定稿）。
    pub fn write_u32(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_u8(&mut self, b: u8) {
        self.hash ^= u64::from(b);
        self.hash = self.hash.wrapping_mul(FNV_PRIME);
    }

    pub fn finish(&self) -> u64 {
        self.hash
    }
}

impl Default for Fnv1a {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_matches_published_ascii_vectors() {
        // FNV-1a 64 公开 ASCII 测试向量（"" 与 "a"）——算法正确性自检，
        // 与模拟态黄金值无关；字节序影响仅作用于 write_u64 的多字节折叠方式。
        let mut h = Fnv1a::new();
        assert_eq!(h.finish(), 0xcbf2_9ce4_8422_2325); // 空串 = offset basis
        h.write_u8(b'a');
        assert_eq!(h.finish(), 0xaf63_dc4c_8601_ec8c); // "a"
    }
}
