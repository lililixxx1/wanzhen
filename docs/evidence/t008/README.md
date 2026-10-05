# T008 证据档索引：M0-07 同种子重放确定性验证（验收④）

- 判定产物 = `matrix.md`（由 `compare_matrix.py` 代码生成；哈希值/判定禁止手抄）。
- 判定结果：**全 PASS**（WP-D D1 运行退出码 0，见 §2）；29 局全 REAL_EXIT=0（见 §6）。
- 任务卡 = `taskset/t008-replay-hash.md`；执行规划留痕 = `dispatches/`（WP-A/B/C/D 派工单全文）。
- 生产日期 2026-10-05（基准机 A 本地时区 +08:00）。

## 1. 口径

### 1.1 矩阵设计（24 局 + 加样 3 + 锚 2 = 29 局，每局独立进程）

| 规模 | 每方 / 总共 | ticks | 标准采样点（+ 终局） | 依据 |
|---|---|---|---|---|
| red200 | 100 / 200 | 1800 | 0/450/900/1350 + 终局 1800 | sim/src/world.rs:145 `TICK_CAP_REDUCED` |
| full10000 | 5000 / 10000 | 14400 | 0/3600/7200/10800/12600 + 终局 14400 | sim/src/world.rs:149 `TICK_CAP_FULL = 8*60*30` |

- 设计矩阵 = 种子 {42,43,44} × 线程档 {1,3,6,12} × 规模 {red200, full10000}；终局哈希取五行摘要 `hash=` 行；哈希输出口径 = T002 起的 `state_hash`（tick → rng 4 状态字 → 单位逐字段，含 rng_state 与 tick 计数）。
- 加样 3 局（跨进程补充，超出 24 局如实标注）：`red200-th12-s42-r2` / `-r3`、`full10000-th12-s42-r2`。
- 锚局 2（跨卡黄金锚，见 §3）：`anchor1-t006-fullscale`、`anchor2-t006-10k300`。

### 1.2 构成映射（bench.rs:284-301 `composition_for` 逐字口径）

```rust
// sim/src/bin/bench.rs:284-301（左侧为实际行号；代码逐字）
284  fn composition_for(units: usize) -> (usize, Vec<(UnitKind, usize)>) {
285      let per_side = units / 2;
286      let q = per_side / 6;
287      let r = per_side % 6;
288      let kinds = [
289          UnitKind::Shieldman,
290          UnitKind::HeavyKnight,
291          UnitKind::Pikeman,
292          UnitKind::Swordsman,
293          UnitKind::Archer,
294          UnitKind::Militia,
295      ];
296      let comp = kinds
297          .iter()
298          .enumerate()
299          .map(|(i, k)| (*k, q + usize::from(i < r)))
300          .collect();
301      (per_side, comp)
```

- 结果串（`gen_comp.py` 重算，留痕 `comp.txt` RESULT: PASS，与主会话速算参考值双盲一致）：
  - red200（units=200 → per_side=100, q=16, r=4）：`shieldman:17,heavyknight:17,pikeman:17,swordsman:17,archer:16,militia:16`
  - full10000（units=10000 → per_side=5000, q=833, r=2）：`shieldman:834,heavyknight:834,pikeman:833,swordsman:833,archer:833,militia:833`
- 完整逐局命令见 `runs/plan.txt`（一行一局，相对仓库根执行）。

### 1.3 采样点选点意图与覆盖域披露

- full10000 的 10800/12600 = 接敌后点（战斗段覆盖）；战斗段已进入哈希的实证 = full 规模 14/14 局终局 `units<10000`（matrix.md §7.2：9959/9969/9970——出现阵亡即索敌/击杀/retain 路径已执行）。
- red200 的 0/450/900/1350 = 移动段采样；1800 ticks 内双方不接敌（T003 观察：队首相距 999m、合速 ≤0.4m/tick；终局 units=200 双证）= 移动段覆盖。
- **覆盖域如实披露**：red200 只覆盖移动段确定性；战斗段确定性由 full10000 覆盖。不得由本档外推「战斗段已在 red200 规模下验证」。

## 2. 对拍结论（引自 matrix.md 判定行）

WP-D D1 判定记录（2026-10-05）：

```
python docs/evidence/t008/compare_matrix.py docs/evidence/t008/runs
→ 退出码 0（0 = 全 PASS；退出码语义 0/1/2/3/5 = 全 PASS / I/O 错 / 输入格式错 / 缺局或未完成 / mismatch）
```

（对应关系：断言 1 = 24 局零 mismatch；断言 2 = 中间哈希一致；加样局行 = 加样跨进程逐列一致；锚局行 = 锚逐字节一致。）

matrix.md §8 原文引用：

