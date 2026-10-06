# T012-RECHECK-A · M0 独立复算派工单（派工对象：plan-code-reviewer · 完整轮 ★）

> 本文件即派工单全文（Lead 下发 2026-10-06；基线 commit `b1b72e7`）。
> 任务卡：`taskset/t012-independent-recheck.md`（含开工裁决 D1~D12，与本单不可冲突，冲突以本单为准并上报）。

## 目标

只读独立复算 M0 全部验收数据与判定：独立复跑（量测抽查 / 确定性对拍 / 渲染复测 / 吞吐复测）+ 判定与原始数据复算 + 止损换算 6.1 过程复核 + 口径混用检查 → 裁决（通过 / 有条件通过 / 不通过）+ 发现清单（P0/P1/P2/S/N）。**不接受「执行者自己写、自己跑、自己交」作为验收证据——你的全部复算必须独立实现/独立复跑，不复用执行侧汇报值。**

## 仓库与环境

- 主仓根 = `<主仓根>`（以下相对路径均以此为根；你写入的档内路径一律用仓库相对路径，**零机器绝对路径**）。
- 基线 = HEAD `b1b72e7`（开工时 `git rev-parse HEAD` 留证）。禁一切 git 写操作（add/commit/checkout 等）；git 只读命令允许。
- 先读 `AGENTS.md`（硬约束 6 条 + 量测口径）与任务卡 `taskset/t012-independent-recheck.md`。
- 环境条款（自读，绝对路径 + 节号）：`<主仓根>\team-prompt\PROJECT-APPENDIX.md` 的 **附录 A**（-j 3 串行 / 量测窗口空闲独占 / commit 预检）、**附录 B.1**（误报清单）、**附录 B.2**（数值纪律）、**附录 G**（证据档规范：自含 / 可公开态 / REAL_EXIT 留痕）。

## 写范围（硬约束）

