# M0 证据总索引（T011 · M0 数据自含成档 · 收官序列第 1 步）

- 生成时刻：2026-10-06T08:43:29+08:00
- 生成者：worker-1（隔离树 t011-a；基线 commit 1d304a8）
- 审核状态：plan-code-reviewer 完整轮**通过**（2026-10-06：有条件通过 P0=0/P1=0/P2×2 → 同日整改闭环转通过；报告 = review-plan-code-reviewer.md；验收断言 1/2/3 全 PASS）
- 冻结计划：docs/万阵-游戏前期策划报告.html（V0.9.1，2026-09-30 复核裁决「通过——可作为 M0 唯一输入」）
- 任务卡：taskset/t011-data-pack.md（开工裁决 D1~D9）；派工单：docs/evidence/m0/dispatches/wp-a.md

## 纪律声明

本档全部验收判定行**逐字引用**自下列代码生成档（均标注「脚本生成，禁止手改」）：

- docs/evidence/t007/summary.md（`bench --summarize` 生成）
- docs/evidence/t008/matrix.md（`compare_matrix.py` 生成）
- docs/evidence/t009/summary.md（`summarize.py` 生成）
- docs/evidence/t010/summary.md（`summarize.py` 生成）
- docs/evidence/t015/summary.md（`bench --summarize` 生成）

本档新增计算仅两处：投入汇总（tally_hours.py，双盲对照留痕 runs/tally.txt）与判定行双向核对（verify_claims.py，留痕 runs/verify.txt）。除此之外本档零手算数字（唯一的例外是 §6 的窗口时长换算 670.6/60=11.18，按派工单要求以算式呈现）。口径唯一来源 = 报告 V0.9.1 表 6-0 / 6.1（docs/万阵-游戏前期策划报告.html）。本档内路径一律仓库相对路径。

## §0 六验收判定快照表

| 验收 | 终态判定 | 关键数字 | 判定行来源档 | 时序注记 |
|---|---|---|---|---|
| ① 四点单线程 µs | PASS（T015 后；T007 曾 4/4 TRIPPED——R2 轮一响应） | 0.060651 / 0.072196 / 0.070617 / 0.108681 µs | t015/summary.md §①（+t007 时序） | T007 如实 TRIPPED → T015 排序序快路径后 4/4 PASS（§1） |
| ② @10k 加速比 | TRIPPED（12t 1.001913×；owner 裁决点→T013，D1） | 12t 实测 1.001913 / 16t 外推 1.112480 | t015/summary.md §② | T007 时代 7.221411 OK → T015 结构性 TRIPPED（§2） |
| ②附则 极限十万 | 保留（17.53ms≤22ms；T007 曾超界） | C(100000)÷speedup16 = 17.532997 ms | t015/summary.md §③ | T007 3907.133373 ms 超界 → T015 翻转保留（§2） |
| ③ 吞吐（降规模） | PASS 1,099,638.6 场/h@12t | games_per_hour@12t = 1099638.6 | t009/summary.md 吞吐节 | 单卡一次交付（§3） |
| ④ 同种子重放 | 全 PASS（退出码 0） | 29 局矩阵 + 加样 + 锚局全一致 | t008/matrix.md §8 | T015 后 29/29 逐字节复现（§4） |
| ⑤ 渲染 10k 同屏 | PASS avg 333.15 / 1% low 202.48 | avg_fps=333.15、1% low=202.48 | t010/summary.md 判定行节 | 单卡一次交付（§5） |
| ⑥ 50k 内存长跑 | 达标 | 稳态 6.8 MiB ≤ 2048 MiB；窗口 670.6 s | t007/summary.md §⑤ | T015 未重跑（D10 预登记，§6） |

三条总注：

1. **② owner 裁决点（本档不仲裁）**：② 字面口径 TRIPPED（12t 实测 1.001913× < 4×）vs 预算侧意图全满足（①③ 达标）——按字面呈现 TRIPPED，不放行不粉饰；裁决出处：AGENTS.md「当前状态」节（「owner 裁决点（② 口径 vs 意图 + t016 去留，建议 T013 报告级仲裁）」）与 taskset/t015-opt-round-1.md 收口记录（「移交 owner 裁决点：② 字面（≥4×）vs 意图（预算全满足）+ 轮二 t016 去留——按 D10 预登记，轮二无法以诚实手段闭合 ②；建议随 T013 报告级仲裁。」）。
2. **③ 镜像 sanity 移交**：t009 双层 12 格 H2 结构性红偏（z=−10.000，apply 索引序先手语义），预注册 H1/H2 双假设之一如实判定，移交 T013 口径仲裁（§3 附带披露）。
3. **D2 编号澄清**：t007/t015 summary.md 内部节名「## ③ 外推十万」实为报告验收②的极限档附则（表 6-0 ②后半句「外推 10 万模拟单位模拟耗时 ≤ 22ms」方保留）；报告验收③ = 实验场吞吐（t009）。本档按报告编号组织，T012 复算者勿以 summary 内部节号对位报告验收号。

## §1 验收① 微秒成本（单线程基线 1k/5k/10k/50k）

### ①口径出处

代码生成档口径行（docs/evidence/t007/summary.md L5，逐字）：

