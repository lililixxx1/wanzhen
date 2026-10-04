//! T006/D5 手写持久线程池执行器（选项 B，任务卡执行裁决留痕：零新依赖、
//! std::thread + std::sync::mpsc，不引入 bevy_ecs）。
//!
//! ## 形态（D5 定稿）
//!
//! 持久 worker ×(threads−1) + 主线程自带一片（最后一片）；Job 经
//! `std::sync::mpsc` 派发，**每次 [`ThreadPool::map_chunks`] 调用建临时回传通道**
//! 回收各片 `Vec<T>`，按片 start 升序拼装成输出（per-i 与索引一一对应）。
//! `Job = Box<dyn FnOnce() + Send + 'static>`；调用方用**快照模式**满足 'static：
//! 闭包只捕获 `Arc<快照输入>`（owned 克隆数据），**零 unsafe**。
//!
//! ## 分片规则（任务卡执行裁决留痕）
//!
//! 连续均摊分块：片 k ∈ [0, T) 的范围 = `[k*n/T, (k+1)*n/T)`（整数除法，
//! 片长差 ≤ 1；`k*n/T == (k+1)*n/T` 的空片合法）。纯执行划分（意图计算 per-i
//! 无共享写）⇒ **分片与结果无关**：结果只依赖各片内容的并与拼装序（start 升序
//! = 索引序），单测 [`chunk_ranges_cover_partition_balanced`] 断言覆盖/不交/均摊。
//!
//! ## 库契约
//!
//! - `threads >= 1`（CLI 已校验 1..=1024）；`new(0)` 按饱和语义退化为 1（不
//!   panic，`threads()` 返回 1）。
//! - 提交的闭包 **f 不得 panic**（模拟意图函数为纯整数运算、索引由
//!   [`chunk_range`] 保证有界，不 panic）。若违约：panic 的 worker 线程随
//!   unwind 结束、其余 worker 经 `join` 时忽略 Err 不死锁，但该次
//!   `map_chunks` 的语义已破——本契约以注释留痕，不做运行时恢复。
//! - `map_chunks` 串行调用（每次等全部片回齐才返回），无重入。

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// 连续均摊分块纯函数（D5，pub 供单测 §3.4 断言覆盖/衔接/均摊）：
/// 片 k 的范围 = `[k*n/chunks, (k+1)*n/chunks)`，k ∈ [0, chunks)。
///
/// 溢出安全：`k * n` 要求 `chunks * n <= usize::MAX`（本仓 n ≤ 2×10^4、
/// chunks ≤ 1024，远不及 2^64 上界，留痕）。
pub fn chunk_range(n: usize, chunks: usize, k: usize) -> (usize, usize) {
    (k * n / chunks, (k + 1) * n / chunks)
}

/// 派发单元（D5 定稿类型）。
type Job = Box<dyn FnOnce() + Send + 'static>;

/// 持久线程池：worker ×(threads−1) + 主线程自带最后一片。
pub struct ThreadPool {
    /// worker 句柄（Drop 时 join）。
    workers: Vec<thread::JoinHandle<()>>,
    /// 任务队列发送端（`None` = 已 Drop 关闭 → worker recv Err 退出）。
    job_tx: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// 建池：持久 worker ×(threads−1)；worker 循环 `recv`，Err（队列关闭）即退出。
    /// threads == 1（或 0 饱和）不建 worker——[`ThreadPool::map_chunks`] 退化为
    /// 主线程直跑唯一一片。
    pub fn new(threads: usize) -> Self {
        let (job_tx, job_rx) = mpsc::channel::<Job>();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let worker_count = threads.saturating_sub(1);
        let mut workers = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let rx = Arc::clone(&job_rx);
            workers.push(thread::spawn(move || loop {
                // 锁作用域只包 recv：取到 job 先放锁再执行，避免长任务持有队列锁。
                let job = {
                    let guard = rx.lock().expect("pool job queue mutex poisoned");
                    guard.recv()
                };
                match job {
                    Ok(job) => job(),
                    // job_tx 全部丢弃（ThreadPool::drop）→ 队列关闭 → 正常退出。
                    Err(_) => break,
                }
            }));
        }
        Self {
            workers,
            job_tx: Some(job_tx),
        }
    }

    /// 总片数 = worker 数 + 主线程一片（new(0) 饱和为 1）。
    pub fn threads(&self) -> usize {
        self.workers.len() + 1
    }

    /// 分片映射（D5 定稿签名）：n 个单元按 [`chunk_range`] 分 T 片，每片调
    /// `f(start, end) -> Vec<T>`（参数 = 片范围 [start, end)），各片结果按片
    /// start 升序拼装为总长 n 的 `Vec<T>`。threads == 1 时主线程直跑唯一一片。
    ///
    /// 回传：每次调用建**临时** `mpsc` 通道；各片闭包持有独立 `Sender` 克隆，
    /// 发完即丢；主线程收集满 T 个结果（全 Sender 关闭后 `into_iter` 自然结束，
    /// 无忙等、无死锁）。
    pub fn map_chunks<T, F>(&self, n: usize, f: F) -> Vec<T>
    where
        T: Send + 'static,
        F: Fn(usize, usize) -> Vec<T> + Send + Sync + Clone + 'static,
    {
        let t = self.threads();
        let (result_tx, result_rx) = mpsc::channel::<(usize, Vec<T>)>();
        for k in 0..t {
            let (start, end) = chunk_range(n, t, k);
            let f = f.clone();
            let tx = result_tx.clone();
            let job: Job = Box::new(move || {
                let out = f(start, end);
                // 回传通道对端为主线程（正在 result_rx 上等待），send 不失败。
                let _ = tx.send((start, out));
            });
            if k + 1 == t {
                // 最后一片主线程自带直跑（闭包内已 send 回传）。
                job();
            } else {
                self.job_tx
                    .as_ref()
                    .expect("job_tx dropped while ThreadPool alive")
                    .send(job)
                    .expect("all pool workers terminated unexpectedly");
            }
        }
        // 丢弃原始 Sender：各片闭包发完各自丢弃克隆 → 全部关闭 → 收集自然结束。
        drop(result_tx);
        let mut parts: Vec<(usize, Vec<T>)> = result_rx.into_iter().collect();
        // 按 start 升序拼装 = 索引序（确定性：与片执行完成先后无关）。
        parts.sort_unstable_by_key(|(start, _)| *start);
        let mut out = Vec::with_capacity(n);
        for (_, mut part) in parts {
            out.append(&mut part);
        }
        out
    }
}

