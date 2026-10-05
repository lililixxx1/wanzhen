# T009 完整轮审核报告（plan-code-reviewer · 量测卡口径）

- 审核日期：2026-10-05；审核对象：隔离树 `trees/wanzhen/t009-a`（基线 45a0741）全部产出 + 证据档
- 审核方式：只读独立复跑（门禁 4 条 + A3 三方复跑）+ 独立重算（A4/A5/A6 从 runs/ 原始档脚本重算，不复用执行侧汇报值）
- 审核期间副作用声明：为取证 touch 了 sim/src/{world,arena 源}并重建 dev/release 二进制（target/ 不入库；重建后 A3 三方哈希仍与量测档逐位一致 = 可复现性反证加强）；未修改任何被审文件内容

## [结论] 通过（✅ 可合并 / 裁决词汇映射：通过）

一票否决五类（安全 / 数据丢失 / 破坏兼容 / 伪造或缺失证据 / 违反验收标准）逐项排查：零命中。全部验收断言独立复算与汇报值逐位一致；已知偏离 6 项披露属实、留痕自含。

## [已核对]

### 门禁（附录 A 清单，树根执行、串行、-j 3，独立复跑）

| # | 命令 | 退出码 | 结果 |
|---|---|---|---|
| 1 | `cargo check --workspace -j 3` | 0 | PASS；touch 重编译后 warning 行计数 = 0（0 警告实证） |
| 2 | `cargo test -p sim -j 3` | 0 | PASS；lib 44（既有 43 + W1）+ arena bin 3（W2/W3/W4）= **47 全绿**，与 exits.txt 台账一致；r0-2-test.stdout 留档含 4 个新测试 `... ok` 真实输出 |
| 3 | `cargo build -p sim --release -j 3` | 0 | PASS |
| 4 | `cargo build -p sim --bin arena --release -j 3` | 0 | PASS |

### 关键断言

- **A1 deploy_versus 镜像等价 —— PASS**。world.rs 新增段与基线 `deploy`（world.rs:488-547）逐句对照同式：清单展开、Fisher-Yates 双洗（`from_seed(seed ^ DEPLOY_SALT)` 独立实例、同式取模）、队列位置同式（GAP_Q32 复用）、蓝方 `lane_q32 − x_blue[k]` 参数化、units 红块后蓝块、tick RNG `from_seed(seed)`、last_hash=布阵快照哈希——镜像情形（red==blue 且 lane==LANE_LEN_Q32）逐位一致成立（双洗同 seed 同盐 ⇒ 序列相同 ⇒ 与 deploy 单序列双用等价）。W1 单测 3 构型 × 3 seed 全组合，last_hash + run_battle final_hash + winner 三重对拍（超出派工单「布阵+终局」两项要求）。
- **A2 既有 43 单测原值零改动 —— PASS**。`git diff --no-index --numstat` 主仓 vs 树 = **`146 0`** 纯增量；unified diff 仅两个纯插入 hunk（impl 内 deploy_versus + tests 模块末尾 W1），既有函数与既有测试零触碰；2100 + 146 = 2246 行自洽。
- **A3 CLI 黄金交叉三方一致 —— PASS（独立复跑留痕）**。重建后三命令均 REAL_EXIT=0：`arena --sampling --games 1` → `0x564cf46fdf191710`；`sim --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 1` → 同；`--threads 12` → 同。三方逐位一致，且与 runs/r5-crosscheck.txt 量测档值一致。
- **A4 接敌实证 —— PASS（独立重算，超抽格要求做了全量）**。从 runs/ 原始 JSONL 重算：口径层 36 格全算 max avg(alive_r+alive_b) = 194.000 < 200；探针层 36 格全算 max = 14.000 < 20；全规模 100 局全算 max = 9980 < 10000。resolved 计数 0/3600、800/3600、0/100 与 README 一致。
- **A5 吞吐判定 —— PASS（独立重算）**。四档从原始 wall_s 数组重算 median 与 gph 全部与档逐位一致：t1=166,557.9 / t3=461,088.1 / t6=793,548.1 / **t12=1,099,638.6 ≥ 10,000（R2 阈值）PASS**；16 线程 OLS 外推复算 c1=0.6632 / c2=10.3642 / elapsed(16)=1.3110 / gph16=1,405,967.6，与 summary.md 判定行一致。阈值常量 R2_GPH_THRESHOLD=10_000 逐字核对（summarize.py:35）。
- **A6 镜像 sanity —— PASS（独立重算）**。两层对角全 6 格（超「至少 2 格」要求）重算：p_blue=0.0000、z=(0−0.5)/sqrt(0.25/100)=−10.000、95% CI ±0.0000（p∈{0,1} 退化）、|z|>1.96 → H2 结构性偏差形态，逐位与 summary.md 一致；H1/H2 为派工单预注册双假设，H2 如实判定 + 三态分布逐格披露，非事后合理化。
- **A7 计时窗纯度 —— PASS（行号级核对）**。sim/src/bin/arena.rs:522 `Instant::now()` → :524 `pool.map_chunks`（对局循环 + 内存收集）→ :525 `elapsed()`；hash_xor 折叠（:526）、median/gph 计算、write_out_file（:548）、println!（:549）全部窗外；warmup（:510-515）在计时循环外。与派工单 §2「Instant 只包批量对局循环」逐字对应。
- **A8 零依赖 / 零 manifest —— PASS**。树内 `sim/Cargo.toml`、根 `Cargo.toml`、`Cargo.lock` 与主仓基线 diff -q 逐字节一致；`main/units/rng/hash/pool/spatial/lib.rs`、`.gitignore`、`task-ledger.md` 亦逐字节一致；arena.rs 无 unsafe、use 仅 std + sim（bin 自动发现，零 manifest 改动）。

