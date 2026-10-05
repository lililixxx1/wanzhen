# T009 派工单 · WP-A（实现+门禁+量测+证据一体单）

[执行者] worker-1 · 隔离树 `C:\Users\Administrator\Desktop\ccc\trees\wanzhen\t009-a\`（下称 `<TREE>`，已由 Lead 建好，树内无 .git——git 命令天然不可用，「只新增不改既有行」由你自查 + Lead 收获时逐行 diff 复核）

[目标] T009 平衡实验场 v0 与吞吐量测（验收③）：sim 新增第二 bin `arena`（胜率矩阵/吞吐/全规模抽样三模式）+ lib 增量 `World::deploy_versus`，产出全部证据档。

[基线 commit] 45a0741（master）

[交付物]
1. `<TREE>\sim\src\bin\arena.rs`（新文件）
2. `<TREE>\sim\src\world.rs`（**只允许两种改动**：①新增 `deploy_versus` 函数含文档注释；②文件末尾 tests 模块内追加新测试函数。其余既有行零改动）
3. `<TREE>\docs\evidence\t009\` 全档（README.md + summarize.py + runs\ + 环境档 + 跑批脚本）

[环境条款引用]（先 Read 再用；该文件为机器态不入库——你的任何入库产物禁止出现其绝对路径或内容转写）
`C:\Users\Administrator\Desktop\ccc\wanzhen\team-prompt\PROJECT-APPENDIX.md` 附录 A（cargo -j 3 串行 / 量测空闲独占 / 门禁清单）、B.1（误报清单）、B.2（数值纪律固化单）、C（锚点保真）、G（证据档规范）

[门禁]（附录 A 清单条目；全部在 `<TREE>` 根执行、串行、期望退出码 0；B.1 纪律：验收命令单独整句执行取退出码，不接管道 `echo $?`）
1. `cargo check --workspace -j 3`（0 警告）
2. `cargo test -p sim -j 3`（全绿：既有 43 项原值零改动 + 新增全过）
3. `cargo build -p sim --release -j 3`
4. `cargo build -p sim --bin arena --release -j 3`

[实现规格]（设计裁决已冻结；本节自含，勿另创口径）

## §1 `deploy_versus` 契约（world.rs 新增）

```rust
pub fn deploy_versus(seed: u64, red: &[(UnitKind, usize)], blue: &[(UnitKind, usize)], lane_q32: i64) -> World
```

- 红方：展开 red 清单为 `seq_red`（清单顺序=块顺序）；Fisher-Yates 全洗，洗牌 RNG `Xoshiro256StarStar::from_seed(seed ^ DEPLOY_SALT)`，`for i in (1..m_r).rev() { j = next_u64() % (i+1); swap(i,j) }`——**与 `deploy` 现行洗法逐字同式（执行前 Read world.rs `deploy` 实现对照，禁凭本单转写写代码）**。
- 蓝方：同式独立洗 `seq_blue`（新洗牌实例、同 seed 同盐——red==blue 时两洗一致 ⇒ 镜像等价性来源）。
- 队列位置：红 `x_red[k]` = 队头 `radius_0`、其后 `x_{k-1} − (r_{k-1}+r_k+GAP_Q32)`（同 deploy 现行式；GAP_Q32 为 world.rs 私有常量，函数在 world.rs 内直接用）；蓝 `x_blue[k]` 同式由 seq_blue 半径计算；蓝最终位置 `x'_k = lane_q32 − x_blue[k]`。
- units 顺序：红块（索引 0..m_r）后蓝块；tick=0、tick 级 RNG `from_seed(seed)`、`last_hash = state_hash()`。
- **`deploy` 既有函数与既有全部行零改动**（重复内联逻辑可接受——语义参照 + 可 diff）。
- 等价性红线：red==blue 且 lane_q32==LANE_LEN_Q32 时与 deploy 逐位一致（单测 W1 断言）。

## §2 `arena.rs` 规格（新 bin，零 manifest 改动——bin 自动发现；零新依赖，stdout 手写 JSON 格式化，禁 serde）