impl Drop for ThreadPool {
    /// D5：`drop(job_tx)` → worker `recv` Err 退出循环 → join 全部。
    fn drop(&mut self) {
        self.job_tx = None;
        for handle in self.workers.drain(..) {
            // 库契约 f 不 panic；若违约，worker unwind 结束后 join 返回 Err——
            // 忽略之（不死锁），语义破坏由契约注释追责。
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 单测 4（§3.4 / D10-4）：分片纯函数——并集恰为 [0, n)、相邻片衔接
    /// （片 k 的 end == 片 k+1 的 start）、片长差 ≤ 1。
    #[test]
    fn chunk_ranges_cover_partition_balanced() {
        // 通用校验器：start 单调、首片 start=0、末片 end=n、相邻衔接、
        // 片长 max-min ≤ 1（空片计入）。
        let check = |n: usize, t: usize| {
            let ranges: Vec<(usize, usize)> = (0..t).map(|k| chunk_range(n, t, k)).collect();
            assert_eq!(ranges[0].0, 0, "n={n} t={t}: 首片 start 应为 0");
            assert_eq!(ranges[t - 1].1, n, "n={n} t={t}: 末片 end 应为 n");
            let mut total = 0;
            for k in 0..t {
                let (s, e) = ranges[k];
                assert_eq!(s, total, "n={n} t={t} 片{k}: start 应衔接前片 end（不交且覆盖）");
                assert!(e >= s, "n={n} t={t} 片{k}: 空片应为 start==end，不得倒置");
                total = e;
            }
            let lens: Vec<usize> = ranges.iter().map(|(s, e)| e - s).collect();
            let max = *lens.iter().max().unwrap();
            let min = *lens.iter().min().unwrap();
            assert!(
                max - min <= 1,
                "n={n} t={t}: 片长差 {max}-{min} 应 ≤ 1（lens={lens:?}）"
            );
            ranges
        };
        // 派工单 §3.4 指定用例（逐片显式值断言）。
        assert_eq!(
            check(10, 3),
            vec![(0, 3), (3, 6), (6, 10)],
            "(n=10,T=3) 应为 [0,3),[3,6),[6,10)"
        );
        assert_eq!(
            check(10, 4),
            vec![(0, 2), (2, 5), (5, 7), (7, 10)],
            "(n=10,T=4) 应为 [0,2),[2,5),[5,7),[7,10)"
        );
        // (n=3,T=5)：含空片 [1,1) 合法。
        let r = check(3, 5);
        assert!(
            r.contains(&(1, 1)),
            "(n=3,T=5) 应含空片 (1,1)，实际 {r:?}"
        );
        // (n=0,T=4)：全空片。
        assert_eq!(check(0, 4), vec![(0, 0), (0, 0), (0, 0), (0, 0)]);
        // 补充边界：n < t、n == t、t=1。
        check(2, 5);
        check(4, 4);
        assert_eq!(check(7, 1), vec![(0, 7)]);
    }

    /// 单测 5（D10-5）：线程池生命周期——threads=1/4 建→用→drop，结果与串行一致。
    #[test]
    fn threadpool_lifecycle_map_chunks() {
        // 串行参照：0..64 的平方表。
        let expect: Vec<u64> = (0..64u64).map(|i| i * i).collect();
        // 闭包纯函数：f(start,end) -> 片内平方表（无共享写，per-i 确定）。
        let squares = |start: usize, end: usize| -> Vec<u64> {
            (start..end).map(|i| (i as u64) * (i as u64)).collect()
        };
        // threads=1：主线程直跑唯一一片。
        {
            let pool = ThreadPool::new(1);
            assert_eq!(pool.threads(), 1);
            assert_eq!(pool.map_chunks(64, squares), expect, "threads=1 应与串行一致");
        } // drop：join（threads=1 无 worker，空 join）
        // threads=4：建→用→drop，多次 map_chunks（含 n < t 空片路径）。
        {
            let pool = ThreadPool::new(4);
            assert_eq!(pool.threads(), 4);
            assert_eq!(pool.map_chunks(64, squares), expect, "threads=4 应与串行一致");
            assert_eq!(pool.map_chunks(64, squares), expect, "复用应稳定");
            assert_eq!(pool.map_chunks(3, squares), expect[..3], "n<t 空片路径");
            assert_eq!(pool.map_chunks(0, squares), Vec::<u64>::new(), "n=0 全空片");
        } // drop：worker ×3 recv Err 退出并 join（不悬挂 = 生命周期断言）
    }
}
