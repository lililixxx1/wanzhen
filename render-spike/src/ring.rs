//! 帧时间环形缓冲（追加语义：写满后拒绝覆盖、直接 panic）。
//!
//! 采集窗口以「已记录帧 delta_ns 之和」计量；容量 240_000 帧对 65 s 窗口在
//! ~3700 fps 以内均有余量。一旦溢出说明口径异常，须显式暴露（panic）而非静默
//! 丢样或覆盖——这是采集完整性纪律的一部分。

use std::collections::VecDeque;

/// 固定容量环形缓冲（VecDeque 承载）。
pub struct RingBuffer<T> {
    buf: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    /// 建立容量为 `capacity` 的空缓冲（capacity ≥ 1）。
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity >= 1, "ring buffer capacity must be >= 1");
        Self {
            buf: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// 追加一个样本；满容时 panic（拒绝覆盖淘汰）。
    pub fn push(&mut self, value: T) {
        if self.buf.len() >= self.capacity {
            panic!(
                "ring buffer overflow: capacity {} reached, refusing to overwrite",
                self.capacity
            );
        }
        self.buf.push_back(value);
    }

    /// 当前样本数。
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// 连续切片视图（按插入序，可能两段）。
    pub fn as_slices(&self) -> (&[T], &[T]) {
        self.buf.as_slices()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W3：容量语义——空、逐条追加、len 与插入序。
    #[test]
    fn w3_capacity_semantics() {
        let mut buf = RingBuffer::with_capacity(4);
        assert_eq!(buf.len(), 0);
        for v in [10u64, 20, 30, 40] {
            buf.push(v);
        }
        assert_eq!(buf.len(), 4);
        // 插入序 = 迭代序（两段切片拼接）。
        let (a, b) = buf.as_slices();
        let seen: Vec<u64> = a.iter().chain(b.iter()).copied().collect();
        assert_eq!(seen, vec![10, 20, 30, 40]);
    }

    /// W3：溢出用例——满容后再次 push 必须 panic（不覆盖、不静默丢弃）。
    #[test]
    #[should_panic(expected = "ring buffer overflow")]
    fn w3_overflow_panics() {
        let mut buf = RingBuffer::with_capacity(2);
        buf.push(1u64);
        buf.push(2u64);
        buf.push(3u64); // 越界：panic
    }

    /// W3：容量 1 边界（最小合法容量）。
    #[test]
    fn w3_capacity_one() {
        let mut buf = RingBuffer::with_capacity(1);
        buf.push(7u64);
        assert_eq!(buf.len(), 1);
    }
}
