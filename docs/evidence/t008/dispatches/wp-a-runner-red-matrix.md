# T008 派工单 WP-A（worker-1）：跑批基建 + 降规模矩阵 + 锚 2 复现

## 纪律（先读，违反即停）

1. 你是执行层：只按本单干活。范围外决策（发现需要改 `sim/` 代码、断言对不上、口径含糊、数值双盲不一致）→ **停下上报主会话，不拍板**。
2. 红线：`sim/` 目录**零改动**（T008 为验证卡，预期零代码变更）。你的唯一产出是 `docs/evidence/t008/` 下的脚本与证据档。
3. 全部命令与产出文件一律**相对仓库根路径**（公开仓卫生：禁止 `C:/Users/...` 等机器绝对路径进入任何产出文件）。
4. 数值纪律（连续四卡派工单笔误教训）：本单给出的构成串是「主会话速算参考值」——你必须按 D2 算法**脚本重算**并断言总和，与参考值逐字一致才使用；不一致 → 上报停止，不得静默改数。
5. 执行中断（配额/消息）→ 已落盘部分保留，向主会话报告断点位置（exits.txt 最后一行 + 在跑局）。
6. 量测窗口机器空闲独占：跑批期间不并行其他重负载（cargo 门禁/构建在跑批前完成）。

## 背景

T008 = 同种子重放确定性验证（验收④）。任务卡 `taskset/t008-replay-hash.md`：3 种子 × 4 线程档 × 2 规模 = 24 次独立进程，终局哈希 + ≥3 个中间采样哈希零 mismatch。本单只做：跑批基建 + 降规模 12 局 + 跨进程加样 2 局 + 锚 2 复现 1 局。全规模 14 局在 WP-C（另一执行者）。

## D1 种子 / 线程档 / 规模（主会话定稿）

- seeds `{42, 43, 44}`；threads `{1, 3, 6, 12}`。
- 降规模 `red200`：每方 100（共 200），`--ticks 1800`（= `sim/src/world.rs:145` `TICK_CAP_REDUCED` 逐字，打开核对）。
- 全规模 `full10000`：每方 5000（共 10000），`--ticks 14400`（= `sim/src/world.rs:149` `TICK_CAP_FULL = 8 * 60 * 30`，打开核对）。

## D2 构成映射（逐字口径 = `sim/src/bin/bench.rs:284-301` `composition_for`）

算法（打开 bench.rs 核对后执行）：`per_side = units / 2`；六兵种 `q = per_side / 6`、余数 `r = per_side % 6` 按**表序前 r 个 +1**；表序 = Shieldman/HeavyKnight/Pikeman/Swordsman/Archer/Militia（bench.rs:288-295）。

主会话速算参考值（**仅供双盲核对，以你的脚本计算为准**）：

- 降规模（units=200 → per_side=100）：`shieldman:17,heavyknight:17,pikeman:17,swordsman:17,archer:16,militia:16`
- 全规模（units=10000 → per_side=5000）：`shieldman:834,heavyknight:834,pikeman:833,swordsman:833,archer:833,militia:833`

动作：写 `docs/evidence/t008/gen_comp.py`（python，零第三方依赖）按上述算法生成两串 + 断言（每方总和 == 100 / 5000，逐 kind 数值 = q 或 q+1）→ 写 `docs/evidence/t008/comp.txt`（含算法引用行 + 两串 + 断言结果）。与参考值逐字比对，**不一致即上报停止**。全规模串写入 plan.txt 供 WP-C 用。

## D3 采样点（主会话定稿，终局哈希走五行摘要 `hash=` 行）

- 降规模：`--hash-samples 0,450,900,1350`（4 个中间点 + 终局 1800）。
- 全规模：`--hash-samples 0,3600,7200,10800,12600`（5 个中间点 + 终局 14400）。选点意图：0=布阵快照、3600/7200=移动段、10800/12600=**接敌后**（T006 归档 `docs/evidence/t006/runs/fullscale.stdout` 终局 units=9959<10000 证明战斗段已进入哈希；接敌 ≈t10000 见 T006 台账）。

## D4 证据契约（主会话定稿；WP-B 分析器按同一契约实现）