> - 口径（报告 V0.9.1 表 6-0 / 6.1，任务卡 D6 逐字）：止损 = 每单位每 tick 成本 > 2µs（单线程基线）或 12 线程实测加速比 < 4×（16 线程外推值 < 4× 同判）；承诺线 = 止损线 × 0.5（µs 侧 1µs）；预算 常态 @60fps ≤ 8ms / 极限 @30fps ≤ 22ms；所需加速比 = 单位数 × 单线程每单位每 tick 成本 ÷ 每帧模拟预算；加速比止损锚定 10k 采样点（万人常态）

报告表 6-0 ①转述（docs/万阵-游戏前期策划报告.html:498）：验收① = 每单位每 tick 微秒成本（模拟层、单线程基线），在 1k/5k/10k/50k（模拟单位数）四采样点测得（基准机 A）。

### ②终态判定行（逐字引自 docs/evidence/t015/summary.md L18-21）

> - [①] units=1000: us_per_unit_tick=0.060651 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）
> - [①] units=5000: us_per_unit_tick=0.072196 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）
> - [①] units=10000: us_per_unit_tick=0.070617 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）
> - [①] units=50000: us_per_unit_tick=0.108681 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）

### ③时序与归因

T007 基线判定行（docs/evidence/t007/summary.md L18-21，逐字）：

> - [①] units=1000: us_per_unit_tick=4.012138 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）
> - [①] units=5000: us_per_unit_tick=19.846795 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）
> - [①] units=10000: us_per_unit_tick=41.010103 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）
> - [①] units=50000: us_per_unit_tick=200.625962 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）

时序注记：T007 如实 TRIPPED → T015 排序序快路径（sim/src/spatial.rs，O(N²)→O(N log N)）后 4/4 PASS 且全达承诺线 1µs；降幅 66×~1846×（docs/evidence/t015/README.md L24-26，逐字）：

> - **① 四采样点单线程 µs：全 PASS 且全达承诺线**——0.060651 / 0.072196 / 0.070617 /
>   0.108681（1k/5k/10k/50k；T007 基线 4.012/19.847/41.010/200.626）——降幅 66×/275×/
>   581×/1846×，距止损线 2µs 裕度 ≥18×。

### ④止损/承诺对照与 6.1 换算（引用，不重算）

docs/evidence/t015/summary.md §④ 换算表（L57-62，表头与四数据行逐字）：

| units | 单线程 us/单位/tick | 所需@8ms（常态） | 对照 | 所需@22ms（极限） | 对照 |
|---:|---:|---:|---|---:|---|
| 1000 | 0.060651 | 0.007581 | 承诺线内（≤2×） | 0.002757 | 承诺线内（≤2×） |
| 5000 | 0.072196 | 0.045122 | 承诺线内（≤2×） | 0.016408 | 承诺线内（≤2×） |
| 10000 | 0.070617 | 0.088271 | 承诺线内（≤2×） | 0.032099 | 承诺线内（≤2×） |
| 50000 | 0.108681 | 0.679255 | 承诺线内（≤2×） | 0.247002 | 承诺线内（≤2×） |

6.1 换算公式（报告 docs/万阵-游戏前期策划报告.html:525）：所需加速比 = 单位数 × 单线程每单位每 tick 成本 ÷ 每帧模拟预算。

### ⑤原始证据档路径

- docs/evidence/t015/（summary.md + matrix.jsonl + runs/ + README.md + equiv/ + hash-crosscheck.txt）
- docs/evidence/t007/（summary.md + matrix.jsonl + runs/ + README.md）

### ⑥复现命令

构建（docs/evidence/t015/run_matrix_t015.sh L29 提示行，逐字；审核轮 P2-1 勘误：原引 L28 偏 1）：

```
cargo build -p sim --bin bench --release -j 3
```

汇总（docs/evidence/t015/run_matrix_t015.sh L22-23 变量定义 + L159-160 汇总步，逐字；审核轮 P2-1 勘误：原引 L24-25 偏 2；展开式附后）：

```
EVID=docs/evidence/t015
BIN=target/release/bench.exe
"$BIN" --summarize "$EVID/matrix.jsonl" --out "$EVID/summary.md" \
  > "$EVID/runs/summarize.stdout.txt" 2> "$EVID/runs/summarize.stderr.txt"
```

展开等价式：`./target/release/bench.exe --summarize docs/evidence/t015/matrix.jsonl --out docs/evidence/t015/summary.md`。量测窗口机器空闲独占纪律适用（附录 A / PROJECT-APPENDIX）。

## §2 验收② 加速比（@10k 锚定）

### ①口径出处

口径同 §1 所引 docs/evidence/t007/summary.md L5 同句（加速比部分逐字）：「12 线程实测加速比 < 4×（16 线程外推值 < 4× 同判）」「加速比止损锚定 10k 采样点（万人常态）」。报告表 6-0 ②转述 + 后半句逐字（docs/万阵-游戏前期策划报告.html:498）：并行扩展曲线：1/3/6/12 线程加速比（16 线程档外推，基准机 A），「外推 10 万模拟单位模拟耗时 ≤ 22ms（极限档模拟预算，表 6-0 帧预算分解）方保留「极限十万」目标」。

### ②终态判定行（逐字引自 docs/evidence/t015/summary.md）

L42：

> - [②] units=10000: 12 线程实测加速比=1.001913 | 止损判定（<4×）: TRIPPED | 16 线程外推加速比=1.112480 | 同判（<4×）: TRIPPED

L68（总判行）：

> - 总判定：TRIPPED（触发项：② 加速比<4×（锚定 10000））；按 6.1/R2 由此进入后续优化决策（本卡不优化）