常量：`LANE_SCALED_Q32 = 50 * sim::units::ONE_Q32_32`（units.rs:23 pub）；`SEED_MATRIX_BASE=1_000_000`、`SEED_PROBE_BASE=1_500_000`、`SEED_THROUGHPUT_BASE=2_000_000`、`SEED_SAMPLING_BASE=3_000_000`；`KINDS: [UnitKind; 6] = {Shieldman, HeavyKnight, Pikeman, Swordsman, Archer, Militia}`（判别值 0..5，units.rs:48-53——执行前 `grep -n` 核对）。

模式（互斥，冲突/解析错误 → stderr + usage + exit 2，同 sim main.rs 纪律；`--out <dir>` 必选，不存在则创建）：

**`--matrix [--per-side N=100] [--per-cell K=100] [--threads T=1] --out DIR`**
- 局清单：`cell_idx = i*6 + j`（i=红兵种序、j=蓝兵种序）、k in 0..K；`seed = base + cell_idx*K + k`（base=1_000_000 当 N==100、1_500_000 当 N==10——两层数据分别调用产出）。
- 每局：`deploy_versus(seed, [(KINDS[i], N)], [(KINDS[j], N)], LANE_SCALED_Q32)` → `run_battle(TICK_CAP_REDUCED)`（串行 None 路径；TICK_CAP_REDUCED=1800 为 world.rs pub 常量）。
- 跨局并发：`ThreadPool::new(T)`（T=1 不建池直跑）+ `map_chunks(36*K, |s,e| …)` 按局分片（pool.rs:95 签名 `Fn(start,end)->Vec<T>`——执行前 Read 该函数核对签名）。
- JSONL 每局一行（**顺序=局索引序**，文件名 `matrix_per100.jsonl` / `matrix_per10.jsonl`）：
  `{"layer":"matrix_per100","cell":0,"red":"shieldman","blue":"heavyknight","per_side":100,"lane_m":50,"seed":1000000,"winner":"red","end_tick":1800,"alive_red":97,"alive_blue":95,"resolved":false,"final_hash":"0x0000000000000000"}`
  —— `resolved = (alive_red==0 || alive_blue==0)`（全灭收束 ⟺ 一方零存活；上限 hp-sum 分支双方必 >0）。

**`--throughput [--games G=512] [--threads T=12] [--repeats R=3] --out DIR`**
- 局清单：g in 0..G：cell = g % 36、per_side=100、lane scaled、`seed = 2_000_000 + g`、cap 1800。
- warmup：16 局（seed = 1_900_000 + w，**不计时、不进数据**）。
- 计时批 r in 0..R：`t0=Instant::now(); rows = pool.map_chunks(G, |s,e| play(s..e)); dt = t0.elapsed();`——**计时窗只包对局循环 + 内存收集**；stdout / 文件 IO 一律窗外（验收断言 4 代码路径指认：本行行号入档）。
- 输出 JSON 文件 `throughput_t{T}.json`：`{"games":512,"threads":12,"warmup":16,"wall_s":[…3 值…],"wall_s_median":m,"games_per_hour":G*3600/m,"per_game_ms":…,"ai_decision_cost":0}`（ai_decision 单列 0 = 表 6-0「AI 池扩档后须重测」钩子字段——镜像 AI v0 无独立决策模块）。

**`--sampling [--games G=100] [--threads T=12] --out DIR`**
- g in 0..G：cell = g % 36、每方 5000、lane = `LANE_LEN_Q32`（world.rs:147 原值 1000m）、`seed = 3_000_000 + g*7919`、cap = `TICK_CAP_FULL`（14400）。
- `ThreadPool::new(T)` 跨局复用；每局 `run_battle_with(14400, Some(&pool))`；局间串行；每局另记 per-game wall_s（Instant 包单局，仅数据不进吞吐口径）。JSONL 同上格式，`layer:"full_per5000"`、`lane_m:1000`、文件名 `sampling.jsonl`。

stdout：每模式结束打**单行 JSON 摘要**（bench 纪律）。

