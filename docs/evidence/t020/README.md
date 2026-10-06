# T020 证据档（断言面与测试集全量，席位 3）

- 派工单：`docs/evidence/t020/dispatch-sheet.md`（Lead，2026-10-06；本档只引用不复制正文）。
- 依据：taskset/t020-assertion-suite.md + 备忘录 §三质量红线/§五判定口径 + T018 P2-2 移交项。
- 实现（隔离树内，基线 = 主仓 f9f30d9）：`host/src/suite.rs` 单文件增 `m5-core` 全量套件
  （9 断言；前 4 复用冒烟集原函数）；其余全部为 `docs/evidence/t020/` 新档。sim/ 零改动
  （本轮全部写操作路径 = `host/src/suite.rs` + 本目录，无 sim/ 写入——mtime 留痕：
  sim/src/* 与 host/src/{main,rpc,presets}.rs 均停在导出时点，仅 suite.rs 为本轮修改）。

## 文件清单

| 文件 | 内容 |
|---|---|
| `replay_matrix.sh` | D2 重放矩阵脚本（七配置 A~G；每配置序列整段重放两遍逐位对拍；含跨线程对与 P2-2 BRP 侧钉死；F lane 算式头注） |
| `replay-matrix-run.log` | 矩阵实录（**25/25 PASS，SCRIPT_EXIT=0**；REQ/RESP 原文全量在档） |
| `replay-matrix.jsonl` | 逐配置观测值（hash / tick / outcome / 锚命中 / 接敌旁证布尔） |
| `error_paths.sh` | D3 错误路径脚本（**整改版 18 发**：非法参数/越界 12（-32602，含未知 preset / suite）+ 未布阵 4001 + 桩 4101 + P1-2 空构成接受现实段 CHK-17/18；尾部进程存活 + 无静默落地复证） |
| `error-paths-run.log` | 错误路径实录（**整改版 30/30 PASS，SCRIPT_EXIT=0**，2026-10-07 复跑；10-06 原版 22/22 已被复跑覆盖——原版计数留在 assertions.md §3 变更注） |
| `m5core_suite.sh` | 门禁 6 / D1 判定面脚本（BRP `game.run_tests {"suite":"m5-core"}`：total/passed/failed + 逐断言名 + 锚 detail） |
| `m5core-suite-run.log` | 套件实录（**19/19 CHK PASS，SCRIPT_EXIT=0**；套件 total 9 / passed 9 / failed 0） |
| `m5core-suite-measure.log` | PIT-M-002 占位首测留痕（断言 6/7 占位 0，passed=7 / failed=2，实测值在 detail） |
| `replay-matrix-measure.log` / `replay-matrix-measure.jsonl` | PIT-M-002 占位首测留痕（C/F 新记录值先测后填） |
| `t018-smoke-rerun.log` | 门禁 3 现场实录副本（T018 冒烟 46/46——原档由脚本按设计重写于 t018 目录，见「运行副产物」） |
| `host-{replay,errors,suite}-stderr.log` / `-stdout.log` | 三脚本宿主运行时留痕（banner：headless / 回环声明 / 监听 127.0.0.1:15714 / 6 方法清单） |
| `assertions.md` | 断言清单档（套件 9 + 矩阵 7 + 错误路径 14 逐条：复跑命令 / 期望 / 实测 / 判定；锚值源指针） |
| `README.md` | 本档（对账 + 门禁 + 上报） |
| `dispatch-sheet.md` | 派工单（Lead） |

## 验收断言对账（任务卡 1~5）

1. **`game.run_tests` 全量套件 pass；断言清单逐条在档可复跑**：`m5core_suite.sh` →
   `total:9 / passed:9 / failed:0`（逐断言名 `"pass":true`，CHK-05 系列）；清单与复跑命令
   见 `assertions.md` §0/§1（每行对应 CHK 名）。
2. **BRP 参数序列重放矩阵零 mismatch（覆盖多 seed × ≥2 线程档 × 含战斗段规模）**：
   `replay_matrix.sh` 25/25（A/B、C/D 为 t1×t12 跨线程对；seed 覆盖 42/7/2026；
   E = 短 lane 真交战至灭绝@878；F = 混编 20 单位战斗段，战损 2/9 旁证接敌）。
3. **M0 黄金锚对拍命中（清单列锚值与源档指针）**：断言 1/2（锚①/②）、6（短 lane 锚）、
   7（seed43 伴随锚）；矩阵 A/B（锚② 经 BRP 路径）、E（短 lane 全四元组）；清单与源指针见
   `assertions.md` §4（t004/t018/t021 归档行号已 `grep -n` 复核）。
4. **错误路径测试全过：结构化错误返回、进程存活**：`error_paths.sh` 整改版 30/30
   （18 发：14 发非法/状态全部结构化 `error.code`（4001 / -32602×12 / 4101）+
   CHK-17/18 空构成接受现实段；CHK-15 复证「无一次非法 deploy 落地」；CHK-16 进程存活）。
   空构成口径修正披露见任务卡范围内节（P1-2 裁决：空数组合法、缺字段才 -32602）。
5. **check 0 警告；sim 零改动**：门禁 1 全绿 0 警告；sim/ 无写入（见「实现」节）。

## 门禁记录（2026-10-06，树根执行；`CARGO_TARGET_DIR` 共享主仓 target；门禁逐条单独整句）

1. `CARGO_TARGET_DIR=… cargo check --workspace -j 2` → `Finished dev profile … in 1.17s`，
   EXIT=0，0 警告。
2. `CARGO_TARGET_DIR=… cargo build -p host --release -j 2` → `Finished release profile …
   in 6.86s`，EXIT=0；产物 host.exe 拷贝至树根 `target/release/host.exe`（运行件，非入库）。
3. `bash docs/evidence/t018/brp_smoke.sh`（默认端口 15702，跑前 tasklist 无残留 host）
   → **PASS=46 FAIL=0，SCRIPT_EXIT=0**（缺省套件不变回归原绿；副本 `t018-smoke-rerun.log`）。
4. `bash docs/evidence/t020/replay_matrix.sh`（15714）→ **PASS=25 FAIL=0，SCRIPT_EXIT=0**。
5. `bash docs/evidence/t020/error_paths.sh`（15714）→ **PASS=22 FAIL=0，SCRIPT_EXIT=0**。
6. `bash docs/evidence/t020/m5core_suite.sh`（15714）→ **PASS=19 FAIL=0，SCRIPT_EXIT=0**；
   套件判定行 `{"failed":0,"passed":9,"total":9,"suite":"m5-core"}`。

## PIT-M-002 留痕（占位 0 → 实测 → 与派工单锚核对 → 回填 → 复跑全绿）

| 锚点 | 首测实测（measure 档） | 派工单/T004 锚 | 核对 | 回填后 |
|---|---|---|---|---|
| 断言 6 短 lane 战斗 | red/878/1/0 `0xfdbc4995554ee691` | 同值（T018 CHK-16） | 一致 | PASS |
| 断言 7 seed43 | `0x54611ed6ded02540` | 同值（T004） | 一致 | PASS |
| 矩阵 C（H_C） | `0xb82a248ff23515e2` | 新记录值（无先验） | 与 t021 同参实测同值旁证 | PASS |
| 矩阵 F（H_F） | `0x185fe8c22adeec36` | 新锚（无先验） | 回填为跨版本回归常量 | PASS |

不符即停手上报的 P0 条款未触发（全部一致）。

## 运行副产物说明

- 门禁 3 的 T018 脚本按设计重写其自写日志（`docs/evidence/t018/brp-smoke-run.log` /
  `host-{stdout,stderr}.log`）——**运行副产物，t018 档内文件零人工改动**（同 T021 先例）；
  现场实录已复副本 `t018-smoke-rerun.log` 于本档自含。t021/ 及本树其余路径本轮无写入。

## 上报节（第 1 项已由完整轮审核 §6 裁定——选项 A 接受，见「完整轮审核与整改」节；原文留档）

1. **D1-8 原文「再 `run(100)` tick 不动」的字面读法与 sim 语义矛盾（实现按分解口径落地）**：
   sim 中 `resolved` 只冻结 `run_battle_with`（收束缓存幂等、tick 不推进）；`run/step` 是
   纯原语、不受 `resolved` 约束（`sim/src/world.rs:32-34` 模块注；`:890-897` run/run_with
   实现无 resolved 检查）——字面断言「run(100) 后 world.tick 不动」实测不可过（tick 878→978）。
   落地口径（断言 8）：①「连调两次四元组全等」+「重入收束路径（含更小 max_ticks=100）
   tick 不推进」= 收束冻结；② 直接 `run(100)` 后冻结缓存四元组不变（end_tick 恒 878），
   world.tick 前移如实写进 detail。**选项**：A 维持本口径（现状，零成本）；B 去掉断言 8 的
   `run(100)` 段只留收束幂等（需改 suite.rs 并重跑门禁 1/2/6，代价小）；C 改 sim 让 run 也
   冻结（超越本卡范围 + 动模拟语义，不建议）。请裁决；未裁决前按 A 交付。

## 工程备注

- 三脚本首跑发现起宿缺 `--port`（宿主落 15702、脚本探 15714）——修正后重跑取证据；
  正式证据档全部来自修正版（失败首跑无判定价值，未留档）。
- F lane 选定算式与实测旁证见 `assertions.md` §2；错误路径发次/消息见 §3。

## 完整轮审核与整改（2026-10-07）

- **审核**：plan-code-reviewer 完整轮（重派轮——首发 agent 因会话续接丢失句柄无产出），
  报告 `review-plan-code-reviewer.md`。**裁决 = 有条件通过：P0=0 / P1×2 / P2×1**；
  双盲复算三档全对账、限定复跑（m5core 19/19 + error_paths，预构建 exe、无 cargo）
  全绿、锚值零抄错、rpc.rs 与 sim/ 零改动核验通过。
- **整改（Lead 执行，同日闭环）**：
  - **P1-1**（断言 8）：`tick_after_run == tick_frozen + 100` 入 pass 条件 + detail 报
    预期值（suite.rs `outcome_freeze_idempotence_shortlane`）。首跑复验即过
    （878→978 断言生效，证明原版实现语义正确、缺的只是判定条件）。
  - **P1-2**（空构成）：实测裁决 = 空数组**合法布阵**（单侧空=立即灭绝冻结@0、双侧空=
    draw@0，与 sim 空局语义一致——T002 锚①先例）；error_paths.sh 增独立段 CHK-17/18
    （+8 检查、锚值双留）；任务卡口径修正披露（范围内节裁决注，原措辞保留）。
    整改首跑 CHK-17c/18c 因 grep 假设 JSON 字段序（end_tick 先于 winner）误判 2 发——
    拆双 grep 修正后 30/30（脚本缺陷非产品缺陷，留痕）。
  - **P2-1**（归属说明）：收获提交 c368204 内 `docs/evidence/t019/dispatch-sheet.md`
    系 Lead 为 T019 派发预备所写（T019 Lead 规划段产物，非 T020 交付物）——在
    审核报告「Lead 处置记录」节留痕。
- **整改门禁复跑（2026-10-07，主仓根执行；预检无在飞 cargo——与 T019 派发串行）**：
  1. `cargo check --workspace -j 3` → EXIT=0，**0 警告**（0.92s）。
  2. `cargo build -p host --release -j 3` → EXIT=0（5.26s）。
  3. `bash docs/evidence/t020/m5core_suite.sh` → **PASS=19 FAIL=0，SCRIPT_EXIT=0**
     （断言 8 强化后原绿）。
  4. `bash docs/evidence/t020/error_paths.sh` → **PASS=30 FAIL=0，SCRIPT_EXIT=0**
     （整改版含 CHK-17/18）。
- **D1-8 上报项处置**：审核 §6 接受选项 A 分解口径（收束重入冻结 tick；直接 run(100)
  推进 tick、冻结缓存不变）——其 tick 推进值现由 P1-1 断言实际校验（978）。上报项闭环。
