# AGENTS.md

## 项目定位

《万阵》（工作名）——米拉奇式万人大战自动对战策略游戏。当前处于**立项预验证阶段（M0 先行，身份未定）**：M0 用三周回答三个立项前提——模拟层能并行跑多大规模 / 渲染层灰盒能同屏多少单位 / 实验场吞吐真实量级；数据经独立复算后重开身份决策（商业产品 / M5 试金石砍规模 / 终止）。

**最高上下文：[`docs/万阵-游戏前期策划报告.html`](./docs/万阵-游戏前期策划报告.html)（V0.9.1，2026-09-30 复核裁决「通过——可作为 M0 唯一输入」）**。改动范围、里程碑、验收阈值、确定性纪律前必须先读对应章节（尤其表 6-0 口径定义、5.2 确定性纪律、6.1 止损换算、7.2 Q5 身份决策）；与其冲突时以该文档为准；M0 实测数据回写与初值替换留待 V1.0（收官修订）。

## 当前状态（2026-10-06 上午更新，W1 D2）

- **预备周** 2026-09-30 ~ 10-04：仓库骨架、M0 全量 taskset 拆卡——不计入首周投入观测。
- **M0 三周正计时自 2026-10-05（周一）起**。硬规则（报告 06/R7）：首周（10-05 ~ 10-11）实测投入 <30h → 降档 20h/周、周期拉长一倍，验收阈值不变只改日历；20h/周仍不可达 → M0 无限期挂起、回归工作流主线。投入数据源 = 任务台账逐任务实测耗时。**投入口径（2026-10-05 grill 定案）**：严格字面——台账挂钟全计（主会话+worker+审核轮，含等待）、按日历窗口切分（预备周内完成的任务不计 W1，跨窗口任务按实际执行日期分摊）；降档若触发即如实接受；推进按依赖链自然节奏，不为凑时数灌水。**W1 快照（T011 汇总档收口终态，2026-10-06）**：T007 跨窗口拆分 99/91（D3-1）后 W1 累计 1246 min = 20.77 h（含 T011 本卡 140；预备周 557 min 不计）——终判未到期（10-11 收口回写终判行，docs/evidence/m0/README.md §8）。
- **进度（2026-10-06 上午，W1 D2）**：T002~T010、T015 已交付（M0 六验收①~⑥全有实测判定行）。**T011 数据自含成档已交付（收官序列第 1 步，10-06 上午收口，审核完整轮通过 P0=0/P1=0/P2×2 整改闭环）**：证据总索引 docs/evidence/m0/（六验收判定快照逐字引自代码生成档 + 环境总档五档汇总 + 投入校准快照/终判条款 + verify_claims.py 81 条双向核对 + 抽查三档重跑取证：t008 局 diff 逐字节空 / bench 哈希 0xc5915d042208e267 / t009 sim_t1 0x564cf46fdf191710）；D3-1 补裁决 = T007 跨预备周/W1 拆分（99 归预备周 / 91 归 W1，快照 20.08h→18.43h 如实修正）。六验收终态速览（详见 m0/README §0）：① PASS（四点 0.061~0.109µs 全达承诺线）、② TRIPPED（1.001913×，结构性归因 + owner 裁决点→T013）、②附则极限十万保留（17.53ms≤22ms）、③ PASS（1,099,638.6 场/h@12t ≈110× 裕度）、④ 全 PASS（29 局逐位一致）、⑤ PASS（avg 333.15 / 1% low 202.48）、⑥ 达标（6.8MiB 稳定）。**T015 R2 优化轮一（10-05 收口）**：sim 意图阶段排序序快路径（spatial.rs）等价性 = T008 矩阵 29/29 局逐字节 + bench 16 配置哈希跨版本一致；② @10k 加速比 1.00× TRIPPED（结构性，理论上限 ≈3.9×<4×）⇒ 轮二无法诚实闭合，owner 裁决点（② 口径 vs 意图 + t016 去留，建议 T013 报告级仲裁）。**执行序现状**：T011 已收口 → **下一卡 T012 ★独立复算（收官序列第 2 步）** → T013 报告 V1.0 实测修订（吸收 ② 裁决点 + t009 镜像 sanity 口径仲裁）→ T014 身份决策评审材料。
- **三层架构工作方式（2026-10-05 grill 修订）**：主会话（GLM-5.3）规划 + 监管（拆解 / 派工 / 把关方向 / 处理上报）；worker-1/2（flash 级，模型 owner 侧配置）只按派工单执行、范围外上报不拍板；**worker-2 可并发 1~3 个**（独立卡并行；量测窗口仍机器空闲独占）；执行中断（配额/用户消息）→ 主会话接管 + 台账如实记（连续四卡先例）。**每卡过审、分轻重**（取代原两节点分级；T007 节点轮已过）：代码卡轻量轮（plan-code-reviewer：diff + 门禁三断言核对 + 抽查复跑）；节点卡 / 量测卡 / 主会话接管的卡完整轮；T012 收官独立复算轮不变。
- 仓库卫生：私有起步、**保持可公开态**（零密钥 / 机器路径 / 个人信息；git 统一 bot 身份 `wanzhen <bot@wanzhen.invalid>`）。

## 硬约束（违反即返工）