### ③时序（T007 基线，逐字引 docs/evidence/t007/summary.md L42）

> - [②] units=10000: 12 线程实测加速比=7.221411 | 止损判定（<4×）: OK | 16 线程外推加速比=9.416188 | 同判（<4×）: OK

### ④归因注记

归因要点（docs/evidence/t015/README.md，逐字引用）——Amdahl 拟合（L37-38）：

>    （分片排序 + 查询）占比骤降——本次 Amdahl 拟合 @10k：c1=630909ns、c2=61780ns
>    （可并行内容占比 ≈ 8.9%，R²=0.19 因短局噪声）。

理论上限推演（L45-48）：

> 4. **上限推演**：即便池开销归零、其余全部完美并行，S(12)@10k 上限 ≈
>    1/(0.19+0.81/12) ≈ 3.9×（哈希链独占串行 19% 时**理论上限已低于 4× 线**；
>    P2-1 勘误：初稿误写 4.1×，审核轮 python 复算 3.883495）——加回必串行
>    apply 后 ≈2.6~3.5×。**轮二（t016：并行归并/扫掠、池开销削减）无法以诚实

转述：可并行 O(N²) 扫描删除后，串行 FNV-1a 哈希链（逐字节依赖链）+ 必串行 apply（索引序语义）主导，线程池开销吃掉残余并行收益（12 线程档实测 ≈ 单线程）；理论上限 3.9×（<4×），轮二无法以诚实手段闭合 ②。

结尾注（owner 裁决点）：② 字面口径 vs 预算侧意图，建议 T013 报告级仲裁（出处：AGENTS.md「当前状态」节 / taskset/t015-opt-round-1.md 收口记录「移交 owner 裁决点」，全文见 §0 总注 1）——本档按字面呈现 TRIPPED，不放行不粉饰。

### ⑤极限十万附则（D2：报告验收②附则，非报告验收③）

T007（docs/evidence/t007/summary.md L49，逐字）：

> - C(100000) ÷ speedup16 = 3907.133373 ms vs 22 ms 极限预算 → [超界：极限十万目标不保留]

T015（docs/evidence/t015/summary.md L49，逐字）：

> - C(100000) ÷ speedup16 = 17.532997 ms vs 22 ms 极限预算 → [保留]

翻转注记：O(N²) 项消除——拟合 b 系数 3.997072319（t007/summary.md L46）→ 0.000918316（t015/summary.md L46），成本近线性；两拟合行逐字：

> - 拟合：a=773.554196350，b=3.997072319，R²=1.000000，最大残差=4560603.291 ns（拟合区间 1k~50k；R²/残差为附加留痕）
> - 拟合：a=62.759210207，b=0.000918316，R²=0.999960，最大残差=24223.712 ns（拟合区间 1k~50k；R²/残差为附加留痕）

### ⑥复现命令

同 §1（构建 + summarize）；matrix.jsonl 来源差异注明：t007 归档 = docs/evidence/t007/matrix.jsonl（优化前基线矩阵），t015 归档 = docs/evidence/t015/matrix.jsonl（T015 复测矩阵，口径与 T007 原样一致：1k×600t / 5k×300t / 10k×300t / 50k×30t × threads{1,3,6,12} seed42 warmup1 repeats5）。

## §3 验收③ 实验场吞吐（降规模口径）

### ①口径出处

报告表 6-0 ③转述（docs/万阵-游戏前期策划报告.html:498）：实验场实测吞吐（降规模口径，基准机 A）= 单场墙钟时长 × 并发线程数（含镜像 AI 决策、不含落库；AI 池扩档后须重测）。降规模对局定义（docs/万阵-游戏前期策划报告.html:485，逐字）：「每方 100 模拟单位（共 200）、单局 ≤ 60s（≤ 1,800 ticks @30Hz）。实验场吞吐与 R2 吞吐阈值均按此口径；与全规模对局两套口径不得混用。」R2 终止判据吞吐线出处（docs/万阵-游戏前期策划报告.html:538，逐字片段）：「实验场吞吐低于 1 万场/小时（降规模口径，表 6-0——全规模对局按止损线约 530 场/小时，仅作质量校验、不计吞吐）」。

### ②判定行（逐字引自 docs/evidence/t009/summary.md）

L180：

> - [吞吐判定行] games_per_hour@12t = 1099638.6 （阈值 >= 10000）→ PASS

L181：

> - [16 线程外推] elapsed(T)=c1+c2/T OLS（T∈{1,3,6,12} 中位）：c1=0.6632s c2=10.3642s·T → elapsed(16)=1.3110s → 吞吐16 = 1405967.6 场/h

### ③附带披露（各 1-2 行 + 档指针）

- 镜像 sanity 双层 12 格 H2 结构性红偏：判定行（docs/evidence/t009/summary.md L150/L169，逐字）「存在 |z| > 1.96 的对角格 → H2 结构性偏差形态（三态分布已逐格披露）」；对角 6 格 z=−10.000、红 100%/蓝 0%/Draw 0%——apply 索引序先手语义，预注册 H1/H2 双假设之一如实判定，移交 T013 口径仲裁。
- 接敌实证三层 PASS（docs/evidence/t009/summary.md L186-188，逐字）：
  > - [接敌实证判定行·口径层] 36 格全部 avg(alive_red+alive_blue) < 200（2*per_side）→ PASS；全场最大 avg 存活(和) = 194.00
  > - [接敌实证判定行·探针层] 36 格全部 avg(alive_red+alive_blue) < 20（2*per_side）→ PASS；全场最大 avg 存活(和) = 14.00
  > - [接敌实证判定行·全规模层] 100 局全部 alive_red+alive_blue < 10000 → PASS；全场最大存活(和) = 9980