## §3 `summarize.py`（python 3，入库 `docs/evidence/t009/`，判定行全部脚本计算、禁手算）

输入 `--out` 目录 → 生成 `summary.md`：
1. 矩阵表：两层数据各一张 6×6（红胜%/蓝胜%/Draw%/resolved%/avg_end_tick/avg 存活）。
2. 镜像 sanity 判定行：对角 6 格（每层）蓝胜率 pooled 二项 `z = (p_blue−0.5)/sqrt(0.25/n)` + 每格 95% CI `±1.96*sqrt(p(1−p)/n)`；|z|≤1.96 →「50/50 一致」，否则「H2 结构性偏差形态」+ 三态分布（红%/蓝%/Draw%）披露。
3. 吞吐判定行：`games_per_hour@12t`（中位）≥ 10_000 → PASS/FAIL（R2 阈值）；附 1/3/6/12 四档表 + 16 线程外推（`elapsed(T)=c1+c2/T` 双参数最小二乘拟合四档中位 → T=16 外推 → 吞吐16 = G*3600/elapsed(16)）。
4. 接敌实证判定行：口径层 36 格全部 `avg(alive_red+alive_blue) < 2*per_side`；探针层同式 < 20；全规模层 100 局全部 `alive_red+alive_blue < 10000`。
5. 分层比对表：逐 cell 三层胜方方向符号一致比例（全规模每 cell ≤3 局，方向=多数胜方、无多数记 mixed，小样本注明）；击溃率（v0 定义：胜方存活率 ≥80% 局占比，三层各列）；avg_end_tick 三层对比。
6. CLI 交叉判定行：§4 R5 三方 final_hash 比对结果。

## §4 量测跑批（release 构建；机器空闲独占——每批开跑前预检声明留档「当前无其他 cargo/重负载进程」；每档一 runs/ 档：命令全文 + 原始输出 + REAL_EXIT）

- R0 门禁四条（上列）先行落档
- R1 快检：`arena --matrix --per-side 10 --per-cell 2 --threads 4 --out <smoke>`（秒级）
- R2 口径层：`--per-side 100 --per-cell 100 --threads 12`
- R3 探针层：`--per-side 10 --per-cell 100 --threads 12`
- R4 吞吐四档：`--threads {1,3,6,12}` × `--repeats 3`
- R5 CLI 黄金交叉：`arena --sampling --games 1`（row0 = cell0/shieldman:5000 双方/seed 3000000/lane1000）vs `<TREE>\target\release\sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400` 的 `--threads 1` 与 `--threads 12` 两次——**三方 final_hash 逐位一致**（sim stdout BattleLog 含 final_hash 行，grep 取值；档存命令+输出+比对结论）
- R6 全规模：`--sampling --games 100 --threads 12`（参考 ≈20 min±50%）
- R7 环境档：CPU/GPU/RAM/OS/rustc/二进制 sha256（arena.exe + sim.exe；复刻 `<TREE>\docs\evidence\t007\collect_environment.ps1` 同式，输出零机器绝对路径）
- R8 `summarize.py` 全量 → summary.md
- 跑批脚本 `run_t009.sh`（bash、断点续跑：产物存在即跳过、全相对路径）

## §5 单测（arena `#[cfg(test)]` + world.rs tests 末尾追加）

- **W1**（world.rs）deploy_versus 镜像等价：构型 `{[(Shieldman,10)], [(HeavyKnight,2),(Militia,3)], [(Militia,5)]}` × seeds `{42, 43, 1_000_007}`：`deploy_versus(s,c,c,LANE_LEN_Q32).last_hash == deploy(s,c).last_hash`，且各自 `run_battle(1800)` 后 final_hash 逐位相等。
- **W2**（arena）确定性+跨线程：同参双跑 state_hash 相等；`(HeavyKnight,3)` vs `(Militia,3)`、lane scaled、cap 1800：threads {1,4} run_battle final_hash 相等。
- **W3**（arena）种子算式自检：matrix 局清单前 5 局 seed == 闭式 `base + cell*K + k` 逐个断言；sampling 前 3 局 == `3_000_000 + g*7919`。
- **W4**（arena）resolved 语义两构型：①必截断 = mirror `(Shieldman,2)` lane=LANE_LEN_Q32、cap 1800 → t_e=(1000−2.0)/0.10=9980>1800 不接敌 → resolved=false（双方存活、镜像同和 → Draw）；②必接敌 = mirror `(Militia,2)` lane=LANE_SCALED_Q32 → t_e=(50−1.6)/0.18=269、首杀 269+(ceil(50/6)−1)*20=269+140=409<1800、全歼两链 ≈1000<1800 → resolved=true（参考值——断言以实测为准）。

