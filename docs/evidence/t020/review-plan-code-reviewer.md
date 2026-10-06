# T020 完整轮审核报告（plan-code-reviewer，2026-10-07）

> 派发：Lead（重派——首发审核 agent 因会话续接丢失句柄无产出，本档为重派轮原文归档）。
> 审核对象：T020 收获提交 c368204。审核器原文未写入本文件（其自述「仅在回复中提供报告正文」），
> 本档 = 审核器最终消息逐字归档（Lead 转录，零改动）。

## §0 裁决

**有条件通过。** 锚值与归档数字均吻合，限定复跑通过，未发现 P0；但断言 8 和空构成错误路径覆盖存在 P1 缺口，整改并复核后关账。

## §1 审核范围与方法

- 范围：任务卡、派工单、`host/src/suite.rs`、`host/src/rpc.rs`、`docs/evidence/t020/` 全部 20 个文件及收获提交 `c368204`；读取仓库规则 `AGENTS.md` 与通用审核规则 `default.md`。
- 检查：审阅提交差异和完整相关代码；复算归档原始日志的判定数；核对锚值来源；按要求仅用预构建 `host.exe` 在仓库外临时目录复跑 `m5core_suite.sh` 与 `error_paths.sh`。复跑前确认端口 15714 无监听，复跑后确认无残留监听及 `host.exe` 进程。
- 未运行 `cargo build/test`；归档中的 `cargo check` 结果仅核验文档记录，未在本轮独立重跑。
- 零越界核验：`rpc.rs` 相对 `fb45b1e` 无差异；提交未改 `sim/`；没有挑战预设实现、统计面、MCP 或模拟行为变更。提交额外新增 `docs/evidence/t019/dispatch-sheet.md`，见 P2。
- 覆盖率：指定审核清单共 24 个文件（任务卡 1、派工单 1、代码 2、T020 证据档 20）—已审 24 / 跳过 0 / 覆盖率 100%。另有未跟踪目录 `docs/evidence/t022/` 不属于本次提交或指定范围，未审。

## §2 逐断言核对表

| 验收断言 | 证据与复核 | 结论 |
|---|---|---|
| 1. `game.run_tests` 全量通过、逐条可复跑 | `README.md` §对账；`assertions.md` §1 列出 9 条断言、期望值、实测值与脚本入口。复跑 19/19，suite 为 total 9 / passed 9 / failed 0。 | 满足；断言 8 的判定条件有缺口，见 P1-1。 |
| 2. 重放矩阵零 mismatch | `assertions.md` §2、`replay-matrix-run.log` 与 JSONL 显示 A–G；含 seed 42/7/2026、线程档 1/3/12、短 lane 战斗及混编战斗段。原始日志为 25/25。 | 满足。 |
| 3. M0 黄金锚命中 | `assertions.md` §4 列源指针；锚值与 T004、T018、T021 对应档案逐值一致。F 锚是本卡新锚，measure 档记录其占位、实测及回填。 | 满足；无锚值错误。 |
| 4. 错误路径结构化错误、进程存活 | `assertions.md` §3、`error-paths-run.log` 记录 14 发、22 项检查、结构化错误码及进程存活。空构成边界未被独立覆盖，见 P1-2。 | 现有用例通过，覆盖不完整。 |
| 5. check 0 警告、sim 零改动 | `README.md` 门禁记录称 check EXIT=0、0 警告；提交差异确认 sim 无改动。`rpc.rs` 也无差异。 | 归档记录支持；本轮未运行 cargo。 |

## §3 双盲复算表

| 原始日志 | 独立计数 | 声称值 | 对账 |
|---|---|---|---|
| `replay-matrix-run.log` | 25 项 CHK、PASS=25、FAIL=0、SCRIPT_EXIT=0 | 25/25 | 一致；A–G 的哈希、tick、终局数据与 JSONL / `assertions.md` 相符。 |
| `error-paths-run.log` | 22 项 CHK、PASS=22、FAIL=0、SCRIPT_EXIT=0 | 22/22 | 一致；包含 14 次请求和进程存活检查。 |
| `m5core-suite-run.log` | 19 项 CHK、PASS=19、FAIL=0、SCRIPT_EXIT=0 | 19/19；total9/pass9/fail0 | 一致；9 条结果均为 `pass:true`。 |

锚值复核：锚② `0x958c5938c8682529` 与 seed43 `0x54611ed6ded02540` 命中 T004；短 lane `0xfdbc4995554ee691` 命中 T018 CHK-16；H_C `0xb82a248ff23515e2` 命中 T021；F 锚 `0x185fe8c22adeec36` 在 T020 measure 与正式档案中一致。附录 C 中 T004 锚与 seed43 伴随值亦核对一致。未发现宣称与原始输出不符。

## §4 复跑记录

复跑脚本为仓库外临时副本，仅将预构建二进制路径指向既有 `target/release/host.exe`；输出写在临时目录，未覆盖仓库证据文件。

