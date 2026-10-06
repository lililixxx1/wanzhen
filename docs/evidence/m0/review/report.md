# T012-RECHECK-A · M0 独立复算审核报告（plan-code-reviewer · 完整轮 ★）

- 派工单 = docs/evidence/m0/review/dispatch-rv.md（Lead 下发 2026-10-06）；任务卡 = taskset/t012-independent-recheck.md（D1~D12）。
- 基线 commit = b1b72e765dbb0f884c70c6878974e56269038797（runs/step0-git-head.txt）；render-spike 源零变更（`git diff --stat 1d304a8..HEAD -- render-spike/` 空输出，runs/step0-git-diff-renderspike.txt）。
- 执行日 = 2026-10-06（W1 D2 窗口内，10:42~11:2x +08:00）；写范围仅 docs/evidence/m0/review/** 新建档；禁改既有文件、禁 git 写操作——全程遵守（唯一例外说明见 §2 命名映射）。
- 复算纪律 = 全部独立实现/独立复跑（median/OLS/拟合/换算公式先读源码再独立实现；判定与原始数据不复用执行侧汇报值）。

## §0 五字段裁决（汇报格式）

```
[结论] 有条件通过（⚠️ 修后可执行）——M0 六验收归档数据与判定独立复算全部逐位通过、零 P0；
       条件 = P1×1（⑤ 干净量测窗口补测或 T013 条件性披露）+ P2×2（派工单锚值/行号勘误入档）闭环。
[已核对]（逐条 · 详见 §3/§4）
  ① 四点单线程 µs            —— 通过：t015 16 配置 median/us/hash 逐位 MATCH，4 t1 点全 ≤1µs；新鲜 bench 2a~2d hash 零容差 OK、us 全 ≤1µs；REAL_EXIT=0
  ② @10k 加速比              —— 通过：1.001913 / 1.112480 / TRIPPED 逐位 MATCH；新鲜 2a/2b 加速比 1.12720 ∈ 0.8~1.3；REAL_EXIT=0
  ②附则 极限十万             —— 通过：17.532997 ms ≤ 22ms [保留] 逐位 MATCH；t007 侧 3907.133373 ms [超界] 逐位 MATCH（但派工单锚值有误，见阻塞项 2）
  ③ 吞吐（降规模）           —— 通过：1,099,638.6 场/h@12t 与 16t 外推 1,405,967.6 逐位 MATCH；新鲜复测 864,409.7（漂移 -21.39%，±25% 带内）判定 ≥10,000 PASS；final_hash_xor 跨卡逐位一致；REAL_EXIT=0
  ④ 同种子重放               —— 通过：29 局矩阵再生成判定内容逐字节一致（5a 仅 argv 自述行机械差异 + 降级排查 exit 0）；新鲜 1a/1b diff+cmp 双 exit 0、1c 黄金 0x564cf46fdf191710 命中；REAL_EXIT=0
  ⑤ 渲染 10k 同屏            —— 有条件：归档 333.15/202.48 三实现复算逐位 PASS（自洽）；但新鲜复测 3 样本 avg 59.41~64.10 / 1% low 22.16~23.87 漂移 -79%~-89%（远超 ±25% 带）、summarize 再生成判定行 FAIL（预注册期望「且 PASS」偏离）、meta window_resolution_actual=0x0（归档/G1 均 2400x1350）且量测窗口非空闲独占 ⇒ 本轮不可确认亦不可证伪，见阻塞项 1；REAL_EXIT=0/3（3=缺档预期）
  ⑥ 内存长跑                 —— 通过：档内核对 n=134 / 窗口 670.6s / 670.6÷60=11.1767→11.18 ≥ 10min / 高水位稳定性口径行（L75）/ 判定行（L77）在档；t015/README.md:31 D10 未重跑披露行在档（按 D1 不重跑）
  6.1 换算（8 格 + 对照词）  —— 通过：逐位 MATCH（含 50k@8ms 全精度 0.679255 与「承诺线内（≤2×）」语义，bench.rs:1449-1458）
  W1 投入                    —— 通过：独立解析 557 min（预备周）/ 1246 min = 20.77h（W1）双 MATCH；tally_hours.py 重跑同值；verify_claims.py 重跑 81 条全 PASS（REAL_EXIT=0）
  口径混用（D5 三对）        —— 通过：无混用（1 处 S 级标签建议）；② owner 裁决点与 ③ 镜像 sanity 两移交项「不放行不粉饰」呈现如实
  工具确定性 5a/5b/5c       —— 通过：5b/5c 再生成逐字节空 diff；5a 除「生成命令」自述行外逐字节一致
[阻塞项]
  1. docs/evidence/m0/review/runs/render-recompute-rv2.txt:4-6（+ render-summary-regen-rv2.md:22）vs docs/evidence/t010/summary.md:31
     —— 违反派工单预注册期望「渲染 summarize 单档再生成…t10000 判定行仍须产出且 PASS」：
        新鲜 3 样本 1% low 22.16/23.27/23.87 全部 < 45 → 判定行 FAIL；avg 59.41/59.43 另低于 60。
        同时 window_resolution_actual=0x0（t010/README.md:90-94 声明各档 2400x1350）+ precheck-3-rv2 非空闲
        （SLDWORKS / AcWebBrowser 系 / 钉钉 / uTools 等负载，CPU 采样 8~38%）⇒ 附录 A「共享负载下红绿均不可信」，
        本轮红不能推翻归档绿、亦不能确认之（归档侧 333.15/202.48 三实现复算自洽 + G1 fresh 329.33/177.95 先例）。
  2. docs/evidence/m0/review/dispatch-rv.md（第 6 步 ②附则 行）vs docs/evidence/t007/summary.md:48
     —— 违反派工单双盲条款（D9 / 附录 B.2⑤）：派工单锚值 speedup16=9.416188 与源档 10.249990（50k 档，bench.rs:100）
        不一致，且与派工单自身目标值 3907.133373 ms 算术不自洽（用 9.416188 复算 = 4253.109497 ms，
        runs/recompute-6-all-rv2.txt:107）。已按上报条件「停该步上报不拍板」处置：该项以源档值复算通过、双取值并列披露，
        口径终裁移交 owner/Lead。
  3.（次级）dispatch-rv.md 第 2 步表注「t015/summary.md 附录 A 表 L72-88」实为 L72-89（2d 行在 L89；
        runs/line-refspotcheck-rv2.txt:19-21）——行号引用类（B.2④）复发计数 +1；值正确、未致误引。
[最小修复指令]
  1. 于真·空闲独占窗口（precheck 进程表无 SLDWORKS/AutoCAD 系/钉钉/uTools 等负载）且 meta
     window_resolution_actual=2400x1350 可复核时，补跑一次 t10000 渲染复测（命令见 §8 复验命令 1），
     新数据并入 T013 重判 ⑤；若无干净窗口可得，T013 必须写入条件性披露「⑤ 归档 PASS 依赖干净量测窗口
     （2026-10-06 两轮 6 样本在带载环境均不可复现）」。渲染腿后果路径（报告 :525）仅在 ⑤ 最终判 FAIL 时触发，
     引用 summarize.py FAIL 分支文本须带此限定。
  2. 派工单勘误入档（dispatch-rv.md 附勘误注记或台账原因列）：t007 ③ speedup16 = 10.249990（50k 档外推，
     bench.rs:100）；9.416188 为 10k 档值，不可用于 ③ 计算。T013 复算口径同步钉死「50k 档」。
  3. 同勘误附行号修正：t015/summary.md 附录 A 表 = L72-89。
[复验命令]
  1. ./target/release/render-spike.exe --units 10000 --seed 42 --warmup-sec 5 --capture-sec 65 --res 1920x1080 --out docs/evidence/m0/review/runs/render-t10000-rv3
     && python docs/evidence/m0/review/runs/render-recompute-rv2.py   （改样本路径后：期望 avg ≥ 60 且 1% low ≥ 45，漂移回 ±25% 带）
  2. python docs/evidence/m0/review/runs/recompute-6-all-rv2.py       （期望 t007 ③ 段与勘误一致：speedup16=10.249990 → 3907.133373 ms）
  3. python docs/evidence/m0/review/runs/recompute-6-anchorfix-rv2.py （期望 3 行 MATCH）+ python docs/evidence/m0/verify_claims.py（期望 81/81、REAL_EXIT=0）
```

## §1 裁决摘要

M0 六验收的**归档数据与判定**经独立复算（独立实现 median / Amdahl OLS / 二次拟合 / 6.1 换算 / 吞吐 OLS）**全部逐位一致**，
确定性与工具确定性证据链成立（bit-exact 零容差项全绿），投入校准数据可复算。**零 P0**（无伪造、无缺失证据、无判定失实、无确定性失败）。
唯一实质开口 = **验收⑤ 的新鲜复测在当前桌面环境不可复现**（P1）：6 个新鲜样本（本轮 3 + 前轮遗留旁证 3）全部落在 avg 59~75 /
1% low 22~25 区间、判定行 FAIL，与归档 333.15/202.48 差 -79%~-89%；但承测窗口非空闲独占且 meta 窗口尺寸 0x0（异常），
按附录 A 该红**不可信、不可推翻归档绿**——归档侧另有 G1 fresh（329.33/177.95）与本轮第三实现重算（333.15/202.48 MATCH）双佐证。
处置 = 干净窗口补测或 T013 条件性披露（阻塞项 1）。另有派工单自身锚值/行号两处缺陷（P2×2）已按「停该步上报不拍板」处置并移交。

## §2 审核范围、覆盖率与留痕规范

### 2.1 环境要素行（D12）

- git HEAD = b1b72e765dbb0f884c70c6878974e56269038797（runs/step0-git-head.txt）
- rustc 1.98.1 (48a229cea 2026-09-01) / cargo 1.98.1 (797e8a9bc 2026-08-05)（runs/bin-sha256-toolchain-rv2.txt；与 m0/README §7 五档同值）
- 二进制 sha256（四 bin；D12 写「五 bin」，本仓实际四 bin = sim/bench/arena/render-spike，措辞勘误级 N-4）：
  - sim.exe `ea79ce438adb877269a299acc716edd83138e05e123b18898f6e76b968a72bf5`
  - bench.exe `8eebc283574cf31d6e69af3ae90e35fe9600882bcc9500a7462bdac51d275350`
  - arena.exe `c302f3a0dfda7aaf7cb8f1d63f3b841bef0468ad18b662eced671d9a4a753c81`
  - render-spike.exe `ce64db4e20fb289bbefb462dc38593c71be05158cda176df8c7715bf0840fbdd`（= Lead 派工前冷编留痕值，核验一致）
- provenance 注记（N-3）：render-spike.exe 历史环境档（m0/README §7 T010 行）为 264A95…，本轮 ce64db4e… —— 构建路径差异属派工单 D2 预告的预期不一致；**确定性锚一律以 final_hash 为准**（本轮 1a/1b/1c/bench×4/throughput hash_xor 全部命中即证）。
- 量测窗口空闲声明指针：runs/precheck-0-rv2.txt（步骤 0）、runs/precheck-2-rv2.txt（bench 批）、runs/precheck-3-rv2.txt / precheck-3b-rv2.txt / precheck-3c-rv2.txt（渲染 3 样本逐次）、runs/precheck-4-rv2.txt（吞吐批）。**如实披露：各窗口均未达成「机器空闲独占」**（实录用户应用负载，CPU 采样 7~38%）——本报告所有 timing 结果均按带载口径解读（附录 A：共享负载下红绿均不可信）；bit-exact 项不受负载影响。

### 2.2 留痕规范与命名映射（-rv2 后缀说明）

- 一命令一档：命令全文（# CMD）+ 原始输出 + 退出码（# REAL_EXIT）同档；对拍项另存 .diff（diff+cmp 双退出码）。
- **零碰撞命名**：本卡开工时 docs/evidence/m0/review/runs/ 内已存在一轮同任务中断遗留档（时间戳 10:04~10:29，命名与断言体系同派工单，来源 = 前一复核轮，非执行侧档）。为守「禁改一切既有文件 / 禁覆写」，本轮全部留痕以 `-rv2` 后缀新建（如 bench-2a-rv2.stdout、render-recompute-rv2.txt），派工单原定文件名（bench-<config>.stdout、render-recompute.txt 等）逐一同名映射加 -rv2；遗留档原样保留、仅作旁证且已降级（其 render-t10000 档 stderr 显示 37s 窗口与 frames 65.015s 不自洽、数据疑与 retry 混位）。遗留档去留移交 Lead（N-1）。
- 本轮工具失误三起如实留痕、零影响判定数据（N-2）：吞吐重算首跑路径假设错误（arena 的 JSON 按线程数命名而非 out 目录名，REAL_EXIT=1，runs/throughput-recompute-rv2.txt，retry = throughput-recompute-rv2b.txt 通过）；W1 解析脚本 heredoc 传输吃反斜杠致正则损坏（REAL_EXIT=1，runs/recompute-w1-rv2.txt，b 版零反斜杠实现通过）；综合复算脚本首次写入被命令长度截断（runs/recompute-6-all-rv2.writefail.txt 留痕，分块重写后运行通过）。

### 2.3 覆盖率（分母 = 本卡判定复算涉及的输入档清单）

| # | 文件 | 结论 |
|---|---|---|
| 1 | taskset/t012-independent-recheck.md | 已审（全文） |
| 2 | docs/evidence/m0/review/dispatch-rv.md | 已审（全文，与派工单消息逐段一致） |
| 3 | AGENTS.md | 已审（硬约束 6 条 + 量测口径 + 当前状态节） |
| 4 | team-prompt/PROJECT-APPENDIX.md | 已审（附录 A / B.1 / B.2 / G） |
| 5 | docs/evidence/m0/README.md | 已审（全文 §0~§10） |
| 6 | docs/evidence/m0/tally_hours.py | 已审（行为级：独立重跑 + 与独立解析互证） |
| 7 | docs/evidence/m0/verify_claims.py | 已审（行为级：独立重跑 81 条逐条输出核对） |
| 8 | docs/evidence/t007/summary.md | 已审（全文） |
| 9 | docs/evidence/t007/matrix.jsonl | 已审（16/16 行全量独立复算） |
| 10 | docs/evidence/t008/README.md | 已审（全文） |
| 11 | docs/evidence/t008/matrix.md | 已审（5a 再生成逐字节对拍 + 内容全读） |
| 12 | docs/evidence/t008/compare_matrix.py | 已审（行为级：执行 + 输出对拍 + 退出码语义核对） |
| 13 | docs/evidence/t008/runs/red200-th1-s42.stdout | 已审（1a 对拍基准） |
| 14 | docs/evidence/t008/runs/full10000-th12-s44.stdout | 已审（1b 对拍基准） |
| 15 | docs/evidence/t008/runs/plan.txt | 已审（:51 定点命中） |
| 16 | docs/evidence/t009/summary.md | 已审（全文） |
| 17 | docs/evidence/t009/summarize.py | 已审（全文 440 行） |
| 18 | docs/evidence/t009/run_t009.sh | 已审（:85 定点命中） |
| 19 | docs/evidence/t009/runs/throughput_t{1,3,6,12}/throughput_t{T}.json | 已审（4 档全量复算，t12 全文） |
| 20 | docs/evidence/t010/summary.md | 已审（全文） |
| 21 | docs/evidence/t010/summarize.py | 已审（全文 259 行，预注册式） |
| 22 | docs/evidence/t010/README.md | 已审（:90-94 DPI 披露定点命中） |
| 23 | docs/evidence/t010/g1-lead-recheck.txt | 已审（全文） |
| 24 | docs/evidence/t010/runs/r5_t10000/out/t10000/frames.csv | 已审（第三实现独立重算 333.15/202.48 MATCH） |
| 25 | docs/evidence/t015/summary.md | 已审（全文） |
| 26 | docs/evidence/t015/README.md | 已审（全文；:31 D10 披露定点命中） |
| 27 | docs/evidence/t015/matrix.jsonl | 已审（16/16 行全量独立复算） |
| 28 | sim/src/bin/bench.rs | 已审（判定语义段 L75-101 / L840-920 / L923-931 / L1193-1531）；measure 与 CLI 解析段带理由略读（本卡为判定复算非代码审查，measure 路径行为经 final_hash 锚间接覆盖） |
| 29 | task-ledger.md | 已审（全文 12 任务行 + 纪律行） |
| 30 | docs/万阵-游戏前期策划报告.html | 已审（L455-569：表 6-0 全文 :477-490、M0 验收 :498、6.1 :525、R2/R7 :538/:543、Q5 :552）；其余章节跳过（理由：口径核对范围 = 表 6-0/6.1/R7/Q5 相关段） |
| 31 | docs/evidence/{t007,t008,t009,t010,t015}/environment.txt（5 档） | 跳过（理由：非本卡判定复算对象，五档一致性经 m0/README §7 汇总核对；独立开档复核移交 T013 环境复核项） |
| 32 | docs/evidence/t010/runs/ 其余档（r0~r8、w1 双盲）+ t007/runs/ 长跑 CSV | 跳过（理由：⑥ 按 D1 档内核对不重跑；t010 其余档判定已经 summary 生成 + G1 复算双路覆盖，本轮以 t10000 新鲜复测为主体） |
| 33 | render-spike/（含 src/stats.rs） | 跳过（理由：git diff 1d304a8..HEAD 空证零变更；统计式以 t010/summarize.py 预注册式为准并经归档 CSV 复算验证） |

共 33 项（多档组展开约 44 档）— 已审 30 / 跳过 3（各附理由）/ 覆盖率 91%（判定数据与判定行权重 100%：六验收全部原始数据档与判定行均已独立复算）。

## §3 执行序逐项结果（0~6 步）

| 步 | 项 | 期望 | 实测 | REAL_EXIT | 留痕档 |
|---|---|---|---|---|---|
| 0.1 | git rev-parse HEAD | b1b72e7… | b1b72e765dbb… 一致 | 0 | step0-git-head.txt |
| 0.2 | git diff --stat 1d304a8..HEAD -- render-spike/ | 空 | 空 | 0 | step0-git-diff-renderspike.txt |
| 0.3 | 空闲预检快照 | 留档 + 声明 | 留档；负载实录见 §2.1 | 0 | precheck-0-rv2.txt |
| 0.4 | cargo build -p sim --release -j 3 | 0 | 0（0.49s 无需重编，产物三 bin 齐） | 0 | build-sim-release-rv2.txt |
| 0.5 | 四 bin sha256 + render-spike 核验 | 与 Lead 留痕一致 | ce64db4e… 一致（余三 bin 留证） | 0 | bin-sha256-toolchain-rv2.txt |
| 0.6 | rustc -V / cargo -V | 1.98.1 | 1.98.1 / 1.98.1 | 0 | bin-sha256-toolchain-rv2.txt |
| 1a | sim red200-th1-s42 复跑 + 对拍 | diff 空 + hash 锚 | diff 0 / cmp 0；hash=0xde91d6a5a6e84d43；sample1350=0xe0cd7861ca5a211f | 0 | step1-1a-rv2.{cmd,stdout,stderr,diff} |
| 1b | sim full10000-th12-s44 复跑 + 对拍 | diff 空 + hash/units 锚 | diff 0 / cmp 0；hash=0x9d55c4ce4f7fd880；units=9970 | 0 | step1-1b-rv2.{cmd,stdout,stderr,diff} |
| 1c | sim t009 r5-sim-t1 复跑 | final_hash 黄金 | 0x564cf46fdf191710 命中 | 0 | step1-1c-rv2.{cmd,stdout,stderr} |
| 2a | bench 10k×t1×300 | hash 零容差 + us ±25% | hash OK；0.0811223（+14.88% IN）≤1µs | 0 | bench-2a-rv2.* |
| 2b | bench 10k×t12×300 | 同上 | hash OK；0.0719681（+2.11% IN）≤1µs | 0 | bench-2b-rv2.* |
| 2c | bench 50k×t1×30 | 同上 | hash OK；0.1206226（+10.99% IN）≤1µs | 0 | bench-2c-rv2.* |
| 2d | bench 50k×t12×30 | 同上 | hash OK；0.1437736（+19.22% IN）≤1µs | 0 | bench-2d-rv2.* |
| 2+ | 加速比带 + 承诺/止损断言 | 0.8~1.3 / ≤1µs / ≤2µs | 1.12720 IN；四配置全 ≤1µs ≤2µs；OVERALL PASS | 0 | bench-2-assertions-rv2.txt（脚本 assert-bench-rv2.py） |
| 3 | render-spike t10000 新鲜复测 | exit 0；判定行 PASS（预注册） | exit 0、4167 帧；avg 64.1002 / low 22.1551 → 判定 FAIL（超带 → 加样 2 次） | 0 | render-t10000-rv2.* |
| 3+ | 加样 2 次（处方） | 全档披露 | s2：3862 帧 59.4140/23.2732 FAIL/FAIL；s3：3864 帧 59.4301/23.8714 FAIL/FAIL | 0/0 | render-t10000-s{2,3}-rv2.* |
| 3r | avg/1% low 独立重算 | 公式同 summarize.py 预注册式 | 三样本全表 + 漂移 -80.76%~-82.17% / -88.21%~-89.06%（全 OUT） | 0 | render-recompute-rv2.{py,txt} |
| 3g | summarize.py 单档再生成 | exit 3 + t10000 判定行 | exit 3（缺档预期）✓；判定行产出但 = FAIL（预注册「且 PASS」偏离，如实披露） | 3 | render-summary-regen-rv2.{exit.txt,md} |
| 4 | arena throughput 512×12t×3 | exit 0；gph ±25% | 864,409.7（-21.39% IN）；per_game 4.165ms；hash_xor 逐位一致 | 0 | throughput_t12-rv2.* |
| 4r | gph 独立重算 | 公式同 t009/summarize.py | own=864,409.7=json 字段；判定 ≥10,000 PASS | 0 | throughput-recompute-rv2b.txt |
| 5a | compare_matrix.py 再生成 + diff | 退出码 0 + diff 空 | 退出码 0；diff 非空 = 仅「生成命令」自述行 1 行（argv 内嵌所致）；tail≥4 cmp 0 = 判定内容逐字节一致 | 0/1/0 | t008-matrix-regen-rv2.{diff,downgrade.txt} |
| 5b | t009 runs 复制 + summarize 再生成 + diff | 退出码 0 + 判定行一致 | 退出码 0；全档 diff 0 / cmp 0（逐字节一致，超预期） | 0 | t009-copy-rv2.diff（+ t009-copy-rv2/） |
| 5c | bench --summarize t015 再生成 + diff | 退出码 0 + diff 空 | 退出码 0；diff 0 / cmp 0（逐字节一致） | 0 | summarize-t015-regen-rv2.diff |
| 6.① | 16 配置 median/us 独立复算 | 与附录 A 逐位一致 | t015 16/16 MATCH、t007 16/16 MATCH（含 hash 列） | 0 | recompute-6-all-rv2.txt |
| 6.② | 加速比/Amdahl/16t 外推 | 逐位一致 | 8 行表 + 8 行拟合全 MATCH；1.001913/1.112480/TRIPPED MATCH | 0 | recompute-6-all-rv2.txt |
| 6.②附则 | 二次拟合 C(100000)÷speedup16 | 逐位一致 | t015 17.532997ms [保留] MATCH；t007 3907.133373ms [超界] MATCH；派工单锚值变体 4253.109497ms 披露 | 0 | recompute-6-all-rv2.txt |
| 6.③ | 吞吐 4 档 + 16t OLS 复算 | 逐位一致 | gph 四档字段一致；c1=0.6632/c2=10.3642/e16=1.3110/g16=1405967.6 全 MATCH | 0 | recompute-6-all-rv2.txt |
| 6.④ | 六哈希表三方一致 | README 表 = matrix 再生成 = 新鲜实测 | 6 值表 = 5a 再生成逐字节一致；1a/1b 新鲜 2/6 直接命中 | 0 | §4.④ |
| 6.⑤ | 归档 CSV 第三实现重算 | 333.15/202.48 自洽 | N=21656；333.15 / 202.48 MATCH；判定 PASS | 0 | recompute-5-archive-rv2.txt |
| 6.⑥ | 档内核对（D1 不重跑） | n=134 / 670.6s / 11.18≥10 / 口径行 / D10 行在档 | 全部在档命中（L72/L73/L75/L77、t015:31）；算式 11.1767→11.18 OK | 0 | recompute-6-all-rv2.txt 尾段 + line-refspotcheck-rv2.txt |
| 6.6.1 | 换算 8 格 + 对照词 | 逐位一致 | 8/8 格 MATCH（全精度 us 口径，含 0.679255 边界值）+ 词义与 bench.rs:1449-1458 一致 | 0 | recompute-6-all-rv2.txt |
| 6.W1 | 台账独立解析 + 脚本重跑 | 557 / 1246=20.77h + 81 条 | 独立解析 MATCH×2；tally 同值；verify 81/81 | 0 | recompute-w1-rv2b.txt / tally-hours-rv2.txt / verify-claims-rv2.txt |
| 6.D5 | 口径混用检查 | 三对口径逐项 | 无混用（表见 §5） | — | §5 |
| ref | 行号引用定点抽查 | :85/:51/:90-94/:31 命中 | 全命中；附录 A 表 span=L72-89（派工单 L72-88 差 1 行） | 0 | line-refspotcheck-rv2.txt |

## §4 六验收复算结论（独立实现，逐位对照）

### ① 四采样点单线程 µs —— 复算通过
t015 侧 16 配置 median / us_per_unit_tick / final_hash 全部 MATCH（median 语义 = bench.rs:923：奇取中、偶取中二均值）；
① 判定（4 t1 点 0.060651/0.072196/0.070617/0.108681）与 t015/summary.md §① 逐位一致。t007 基线侧 16 配置同样全 MATCH
（4.012138/19.846795/41.010103/200.625962）。新鲜 bench 4 配置 hash 零容差全 OK、us 全部 ≤1µs（承诺）≤2µs（止损）。

### ② @10k 加速比 —— 复算通过（TRIPPED 如实）
逐行表与 Amdahl 拟合表（c1/c2/R²/残差/×16）t015 与 t007 双侧 8+8 行全 MATCH；
锚定 10k 判定行 = 1.001913 / 1.112480 / TRIPPED 逐位一致（t007 时序 7.221411 / 9.416188 / OK 同样逐位一致，
runs/recompute-6-anchorfix-rv2.txt 3 行 MATCH）。新鲜加速比 2a/2b = 243366900/215904400 = 1.12720 ∈ 0.8~1.3
（结构性 ≈1.0 判定复现）。注：recompute-6-all-rv2.txt 内 T007 块的 anchor 对照行显示 FAIL 系脚本把 t015 参照常量
复用于共享函数的标签瑕疵，计算值本身与 t007/summary.md:42 逐位一致（anchorfix 档钉死）。

### ②附则 极限十万 —— 复算通过（[保留] 如实）
二次拟合（bench.rs:887-920 语义）：t015 a=62.759210207 / b=0.000918316 / R²=0.999960 / 残差 24223.712 全 MATCH，
C(100000)=15459082.3 ns/tick MATCH，speedup16（50k 档，bench.rs:100）=0.881714 MATCH，17.532997 ms ≤ 22ms [保留] MATCH。
t007 侧 a=773.554196350 / b=3.997072319 MATCH，C=40048078610.0 MATCH，speedup16（50k）=10.249990 MATCH，
3907.133373 ms [超界] MATCH。**派工单锚值专项**：派工单写 speedup16=9.416188（实为 10k 档值），
代入得 4253.109497 ms ≠ 归档 3907.133373 ms——归档值只能由 50k 档 10.249990 生成，与 bench.rs:100 语义自洽；
按上报条件停该步上报（阻塞项 2），双取值并列披露、不拍板。

### ③ 吞吐（降规模）—— 复算通过
从 t009 四档 JSON 原始 wall_s 数组独立取中位（statistics.median）重算：四档 gph/per_game_ms 与字段值逐位一致
（166557.9 / 461088.1 / 793548.1 / 1099638.6）；16t OLS（elapsed=c1+c2/T）c1=0.6632 / c2=10.3642 /
elapsed(16)=1.3110s / 吞吐16=1,405,967.6 全 MATCH（t009/summary.md:180-181）。新鲜复测 864,409.7 场/h
（-21.39%，±25% 带内）判定 ≥10,000 PASS；final_hash_xor=0xd1b28b373f7791e3 与 t009 归档逐位一致
（跨卡、跨二进制世代的行为同一性旁证）。

### ④ 同种子重放 —— 复算通过（bit-exact）
5a 再生成（compare_matrix.py 对 t008/runs 归档重算）退出码 0，全 29 局矩阵/加样/锚局/种子互异/断言 1~3 判定内容与
docs/evidence/t008/matrix.md **逐字节一致**（唯一差异 = 第 3 行「生成命令」自述行，机械性 argv 内嵌；tail≥4 cmp 0 佐证）。
新鲜复跑：1a/1b 对归档 stdout diff 0 + cmp 0（逐字节），1c 黄金 0x564cf46fdf191710。
六哈希表三方一致：t008/README.md §2 六值 = 5a 再生成表逐字节一致 = 新鲜实测 2 值（red200-s42、full10000-s44）直接命中，
其余 4 值经再生成对拍覆盖。覆盖域披露（red200=移动段 / full10000=战斗段）在档、未见外推滥用。

### ⑤ 渲染 10k 同屏 —— 归档自洽 PASS；新鲜复测不可复现（本轮唯一实质开口）
- 归档侧：对 docs/evidence/t010/runs/r5_t10000/out/t10000/frames.csv 以第三实现重算（预注册式）：
  N=21656、avg_fps=333.15、1% low=202.48 —— 与 t010/summary.md:14/:31 逐位 MATCH、判定 PASS（归档自洽，三实现一致）。
- 新鲜侧：3 样本（65s 采集逐样本 precheck 在档）avg 64.1002 / 59.4140 / 59.4301，1% low 22.1551 / 23.2732 / 23.8714，
  漂移 -80.76%~-82.17% / -88.21%~-89.06%（全超 ±25% 带，已按处方加样 2 次全档披露）；summarize.py 单档再生成
  （exit 3 缺档预期 ✓）判定行 = **FAIL**（22.16 < 45；另 s2/s3 的 avg 亦 <60）——与派工单预注册「判定行仍须产出且 PASS」
  偏离，如实披露、禁止事后改预期。
- 环境判定：meta window_resolution_actual="0x0"（本轮 3 样本 + 前轮遗留 3 样本全同）vs t010/README.md:90-94
  「各档 meta.json window_resolution_actual 一致 = 2400x1350（125% DPI，保守方向）」与 G1 fresh 同值——窗口呈现路径
  已实质变化；precheck 档实录非空闲负载（SLDWORKS / AcWebBrowser 系 / 钉钉 / uTools 等，CPU 7~38%）。
  按附录 A「共享负载下的红/绿均不可信」：本轮红**不能推翻**归档绿（另有 G1 fresh 329.33/177.95 PASS 先例），
  亦**不能确认**之。⇒ 判定：⑤ 归档数据自洽、判定在其量测窗口内成立；当前环境不可独立复核；
  必改 = 干净窗口补测或 T013 条件性披露（阻塞项 1）。渲染腿后果路径（报告 :525）在 ⑤ 最终判 FAIL 时才触发，不提前引用。

### ⑥ 内存长跑 —— 档内核对通过（按 D1 不重跑）
t007/summary.md L72（n=134）、L73（窗口 670.6s）、L75（高水位稳定性口径行）、L77（判定行 达标）全部在档；
670.6/60 = 11.1767 → 11.18 ≥ 10 min 算式核 OK（runs/recompute-6-all-rv2.txt 尾段）。
t015/README.md:31 「⑥ 内存未重跑（无内存行为变更，D10 预登记；summarize 披露行在档）」定点命中——D10 预登记如实。

### 6.1 换算复核 —— 通过
公式 = 报告 :525 加粗式（所需加速比 = 单位数 × 单线程每单位每 tick 成本 ÷ 每帧模拟预算）；
实现 = bench.rs:1449-1458（req = u × us / (budget_ms×1000)，us 取全精度记录值）。
t015/summary.md §④ 表 8 格逐位 MATCH（含 50k@8ms = 0.679255 全精度边界值——用显示位 0.108681 反算得 0.679256，
证实归档用全精度，复算同式同值）；对照词「承诺线内（≤2×）」与 PROMISE_SPEEDUP=2.0 / STOP_SPEEDUP=4.0 三段语义一致。

### W1 投入复算 —— 通过
独立解析 task-ledger.md 耗时列（转义竖线切分）：预备周 = 150+15+18+35+110+130 + T007(10-04 段) 99 = **557 min = 9.28h**；
W1 = T007(10-05 段) 91 + 605+145+125+140+140 = **1246 min = 20.77h** —— 与 m0/README §8 小计行双 MATCH。
tally_hours.py 独立重跑输出同值（557 / 1246 / 20.77h）；verify_claims.py 独立重跑 = **汇总: 81 条 | PASS 81 | FAIL 0**（REAL_EXIT=0）。

## §5 口径混用检查（D5：三对口径逐项清单）

比对源：报告表 6-0 原文（:485 / :490 / :498）+ 6.1（:525）+ R2（:538）vs m0/README §0~§8 + t007/t009/t010/t015 判定行 + AGENTS.md「当前状态」节。

| 口径对 | 报告原文锚 | m0/README 与 summary 判定行 | AGENTS.md 状态节 | 结论 |
|---|---|---|---|---|
| 降规模 / 全规模 | :485「每方 100 模拟单位（共 200）、单局 ≤ 60s（≤ 1,800 ticks）…与全规模对局两套口径不得混用」；:538「1 万场/小时（降规模口径…全规模约 530 场/小时，仅作质量校验、不计吞吐）」 | §0「③ 吞吐（降规模）」带标签；§3 ①口径逐字引 :485+:538；t009 判定行 games=512 降规模参数在档；④ red200/full10000 分规模标注 + 覆盖域披露「不得外推」 | 「③ PASS（1,099,638.6 场/h@12t ≈110× 裕度）」未带「降规模」字样（数字为降规模值；全规模 530 未混入） | 无混用；S-1：AGENTS.md 补「降规模」标签 |
| 单线程基线 / 多线程 | :483「①…单线程基线；多线程收益由②加速比单独刻画」；:498① 同句加粗 | §1①标题「四点单线程 µs」、判定行全单线程；§2② 12t 实测/16t 外推独立成节；summary ①表「单线程」、②表分线程档、④表「单线程 us」列 + 换算公式含「单线程」 | ①（四点 µs）与 ②（1.001913×@12t）分列，无交叉 | 无混用 |
| 含镜像 AI 决策 / 不含落库 | :498③「（含镜像 AI 决策、不含落库；AI 池扩档后须重测）」 | §3 ①口径逐字引全句；t009 JSON ai_decision_cost 键在档；BattleLog「不含落库——表 6-0 口径」（T005 起） | ③ 未重复短语但无矛盾表述 | 无混用 |

移交项呈现核查（不放行不粉饰）：② owner 裁决点（m0/README §0 总注 1「按字面呈现 TRIPPED，不放行不粉饰」+ AGENTS.md「owner 裁决点」+ T015 收口记录三方一致）✓ 如实；③ 镜像 sanity H2 结构性红偏（§0 总注 2 + §3 ③附带披露 + t009 判定行）✓ 如实移交 T013。均未放行、未粉饰。

## §6 预注册预期 vs 实测（偏差逐条，禁止事后合理化）

| 类别 | 预注册（派工单） | 实测 | 处置 |
|---|---|---|---|
| bit-exact（1a/1b/1c、5c） | 零容差 | 全部命中 | — |
| 5a 再生成 diff | 逐字节空 | 非空（仅 argv 自述行 1 行；判定内容逐字节一致） | 按派工单预案「如实披露 diff 内容并降级排查」执行（t008-matrix-regen-rv2.downgrade.txt）；根因 = 工具内嵌生成命令自述行，其自带命令形态机械性不可达 byte-empty（S-2） |
| 5b 判定行一致 | 判定行逐字节一致 | 全档逐字节一致（diff 0/cmp 0，优于预期） | — |
| bench timing（①µs 等） | ±25% 带 | +2.11%~+19.22% 全带内 | — |
| ② 新鲜加速比（2a/2b） | 0.8~1.3 | 1.12720 带内 | — |
| ③ 吞吐 | ±25% 带、≥10,000 | -21.39% 带内、PASS | — |
| ⑤ fps 漂移 | ±25% 带 | -79%~-89%（3 样本全超带） | 处方「加样复测 2 次全档披露」已执行（+2 样本同向）；判定仍按验收线独立判 = 本轮 FAIL 行如实入档 |
| 渲染 summarize 单档再生成 | exit 3 + 判定行 PASS | exit 3 ✓ + 判定行 FAIL | **偏离如实披露**（本报告 §4⑤ / 阻塞项 1），不改预期 |
| render-spike.exe sha256 | = Lead 留痕 ce64db4e… | 一致 | — |
| 双盲锚值（本单 vs 源档） | 一致 | 1 处不一致（t007 ③ speedup16） | 按上报条件「停该步上报不拍板」处置（阻塞项 2） |

## §7 发现清单（P0 / P1 / P2 / S / N）

**P0（伪造/缺失证据/判定失实/确定性失败）：无。**

**P1（必改）**
- **P1-1（置信度：数据面 100）验收⑤ 新鲜复测不可复现且判定行 FAIL，预注册「PASS」期望偏离；承测环境不合规（非空闲 + window_resolution_actual=0x0）故不可推翻亦不可确认归档 PASS。**
  依据：runs/render-recompute-rv2.txt:4-6、render-summary-regen-rv2.md:22-26、render-t10000-rv2/t10000/meta.json（0x0）、
  precheck-3-rv2.txt（负载实录）、t010/README.md:90-94（归档 2400x1350 声明）、t010/g1-lead-recheck.txt:19-20（G1 fresh PASS 先例）。
  修复：干净量测窗口补测一次 t10000 并重判（最小修复指令 1）；或 T013 写入条件性披露 + 渲染腿后果路径限定语。

**P2（次级必改）**
- **P2-1（置信度 100）派工单 t007 ③ speedup16 锚值错误**：dispatch-rv.md 第 6 步「speedup16=9.416188 → 3907.133373 ms」
  内部不自洽；9.416188 为 10k 档值（t007/summary.md:42），③ 计算用 50k 档 10.249990（t007/summary.md:48、bench.rs:100）。
  复算证据：runs/recompute-6-all-rv2.txt:101-107（用 10.249990 → 3907.133373 MATCH；用 9.416188 → 4253.109497 不符）。
  已按 D9/附录 B.2⑤ 停该步上报处置；修复 = 勘误入档 + T013 钉死「50k 档」口径。
- **P2-2（置信度 95）派工单行号引用 off-by-one（B.2④ 类复发）**：「t015/summary.md 附录 A 表 L72-88」实为 L72-89
  （2d 行在 L89；runs/line-refspotcheck-rv2.txt:19-21）。值正确、未致误引后果；计入固化检查单升级计数
  （T008 P1-1 后同类第 2 次；连同 P2-1 本单共 2 处派工单数值/行号类缺陷，建议 Lead 按 B.2 复盘派工质量）。

**S（建议）**
- S-1：AGENTS.md「当前状态」③ 行补「降规模」口径标签（沿 T011 M-1 精度勘误先例，随 T013/收口统一）。
- S-2：再生成比对类预注册措辞修订为「除『生成命令』自述行外逐字节一致」或固定调用形态（compare_matrix.py 内嵌 argv 的机制性后果）。
- S-3：T013 引用 summarize.py FAIL 分支「渲染腿后果记录行」文本时须带触发条件限定（仅 ⑤ 最终判 FAIL 时生效），防提前误引。

**N（注记）**
- N-1：review/runs/ 开工前存在一轮同任务中断遗留档（10:04~10:29）——本轮 -rv2 零碰撞命名、遗留档原样保留仅作旁证并降级（其 render-t10000 档 stderr 37s 窗口与 frames 65.015s 不自洽、数据疑与 retry 混位）；去留移交 Lead。
- N-2：本轮审核工具三起失误如实留痕（吞吐路径假设首跑 exit 1 → retry 通过；W1 正则反斜杠被 heredoc 传输吃掉首跑 exit 1 → b 版通过；综合脚本写入被命令长度截断 → 分块重写通过）；零影响判定数据。
- N-3：render-spike.exe sha256 与历史环境档（264A95…）不一致 = D2 预告的构建路径 provenance 差异；确定性锚以 final_hash 为准全数命中。
- N-4：D12「五 bin sha256」实为四 bin（sim/bench/arena/render-spike）——措辞勘误级。
- N-5：t007/t015 summary 内部节号「③ 外推十万」= 报告验收②附则（m0/README §0 总注 3 已澄清）——本轮按报告编号组织复算，未发现节号混用。

## §8 优点（Strengths）

- T011 成档纪律经受住独立复算：verify_claims 81 条双向核对独立重跑全 PASS、m0/README 判定行逐字引用、tally 双盲分段——本轮全部独立复算逐位通过（recompute-6-all-rv2.txt 全 MATCH）。
- 确定性证据链跨二进制世代成立：throughput final_hash_xor 跨 T009→本轮逐位一致、bench 16 配置 hash 全对、1a/1b 逐字节 diff+cmp 双零、1c 黄金命中。
- 工具确定性扎实：5b/5c 再生成逐字节一致（零时间戳、零手改空间），5a 判定内容逐字节一致。
- bench.rs 判定口径常量化显式（PROMISE/STOP/ANCHOR/SPEEDUP16_SOURCE_UNITS + ④ 对照词代码判定），复算可逐位对照、无手算漂移空间。

（完；留痕索引 = docs/evidence/m0/review/runs/ 全部 -rv2 档 + 既有遗留档，一命令一档含 REAL_EXIT。）