- **仅允许新建** `docs/evidence/m0/review/**` 下的文件（`report.md` + `runs/` 子目录）。
- **禁改一切既有文件**（含 docs/evidence/**、taskset/**、AGENTS.md、源码、Cargo.*）；禁覆写任何归档档（所有再生成输出一律重定向到 review 目录内）。
- 交付物：`docs/evidence/m0/review/report.md`（审核报告主体）+ `docs/evidence/m0/review/runs/`（一命令一档：命令全文 + 原始输出 + 退出码 REAL_EXIT 行）。

## 执行序（串行；每条命令单独整句执行、真实留痕）

### 第 0 步 · 环境与前置

1. `git rev-parse HEAD` → 期望 `b1b72e7...`（前 7 位 b1b72e7）。
2. `git diff --stat 1d304a8..HEAD -- render-spike/` → 期望**空输出**（1d304a8 = T010 收口提交；render-spike 源自 T010 后零变更——证明 Lead 预编译二进制与归档行为同源；非空即停并上报）。
3. 空闲预检快照（进程表）→ 存 `runs/precheck-0.txt`；量测窗口机器空闲独占声明写入 report.md（每 timing 批前各留一份 precheck 档）。
4. `cargo build -p sim --release -j 3`（主仓根；期望退出码 0；产物 sim.exe/bench.exe/arena.exe）。**沿 T011 D8 先例：本卡无代码变更，不跑 --workspace check（bevy full 冷编无必要）。**
5. 五 bin sha256 留证（sim/bench/arena + render-spike）：`certutil -hashfile target/release/<bin>.exe SHA256`。render-spike.exe 已由 Lead 于派工前冷编完成（-j 2 + ≥12G 预检过）；若缺失即停并上报（禁止自行冷编）。二进制 sha 与历史环境档不一致属预期（源经多卡演进），report.md 披露 provenance 注记即可；**确定性锚以 final_hash 为准，不以 bin sha 为准**。
6. `rustc -V` && `cargo -V` 留证（期望 1.98.1，与 m0/README §7 五档同值）。

### 第 1 步 · 确定性新鲜复跑（bit-exact，零容差）

对拍方式：stdout 重定向到 `runs/`，然后与归档档 `diff`（cmp/diff 逐字节；**只 diff stdout，stderr 含计时行不比**）。每档留 `.cmd`（命令全文）、`.stdout`、`.diff`（diff 输出）与 REAL_EXIT 行。

| # | 命令（主仓根执行） | 归档对照档 | 断言 |
|---|---|---|---|
| 1a | `./target/release/sim.exe --comp "shieldman:17,heavyknight:17,pikeman:17,swordsman:17,archer:16,militia:16" --ticks 1800 --hash-samples 0,450,900,1350 --threads 1 --seed 42` | `docs/evidence/t008/runs/red200-th1-s42.stdout` | diff 空；终局 `hash=0xde91d6a5a6e84d43`；采样 1350 点 `0xe0cd7861ca5a211f`（来源：t008/README.md:82-89 哈希表 + m0/README §10 档 a） |
| 1b | `./target/release/sim.exe --comp "shieldman:834,heavyknight:834,pikeman:833,swordsman:833,archer:833,militia:833" --ticks 14400 --hash-samples 0,3600,7200,10800,12600 --threads 12 --seed 44` | `docs/evidence/t008/runs/full10000-th12-s44.stdout` | diff 空；终局 `hash=0x9d55c4ce4f7fd880`、units 9970（t008/README.md:88；运行 ≈12s，战斗段+多线程档） |
| 1c | `./target/release/sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 1` | 无逐字节档（锚=哈希） | stdout 末行 `final_hash=0x564cf46fdf191710`（t009/summary.md:245-247 三方黄金交叉同值；命令来源 t009/run_t009.sh:85；运行 ≈11s） |

### 第 2 步 · 量测套件新鲜复跑（4 配置；timing 类，容差政策见「预注册预期」）

逐条串行执行，stdout JSON 存 `runs/bench-<config>.stdout`（单行 JSON：samples_ns 全量/median/CV/us_per_unit_tick/final_hash；同配置多局哈希不一致 exit 4）。命令骨架与口径来源 = AGENTS.md:32（默认 warmup=1/repeats=5/seed=42）：

| # | 命令 | final_hash 期望（零容差） | us_per_unit_tick 参考值（±25% 带） |
|---|---|---|---|
| 2a | `./target/release/bench.exe --units 10000 --threads 1 --ticks 300 --seed 42` | `0xc5915d042208e267` | 0.070617 |
| 2b | `./target/release/bench.exe --units 10000 --threads 12 --ticks 300 --seed 42` | `0xc5915d042208e267` | 0.070482 |
| 2c | `./target/release/bench.exe --units 50000 --threads 1 --ticks 30 --seed 42` | `0x022c5abdfae119dc` | 0.108681 |
| 2d | `./target/release/bench.exe --units 50000 --threads 12 --ticks 30 --seed 42` | `0x022c5abdfae119dc` | 0.120594 |

（哈希与参考值来源：docs/evidence/t015/summary.md 附录 A 表 L72-88——**双盲纪律：先从该源档独立取值核对上表，不一致即停并上报**。）

附加断言：2a/2b 新鲜加速比 = median(2a)/median(2b) ∈ 0.8~1.3（② 结构性 ≈1.0 判定复现）；四配置 us_per_unit_tick 全部 ≤1µs（承诺线）且 ≤2µs（止损线）。

### 第 3 步 · 渲染复测 1 档（t10000）

```
./target/release/render-spike.exe --units 10000 --seed 42 --warmup-sec 5 --capture-sec 65 --res 1920x1080 --out docs/evidence/m0/review/runs/render-t10000
```

- 期望退出码 0；窗口弹出 ≈70s 属预期（report 披露）；meta.json 记录 `window_resolution_actual`（参考历史值 2400×1350@125% DPI，保守方向注记）。
- 判定：`--out` 产物 frames.csv + meta.json。然后独立重算 avg_fps 与 1% low（**自己写 python，公式先读 `docs/evidence/t010/summarize.py` 文件头预注册式**：avg_fps=N/(Σdelta_ns/1e9)；1% low=1e9/mean(最慢 k1 帧)，k1=(N+99)//100）→ 存 `runs/render-recompute.txt`。
- 归档参考值：avg_fps=333.15 / 1% low=202.48（docs/evidence/t010/summary.md:14 与 :31）；判定线 avg≥60 且 1%low≥45 独立判定（预期 PASS，漂移 ±25% 带内；先例 G1 fresh 329.33/177.95）。
- 再生成旁证（可选加分）：把产物按 `root/t012/out/t10000/` 布局放置后 `python docs/evidence/t010/summarize.py --root <该root> --out runs/render-summary-regen.md` → **期望退出码 3（缺其余三档=预期）**且产出 t10000 判定行。

### 第 4 步 · 吞吐复测 1 轮（@12t）

```
./target/release/arena.exe --throughput --games 512 --threads 12 --repeats 3 --out docs/evidence/m0/review/runs/throughput_t12
```

- 期望退出码 0；从产物 JSON 重算 games_per_hour@12t（公式先读 `docs/evidence/t009/summarize.py` 源码后独立实现）→ 存 `runs/throughput-recompute.txt`。
- 归档参考值 1,099,638.6 场/h（docs/evidence/t009/summary.md:180，来源档 docs/evidence/t009/runs/throughput_t12/throughput_t12.json）；判定线 ≥10,000 PASS 独立判定；漂移 ±25% 带内（裕度 ≈110×）。

### 第 5 步 · 归档再生成比对（工具确定性）

| # | 命令 | 期望 |
|---|---|---|
| 5a | `python docs/evidence/t008/compare_matrix.py docs/evidence/t008/runs docs/evidence/m0/review/runs/t008-matrix-regen.md` | 退出码 0；再生成档与 `docs/evidence/t008/matrix.md` diff **逐字节空**（档无时间戳；非空即如实披露 diff 内容并降级排查） |
| 5b | 复制 `docs/evidence/t009/runs` → `docs/evidence/m0/review/runs/t009-copy/runs`（≈1.8M），然后 `python docs/evidence/t009/summarize.py --out docs/evidence/m0/review/runs/t009-copy` | 退出码 0；复制档内新生成 summary.md 与 `docs/evidence/t009/summary.md` 的**判定行逐字节一致**（全档 diff 如实披露） |
| 5c | `./target/release/bench.exe --summarize docs/evidence/t015/matrix.jsonl --out docs/evidence/m0/review/runs/summarize-t015-regen.md` | 退出码 0；再生成档与 `docs/evidence/t015/summary.md` diff **逐字节空**（主仓根执行使内嵌数据源相对路径一致） |

### 第 6 步 · 判定与原始数据独立复算（自己写 python，逐项存 `runs/recompute-*.txt`）

- **①**：从 `docs/evidence/t015/matrix.jsonl` 的 samples_ns 重算 16 配置 median 与 us_per_unit_tick（median 语义先读 `sim/src/bin/bench.rs:923` median_f64 后独立实现）→ 与 t015/summary.md 附录 A（L72-88）逐位对照。
- **②**：从 t015 matrix 重算 @10k 加速比（t1/t12）、Amdahl OLS（elapsed=c1+c2/T，T∈{1,3,6,12}）与 16 线程外推（`sim/src/bin/bench.rs:1193-1348`，speedup16 式在 :1249）→ 对照 1.001913 / 1.112480 / TRIPPED（summary.md:42）。
- **②附则（极限十万）**：四采样点单线程 ns/tick 拟合 C(N)=a·N+b·N²（bench.rs:1361 起）→ C(100000)÷speedup16 对照 17.532997 ms ≤22ms → 保留（t015/summary.md:49；speedup16 取 50k 档，bench.rs:100）。同法复算 t007 归档侧：a=773.554196350/b=3.997072319/speedup16=9.416188 → 3907.133373 ms 超界（t007/summary.md:42/46/49）。
- **③**：从 `docs/evidence/t009/runs/throughput_t{1,3,6,12}/throughput_t{T}.json` 重算 games_per_hour@12t=1,099,638.6 与 16t OLS 外推（c1=0.6632/c2=10.3642/elapsed(16)=1.3110s/吞吐16=1,405,967.6 场/h；summary.md:180-181）。
- **④**：5a 再生成即④复算主体；另核 t008/README.md:82-89 六哈希表与你 1a/1b 实测值三方一致。
- **⑤**：第 3 步独立重算即⑤复算主体。
- **⑥**：档内核对（不重跑，D1）：t007/summary.md L72-73/L77——n=134 样本、窗口 670.6s（670.6/60=11.18≥10min 算式核）、高水位稳定性口径行在档；D10 预登记未重跑披露行在 t015/README.md:31。
- **6.1 换算复核**：所需加速比 = 单位数 × µs ÷ 预算 ms 数 × 10⁻³（口径句 = 报告 :525 加粗式 + t007/summary.md L5）→ 复算 t015/summary.md §④ 表 8 格（如 10k@8ms：10000×0.070617/8000=0.088271）与「承诺线内（≤2×）」对照词（语义读 bench.rs）。
- **W1 投入复算**：从 task-ledger.md 逐任务行手工/独立解析——预备周 458+99=557、W1 91+605+145+125+140+140=1246（min）=20.77h（m0/README §8 小计行）；再跑 `python docs/evidence/m0/tally_hours.py`（stdout 重定向到 runs/）与 `python docs/evidence/m0/verify_claims.py`（stdout 重定向；期望 `汇总: 81 条 | PASS 81 | FAIL 0`、REAL_EXIT=0）对照。
- **口径混用检查（D5）**：报告表 6-0 原文（`docs/万阵-游戏前期策划报告.html`:485/:490/:498）vs `docs/evidence/m0/README.md` §0~§8 + 各 summary 判定行 + AGENTS.md「当前状态」节——逐验收核对三对口径（**降规模/全规模、单线程基线/多线程、含镜像 AI 决策/不含落库**）无混用；② owner 裁决点与③ 镜像 sanity 两个移交项的「不放行不粉饰」呈现是否如实。产出逐项清单表入 report.md §5。

## 预注册预期（偏差如实披露，禁止事后改预期合理化）

| 类别 | 容差 | 判定语义 |
|---|---|---|
| bit-exact（终局哈希/采样列/CLI 黄金/stdout diff/5a/5c 再生成 diff） | **零容差** | 任何一位不符 = 确定性失败 → P0 级上报 |
| timing 漂移（①µs/②加速比档外其余/③吞吐/⑤fps） | 归档值 ±25% 内 = 容差内；超带 = 加样复测 2 次全档披露（沿 T015 处方），判定仍以验收线独立判 | 验收线：µs≤1（承诺）/≤2（止损）；games/h≥10,000；fps≥60/45 |
| ② 新鲜加速比（2a/2b） | 0.8~1.3× | 结构性 ≈1.0 复现；② 判定以 <4× 为准（TRIPPED 预期复现） |
| 渲染 summarize 单档再生成 | 退出码 3 = 缺档**预期** | t10000 判定行仍须产出且 PASS |

## 上报条件（任一即停该步并上报，不拍板）

- 双盲核对发现本单锚值与源档不一致；diff 非空的 bit-exact 项；render-spike.exe 缺失或 `git diff 5ab440b..HEAD -- render-spike/` 非空；任何退出码非预期且重试 1 次仍复现（附录 B.1 误报先自查：管道退出码/并发纪律）；超时间盒。

## 时间盒

**90 min**（超盒即上报当前进度，不默认续做）。量测命令均秒级~75s；全部串行。

## 汇报格式（最终消息 + report.md §8 五字段）

```
[结论] 通过 / 有条件通过 / 不通过
[已核对] 逐条列验收标准 + 通过情况 + 退出码（只回「通过」二字 = 无效审核）
[阻塞项] 文件:行 + 违反哪条验收标准
[最小修复指令]
[复验命令]
```

发现清单定级：P0（伪造/缺失证据/判定失实/确定性失败）/ P1 必改 / P2 次级必改 / S 建议 / N 注记。验收断言（任务卡）：① 审核报告入档 docs/evidence/m0/review/（含独立复跑命令与输出摘要）；② 裁决明确，必改项闭环记录；③ 复算与自报数字的偏差全部披露（容差内/超差逐条）。

---

## 修正留痕（2026-10-06，审核轮后追加；P2×2 闭环记录——禁止静默改，原文上文不动）

- **勘误 1（P2-1，审核轮停步上报证实）**：第 6 步「②附则（极限十万）」行中 t007 侧锚值「speedup16=9.416188」**有误**——9.416188 是 **10k 档**的外推值（t007/summary.md:42 [②] 行），③ 极限十万计算按 bench.rs:100 取 **50k 档 speedup16=10.249990**（t007/summary.md:48 拟合表 50k 行 ×16 列）。算术自洽核：用 10.249990 → C(100000)÷speedup16 = 3907.133373 ms 与归档一致；用 9.416188 → 4253.109497 ms 与归档不符（runs/recompute-6-all-rv2.txt:101-107 双取值并列披露）。原单该锚值系 Lead 从 [②] 行误取，违反 B.2①「整数算式 + 权威来源」——归因 Lead 派工单，台账如实计返工。**T013 复算口径钉死「③ speedup16 取 50k 档」**（吸收为移交项）。
- **勘误 2（P2-2）**：第 2 步表注与第 6 步①中「t015/summary.md 附录 A 表 **L72-88**」实为 **L72-89**（2d 行在 L89；runs/line-refspotcheck-rv2.txt:19-21）。值正确未致误引；行号引用类（B.2④）同类第 2 次（T008 P1-1 后），固化计数 +1，随本勘误闭环。
- 前置修正（下发前自查，2026-10-06）：render-spike 零变更比对基线 5ab440b→**1d304a8**（5ab440b 是 T010 之前的基线，含 T010 建档 diff 不可用作零变更证明）。
- 附带措辞勘误（N-4）：「五 bin sha256」实为四 bin（sim/bench/arena/render-spike）。