> - 断言 1（零 mismatch：每 (scale,seed) 组合 4 线程档 × 全部采样列 + 终局列逐位一致）: PASS — 6/6 组合逐位一致
>   - 组合明细: red200-s42 PASS；red200-s43 PASS；red200-s44 PASS；full10000-s42 PASS；full10000-s43 PASS；full10000-s44 PASS
> - 断言 2（中间哈希：每 scale 标准采样列组内一致）: PASS
>   - red200: PASS（12/12 判定点 = 3 组合 × 4 中间列组内一致）
>   - full10000: PASS（15/15 判定点 = 3 组合 × 5 中间列组内一致）
> - 断言 3（mismatch 全量如实列出）: 未触发（无 mismatch）
> - 加样局（跨进程同配置逐列一致）: PASS — red200-th12-s42-r2 PASS；red200-th12-s42-r3 PASS；full10000-th12-s42-r2 PASS
> - 锚局（vs T006 归档逐字节 diff）: PASS — anchor1-t006-fullscale PASS；anchor2-t006-10k300 PASS
> - 种子互异 sanity（同 (scale,threads) 下 3 种子终局哈希两两不同）: PASS — 8/8 个 (scale,threads) 下 3 种子终局哈希两两不同
> - 退出码: 0

关键终局哈希（引自 matrix.md §1/§2 判定表；同组合 4 线程档逐位一致）：

| 规模 | seed | 终局哈希 | 终局 units |
|---|---|---|---|
| red200 | 42 | 0xde91d6a5a6e84d43 | 200 |
| red200 | 43 | 0x999a5237d3b1a9b2 | 200 |
| red200 | 44 | 0x44348b8042997ed3 | 200 |
| full10000 | 42 | 0xd921c95a9bf1db66 | 9959 |
| full10000 | 43 | 0x7d34b08102e34260 | 9969 |
| full10000 | 44 | 0x9d55c4ce4f7fd880 | 9970 |

逐列全量矩阵表（含每列判定）见 matrix.md §1/§2；§5「不一致全量明细」为空（无 mismatch）；§6「缺局/失败局清单」为空（29 局齐备且 REAL_EXIT=0）。

## 3. 锚对照表（跨卡黄金锚，逐字节 diff）

| 锚局 | 期望值来源（现场读取） | 判定（matrix.md §4 原文） | 实测 = 参照 hash |
|---|---|---|---|
| anchor1-t006-fullscale | docs/evidence/t006/runs/fullscale.stdout（72 bytes） | PASS — 逐字节一致（72 bytes） | 0x29980473140ed39e |
| anchor2-t006-10k300 | docs/evidence/t006/runs/10k-s42-t12.stdout（207 bytes） | PASS — 逐字节一致（207 bytes） | 0x9e17408bc4b7b909 |

- 锚局命令逐字复现 T006 归档：
  - anchor1 = T006 fullscale 命令逐字复现（`--ticks 14400 --threads 12 --seed 42`，无 `--hash-samples`）→ 与 T006 归档逐字节 diff（留痕 runs/anchor1-diff.txt，WP-C 侧；双侧 sha256 一致）。
  - anchor2 = T006 `10k-s42-t12` 命令逐字复现（`--ticks 300 --hash-samples 0,100,200,300 --threads 12 --seed 42`）→ 逐字节 diff（留痕 runs/anchor2-diff.txt，WP-A 侧；双侧 sha256 一致）。
- 两套构成串用途差异（如实披露）：
  - **锚局用 T006 的 COMP5000 串**（逐字取自 docs/evidence/t006/batch.sh:8）：`shieldman:840,heavyknight:830,pikeman:830,swordsman:830,archer:830,militia:840`。用途 = 跨卡复现零漂移验证——必须与 T006 同命令同构成才能逐字节对拍；锚局不属于 24 局矩阵。
  - **矩阵局用 bench 映射串**（bench.rs:284-301，见 §1.2）：`shieldman:834,heavyknight:834,pikeman:833,swordsman:833,archer:833,militia:833`。用途 = D2 口径的 24 局对拍矩阵（与 T007 bench 套件同一映射）。
  - 两串 total 均为 10000，但兵种分布不同（840/830 vs 834/833）——锚局哈希与矩阵局哈希不可互比；锚局只与 T006 归档比。

## 4. 复现命令

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

- run_t008.sh 幂等语义：对已全 0 的 exits.txt 全部 SKIP（check.txt 门禁 3 实测 15/15 SKIP 留痕）；从头全量执行需以空 exits.txt / 干净 runs 目录为前提。

## 5. 证据文件索引（本目录）