- `docs/evidence/t008/runs/plan.txt`：**全 29 局**清单，每行 `<run_id> <完整相对命令>`，固定顺序（见 D6）。
- `runs/<run_id>.cmd`：该局完整命令一行（相对路径）。
- `runs/<run_id>.stdout` / `runs/<run_id>.stderr`：逐字捕获。
- `runs/exits.txt`：每局一行 `<run_id> REAL_EXIT=<code> wall_ms=<ms>`。
- run_id 命名：矩阵局 `<scale>-th<T>-s<SEED>`（scale ∈ `red200`|`full10000`）；跨进程加样后缀 `-r2`/`-r3`；锚局 `anchor1-t006-fullscale` / `anchor2-t006-10k300`。
- 矩阵局 24 = red200×th{1,3,6,12}×s{42,43,44} + full10000×同 12。
- 加样局 3 = `red200-th12-s42-r2`、`red200-th12-s42-r3`（WP-A 做）、`full10000-th12-s42-r2`（WP-C 做）。
- 锚局 2 = 见 D7（anchor2 在本单；anchor1 在 WP-C）。
- stdout 行语法（分析器解析契约，出处 `sim/src/main.rs`）：sample 行 `sample=<tick> hash=0x<16hex>`（main.rs:316）；摘要 `seed=` / `ticks=` / `units=` / `final_tick=` / `hash=0x<16hex>`（main.rs:326-330，审核轮 P1-1 修订）；stderr `threads=` / `elapsed_ms=`（main.rs:348-349）。

## D5 跑批脚本 `docs/evidence/t008/run_t008.sh`（断点续跑版）

- 用法：`./docs/evidence/t008/run_t008.sh <run_id 前缀>...`（如 `anchor2 red200` / `anchor1 full10000` / `all`）；无参 = all。
- 逐局执行 plan.txt 中匹配前缀的局：
  - exits.txt 已有该 run_id 的 `REAL_EXIT=0` 行 → skip（幂等）；
  - 已有**非 0** 行 → 不覆盖不重跑（断言 3：不得静默重跑遮蔽），脚本结束时汇总报告该行；
  - 无 exit 行但有残 stdout（被中断的半局）→ 覆盖重跑（半局非证据）。
- watchdog（防死锁兜底，非验收判据）：`timeout`——red 局 1800s；full th12 3600s / th6 7200s / th3 14400s / th1 28800s（估值来源：T006 归档 fullscale `elapsed_ms=1017022` @th12 与 T007 12 线程加速比 7.22× 外推，约 4× 余量；如你按归档数据重算出显著不同估值，按你的估值执行并在脚本注释留痕换算式）。watchdog 杀 = REAL_EXIT=124 如实入档。
- 头部注释：机器空闲独占声明 + 复现命令 + 断点续跑语义。全程相对路径。
- 开跑前门禁（跑批前一次完成）：`cargo check --workspace -j 3` **0 警告**留痕 `docs/evidence/t008/check.txt`；`cargo build -p sim --release -j 3`。
- 脚本结尾打印 `BATCH_DONE rc=<n>`（0 = 本次选择范围内全部计划局 REAL_EXIT=0）。

## D6 本单执行顺序

1. 门禁 + 构建（check.txt）。
2. gen_comp.py → comp.txt（双盲核对通过才继续）。
3. 生成 plan.txt（全 29 局，固定顺序：anchor1 → full th12 s42/43/44 → full th6 → full th3 → full th1 → full-r2 → red th12 → red th6 → red th3 → red th1 → red-r2/-r3 → anchor2；快档先行尽早暴露 mismatch——顺序可调但须在 plan.txt 注释留痕理由）。
4. `./docs/evidence/t008/run_t008.sh anchor2 red200`（15 局：12 矩阵 + 2 加样 + 1 锚 2；降规模局秒级）。
5. 幂等自检：重跑同一命令，验证全部 skip + `BATCH_DONE rc=0`，输出并入 check.txt 尾部。

## D7 锚 2 复现（跨卡黄金锚，逐字节 diff）

`anchor2-t006-10k300` 命令**逐字复现** T006（comp 串逐字取自 `docs/evidence/t006/batch.sh`，**打开该文件核对后再写入 plan.txt，禁止凭记忆**）：

```
./target/release/sim.exe --comp shieldman:840,heavyknight:830,pikeman:830,swordsman:830,archer:830,militia:840 --ticks 300 --hash-samples 0,100,200,300 --threads 12 --seed 42
```

判定 = stdout 与 `docs/evidence/t006/runs/10k-s42-t12.stdout` **逐字节 diff**（预期 identical）→ 结果写 `runs/anchor2-diff.txt`。差异 → 如实记录 + 上报（禁止静默重跑）。

## 产出清单（WP-A）

`docs/evidence/t008/`：`gen_comp.py`、`comp.txt`、`run_t008.sh`、`check.txt`、`runs/plan.txt`、`runs/anchor2-*`（.cmd/.stdout/.stderr/anchor2-diff.txt）、`runs/red200-*` ×14（.cmd/.stdout/.stderr）、`runs/exits.txt`（15 行）。

## 报告格式（返回主会话）

1. 双盲核对结果（参考值 vs 脚本值，逐串）；2. 门禁三断言（check 0 警告 / 构建 REAL_EXIT / 幂等自检）；3. 15 局 REAL_EXIT 清单 + red 矩阵 12 局终局 hash 一览（读 stdout，不手抄——粘贴原文行）；4. 锚 2 diff 判定；5. 任何上报事项。