### 已知上报/偏离 6 项披露如实性

| # | 偏离 | 核对结果 |
|---|---|---|
| 1 | W4② 参考值自相矛盾（140→应为 160、首杀 ≈429 非 409） | 属实：arena.rs:727-728 注释明示分歧；W4 断言以实测落（resolved=true + end_tick<cap + 非 Draw），未采任一速算值；README 披露 1 一致 |
| 2 | `--per-side` 域收窄 {10,100} | 属实：arena.rs:409-414；审核实测 `--per-side 50` → exit 2 |
| 3 | throughput JSON 附加键 `final_hash_xor` | 属实：四档 JSON 全等 `0xd1b28b373f7791e3`（跨档确定性旁证成立）；其余键与派工单样例逐键一致 |
| 4 | r1~r6 precheck .txt 缺失 | 属实：runs/precheck-*.out 留有 PowerShell 5.1 ANSI 解析报错原文；precheck-post-run.txt 补检 idle（(no match → idle)、CPU 1%）；r4-t12 CV 复算 = 1.09% < 3% 旁证成立；r7/r8 起正常落档 |
| 5 | summarize.py 区间标注 ±50% 误写修正重生成 | 属实：exits.txt r8-summary-regen REAL_EXIT=0；summary.md 现值 [0.0, 23.2] = 11.6s ±100% 正确（实测中位 13.12 两版区间均含，判定方向不受影响） |
| 6 | check attempt1=101（usage 串 {T} 格式化解析） | 属实：exits.txt r0-1 行留痕（含「原 stderr 未保留，特此声明」）；现源码 usage 串已用 `{{T}}` 转义；编译期错误非量测数据，attempt2=0 + 审核复跑佐证 |

### 附带核对（均一致）

seed 闭式抽查（cell0/cell7 首局/cell35 末局 × 两层 + 全规模前 3 局）；分层比对方向翻转格 cell 29/34 与全规模 draw 格 cell 35 从原始数据复算与 summary 一致（「不一致」格如实披露未粉饰）；击溃率三层 100%/50%/100%（分母 = 有胜方局）复算一致；探针层 avg_end_tick 1676.6 一致；CLI 边界（模式互斥 / per-side 域外 / 缺 --out 均 exit 2，smoke 复跑 exit 0）；证据档可公开态 grep 机器路径零命中；environment.txt 含 arena.exe/sim.exe sha256 链；run_t009.sh 断点续跑语义 + watchdog、crosscheck_r5.sh 判定逻辑、summarize.py 阈值常量逐字核对无手改痕迹；G1 Lead 档（g1-lead-recheck.txt）自含且与本次复算相容（其 t12 抽跑 1,114,633.7 vs 档 1,099,638.6，drift 1.4% 同数量级，判定裕度 ≈110×）。

## [阻塞项]

无。

## [最小修复指令]

无阻塞项。以下为不阻塞的 Minor 建议（收获时随手 / 移交后续，均不构成返工条件）：

1. sim/src/bin/arena.rs:194-195 — `throughput_game` doc 注释残留旧签名表述（「`warmup` = true 时 seed = …」；函数实际以 `seed_base` 参数区分，无 warmup 布尔），注释与实现不符，无行为影响。
2. sim/src/bin/arena.rs:427-441 — `--throughput` 模式未拒绝 `--per-side`/`--per-cell` 等无关参数（静默忽略；`--sampling` 分支 :448-453 有显式拒绝）——CLI 卫生不一致，超出派工单要求范围，登记即可。
3. docs/evidence/t009/README.md:68 — 「既有 2101 行零改动」应为 2100（wc -l 实测；2100+146=2246 自洽）。文档笔误。
4. 提示 Lead（非 worker 缺陷）：主仓 `task-ledger.md` 尚无 T009 行——本审核先于收获属正常顺序，收获提交时按附录 F 回写台账 + taskset 状态。

## [复验命令]

仓库根 = 隔离树根；串行执行。门禁见上表 4 条（退出码全 0）。关键断言复验：

```bash
# A2 纯增量
git diff --no-index --numstat <主仓>/sim/src/world.rs <树>/sim/src/world.rs   # → 146 0
# A3 三方（三方哈希均应 = 0x564cf46fdf191710）
./target/release/arena.exe --sampling --games 1 --out <tmp> && head -1 <tmp>/sampling.jsonl
./target/release/sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 1
./target/release/sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 12
# A5 gph 重算（t12）
python -c "import json; j=json.load(open('docs/evidence/t009/runs/throughput_t12/throughput_t12.json')); w=sorted(j['wall_s']); print(512*3600/w[1])"   # → ≈1099638.6
# 判定行再生成（幂等，数据输入零改动应得同文件）
python docs/evidence/t009/summarize.py --out docs/evidence/t009
```