| 文件 | 角色 |
|---|---|
| matrix.md | 对拍矩阵判定产物（compare_matrix.py 生成；WP-D D1 退出码 0） |
| compare_matrix.py | 对拍分析器（退出码 0/1/2/3/5；夹具自测见 fixtures-selftest.txt） |
| fixtures-selftest.txt | 分析器夹具自测记录（PASS/FAIL/缺局 3 必选路径 + 锚/加样/格式错/失败局 4 附加分支；防假绿） |
| run_t008.sh | 跑批脚本（断点续跑/幂等/watchdog 兜底；按 run_id 前缀选择集） |
| gen_comp.py | 构成映射重算脚本（bench.rs:284-301 转写 + 双盲核对） |
| comp.txt | 构成映射生成记录（两串 + 断言 PASS） |
| check.txt | WP-A 门禁留痕（check/build 0 警告 + 幂等自检 15/15 SKIP） |
| environment.txt | 环境档（基准机 A 参数/工具链/二进制 sha256/git HEAD/空闲独占声明） |
| events.md | 执行事件流水（主会话维护；本 README §6 转记） |
| dispatches/ | WP-A/B/C/D 派工单全文（流程文档，非证据档） |
| runs/plan.txt | 29 局清单 + 完整相对命令（跑批契约） |
| runs/&lt;run_id&gt;.cmd / .stdout / .stderr | 29 局 × 3 档（相对命令 / 控制台输出 / stderr）；共 87 档 |
| runs/exits.txt | 29 行 REAL_EXIT + wall_ms |
| runs/anchor1-diff.txt、runs/anchor2-diff.txt | 锚局独立逐字节 diff 留痕（WP-C / WP-A） |
| runs/wp-c-batch.log | WP-C 运营日志（控制台捕获；**非 D4 契约档位**，如实标注） |

runs/ 共 92 档 = 29 局 × 3 + plan.txt + exits.txt + 2 锚 diff + 1 运营日志。

## 6. 执行事件（转记 events.md 与 exits.txt，如实）

1. 2026-10-05 09:45 T008 启动：主会话规划定稿 D1~D9，四份派工单落盘 dispatches/，三层架构执行（worker 执行 + 主会话监管 + plan-code-reviewer 收尾审核）。
2. Phase 1（并行）：WP-A（worker-1）与 WP-B（worker-2#1）双双完成全绿——WP-A：双盲构成核对零分歧、门禁三断言过、15 局（red200 矩阵 12 + 跨进程加样 2 + anchor2 锚局 1）REAL_EXIT 全 0、anchor2 与 T006 归档逐字节 IDENTICAL、red200 三种子跨 4 线程档零 mismatch；WP-B：compare_matrix.py（退出码 0/1/2/3/5）+ 夹具自测 3 必选路径 + 4 附加分支 + 对 WP-A 真数据只读预跑契约对齐 + environment.txt。
3. 主会话上报裁决 D10（任务卡留痕）：①T006 batch.sh:4 机器绝对路径顺手清理（内容零改动）；②夹具 PASS 路径退出码语义维持现状（不引入 --partial）；③退出码归类口径采纳（mismatch=5 仅限真实哈希/字节不一致；缺局/未完成=3）。
4. 10:12:22 WP-C（worker-2#2）开工：全规模 14 局长跑（anchor1 → th12 → th6 → th3 → th1 → r2；挂钟 8h32m46s；开工时空闲声明成立）。
5. 11:16 执行事件（WP-C 简报，进程树经 WMI 核实存活）：harness 后台包装壳（父 PID）被系统通知 stopped/killed，**实际跑批进程树未受影响续跑**（run_t008.sh → timeout → sim.exe 存活）。影响留痕：包装壳死后脚本收尾行不再写；判完成以 exits.txt 满 29 行 + BATCH_DONE 行为准；无数据损失、无重跑。
6. 11:18 WP-C 进度快照：anchor1 REAL_EXIT=0 且逐字节 IDENTICAL（留痕 runs/anchor1-diff.txt）；full10000-th12 s42/s43/s44 三局 REAL_EXIT=0（793~816s/局）；th6-s42 在跑（tick≥7200），抽检 sample 0/3600/7200 与 th12-s42 逐位一致。
7. 18:45:08 WP-C 收官：**14/14 局 REAL_EXIT=0、BATCH_DONE rc=0、零重跑、无看门狗杀**。exits.txt 满 29 行全 0（WP-A 15 + WP-C 14）。全规模 12 局终局三种子跨 4 线程档逐位一致（见 §2 表）；加样 r2 与基准局逐列一致；种子互异 sanity 成立；锚 1 diff IDENTICAL。
8. 耗时观察（如实披露）：本卡各档耗时快于 T006 归档同配置（anchor1 775.9s vs T006 1017.0s；总体 8.6h vs 预估 10~11h）——纯挂钟差（T008 非计时卡），stdout 逐字节一致性证明零漂移。
9. WP-D（本档）：2026-10-05 D1 执行 compare_matrix.py → 退出码 0，matrix.md 由该次运行生成；本 README 为证据档索引。