- 分层方向一致 33/36（docs/evidence/t009/summary.md L233，逐字；3 格翻转小样本披露：cell 29/34 方向随规模翻转、cell 35 全规模层 hp-sum 同值 Draw，n=2 小样本如实披露）：
  > - [分层方向一致判定行] 33/36 cell 三层方向一致（比例 91.7%）
- 口径层 resolved 0/3600 = 预注册预期（docs/evidence/t009/summary.md L254，逐字）：
  > - 口径层多数格 resolved=false（预注册设计预期）→ 实测口径层 resolved 局数 0/3600；全规模层 resolved 0/100

### ④复现命令

docs/evidence/t009/README.md 复现指引节（L74-79，逐字）：

```bash
cargo test -p sim -j 3                      # 47 全绿
cargo build -p sim --release -j 3
./docs/evidence/t009/run_t009.sh            # 断点续跑：exits.txt 已有 REAL_EXIT=0 的批次自动跳过
python docs/evidence/t009/summarize.py --out docs/evidence/t009
```

吞吐四档批命令（docs/evidence/t009/run_t009.sh L80-83，逐字）：

```
run_batch r4-t1             1800 './target/release/arena.exe --throughput --games 512 --threads 1 --repeats 3 --out docs/evidence/t009/runs/throughput_t1'
run_batch r4-t3              900 './target/release/arena.exe --throughput --games 512 --threads 3 --repeats 3 --out docs/evidence/t009/runs/throughput_t3'
run_batch r4-t6              900 './target/release/arena.exe --throughput --games 512 --threads 6 --repeats 3 --out docs/evidence/t009/runs/throughput_t6'
run_batch r4-t12             900 './target/release/arena.exe --throughput --games 512 --threads 12 --repeats 3 --out docs/evidence/t009/runs/throughput_t12'
```

## §4 验收④ 同种子重放确定性

### ①口径出处

报告表 6-0 ④转述（docs/万阵-游戏前期策划报告.html:498）：同种子重放哈希一致（1/3/6/12 线程 × 3 组种子，基准机 A）。

### ②判定（docs/evidence/t008/matrix.md §8，逐字）

> - 断言 1（零 mismatch：每 (scale,seed) 组合 4 线程档 × 全部采样列 + 终局列逐位一致）: PASS — 6/6 组合逐位一致
> - 断言 2（中间哈希：每 scale 标准采样列组内一致）: PASS
> - 断言 3（mismatch 全量如实列出）: 未触发（无 mismatch）
> - 加样局（跨进程同配置逐列一致）: PASS — red200-th12-s42-r2 PASS；red200-th12-s42-r3 PASS；full10000-th12-s42-r2 PASS
> - 锚局（vs T006 归档逐字节 diff）: PASS — anchor1-t006-fullscale PASS；anchor2-t006-10k300 PASS
> - 种子互异 sanity（同 (scale,threads) 下 3 种子终局哈希两两不同）: PASS — 8/8 个 (scale,threads) 下 3 种子终局哈希两两不同
> - 退出码: 0

终局哈希 6 值表（docs/evidence/t008/README.md §2 L82-89，逐字）：

| 规模 | seed | 终局哈希 | 终局 units |
|---|---|---|---|
| red200 | 42 | 0xde91d6a5a6e84d43 | 200 |
| red200 | 43 | 0x999a5237d3b1a9b2 | 200 |
| red200 | 44 | 0x44348b8042997ed3 | 200 |
| full10000 | 42 | 0xd921c95a9bf1db66 | 9959 |
| full10000 | 43 | 0x7d34b08102e34260 | 9969 |
| full10000 | 44 | 0x9d55c4ce4f7fd880 | 9970 |

### ③覆盖域披露（docs/evidence/t008/README.md §1.3 L54，逐字）

> - **覆盖域如实披露**：red200 只覆盖移动段确定性；战斗段确定性由 full10000 覆盖。不得由本档外推「战斗段已在 red200 规模下验证」。

要点转述：full10000 的 10800/12600 采样点 = 接敌后点（战斗段覆盖），实证 = full 规模 14/14 局终局 units<10000（9959/9969/9970）；red200 的 0/450/900/1350 = 移动段采样（1800 ticks 内双方不接敌，终局 units=200 双证）。

### ④T015 后等价性（逐字引自 docs/evidence/t015/README.md）

L13：

> | `equiv/` | **T008 对拍矩阵全 29 局逐字节复现**（equiv-exits.txt：29/29 IDENTICAL；含 14 局全规模战斗段中间采样点 10800/12600 与双锚局） |

L15：

> | `hash-crosscheck.txt` | 16 配置 final_hash vs T007 归档动态比对：**16/16 MATCH** |

### ⑤复现命令（docs/evidence/t008/README.md §4 L110-125，逐字）

```bash
# 门禁（0 警告）与构建（量测窗口机器须空闲独占）
cargo check --workspace -j 3
cargo build -p sim --release -j 3

# 构成映射重算（双盲核对；退出码非 0 = 按纪律停止上报）
python docs/evidence/t008/gen_comp.py

# 跑批（断点续跑幂等：exits.txt 已有 REAL_EXIT=0 行即 SKIP；非 0 行不覆盖不重跑）
./docs/evidence/t008/run_t008.sh all
# 分选择集：./docs/evidence/t008/run_t008.sh anchor2 red200
#           ./docs/evidence/t008/run_t008.sh anchor1 full10000

# 对拍判定（WP-D D1 本次命令；matrix.md 由该脚本生成）
python docs/evidence/t008/compare_matrix.py docs/evidence/t008/runs
```

