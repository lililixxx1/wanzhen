# 派工单 WP-A · T011 M0 数据自含成档（worker-1 · 树 t011-a）

- 树根：`<trees-root>\t011-a\`（git archive 基线 1d304a8 导出，无 .git；工作目录 = 树根）
- 任务卡：`taskset/t011-data-pack.md`（含开工裁决 D1~D9，先通读）
- 本单路径（主仓，只读）：`<repo>\docs\evidence\m0\dispatches\wp-a.md`
- **先读项目附录**（只读，禁止转写入任何入库文件）：`<repo>\team-prompt\PROJECT-APPENDIX.md`——附录 A（并发预算/门禁纪律）、B.1（误报清单）、B.2（数值三查五类笔误+双盲）、G（证据档规范）全部适用。
- 上报：范围外决策一律上报不拍板（SendMessage 或完成简报）；判定语义零裁量——本卡全部判定行**逐字引用既有代码生成档**，你不产生任何新判定。

## 0. 任务总览

在树内新增 `docs/evidence/m0/`（纯新增；**禁改** `sim/`、`render-spike/`、`Cargo.*`、`rust-toolchain.toml`、既有 `docs/evidence/t002~t015/`、`task-ledger.md`、`taskset/`、`AGENTS.md`——发现需要改动即停手上报）。产出 5 件：

| # | 产出 | 说明 |
|---|---|---|
| 1 | `docs/evidence/m0/README.md` | M0 证据总索引（§1 骨架规格见下） |
| 2 | `docs/evidence/m0/tally_hours.py` | 台账投入汇总脚本（§3 规格） |
| 3 | `docs/evidence/m0/verify_claims.py` | 判定行双向核对脚本（§4 规格） |
| 4 | `docs/evidence/m0/runs/` | 抽查三档取证（§5）+ tally/verify 输出档 |
| 5 | `docs/evidence/m0/selfcheck.md` | 宣称-证据对齐自查记录（§6） |

## 1. m0/README.md 逐节规格

总纪律：
- 判定行/关键数字**逐字引用**（下文清单给出锚子串全文），写入前 `grep -n '锚子串片段' 源档` 核对行号与全文，逐字符复制；
- 本档内路径一律仓库相对路径（`docs/evidence/t0xx/...`），零机器绝对路径（可公开态，附录 G）；
- 每验收节固定结构：①口径出处 → ②终态判定行（逐字）→ ③时序与归因 → ④止损/承诺对照与 6.1 换算（引用，不重算）→ ⑤原始证据档路径 → ⑥复现命令。

### 篇首

- 标题 + 生成信息（生成时刻 ISO +08:00、生成者 worker-1、基线 1d304a8、审核状态留空由 Lead 回填）。
- 纪律声明段：本档判定行全部逐字引自下列代码生成档（列表：t007/summary.md、t008/matrix.md、t009/summary.md、t010/summary.md、t015/summary.md——均「脚本生成禁止手改」档）；本档新增计算仅两处——投入汇总（tally_hours.py）与判定行双向核对（verify_claims.py）；口径唯一来源 = 报告 V0.9.1 表 6-0/6.1（`docs/万阵-游戏前期策划报告.html`）。

### §0 六验收判定快照表

一张表，列：验收 | 终态判定 | 关键数字 | 判定行来源档 | 时序注记。六行内容（判定词按下列源档判定行原词）：

| 验收 | 终态 | 来源 |
|---|---|---|
| ① 四点单线程 µs | PASS（T015 后；T007 曾 4/4 TRIPPED——R2 轮一响应） | t015/summary.md §①（+t007 时序） |
| ② @10k 加速比 | TRIPPED（12t 1.001913×；owner 裁决点→T013，D1） | t015/summary.md §② |
| ②附则 极限十万 | 保留（17.53ms≤22ms；T007 曾超界） | t015/summary.md §③ |
| ③ 吞吐（降规模） | PASS 1,099,638.6 场/h@12t | t009/summary.md 吞吐节 |
| ④ 同种子重放 | 全 PASS（退出码 0） | t008/matrix.md §8 |
| ⑤ 渲染 10k 同屏 | PASS avg 333.15 / 1% low 202.48 | t010/summary.md 判定行节 |
| ⑥ 50k 内存长跑 | 达标 | t007/summary.md §⑤ |

表下三条总注：② owner 裁决点（引 taskset/t015-opt-round-1.md 与 AGENTS.md 现状表述：字面 TRIPPED vs 预算侧全满足，建议 T013 报告级仲裁，本档不仲裁）；③ 镜像 sanity H2 结构性红偏移交 T013 口径仲裁（引 t009）；D2 编号澄清（t007/t015 summary 内部节名「③ 外推十万」= 报告验收②附则；报告验收③ = 实验场吞吐 t009）。

### §1 验收① 微秒成本（单线程基线 1k/5k/10k/50k）

- 口径：引 `docs/evidence/t007/summary.md` 第 5 行口径节（「止损 = 每单位每 tick 成本 > 2µs（单线程基线）…」一行逐字）+ 报告表 6-0 ①原文转述（对照 `docs/万阵-游戏前期策划报告.html` 内「① 每单位每 tick 微秒成本」段）。
- 终态判定行：逐字引 t015/summary.md L18-21 四条（锚见 §2 清单 A）。
- 时序：t007/summary.md L18-21 四条逐字（清单 B）+ 一句注记「T007 如实 TRIPPED → T015 排序序快路径（spatial.rs）后 4/4 PASS 且全达承诺线 1µs；降幅 66×~1846×（引 t015/README.md 相应行，grep '1846' 定位）」。
- 换算：逐字引 t015/summary.md §④ 换算表 4 数据行（清单 C）。
- 证据档路径 + 复现命令：`cargo build -p sim --bin bench --release -j 3` + `./target/release/bench.exe --summarize docs/evidence/t015/matrix.jsonl`（引 t015 档实际命令格式，grep t015/README.md 'summarize' 定位后逐字）。

### §2 验收② 加速比（@10k 锚定）

- 口径：引 t007/summary.md 第 5 行口径节同句（加速比部分逐字）+ 表 6-0 ②原文转述（含「外推 10 万模拟单位模拟耗时 ≤ 22ms 方保留」后半句）。
- 终态判定行：t015/summary.md L42（清单 D）+ L68 总判行（清单 E）。
- 时序：t007/summary.md L42（清单 F：7.221411 OK / 9.416188 OK）。
- 归因注记：从 t015/README.md 提取归因要点（grep -n '3.9' 与 'Amdahl' 定位），转述 + 关键行逐字引用；结尾注「owner 裁决点：② 字面口径 vs 预算侧意图，建议 T013 报告级仲裁（出处：AGENTS.md 当前状态节 / taskset/t015-opt-round-1.md 收口记录）——本档按字面呈现 TRIPPED，不放行不粉饰」。
- 极限十万附则：t007 L49（清单 G：超界不保留）+ t015 L49（清单 H：保留）两行逐字 + 一句翻转注记（O(N²) 项消除，引 t015/summary.md §③ 拟合行 b 系数）。
- 复现命令同 §1（matrix.jsonl 来源差异注明）。

### §3 验收③ 实验场吞吐（降规模口径）

- 口径：表 6-0 ③原文转述 + R2 终止判据吞吐线出处（报告 R2 节「实验场吞吐低于 1 万场/小时」——grep 报告 html '1 万场' 核对后转述）；降规模对局定义（表 6-0：每方 100、≤1,800 ticks）。
- 判定行：t009/summary.md L180 吞吐判定行 + L181 16 线程外推行（清单 I）。
- 附带披露（各 1-2 行 + 档指针）：镜像 sanity 双层 12 格 H2 结构性红偏（z=−10，apply 索引序先手语义，移交 T013 口径仲裁）；接敌实证三层 PASS（194<200/14<20/9980<10000）；分层方向一致 33/36（3 格翻转小样本披露）；口径层 resolved 0/3600 = 预注册预期。
- 复现命令：引 t009/README.md 复现节（grep 'throughput' 定位逐字）。

### §4 验收④ 同种子重放确定性

- 口径：表 6-0 ④原文转述（1/3/6/12 × 3 种子）。
- 判定：逐字引 t008/matrix.md §8 断言节（在 t008/matrix.md 内 grep -n '断言 1' 定位，含断言 1/2/3、加样局、锚局、种子互异、退出码行——清单 J 锚子串）；终局哈希 6 值表逐字引 t008/README.md §2 表（清单 K）。
- 覆盖域披露：引 t008/README.md §1.3 覆盖域段要点（red200 移动段-only / full10000 战斗段，不得外推）。
- T015 后等价性：从 t015/README.md 提取（grep -n '29' 定位 29/29 IDENTICAL 与 16/16 MATCH 判定行，逐字引用；清单 L 锚）。
- 复现命令：引 t008/README.md §4 复现块逐字。

### §5 验收⑤ 渲染 spike

- 口径：表 6-0 ⑤原文转述（含「灰盒无动画/LOD/分层渲染，指标高于 M2/M3 属预期，不作外推依据」句——此句必须保留）。
- 判定行：t010/summary.md L31（清单 M）+ 四档表（引 t010/summary.md 四档表 4 行逐字，清单 N）。
- 披露：物理窗口 2400×1350@125% DPI 保守方向（从 t010/README.md grep '125%' 定位逐字引）；vsync 旁证四档均「有」。
- 复现命令：引 t010/README.md（render-spike 构建/运行行；**注明 cargo -j 2 从严与 commit 预检门槛**——AGENTS.md 常用命令节，逐字引）。

### §6 验收⑥ 内存长跑

- 口径：表 6-0 ⑥原文转述（50k 单位 ≥10 分钟、≤2GB、无单调增长）。
- 判定行：t007/summary.md L77（清单 O）+ 窗口对照行「样本 n=134 / 窗口时长 670.6 s」逐字（L72-73，清单 P）+ 一句对照注记：670.6s = 11.18min ≥ 10min 口径（**给出算式 670.6/60=11.18，以算式为准**）。
- T015 未重跑注记：无内存行为变更，D10 预登记不重跑（引 t015 任务卡/README 对应行，grep 'D10' 定位）。
- 复现命令：引 t007 档长跑命令（run_longrun.ps1 相关行）。

### §7 环境总档

- 环境要素表一张（行 = 要素：CPU / GPU+驱动 / RAM / OS / rustc / cargo / rust-toolchain / bevy lock / 构建 profile；列 = 值 + 出处档）。从 5 份 environment.txt 提取：t007/t008/t009/t010/t015（值不一致时分行列出并注明档，**不合并不择优**）。
- 分卡指纹表：卡 | 环境档路径 | 生成时间 | 量测窗口空闲声明指针（t007 双窗口预检档、t009 precheck_load.ps1、t010/t015 声明行——各 grep 定位）| 二进制 sha256（各卡环境档内值）。
- 基准机 A 固化出处：表 6-0「基准机 A 已固化（2026-09-29，开发机本机）」。

### §8 首周投入校准（快照 + 终判条款）

- 口径段：报告 06 章 R7 规则转述（首周 <30h → 降档 20h/周、阈值不变只改日历；20h 仍不可达 → 无限期挂起）+ 2026-10-05 grill 定案口径逐字转述（严格字面、挂钟全计、按实际执行日期切分、预备周不计、不为凑时数灌水——出处 AGENTS.md 当前状态节，grep '投入口径' 定位）。
- 逐任务表：预备周 T001~T006 六行 + W1 窗口 T007/T008/T009/T015/T010 五行 + T011 占位行（耗时列写「本卡，收口回填」）。耗时数字**只从 task-ledger.md 耗时列解析**（`≈NNN min`）。
- 小计行：预备周小计（不计 W1）、W1 累计（截至本档生成时刻，含 T011 为 0 的口径注明「T011 收口回填后更新」）。
- 判定快照行：`W1 终判未到期（窗口 2026-10-05~10-11，截至 <生成时刻> 累计 <X> h < 30h）——不可终判`。
- 终判条款：W1 收口后由 Lead 回写终判行；判定式 = W1 全周累计 <30h → 触发降档（20h/周、周期拉长一倍、阈值不变），如实接受；数据源 = 本表 + 台账。

### §9 宣称-证据对齐自查

- verify_claims.py 运行记录（全 PASS 截录 + 档指针 runs/verify.txt）。
- 措辞自查：本档验收级数字无「约/大概」级措辞的声明 + 投入耗时「≈」合法性边界注明（台账口径 = 分针近似）。
- AGENTS.md 顺带核对：「当前状态」节 4 个验收级数字抽查（333.15 / 202.48 / 1,099,638.6 或 1099638.6 / 0.061 系）与源档 grep -F 一致性，结果记入 selfcheck.md。

### §10 抽查重跑记录（验收断言 2）

三小节，每节 = 命令来源（档:行）+ 命令全文 + 期望（哈希/diff）+ 实测 + REAL_EXIT。数据取自 §5 执行留档。

## 2. 判定行锚子串清单（写入前逐条 grep -n 复核；逐字符复制）

> 行号为 Lead 写单时实测（源档 Read 行号），执行时以 grep -n 复核为准，不一致即上报。

- **A（t015/summary.md L18/19/20/21，①终态四条）**：
  `- [①] units=1000: us_per_unit_tick=0.060651 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）`（另三条 units=5000→0.072196 / 10000→0.070617 / 50000→0.108681 同式）
- **B（t007/summary.md L18-21，①时序四条）**：`- [①] units=1000: us_per_unit_tick=4.012138 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）`（另三条 19.846795 / 41.010103 / 200.625962 同式）
- **C（t015/summary.md §④ 换算表 L59-62 四数据行）**：`| 1000 | 0.060651 | 0.007581 | 承诺线内（≤2×） | 0.002757 | 承诺线内（≤2×） |` 等四行
- **D（t015/summary.md L42）**：`- [②] units=10000: 12 线程实测加速比=1.001913 | 止损判定（<4×）: TRIPPED | 16 线程外推加速比=1.112480 | 同判（<4×）: TRIPPED`
- **E（t015/summary.md L68）**：`- 总判定：TRIPPED（触发项：② 加速比<4×（锚定 10000））；按 6.1/R2 由此进入后续优化决策（本卡不优化）`
- **F（t007/summary.md L42）**：`- [②] units=10000: 12 线程实测加速比=7.221411 | 止损判定（<4×）: OK | 16 线程外推加速比=9.416188 | 同判（<4×）: OK`
- **G（t007/summary.md L49）**：`- C(100000) ÷ speedup16 = 3907.133373 ms vs 22 ms 极限预算 → [超界：极限十万目标不保留]`
- **H（t015/summary.md L49）**：`- C(100000) ÷ speedup16 = 17.532997 ms vs 22 ms 极限预算 → [保留]`
- **I（t009/summary.md L180/L181）**：`- [吞吐判定行] games_per_hour@12t = 1099638.6 （阈值 >= 10000）→ PASS`；`- [16 线程外推] elapsed(T)=c1+c2/T OLS（T∈{1,3,6,12} 中位）：c1=0.6632s c2=10.3642s·T → elapsed(16)=1.3110s → 吞吐16 = 1405967.6 场/h`
- **J（t008/matrix.md §8，锚子串自行 grep 定位后逐字）**：`断言 1（零 mismatch` / `断言 2（中间哈希` / `断言 3（mismatch 全量如实列出` / `加样局（跨进程同配置逐列一致` / `锚局（vs T006 归档逐字节 diff` / `种子互异 sanity` / `退出码: 0`
- **K（t008/README.md §2 终局哈希表 L84-89 六行）**：`| red200 | 42 | 0xde91d6a5a6e84d43 | 200 |` 等六行
- **L（t015/README.md，grep -n '29/29' 与 '16/16' 定位逐字）**
- **M（t010/summary.md L31）**：`- t10000: avg_fps=333.15 (≥60 ? 是)、1% low=202.48 (≥45 ? 是) → **PASS**`
- **N（t010/summary.md L11-14 四档行）**：`| t1000 | 34843 | 536.03 | 260.07 | 151.61 | 1.745 | 2.567 | 3.249 | 24.113 | 34841 |  |` 等四行
- **O（t007/summary.md L77）**：`- [⑥] 判定：达标（附：PrivateMemorySize64 max=4.7 MiB）`
- **P（t007/summary.md L72/L73）**：`- 样本：n=134 条（采样间隔 5s；每 5s 读 Get-Process WorkingSet64 + PrivateMemorySize64；判定用工作集）`；`- 窗口时长（末样本 elapsed_ms）：670.6 s`

## 3. tally_hours.py 规格

- 纯标准库 python，无参数运行；解析 `task-ledger.md`「## 记录」表：行首 `| T0xx |` 的行，取耗时列中 `≈(\d+)\s*min` 首个匹配（单位 min，int）。
- 窗口映射常量：`PREP = ["T001","T002","T003","T004","T005","T006"]`；`W1 = ["T007","T008","T009","T015","T010","T011"]`。
- 输出（stdout）：逐任务行（ID、耗时 min、窗口归属）+ 两窗口小计（min 与 h，h = min/60 保留 2 位）+ 「未登记：T011（收口回填）」类提示行（缺失任务列出但退出码仍 0）。
- 双盲（B.2⑤）：**先**手工从 task-ledger.md 抄录六预备周 + 五 W1 任务耗时做速算表（sum 算式写全，如 `190+605+145+125+140=1205`），**再**跑脚本对照——两值不一致即停止上报；一致才把脚本输出落 `runs/tally.txt`（含速算表与脚本输出两段）。参考值仅供核对：预备周 ≈458 min、W1（不含 T011）≈1205 min——以脚本为准。

## 4. verify_claims.py 规格

- 纯标准库；内置 `CLAIMS: list[(label, source_relpath, needle)]`，内容 = §2 清单 A~P 全部锚子串（J/K/L 按实际定位行补全 needle）。
- 双向核对：`needle` 在 `source_relpath` 命中 ∧ 在 `docs/evidence/m0/README.md` 命中（子串级，编码 utf-8）。
- 输出逐条 `PASS/FAIL label source` + 汇总行；退出码 0=全过、1=有 FAIL。结果落 `runs/verify.txt`。FAIL 任何一条即停止上报（不得改 needle 凑过）。

## 5. 抽查三档（验收断言 2）

执行前置：`cargo build -p sim --release -j 3`（树根执行；**只此一条 cargo，-j 3，串行**——禁止 --workspace 触发 bevy full 冷编；此为 D8 门禁修正，理由：纯文档卡预期零代码变更）。产物三 bin：sim/bench/arena。留档目录 `docs/evidence/m0/runs/`：

1. **t008 局**：在 `docs/evidence/t008/runs/` 找 red200 th1 s42 局（ls 档名，`.cmd` 档 = 命令存档）→ 逐字执行该命令 → stdout 与同 run_id `.stdout` 归档 `diff` 逐字节比对（留 .diff 档，期望 identical）→ cmd/新 stdout/diff 三档留档（`spot-t008-*.{cmd,stdout,diff}`）+ REAL_EXIT 行。
2. **bench 单配置**：`./target/release/bench.exe --units 10000 --threads 1 --ticks 300 --seed 42`（与 t015 附录 A 同配置；来源 AGENTS.md 常用命令节用法行）→ stdout 单行 JSON 中 `final_hash` == `0xc5915d042208e267`（python -c 或 grep 提取比对，留档 `spot-bench-10k-t1.{stdout,check}`）；**只核哈希不比耗时**（计时属量测窗口产物，D4 口径注明）。
3. **t009 CLI 交叉腿**：从 `docs/evidence/t009/crosscheck_r5.sh` 取 sim_t1 命令（grep -n 定位）→ 逐字执行 → final_hash == `0x564cf46fdf191710`（同上留档 `spot-t009-simt1.{cmd,stdout,check}`）。

三档命令均相对树根、无机器绝对路径；任何一档 diff 不一致/哈希不符 → 停止上报。

## 6. selfcheck.md 规格

四节：① verify_claims.py 全 PASS 声明 + runs/verify.txt 指针；② 措辞自查（m0/README 验收级数字零「约/大概」；「≈」仅投入耗时口径）；③ AGENTS.md「当前状态」节验收数字抽查 ≥4 项的 grep -F 对照结果（逐项：AGENTS.md 片段 ↔ 源档命中）；④ 环境总档五档一致性观察（rustc/bevy lock 是否五档同值；差异如实列）。

## 7. 门禁与完成判定

1. `cargo build -p sim --release -j 3` 退出码 0（Bash 单独整句执行，验证与退出码判定不接管道——附录 B.1）。
2. tally_hours.py / verify_claims.py 运行退出码 0；双盲对照一致。
3. 抽查三档 REAL_EXIT 全 0、diff/哈希全符。
4. 纯净性自查：树根执行 `find . -newer <标记文件> -type f -not -path './target/*'`（标记文件 = 建树后你自建的一次性 timestamp 文件，放树外临时处或用 `touch /tmp/t011a-mark` 前置）——改动清单应**仅含** `docs/evidence/m0/**`（清单落 runs/changed-files.txt）；出现其他路径即停上报。
5. 完成简报：产出清单 + 各门禁退出码 + 双盲两值 + 抽查三行结果 + 挂钟起止。

时间盒 ≈90 min；ZCode Bash 单命令 ≤10 分钟硬超时（附录 A），构建按经验 ≤5 min 单条可完。
