# T009 证据档 · 平衡实验场 v0 与吞吐量测（M0 验收③）

- 任务：T009（WP-A 一体单：实现 + 门禁 + 量测 + 证据）；执行者 worker-1（隔离树内执行，收获由 Lead 完成）
- 执行日期：2026-10-05；基线 commit `45a0741`（master）
- 量测口径：基准机 A = 开发机本机（i5-12490F / 32GB / RTX 3050，见 `environment.txt`）；release 构建；tick 固定 30Hz

## 交付物

| 文件 | 说明 |
| --- | --- |
| `sim/src/bin/arena.rs` | 新增第二实验场 bin（`--matrix` / `--throughput` / `--sampling` 三模式；零 manifest 改动、零新依赖、stdout 手写 JSON） |
| `sim/src/world.rs` | **只新增** `World::deploy_versus`（含文档注释）+ tests 末尾追加 W1 单测；既有行零改动（自查与逐行 diff 复核见下「零改动自查」） |
| `docs/evidence/t009/` | 本档：README + summarize.py + run_t009.sh + crosscheck_r5.sh + precheck_load.ps1 + collect_environment.ps1 + summary.md + environment.txt + runs/ |

## 门禁（R0，全过；明细见 runs/r0-*）

| 门禁 | 命令 | 结果 |
| --- | --- | --- |
| check | `cargo check --workspace -j 3` | REAL_EXIT=0，0 警告（首验 attempt 1 = 101：arena usage 串 `{T}` 被格式化解析 E0425，修一处分歧后 attempt 2 = 0，留痕 exits.txt） |
| test | `cargo test -p sim -j 3` | REAL_EXIT=0，**47 全绿 = 既有 43（原值零改动）+ 新增 4**（W1 world.rs；W2/W3/W4 arena.rs） |
| build sim | `cargo build -p sim --release -j 3` | REAL_EXIT=0 |
| build arena | `cargo build -p sim --bin arena --release -j 3` | REAL_EXIT=0 |

## 跑批（run_t009.sh，断点续跑；逐批命令全文 + stdout/stderr + REAL_EXIT 见 runs/）

| 批次 | 内容 | REAL_EXIT | 关键结果 |
| --- | --- | --- | --- |
| r1 | 快检 `--matrix --per-side 10 --per-cell 2 --threads 4` | 0 | 72 局、wall 0.05s；JSONL 局索引序、种子闭式逐格核对 |
| r2 | 口径层 `--per-side 100 --per-cell 100 --threads 12` | 0 | 3600 局 11.26s；resolved 0/3600 |
| r3 | 探针层 `--per-side 10 --per-cell 100 --threads 12` | 0 | 3600 局 1.52s；resolved 800/3600 |
| r4 | 吞吐四档 `--threads {1,3,6,12} --repeats 3 --games 512` | 0 | 见下判定行；四档 final_hash_xor 全等（跨线程确定性旁证） |
| r5 | CLI 黄金交叉（arena sampling row0 vs sim ×threads{1,12}） | 0 | 三方 final_hash `0x564cf46fdf191710` 逐位一致 → PASS |
| r6 | 全规模 `--sampling --games 100 --threads 12` | 0 | 100 局 1321.4s（单局中位 13.12s、区间 10.4~18.6s）；resolved 0/100 |
| r7 | 环境档 collect_environment.ps1 | 0 | environment.txt（含 arena.exe / sim.exe sha256） |
| r8 | summarize.py → summary.md | 0 | 判定行全脚本计算 |

## 判定行汇总（详见 summary.md；全部由 summarize.py 计算）

- **吞吐判定行**：games_per_hour@12t = 1,099,638.6（阈值 ≥ 10,000）→ **PASS**；四档 1/3/6/12 = 166,557.9 / 461,088.1 / 793,548.1 / 1,099,638.6；16 线程外推（elapsed(T)=c1+c2/T OLS）= 1,405,967.6 场/h
- **镜像 sanity 判定行**：两层对角 6 格全部 |z| = 10.000 > 1.96 → **H2 结构性偏差形态**（红先手 100% 胜；三态分布已逐格披露——v0 战斗结算按索引序红块先手，「先手击杀剥夺后手目标选择」在镜像对局中系统性偏红；派工单预注册 H1/H2 双假设之一，如实判定）
- **接敌实证判定行**：口径层 36 格 max avg 存活(和) 194.00 < 200 → PASS；探针层 max 14.00 < 20 → PASS；全规模层 100 局 max 9980 < 10000 → PASS
- **分层方向一致判定行**：33/36 cell 三层方向一致（91.7%）；不一致格 29/34（archer-militia 方向随规模翻转）与 35（militia 镜像全规模层 hp-sum 同值 Draw）
- **击溃率（v0：有胜方局中 胜方存活 ≥0.8×per_side）**：口径层 100.0%（3600 有胜方局）/ 探针层 50.0% / 全规模层 100.0%（98 有胜方局）
- **CLI 交叉判定行**：三方 final_hash 逐位一致 → PASS