## §6 接敌可达性参考表（lane=50m；t_e=(50−2*(rR+rB))/(vR+vB)；源=units.rs 速度/半径表）

21 对（m、m/tick、参考 tick）：shieldman-shieldman 480 · shieldman-heavyknight 190 · shieldman-pikeman 320 · shieldman-swordsman 283 · shieldman-archer 371 · shieldman-militia 345 · heavyknight-heavyknight 117 · heavyknight-pikeman 158 · heavyknight-swordsman 149 · heavyknight-archer 170 · heavyknight-militia 165 · pikeman-pikeman 240 · pikeman-swordsman 219 · pikeman-archer 268 · pikeman-militia 254 · swordsman-swordsman 200 · swordsman-archer 241 · swordsman-militia 230 · archer-archer 303 · archer-militia 285 · militia-militia 269。全部 < 1800 ✓。
最坏首杀链（S,S 镜像）：480 + (ceil(120/8)−1)*30 = 480+420 = 900 < 1800 ⇒ 口径层每格必有死亡（断言 4 依据）。
全规模层（lane=1000）：最慢对 S,S t_e = (1000−2.0)/0.10 = 9980 < 14400、首杀 = 9980+420 = 10400 < 14400 ⇒ 100 局必有死亡（T008 full 终局 9959/9969/9970 旁证）。

## [数值纪律与双盲]（附录 B.2⑤）

本单全部速算参考值（§5/§6 预期、吞吐量级、采样时长）**仅供人眼核对，以代码/脚本/实测为准；任何不一致 = 立即停止上报，不得自行取舍**。行号引用（units.rs:23/48-53、world.rs:147/158/162、pool.rs:95）执行前逐条 `grep -n` 核对，对不上即上报。

## [写范围] / [禁改]

写范围 = 交付物三处。禁改 = 其余一切（含 `sim/src/{main,units,rng,hash,pool,spatial,lib}.rs`、`sim/Cargo.toml`、根 `Cargo.toml`、`Cargo.lock`、`.gitignore`、`taskset/**`、`AGENTS.md`、`task-ledger.md`、`docs/evidence/` 其他任务档——只读）。

## [上报条件]（任一即停并上报，附命令+原始输出）

既有 43 单测任何失败或需改期望值（锚点红线，不许修）；W1 镜像等价失败（契约疑义，不许自行改 deploy）；CLI 交叉三方不一致；断言 4 任一格/局零死亡；吞吐 <10_000 场/h；采样单局均值偏离 ±100%；cargo 假失败嫌疑（B.1 形态——自查并发纪律后上报）；同一异常 ≤2 次尝试；配额/中断类报错直接上报。

## [时间盒 · 预期登记]

时间盒 ≤3h，超盒即上报不默认续做。预注册预期：吞吐@12t ≥100_000 场/h 量级（判定线 10_000）；镜像 sanity 双假设 H1(50/50)/H2(结构性红偏或 Draw 主导) 皆如实判定；采样单局 ≈11.6s（T015 同规模同档参考）；口径层多数格 resolved=false（v0 单 lane 序贯决斗 1800t 不可全歼、截断 hp-sum 主导——README 解读如实，此为预注册设计预期非缺陷）。

## [汇报]

按 agent 定义四要素：完成项 / 上报项 / 证据（改动文件、命令与退出码、结果、未决点）/ 偏离——含「既有行零改动」自查声明与单测计数（既有 43 + 新增 N）。