## §5 验收⑤ 渲染 spike

### ①口径出处

报告表 6-0 ⑤转述（docs/万阵-游戏前期策划报告.html:498）：渲染 spike：胶囊体灰盒 1 万同屏 @60fps、1% low ≥ 45（基准机 A；「灰盒无动画/LOD/分层渲染，指标高于 M2/M3 属预期，不作外推依据」——此句为表 6-0 原文，必须保留，不得外推 M2/M3 常态渲染结论）。

### ②判定行（逐字引自 docs/evidence/t010/summary.md）

L31：

> - t10000: avg_fps=333.15 (≥60 ? 是)、1% low=202.48 (≥45 ? 是) → **PASS**

四档表（docs/evidence/t010/summary.md L9-14，表头与四档行逐字）：

| 档位 | 帧数 N | avg_fps | 1% low | 0.1% low | p50 (ms) | p95 (ms) | p99 (ms) | max (ms) | 非vsync旁证帧数 | 判定 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| t1000 | 34843 | 536.03 | 260.07 | 151.61 | 1.745 | 2.567 | 3.249 | 24.113 | 34841 |  |
| t2000 | 33933 | 522.03 | 268.28 | 213.06 | 1.800 | 2.666 | 3.324 | 6.026 | 33933 |  |
| t5000 | 32936 | 506.70 | 242.93 | 191.98 | 1.824 | 2.866 | 3.650 | 6.163 | 32936 |  |
| t10000 | 21656 | 333.15 | 202.48 | 153.40 | 2.935 | 3.727 | 4.395 | 7.743 | 21656 | PASS |

### ③披露

物理窗口保守方向（docs/evidence/t010/README.md L90-94，逐字）：

> - **实际窗口尺寸披露**：`--res 1920x1080` 为请求值；实际物理窗口 **2400×1350**（各档 meta.json
>   `window_resolution_actual` 一致）。成因：显示器 125% DPI 缩放，bevy_winit 无 scale override 时按
>   **逻辑尺寸**请求、由 OS 套用缩放（源码 `bevy_winit-0.19.1/src/winit_windows.rs:114-120`）。
>   即实测画布面积 ≈ 3.24 Mpix = 1080p 的 1.56×——**渲染负载更重，判定为保守方向**；
>   判定口径（avg/1% low）与阈值不变，不改判。详见 api-notes.md。

vsync 旁证四档均「有」（docs/evidence/t010/summary.md L24-27，逐字）：

> - t1000: 非 vsync 锁定旁证：有（34841 帧 < 15.865 ms）
> - t2000: 非 vsync 锁定旁证：有（33933 帧 < 15.865 ms）
> - t5000: 非 vsync 锁定旁证：有（32936 帧 < 15.865 ms）
> - t10000: 非 vsync 锁定旁证：有（21656 帧 < 15.865 ms）

### ④复现命令（AGENTS.md 常用命令节 L34，逐字；含 -j 2 从严与 commit 预检门槛）

> - `cargo build -p render-spike --release -j 2` / `./target/release/render-spike.exe --units <N> [--seed 42] [--warmup-sec 5] [--capture-sec 65] [--res 1920x1080] --out <dir>` — M0 渲染 spike（T010，验收⑤：窗口化帧采集 frames.csv 逐帧原始档；判定汇总 `python docs/evidence/t010/summarize.py`）。涉 render-spike 的 cargo 一律 **-j 2 从严 + 前置 commit 预检（check/test ≥10G、release ≥12G）**——bevy full 冷编属依赖重型足迹（T010 实测冷编 19m45s）

## §6 验收⑥ 内存长跑

### ①口径出处

报告表 6-0 ⑥转述（docs/万阵-游戏前期策划报告.html:498）：内存：50k 模拟单位 ≥ 10 分钟长跑稳态工作集 ≤ 2 GB 且无单调增长（基准机 A）。无单调增长的判定口径为再裁决定稿口径 = 高水位稳定性（稳态后半 max ≤ 前半 max × 1.01；docs/evidence/t007/summary.md §⑤ 节首行注明）。

### ②判定行（逐字引自 docs/evidence/t007/summary.md）

L77：

> - [⑥] 判定：达标（附：PrivateMemorySize64 max=4.7 MiB）

窗口对照行（L72/L73，逐字）：

> - 样本：n=134 条（采样间隔 5s；每 5s 读 Get-Process WorkingSet64 + PrivateMemorySize64；判定用工作集）
> - 窗口时长（末样本 elapsed_ms）：670.6 s

对照注记（算式为准）：670.6/60 = 11.177 → 11.18 min ≥ 10 min 口径。

### ③T015 未重跑注记（docs/evidence/t015/README.md L31，逐字）

> - ⑥ 内存未重跑（无内存行为变更，D10 预登记；summarize 披露行在档）。

### ④复现命令

长跑脚本：docs/evidence/t007/run_longrun.ps1（docs/evidence/t007/README.md 文件索引 L24：「run_longrun.ps1 | ⑥ 长跑 + 5s 内存采样器（探针校准 K6 → 单局 → CSV）」）。长跑命令留痕（docs/evidence/t007/runs/longrun_meta.txt L5，逐字）：

