# T018 证据总索引（M5-01 BRP 宿主与 game.* 初始集）

- 任务卡：`taskset/t018-brp-host.md`（范围 + 验收断言 1~6 + 设计裁决 D1~D11 执行记录）。
- 口径唯一来源：备忘录定本 v1.0（工作流仓 `docs/m5-game-selection.md`）§五 + 本仓确定性纪律。

## 文件清单

| 文件 | 内容 |
|---|---|
| `dispatch-sheet.md` | 派工单（worker-1 实现段）：API 事实 9 条（registry 源码行号级）+ 实现清单 + 门禁 + 红线 |
| `brp_smoke.sh` | BRP 直调冒烟脚本（curl；46 项判定 CHK-00~CHK-16，含 6 方法全调用 + 错误路径五发 + 锚对拍 + 跨线程 + 灭绝冻结两路径） |
| `brp-smoke-run.log` | 冒烟实录（2026-10-06 21:48 整改版，**46/46 PASS，SCRIPT_EXIT=0**；请求/响应 JSON-RPC 原文全量在档——首轮 33/33 版 REQ/RESP 未落档系脚本 `$( )` 捕获缺陷，审核轮 P0-1 指出后 `>&2` 整改重跑） |
| `host-stderr.log` / `host-stdout.log` | 宿主运行时留痕（banner：headless 形态 / 回环声明 / 6 方法清单） |
| `api-notes.md` | 0.19.1 API 查证记录（6 条，registry 源码行号；沿 T010 先例） |
| `env.md` | 环境档（同日同机引 T010 基线 + 本卡增量：依赖图证据 / 产物指纹 / 构建纪律） |

## 验收断言对账（任务卡 1~6）

1. **check 0 警告 + sim 零 bevy + 宿主只调公共 API**：worker 轮 + Lead 复跑均 0 警告；依赖图证据见
   `env.md`（grep sim 零 bevy API 使用；host 引用全限定 sim 公共模块）。
2. **6 方法 BRP 直调可达 + 签名语义符合备忘录 §五**：`brp-smoke-run.log` CHK-02（discover ×6）/
   CHK-04~07（deploy/run_to_tick/state_hash/outcome）/ CHK-11（run_tests）/ CHK-12（screenshot 桩 =
   结构化错误 4101 不击穿，CHK-14 进程存活）；**请求/响应原文逐条在档**（整改版起——见文件表注）。
   灭绝冻结分支覆盖（审核轮 P2-3 整改补面）：CHK-15 系列（空侧 deploy → tick0 冻结 blue 胜 → 4002）+
   CHK-16 系列（短 lane 5v5 真交战 → 灭绝@tick878 red 胜 alive_red=1 → 幂等 outcome 同锚 → 4002）。
3. **重放逐位一致 + 黄金锚对拍 + 跨线程 ≥2 档**：CHK-09（BRP 重放同锚）/ CHK-05（锚②
   `0x958c5938c8682529` 经 BRP 路径命中——与 M0 归档 `t004/s42_run1.txt` 等三档逐字核对）/ CHK-10
   （threads=12 同锚——1/12 两档）；锚① `0xd3b6408fd46c2008`（T002 归档双跑核对）经 `game.run_tests`
   进程内断言（CHK-11b 4/4）。注（D5 语义事实）：空构成经 BRP 在 tick 0 冻结（Draw），锚①的 1800-tick
   裸推进仅在进程内 World::new 路径存在——「至少一枚对拍命中」由锚②承担。
4. **run_tests 冒烟集 pass + 判定面骨架**：CHK-11 系列（total 4 / passed 4 / failed 0 + 套件名 +
   未知套件 -32602）。
5. **回环硬约束双留痕**：代码侧 `rpc.rs` 显式 `Ipv4Addr::LOCALHOST`（api-notes §4）+ 运行时 banner
   （`host-stderr.log`，CHK-01/01b）。
6. **构建纪律**：-j 2 + 内存预检 24.8G（`env.md`）；bevy facade 重编 14m10s 属 D10 预期内。

## 语义留痕（实现层裁决）

- run_to_tick 逐 tick 灭绝检查 + `run_battle_with(当前 tick)` 冻结（end_tick 真值——越过灭绝推会失真，
  D5）；已冻结再推 → 4002。
- 空构成（red=blue=[]）deploy 后立即 Draw@tick0（tick 0 双空语义，sim T005 既有边界）。
- outcome 未收束时驱动至 deploy 配置 max_ticks（默认 1800 = TICK_CAP_REDUCED）后按 hp 判定（幂等冻结）。