1. **版本锁**：`bevy = "0.19"`（workspace.dependencies 统一锁定，成员经 `{ workspace = true }` 继承），`Cargo.lock` 入库。版本变更只允许发生在升级窗口；Bevy 0.20 处置挂起（Q5 挂起清单项，身份定案时一并回答）。
2. **禁凭记忆写 Bevy API**：不确定的 API 必须查 docs.rs 对应版本（0.19）或官方 examples。
3. **每次代码变更必须过 `cargo check --workspace`（0 警告）**；模拟行为变更须附确定性验证证据（同种子状态哈希一致）。
4. **模拟态确定性纪律**（报告 5.2，验收④的地基）：模拟态禁止依赖 HashMap 迭代序（Rust RandomState 每次运行不同——改 BTreeMap / 索引数组 / 固定序遍历）；浮点归约按固定索引序执行（并行求和结果不进模拟态）；跨单位间接影响两阶段更新（先并行写标记、下一 tick 按固定序应用，结果与线程数无关）；rustc 工具链钉版（`rust-toolchain.toml`，当前 1.98.1）；超越函数（sin/cos/powf 等）禁入模拟态。
5. **平台仅 PC（Windows 独占）**，语言 Rust（stable 钉版）。不做移动端 / 网络多人 / 编辑器。
6. **量测口径**（表 6-0，不得混用）：基准机 A = 开发机本机（i5-12490F / 32GB / RTX 3050）；tick 固定 30Hz；微秒成本 = **单线程基线**；线程档位 1/3/6/12 实测 + 外推 16；帧预算常态 @60fps 模拟 ≤8ms、极限 @30fps 22+11ms；**降规模对局 = 每方 100 模拟单位（共 200）、单局 ≤1,800 ticks**；止损阈值不放松（每单位每 tick >2µs 或 12 线程加速比 <4× 即触发，6.1 换算公式同表）；量测一律 release 构建。

## 常用命令

- `cargo check --workspace -j 3` — 每次代码变更的门禁（**cargo 编译并行 ≤3、注意内存**——owner 2026-09-30 指令；量测与 doctest 门禁期间机器须空闲独占，避免负载敏感假红，见 docs/evidence 经验）
- `cargo run -p sim --release` — headless 模拟入口（骨架期仅验证工具链）
- `cargo run -p sim --release -- [--comp <kind:count,...> | --units <N>] [--ticks <N>] [--seed <u64>] [--threads <1..=1024>] [--hash-samples <t1,t2,...>] [--battle]` — 对局入口（T006 起）：`--threads` 线程档位（默认 1=串行，stdout 与线程数逐字节无关，`threads=N` 与耗时同打 stderr）；`--hash-samples` 严格升序采样 tick 列表（0=布阵快照哈希；仅 run 路径，×`--battle` 互斥、超 `--ticks` 均 exit 2）
- `cargo build -p sim --bin bench --release -j 3` — M0 量测套件构建（T007：`sim` 第二 bin，零新依赖、lib 零改动）
- `./target/release/bench.exe --units <N> --threads <T> --ticks <K> [--warmup <W>=1] [--repeats <R>=5] [--seed <S>=42]` — M0 量测入口（T007，验收①②⑥）：stdout 单行 JSON（samples_ns 全量/median/CV/us_per_unit_tick/final_hash；同配置多局哈希不一致 exit 4）；`--summarize <matrix.jsonl> [--memory-csv <mem.csv>] [--out <summary.md>]` = 判定汇总（①②③④⑥ 全判定行代码计算，防手算漂移；口径常量逐字取自报告表 6-0/6.1）。全矩阵/复测/⑥ 长跑可复现跑批与两窗口空闲预检声明见 docs/evidence/t007/（run_matrix.sh / finish_takeover.sh / run_longrun.ps1）；量测窗口机器空闲独占纪律同前
- `cargo build -p sim --bin arena --release -j 3` / `./target/release/arena.exe --matrix|--throughput|--sampling … --out <dir>` — M0 实验场入口（T009，验收③：`--matrix [--per-side 100|10] [--per-cell K]` 两层胜率矩阵 / `--throughput [--games 512] [--threads T] [--repeats 3]` 吞吐 / `--sampling [--games 100]` 全规模抽样；模式互斥 exit 2、stdout 单行 JSON；判定汇总 `python docs/evidence/t009/summarize.py --out <dir>`）
- `cargo build -p render-spike --release -j 2` / `./target/release/render-spike.exe --units <N> [--seed 42] [--warmup-sec 5] [--capture-sec 65] [--res 1920x1080] --out <dir>` — M0 渲染 spike（T010，验收⑤：窗口化帧采集 frames.csv 逐帧原始档；判定汇总 `python docs/evidence/t010/summarize.py`）。涉 render-spike 的 cargo 一律 **-j 2 从严 + 前置 commit 预检（check/test ≥10G、release ≥12G）**——bevy full 冷编属依赖重型足迹（T010 实测冷编 19m45s）

## 知识资产纪律

- **跨仓回流**（报告 R6，双仓边界）：引擎无关教训 → `bevy-ai-workflow` 仓 `assets-methodology/`；Bevy 特定教训 → `bevy-ai-workflow` 仓 `bevy-dev/`；**跨仓引用不复制正文**；任务台账随本仓独立成册。
- 未过验证的结论不入库、不写进文档（宣称与证据对齐）。
- 行为、接口、口径相关变更必须同步 `docs/` 或任务卡；纯重构豁免。

## 任务台账

从 T001 起逐条记录（`task-ledger.md`）：类型 / 一次通过 / 返工次数 / 实测耗时 / 原因。M0 增值证明与三周投入校准（首周 30h 判定）的唯一数据源，任何任务开始前不可缺位。

## 术语口径

- **M0 三问** = 模拟规模 / 渲染灰盒 / 吞吐量级（报告 6.1）；**止损线 / 承诺线** = 表 6-0 余量政策（承诺线 = 止损线 × 0.5）；**收官序列** = 数据自含成档 → 独立复算 → 报告 V1.0 实测修订 → 身份决策评审（顺序固定，不得倒置）。