```
长跑命令: target/release/bench.exe --units 50000 --threads 12 --ticks 478 --warmup 0 --repeats 1 --seed 42
```

## §7 环境总档

### 环境要素表（来源：五份 environment.txt；值不一致时分行列出，不合并不择优）

| 要素 | 值 | 出处档 |
|---|---|---|
| CPU | 12th Gen Intel(R) Core(TM) i5-12490F（6 核 / 12 逻辑处理器 / Max 3000 MHz） | t007/t008/t009/t010/t015 environment.txt 五档同值 |
| GPU + 驱动（WMI） | NVIDIA GeForce RTX 3050，DriverVersion 31.0.15.3758（DriverDate 10/04/2023）；另有 GameViewer Virtual Display Adapter（15.6.5.199） | t007/t008/t009/t015/t010 environment.txt 五档同值 |
| GPU + 驱动（nvidia-smi 旁行） | NVIDIA GeForce RTX 3050, 537.58 | 仅 t010/environment.txt（采集器多采一行，与 WMI 行同卡不同版本记法，分行列出） |
| RAM | TotalPhysicalMemory 34187943936 bytes；32 GiB 3600 MHz Kingston | 五档同值 |
| OS | Microsoft Windows 10 IoT 企业版 LTSC，10.0.19044（Build 19044），64 位 | 五档同值 |
| rustc | rustc 1.98.1 (48a229cea 2026-09-01) | 五档同值 |
| cargo | cargo 1.98.1 (797e8a9bc 2026-08-05) | 五档同值 |
| rust-toolchain.toml | channel = "1.98.1"（t008 档附全文三行：注释 + [toolchain] + channel） | 五档同值（详略差异：t008 逐行、其余摘要） |
| bevy lock | bevy 0.19.1（注：sim 经 'bevy'(default-features=false) 最小面、render-spike 经别名 bevy_full(默认特性全量) 同锁——该注仅 t010 档此表述，其余四档注为「bench/sim 代码路径零 bevy API 参与」类表述） | 五档同值 0.19.1，注释措辞分行如实列出 |
| 构建 profile | Cargo.toml [profile.release] 声明行 debug = false（opt-level 未声明 = 默认 3；lto 未声明 = 默认 off；.cargo/config.toml 不存在）；t008/t010 档无 profile 节 | t007/t009/t015 档含 profile 自证节；t008/t010 档无此节（如实列） |

### 分卡指纹表（各自列出不合并）

| 卡 | 环境档路径 | 环境档生成时间 | git HEAD / 基线（档内所得） | 量测窗口空闲声明指针 | 二进制 sha256（档内值） |
|---|---|---|---|---|---|
| T007 | docs/evidence/t007/environment.txt | 2026-10-04 15:28:02 +08:00 | 档内无 git HEAD 字段 | docs/evidence/t007/runs/preflight.txt（矩阵窗口 2026-10-04 15:28）+ docs/evidence/t007/runs/takeover_preflight.txt（收尾窗口 2026-10-05 08:08；见 t007/README.md §4） | bench.exe `D8E251B06C4FFA01F94527B1E5730A139BF24573B35A703A72DDF5D7D73DF089`（2026-10-04 采集态；后续两态见 t007/README.md §5：`09971df2115e0b9157e02ea190bbbcc22c18bca4f6165504a64d6a636e15e749`（产出 summary.md 态）、`80ebccb0cabb7c607d60eb529548e5c1f9b2de6c625cacb22cd482467123af0d`（提交态）） |
| T008 | docs/evidence/t008/environment.txt | 2026-10-05 10:04:57 +08:00 | b2dad10a438d295bdcfbdfd5cfd7bce0d5a8b917（档内 git HEAD 字段） | 档内声明行：「声明: T008 量测窗口机器空闲独占（2026-10-05）」 | sim.exe `C8006C0A104507B8014612EDC16736928C989D3E31083130988A39F462827EE9`；bench.exe `80EBCCB0CABB7C607D60EB529548E5C1F9B2DE6C625CACB22CD482467123AF0D` |
| T009 | docs/evidence/t009/environment.txt | 2026-10-05 22:46:12 +08:00 | 基线 commit 45a0741（t009/README.md 执行信息行；环境档无 HEAD 字段） | docs/evidence/t009/runs/precheck-*.out（r1~r6 为操作者手工预检 + post-run 补充 precheck-post-run.txt；r7 起 precheck-r7-env.txt 等——t009/README.md 披露节 2） | arena.exe `B541460316D464B5DB0AC9E64DA3BE67B34E693C970E4EE5667F5AAA018DE062`；sim.exe `6E113DC1FBFB401FC91EB2213810DE3589F2DCD0B332735FF121965CD01507D9` |
| T010 | docs/evidence/t010/environment.txt | 2026-10-06 00:42:16 +08:00 | 基线 commit 5ab440b（t010/README.md 执行行；环境档无 HEAD 字段） | docs/evidence/t010/runs/r0_precheck/idle.log（采集前后；t010/README.md 环境披露节） | render-spike.exe `264A95931B74D81932DBC2B5B115BC08CF13FF74A0B97FF9AA1278FEC4605272` |
| T015 | docs/evidence/t015/environment.txt | 2026-10-05 21:15:07 +08:00 | 树 t015-a 基线 1c4dd87（t015/README.md 结构行；环境档无 HEAD 字段） | docs/evidence/t015/runs/preflight.txt（含 preflight_tasklist.csv；t015/README.md 执行事件节） | bench.exe `5A2312590216D9002BE91F1E9785DBE2B0EB43ACA9E755A9745EB6F3E79D48D3` |

