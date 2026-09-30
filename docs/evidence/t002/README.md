# T002 证据档——最小确定性 tick 循环（M0 W1）

## 各文件用途

| 文件 | 内容 |
| --- | --- |
| `check.txt` | `cargo check --workspace -j 3` 完整输出 + 真实退出码（0 警告门禁，REAL_EXIT=0） |
| `test.txt` | `cargo test -p sim -j 3` 完整输出 + 真实退出码（8/8 全绿，REAL_EXIT=0） |
| `build-release.txt` | `cargo build -p sim --release -j 3` 构建记录（跨进程对拍所用二进制的来源，REAL_EXIT=0） |
| `run1.txt` / `run2.txt` | release 二进制 `./target/release/sim.exe --seed 42 --ticks 1800` 两次独立进程运行全记录（stdout 五行确定性输出 + stderr elapsed_ms + REAL_EXIT） |
| `cross-check.txt` | run1 vs run2 逐字节对拍（cmp/diff 均无差异）+ seed 43 对照（哈希不同） |
| `smoke-bench.txt` | 能力烟雾：`--seed 42 --ticks 1800 --units 10000`，elapsed_ms=18（验收阈值 <1,000ms，REAL_EXIT=0；能力烟雾非正式量测） |
| `rng-golden.txt` | RNG/World 黄金值实测固化全过程：占位 0 实测打印 → python 独立重实现对拍 → 回填断言 → 复跑全绿（PIT-M-002 纪律） |

## 执行摘要

T002 交付 headless 最小确定性 tick 循环：`cargo check --workspace -j 3` 0 警告、`cargo test -p sim -j 3` 8/8 全绿；release 二进制两次独立进程运行 stdout 逐字节一致（seed 42 / 1800 ticks → `hash=0xd3b6408fd46c2008`），seed 43 同参数得 `hash=0xa81b59deeb98e700`（不同 seed 哈希不同）；10,000 空单位 × 1,800 ticks 能力烟雾 elapsed_ms=18ms（阈值 <1,000ms）。RNG（SplitMix64 seed 扩展 + Xoshiro256\*\*）与状态哈希（FNV-1a 64）均为零依赖手写实现，黄金值全部实测产出并经独立 python 重实现逐值对拍一致；模拟态代码路径无浮点、无超越函数、无挂钟时间源、无 HashMap（计时仅在 main 外壳与测试代码）。