## 预注册预期对照（派工单[时间盒·预期登记]；结果如实披露）

| 预注册 | 实测 | 结论 |
| --- | --- | --- |
| 吞吐@12t ≥ 100,000 场/h 量级 | 1,099,638.6 场/h | 超预期一个量级 |
| 镜像 sanity H1(50/50) / H2(结构性红偏或 Draw 主导) 双假设 | H2 结构性红偏（对角红 100%） | 如实判定为 H2 |
| 采样单局 ≈11.6s（±100% 区间 [0, 23.2]） | 中位 13.12s | 区间内 |
| 口径层多数格 resolved=false（v0 单 lane 序贯决斗 1800t 截断 hp-sum 主导） | resolved 0/3600 | 与预注册设计预期一致 |

## 披露（与派工单的偏离与过程缺陷，全部如实留痕）

1. **W4② 参考值自相矛盾**（不阻断，按 B.2⑤ 双盲上报）：派工单参考链「首杀 269+(ceil(50/6)−1)×20=269+140=409」中 (ceil(50/6)−1)×20 = 8×20 = **160**（首杀 ≈429），非 140/409——派工单自身公式与速算不一致；W4 断言按其自身「断言以实测为准」条款以实测落断言（② 必接敌、全歼早于 cap，实测通过），未采用任一速算值作断言。
2. **precheck_load.ps1 首版编码缺陷**：PowerShell 5.1 将无 BOM UTF-8 中文注释按 ANSI 解析报错 → r1~r6 各批 per-batch precheck .txt 未生成（错误日志留痕 runs/precheck-*.out）；该窗口的空闲声明 = 操作者开跑前手工预检（进程表仅 ZCode 框架遥测 pythonw.exe、CPU 负载 3%）+ 全程单脚本串行独占窗口 + 跑批后补充预检（runs/precheck-post-run.txt，(no match -> idle)）。脚本已改 ASCII-only，r7/r8 起正常落档（runs/precheck-r7-env.txt 等）。
3. **collect_environment.ps1 无 BOM** 会致输出乱码（同源缺陷，未发生实际损坏）——已补 UTF-8 BOM 后执行（r7 REAL_EXIT=0）。
4. **summarize.py 区间标注修正**：首版把「偏离 ±100%」误算为 ±50% 区间（[5.8, 17.4]）；实测 13.12s 两区间均在内、判定方向不受影响，脚本修正为 [0.0, 23.2] 后重生成 summary.md（台账行 r8-summary-regen REAL_EXIT=0；数据输入零改动）。
5. **机器路径清洗**（附录 G「只改前缀、内容原样」先例）：runs/ 下 17 个文本档（cargo 编译输出 3 + precheck 失败日志 14）含机器绝对路径前缀，已仅摘除前缀 `C:\Users\...\t009-a\`，命中行内容原样留痕。
6. **matrix `--per-side` 合法域收窄为 {10, 100}**（exit 2 拒绝其他值）：派工单只定义了两层种子基（1_000_000 / 1_500_000），其他 per_side 的种子空间未定义——按「冲突/解析错误 → exit 2」纪律拒绝，未自创新口径。
7. **throughput JSON 增设 `final_hash_xor` 字段**（派工单键集之外的一个附加键）：防优化器省略 + 跨档确定性旁证（四档全等 `0xd1b28b373f7791e3`）；其余键与派工单样例逐键一致。
8. run_t009.sh 未实现批次前缀过滤（与 t008 脚本的差异）：本卡批次一次串行跑完，全部 REAL_EXIT=0，未触发断点续跑分支；幂等语义保留。

## 零改动自查（worker 自查声明；收获时由 Lead 逐行 diff 复核）

- `sim/src/world.rs`：仅两处**纯插入**——`deploy_versus`（impl World 内、deploy 之后）+ tests 模块末尾 W1；既有 2100 行零改动（`git diff --no-index` 复核：删除行 0）。
- 其余交付清单外文件零改动（`diff -rq` 主仓基线 vs 隔离树，排除本档新增目录与 target/，输出为空）。
- 既有 43 单测期望值原值原绿（r0-2-test）。

## 复现指引

```bash
cargo test -p sim -j 3                      # 47 全绿
cargo build -p sim --release -j 3
./docs/evidence/t009/run_t009.sh            # 断点续跑：exits.txt 已有 REAL_EXIT=0 的批次自动跳过
python docs/evidence/t009/summarize.py --out docs/evidence/t009
```
