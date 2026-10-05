# T008 · M0-07 同种子重放确定性验证（验收④）

- 周位：W2（T006 后即可，可与 T007 并行）
- 前置：T006
- 目标：验收④闭环——同种子重放哈希一致，1/3/6/12 线程 × 3 组种子（基准机 A 单机档；跨机档 M1 起入 CI）。

## 范围内

- 3 组固定种子 × 4 线程档 × **跨进程**（每次独立进程启动）：终局哈希 + ≥3 个关键 tick 中间哈希一致。
- 两套规模各跑一轮：降规模（每方 100 单位 ≤1,800 ticks）与全规模（万人级，tick 上限按报告）。
- 哈希输出口径与 T002 一致（含 rng_state 与 tick 计数）。
- 对拍矩阵表：组合 × 哈希值 × 判定。

## 范围外

跨机对拍（M1+）；CI 化（挂 M1）；lockstep 联机（M4）。

## 验收断言

1. 3 种子 × 4 线程 × 2 规模 = 24 次独立进程运行，零 mismatch（每组合内哈希逐位一致）。
2. 中间 tick 哈希（≥3 个采样点）一致，非仅终局。
3. 任一 mismatch → 全量如实记录并上报（不得静默重跑遮蔽）。

## 证据要求

docs/evidence/t008/：对拍矩阵 + 每次运行的命令/哈希行 + REAL_EXIT。

## 执行裁决记录（主会话规划定稿，2026-10-05 W1 D1；三层架构 worker 执行 + plan-code-reviewer 审核）

- **D1 种子/线程档**：seeds {42,43,44} × threads {1,3,6,12}；矩阵 24 局 = red200 12 + full10000 12，每局独立进程。
- **D2 规模与构成映射**：降规模 red200 = 每方 100（共 200）`--ticks 1800`（world.rs:145）；全规模 full10000 = 每方 5000（共 10000）`--ticks 14400`（world.rs:149 `TICK_CAP_FULL = 8*60*30`——「tick 上限按报告」落此）。构成映射逐字口径 = bench.rs:284-301 `composition_for`（per_side=units/2、六兵种均分余数按表序前 r 个 +1）；主会话速算参考值（17/16 分与 834/833 分）仅作双盲核对，worker 脚本重算为准，不一致即上报（连续四卡派工单数值笔误教训的制度化防呆）。
- **D3 采样点**：red200 `0,450,900,1350`；full10000 `0,3600,7200,10800,12600`（10800/12600 为接敌后点——战斗段哈希覆盖，T006 归档 fullscale 终局 units=9959<10000 实证战斗进哈希）；终局哈希走五行摘要 `hash=` 行。哈希输出口径 = T002 起的 state_hash（tick→rng 4 状态字→单位逐字段，含 rng_state 与 tick 计数）。
- **D4 覆盖域披露**：red200 1800 ticks 内不接敌（T003 观察：队首相距 999m、合速≤0.4m/tick）= 移动段确定性覆盖；战斗段（combat/retain）由 full10000 覆盖。如实写入证据 README。
- **D5 加样（跨进程补充，超出 24 局如实标注）**：`red200-th12-s42-r2/-r3`（同配置独立进程 ×2）+ `full10000-th12-s42-r2`（×1）。
- **D6 跨卡黄金锚 ×2（逐字节 diff）**：anchor1 = 逐字复现 T006 fullscale 命令（COMP5000 串逐字取自 docs/evidence/t006/batch.sh，无 samples）→ diff `docs/evidence/t006/runs/fullscale.stdout`（含 `hash=0x29980473140ed39e` 锚行）；anchor2 = 逐字复现 T006 `10k-s42-t12` 命令（samples 0,100,200,300）→ diff `10k-s42-t12.stdout`。锚局用 T006 COMP5000 串、矩阵局用 D2 映射串，两串用途差异如实披露。分段采样哈希中性有代码级论证（state_hash 为纯 &self，world.rs:552；run_with = 逐 tick 循环，world.rs:749）+ anchor2 实证。
- **D7 跑批纪律**：run_t008.sh 断点续跑（exits.txt 有 REAL_EXIT=0 行即 skip；非 0 行不覆盖不重跑；半局残档覆盖重跑）；watchdog 防死锁兜底（red 1800s / full th12 3600 / th6 7200 / th3 14400 / th1 28800，≈4× 外推余量，非验收判据）；量测窗口机器空闲独占；执行序快档先行（anchor1 → th12 → th6 → th3 → th1 → r2 → red → anchor2）尽早暴露 mismatch。
- **D8 零代码变更红线**：T008 为验证卡，`sim/` 零改动；CLI 若有缺口 worker 上报主会话裁决（预期无缺口：`--hash-samples` T006 已具备）。
- **D9 断言 3 制度化**：mismatch/失败局全量如实入档上报，禁止静默重跑遮蔽；分析器 exit 5 路径列出全部不一致值。
- **执行序**：WP-A（worker-1：跑批基建+降规模 15 局+锚2）∥ WP-B（worker-2#1：compare_matrix.py+环境档）→ WP-C（worker-2#2：全规模 14 局长跑 ≈10~11h，断点续跑，中断主会话接管）→ WP-D（全量对拍 matrix.md+README）→ 主会话收尾独立复现 + plan-code-reviewer 审核轮 → 台账 + 提交。派工单全文留痕 docs/evidence/t008/dispatches/。
- **D10 上报裁决（主会话，2026-10-05 执行中）**：①WP-A 上报 T006 既有证据 `batch.sh:4` 机器绝对路径（已入库历史档）——裁决顺手清理（T003/T007 P1-2 先例：路径前缀改相对 `$(dirname "$0")/../../..`、comp 串等内容零改动、文件内留痕注释；不重跑 T006 任何数据）；②WP-B 上报夹具 PASS 路径退出码语义——裁决选 A 维持现状（齐全性固定 29 局、缺局必 exit 3 不可配置，夹具以组合判定行验收 PASS 路径；不引入 `--partial` 稀释正式口径）；③WP-B 退出码归类口径（mismatch=5 仅限有效局真实哈希/字节不一致；缺标准列/REAL_EXIT≠0/锚局未成功→INCOMPLETE=3 且 §5 全量披露）——采纳，与断言 3 对齐。
- **执行结果（2026-10-05 收官）**：29 局（24 矩阵 + 3 跨进程加样 + 2 跨卡锚）REAL_EXIT 全 0、零重跑。compare_matrix.py 判定退出码 0 全 PASS——断言 1 零 mismatch（6/6 组合逐位一致）、断言 2 中间哈希一致（red 12/12 + full 15/15 判定点）、断言 3 未触发、加样 3/3、双锚逐字节 IDENTICAL、种子互异 8/8；战斗段覆盖实证 = full 终局 units 9959/9969/9970 < 10000。主会话独立复现（recheck-main-session.txt）：判定复算可重生成、双锚独立 diff、新进程第 3 次重跑逐字节一致、路径扫描 0、sim/ 零改动。证据档 docs/evidence/t008/（README + matrix.md + 29 局 runs + 双盲 comp + 分析器 + 环境档 + 事件流水 + 复现记录 + 派工单 4 份）。验收④闭环成立，数据放行 T013；按 AGENTS.md 审核粒度（量测卡完整轮）过 plan-code-reviewer 审核。