### 基准机 A 固化出处

docs/万阵-游戏前期策划报告.html:490 表 6-0 题注（逐字）：「表 6-0 · 测试基准机与口径定义 · 基准机 A 已固化（2026-09-29，开发机本机）；基准机 B 待 M2 前锁定」。

## §8 首周投入校准（快照 + 终判条款）

### 口径段

报告 06 章 R7 规则（docs/万阵-游戏前期策划报告.html:471，转述）：M0 按全职三周执行，首周实测投入不足 30 小时即降档为 20h/周、周期拉长一倍，验收阈值不变只改日历；R7 补充（同文档 :543）：20h/周仍不可达则 M0 无限期挂起、回归工作流主线。投入口径 = 2026-10-05 grill 定案（AGENTS.md L12，逐字）：「**投入口径（2026-10-05 grill 定案）**：严格字面——台账挂钟全计（主会话+worker+审核轮，含等待）、按日历窗口切分（预备周内完成的任务不计 W1，跨窗口任务按实际执行日期分摊）；降档若触发即如实接受；推进按依赖链自然节奏，不为凑时数灌水。」

### 逐任务表（耗时只从 task-ledger.md「## 记录」耗时列解析；解析器 = tally_hours.py）

| 编号 | 任务（摘要） | 窗口归属 | 耗时（台账） |
|---|---|---|---|
| T001 | 仓库骨架搭建与台账启用 | 预备周（不计 W1） | ≈150 min |
| T002 | M0-01 最小确定性 tick 循环 | 预备周（不计 W1） | ≈15 min |
| T003 | M0-02 六兵种数据与单 lane 移动 | 预备周（不计 W1） | ≈18 min |
| T004 | M0-03 索敌攻击与克制结算 | 预备周（不计 W1） | ≈35 min |
| T005 | M0-04 胜负判定与终局 | 预备周（不计 W1） | ≈110 min |
| T006 | M0-05 并行化 v0 与两阶段更新 | 预备周（不计 W1） | ≈130 min |
| T007 | M0-06 基准量测套件 | 跨预备周/W1 拆分（D3-1）：10-04 worker-2 执行段 99 min 归预备周；10-05 接管收尾 65 + 审核轮 25 + 尾差 1 = 91 min 归 W1 | ≈190 min |
| T008 | M0-07 同种子重放确定性验证 | W1 | ≈605 min |
| T009 | M0-08 平衡实验场 v0 与吞吐量测 | W1 | ≈145 min |
| T015 | R2 优化轮一 | W1 | ≈125 min |
| T010 | M0-09 渲染 spike | W1（跨 10-05/10-06 两日均在 W1 窗口内，整卡计入） | ≈140 min |
| T011 | 本卡（M0 数据自含成档） | W1 | 本卡，收口回填 |

注：「≈」合法性边界 = 台账口径为分针近似（task-ledger.md 纪律行「耗时以分针近似」），仅投入耗时可用；验收级数字禁用。
T007 拆分依据（D3-1 补裁决，Lead G1 修正 2026-10-06）：投入口径句「跨窗口任务按实际执行日期分摊」（AGENTS.md）——T007 是唯一跨预备周（10-04）→W1（10-05）边界的卡（证据：t007/environment.txt 生成时刻 2026-10-04 15:28 = worker-2 段在 10-04；runs/takeover_preflight.txt 2026-10-05 08:08 = 接管段在 10-05）；阶段分解取台账 T007 行（worker-2 ≈99 + 接管 ≈65 + 审核 ≈25 = 189 ≈ 190，尾差 1 min 归 W1 侧）。设计侧初版（worker 产出）整卡归 W1 属 D3 疏漏，如实归因并修正。

### 小计行

- 预备周小计（T001~T006 共 458 + T007 的 10-04 段 99，不计 W1）：458+99 = 557 min = 9.28 h。
- W1 累计（T007 的 10-05 段 91 + T008/T009/T015/T010 四卡 + **T011 本卡 140**，截至 T011 收口 2026-10-06）：91+605+145+125+140+140 = 1246 min = 20.77 h。
- 解析器输出留痕：runs/tally.txt（三段——worker 双盲段：手工速算 458/1205 = 脚本 458/1205（整卡口径，D3 疏漏版）；G1 修正段：557/1106（D3-1 拆分口径，T011 未计）；收口段：557/1246（T011 ≈140 min 回填后终态））。

### 判定快照行

`W1 终判未到期（窗口 2026-10-05~10-11，截至 T011 收口 2026-10-06 累计 20.77 h < 30h）——不可终判；W1 剩余窗口（~10-11）内继续投入按台账滚动，周期末由 Lead 回写终判行`

### 终判条款

W1 收口（2026-10-11 窗口结束）后由 Lead 回写终判行；判定式 = W1 全周累计 <30h → 触发降档（20h/周、周期拉长一倍、验收阈值不变只改日历），如实接受；若 20h/周仍不可达 → M0 无限期挂起、回归工作流主线（R7 补充）。数据源 = 本表 + task-ledger.md（唯一数据源）。

## §9 宣称-证据对齐自查

