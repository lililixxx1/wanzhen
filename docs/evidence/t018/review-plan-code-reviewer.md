# T018 plan-code-reviewer 轻量审核报告（2026-10-06，节录存档 + 复验结果）

> 审核 agent 回传的忠实节录（裁决/分级条目/置信度/覆盖面/复跑结果逐项一致；「优点」「遗漏/疑问」两节从略——其中含「备忘录 §五原文未逐字打开（跨仓），签名语义对照以任务卡 D5 + 派工单为准」的不确定标注）。整改闭环记录见 `taskset/t018-brp-host.md`「审核与收口记录」节。

> **复验轮（2026-10-06 同日，整改后）：最终裁决 = 通过（✅ 可合并）**——P0-1/P1-1/P2-1/P2-3/S-2/S-3 六项整改逐字属实、无新缺陷、check 独立复跑 0 警告；确定性脚注 = deploy_hash `0xe2706f0b91a2be8e` 三轮运行逐字一致。遗留非阻断备注 4 条（env 指纹刷新 / grep 模式表述 / S-1 log 自截断 / 本档改题节录）已随收口处理前三条 + 本条即第四条。

### 合并裁决
**有条件通过（⚠️ 修后可合并）** — 实现本体可靠（0 警告、33/33 复跑全绿、黄金锚逐字无误、sim 语义核实吻合），但证据档有一处不实宣称 + 一处事实性错误注释须整改后收口（P0×1、P1×1）。

### 审核范围
- **模式**：代码卡轻量轮（diff + 门禁三断言核对 + 抽查复跑）。加载规则：review-protocol 七条协议 + `rules/default.md`（Rust 无专属规则文件，按索引表走 default）。
- **diff 面**：`git status --porcelain`（`M Cargo.lock` / `M Cargo.toml` / `M taskset/t018-brp-host.md` / `?? docs/evidence/t018/` / `?? host/`）+ 全量 `git diff`。
- **文件覆盖清单**：15/15 全部已审（host/ 四文件 + 根 manifests + 任务卡 + 证据六档 + host 两日志），覆盖率 100%。

### Critical（阻塞合并）
**P0-1** | 置信度 96 | **「请求/响应原文全量在档」宣称不实，验收断言 2 证据要求未达成** | `README.md:12`、`api-notes.md:61`；机制在 `brp_smoke.sh:23-31` 及其 14 处调用点。`brp()` 的 `--- REQ/--- RESP` echo 全部落在 `$( )` 命令替换内被捕获进变量后丢弃，从未进 log（归档 log `grep -- "--- REQ\|--- RESP"` = 0 命中；复跑新档同样 0 命中，现场再证）。修复：echo 改 `>&2` 或直写旁档，重跑补档；同步修正 README/api-notes 措辞。

### Important（强烈建议修复）
**P1-1** | 置信度 92 | **「无 sim from_id 公共 API」宣称不实（实有 `sim::units::kind_from_id`，sim/src/units.rs:171-182 pub），导致本地重复表** | `taskset/t018-brp-host.md:51`（D5）、`host/src/rpc.rs:105-106` 注释、`dispatch-sheet.md:20-28`（API 事实 8 漏列）。同一 id 表现存 4 处。修复：改调 sim 公共实现或如实改写理由；派工单补列。

### Minor
- **P2-1** | run_tests 对非字符串 `suite` 静默回落默认套件（`rpc.rs:349-353`）——建议 Some 且非字符串即 INVALID_PARAMS。
- **P2-2** | run_to_tick 不受 max_ticks 约束，超限后 outcome 上限判定发生在超限 tick（调用序可影响 winner）——D5 规格缺口，实现忠实于 D5；建议 T020 钉死语义。
- **P2-3** | 冒烟零覆盖 run_to_tick 灭绝冻结分支（宿主最复杂逻辑；CHK-05/07 实走无交战路径，CHK-08 的 4002 走上限冻结路径）——建议补空侧 deploy 与短 lane 灭绝两例。

### S（建议性备注）
- S-1：`brp_smoke.sh:15` `: > "$LOG"` 每次运行自截断归档证据（本审复跑已覆盖 21:16 版，原文备份 `/tmp/t018-review-backup/`，log sha256 `caef4e6eb68e5cfa…`）。
- S-2：`api-notes.md:37` 转写笔误 `impl Into<IpAddr)`（缺 `>`）。
- S-3：`rpc.rs:307/322` 读取路径用 `get_resource_mut`（可用 `get_resource`）；nit。

### 验收断言 1~6 核对面
| # | 判定 | 摘要 |
|---|---|---|
| 1 | 达成 | check 0 警告复跑 + sim 零 bevy grep + 公共 API 逐一实核 |
| 2 | 部分达成 | 6 方法可达 ✓、screenshot 桩 ✓；「原文留档」未达成（P0-1） |
| 3 | 达成 | 重放/锚② BRP 命中/跨线程 1+12 两档；锚值与 t002/t004 归档逐字一致 |
| 4 | 达成 | run_tests 判定面骨架（4/4 + 未知套件 -32602） |
| 5 | 达成 | 代码 + banner 双留痕；回环唯一监听面论证（render 端口 15703 在 MinimalPlugins 下不可达） |
| 6 | 达成 | -j 2 + 内存预检留档 + 重编时长记录 |

### 抽查复跑（三项独立复跑）
① `cargo check --workspace -j 2` PASS（0 警告）；② `bash docs/evidence/t018/brp_smoke.sh` PASS（33/33，无残留进程；新 log REQ/RESP 仍 0 命中 = P0-1 复证）；③ `git diff --stat -- sim/` 空（sim 零改动）。

---

**最终裁决：有条件通过（⚠️ 修后可合并）** — P0-1 与 P1-1 整改闭环后收口；P2×3 与 S×3 可随 T020/下卡处置。