- `bash docs/evidence/t020/m5core_suite.sh`：退出码 **0**；`SUMMARY: PASS=19 FAIL=0`、`SCRIPT_EXIT=0`；返回 `total=9, passed=9, failed=0`，9 条断言全通过。
- `bash docs/evidence/t020/error_paths.sh`：退出码 **0**；`SUMMARY: PASS=22 FAIL=0`、`SCRIPT_EXIT=0`。
- 两次运行前端口 15714 均无监听；各次运行后端口均无监听且没有残留 `host.exe` 进程。

## §5 发现清单

### P0

无。日志判定数与 README / 清单一致；核验的锚值均未抄错。

### P1

1. **置信度 95 | 断言 8 未把直接 `run(100)` 后的 tick 变化纳入 pass 条件**
   `host/src/suite.rs:284-288`
   `pass` 检查了三次 `run_battle_with` 的 outcome 一致、收束重入时 tick 不变，以及 `outcome()` 缓存未变化；但没有验证 `tick_after_run == tick_frozen + 100`。因此，即使直接 `run(100)` 未推进 tick，断言仍可能通过，而 `detail` 仍宣称已覆盖该语义。
   **建议：** 将直接 run 后 tick 的预期值纳入 `pass` 条件，并让失败 detail 明确报告预期与实测 tick。

2. **置信度 91 | 错误路径缺少任务卡要求的独立空构成案例**
   `taskset/t020-assertion-suite.md:13`；`docs/evidence/t020/error_paths.sh:57-76`
   任务卡把"空构成"列为非法参数验证对象；脚本中的 `blue:[]` 只出现在同时带有未知兵种的请求中，错误会先由 `red` 的未知兵种触发，并未单独验证空构成路径。T018 证据还显示单侧空阵营是允许的，因此应明确区分"单侧为空"与"双方均空/必需字段缺失"的预期语义。
   **建议：** 澄清目标边界后增加独立请求，断言对应结构化错误码、进程存活和未部署状态；若该构成实际应被接受，则同步更正任务卡措辞与验收口径。

### P2

- 收获提交另新增 `docs/evidence/t019/dispatch-sheet.md`，不属于 T020 的交付文件清单，也未在 T020 README 中解释。它未造成代码越界，但建议在提交说明或后续任务记录中说明该文件的归属，避免混淆任务范围。

## §6 整改清单与关账意见

关账前处理 P1-1、P1-2，并复跑受影响的 m5-core / 错误路径脚本、更新对应证据记录。对 D1-8 上报项，基于 `sim/src/world.rs:890-897` 的 `run` 原语实现，本审核接受 README 所述选项 A 的语义拆分：收束重入冻结 tick，而直接 `run(100)` 推进模拟 tick、保留冻结 outcome 缓存。其具体 tick 推进值仍应由 P1-1 的断言实际校验。

---

**最终裁决：有条件通过。** P0=0；P1×2：断言 8 未验证直接 run 的 tick 推进、空构成错误路径缺少独立覆盖；P2×1：提交包含未说明的 T019 派工单文件。

本次仅在回复中提供报告正文，未写入用户指定的报告文件。相关文件绝对路径：

- `C:\Users\Administrator\Desktop\ccc\wanzhen\host\src\suite.rs`
- `C:\Users\Administrator\Desktop\ccc\wanzhen\docs\evidence\t020\error_paths.sh`
- `C:\Users\Administrator\Desktop\ccc\wanzhen\taskset\t020-assertion-suite.md`
- 指定报告路径：`C:\Users\Administrator\Desktop\ccc\wanzhen\docs\evidence\t020\review-plan-code-reviewer.md`（未创建）

---

## Lead 处置记录（转录后回填）

- 报告归档：本档（审核器最终消息逐字转录，§0~末尾为原文）。
- P2-1 归属说明：`docs/evidence/t019/dispatch-sheet.md` 系 Lead 于 T020 收获当日（2026-10-06 深夜）为 T019 派发预备所写，随 T020 收获提交 c368204 一并入库——**属 T019 Lead 规划段产物，非 T020 交付物**；T020 证据 README 不引用它，归属在此留痕（整改段回填台账时同步注明）。
- 整改与复验（2026-10-07 同日闭环，Lead 执行）：**P1-1** = suite.rs 断言 8 增 `tick_after_run == tick_frozen + 100` pass 条件 + detail 预期值，复跑 m5core 19/19 原绿（878→978 生效）；**P1-2** = 实测裁决空数组合法布阵（单侧空=灭绝冻结@0 blue、双侧空=draw@0、锚值双留），error_paths.sh 增 CHK-17/18（+8 检查），任务卡口径修正披露（原措辞保留 + 裁决注），复跑 30/30（首跑 2 发为脚本 grep 字段序假设缺陷，拆双 grep 修正——脚本缺陷留痕）；门禁 = check 0 警告 + release 5.26s + 两脚本全绿（明细见 README「完整轮审核与整改」节）。D1-8 上报项经 §6 接受选项 A，tick 推进值已入断言。**整改闭环，T020 转通过。**