- verify_claims.py 运行记录：全 PASS——`汇总: 81 条 | PASS 81 | FAIL 0`，REAL_EXIT=0（运行时刻 2026-10-06T08:50:23+08:00，树内终态复跑；整改后主仓复跑同 81/81 见 runs/verify-g1.txt——审核轮 M-2 补档，内嵌时刻与 REAL_EXIT 行；逐条 PASS/FAIL 截录见 runs/verify.txt；CLAIMS = 派工单 §2 清单 A~P 全部锚子串 42 条 + 引用点扩充 39 条，双向核对 = 源档命中 ∧ 本档命中）。
- 措辞自查：本档验收级数字无「约/大概」级措辞——全部判定行逐字引自代码生成档；本档自身产出的两处计算（投入小计与 670.6/60 换算）均给出算式；「≈」仅出现在投入耗时口径（台账分针近似）与**引用原文**中（引用档自带字符，如「≈ 3.9×」，非本档措辞）。
- AGENTS.md「当前状态」节验收级数字抽查（4 项，grep -F 对照；详录 runs/../selfcheck.md §3）：333.15 / 202.48 / 0.061 系三项逐字一致；吞吐项 AGENTS.md 作「1,099,639 场/h」（整数取整形态），源档判定行为 1099638.6——取整一致、字面不同，如实记录（AGENTS.md 为状态摘要档，本卡禁改，是否回改精度移交 Lead）。
- 措辞与覆盖自查详档：selfcheck.md（四节：verify 全 PASS 声明 / 措辞自查 / AGENTS.md 抽查 / 环境总档五档一致性观察）。

## §10 抽查重跑记录（验收断言 2：证据档内命令可直接重跑）

执行前置门禁：`cargo build -p sim --release -j 3`（树根执行，REAL_EXIT=0；产物 sim/bench/arena 三 bin）。三档命令均相对树根、零机器绝对路径。

### 档 a · t008 局（red200-th1-s42）

- 命令来源：docs/evidence/t008/runs/plan.txt:51（与 runs/red200-th1-s42.cmd 档逐字一致）。
- 命令全文：`./target/release/sim.exe --comp "shieldman:17,heavyknight:17,pikeman:17,swordsman:17,archer:16,militia:16" --ticks 1800 --hash-samples 0,450,900,1350 --threads 1 --seed 42`
- 期望：stdout 与归档 docs/evidence/t008/runs/red200-th1-s42.stdout 逐字节一致（diff 空）。
- 实测：diff 空（DIFF_EXIT=0，留痕 runs/spot-t008-red200-th1-s42.diff 0 差异）；终局 `hash=0xde91d6a5a6e84d43` 与 §4 终局哈希表 red200 s42 行一致；采样 1350 点 `0xe0cd7861ca5a211f` 与 t008/matrix.md §1 表一致。
- REAL_EXIT=0（留痕 runs/spot-t008-red200-th1-s42.cmd 尾行）。

### 档 b · bench 单配置（10k × t1 × 300t × seed 42）

- 命令来源：AGENTS.md 常用命令节 bench 用法行（`./target/release/bench.exe --units <N> --threads <T> --ticks <K> [--warmup <W>=1] [--repeats <R>=5] [--seed <S>=42]`）；配置与 docs/evidence/t015/summary.md 附录 A 及 docs/evidence/t007/summary.md 附录 A 的 10000×1×300 行同配置（派工单 D4-b）。
- 命令全文：`./target/release/bench.exe --units 10000 --threads 1 --ticks 300 --seed 42`
- 期望：stdout 单行 JSON 中 `final_hash` == `0xc5915d042208e267`（T007/T015 双档附录 A 同值）。
- 实测：final_hash=0xc5915d042208e267，MATCH（留痕 runs/spot-bench-10k-t1.check）。
- 口径注（D4）：只核哈希确定性，不比耗时——计时数字属量测窗口产物，不在本卡重判。
- REAL_EXIT=0。

### 档 c · t009 CLI 交叉腿（sim_t1）

- 命令来源：docs/evidence/t009/run_t009.sh:85（与 docs/evidence/t009/runs/r5-sim-t1.cmd 档逐字一致；crosscheck_r5.sh:16 即取该局 stdout 判定）。
- 命令全文：`./target/release/sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 1`
- 期望：final_hash == `0x564cf46fdf191710`（t009/summary.md CLI 黄金交叉：arena=sim_t1=sim_t12 三方同值）。
- 实测：stdout 末行 `final_hash=0x564cf46fdf191710`，MATCH（留痕 runs/spot-t009-simt1.check）。
- REAL_EXIT=0（留痕 runs/spot-t009-simt1.cmd 尾行）。

### 未重跑重型档的复跑旁证指针（D4；审核轮 P2-2 补档）

t010/t015 两档不重跑（bevy full 冷编 19m45s 与 -j 2 门槛成本不值为本卡重复），既有复跑旁证在库可点开：

- t010：docs/evidence/t010/g1-lead-recheck.txt（Lead G1 独立复跑 t10000 fresh：avg 329.33 / 1% low 177.95 PASS 独立确认）+ docs/evidence/t010/review-plan-code-reviewer.md（审核轮四类独立复算：四档重算 / 黄金 8 值第三实现 / 端到端短采集 / 复构建 sha256 指纹）。
- t015：docs/evidence/t015/g1-lead-recheck.txt（门禁 43/43 + 五组锚点独立复验 + T008 对拍矩阵复跑）+ docs/evidence/t015/review-plan-code-reviewer.md（审核轮独立复算 + 全量 29 局 cmp 0 mismatch）。
