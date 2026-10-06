# T023 证据档（M5-06 统计面接入，席位 8）

- 日期：2026-10-07（worker-2 执行段；隔离树 t023-a，基线 commit = bb3f4ba；产出全落树内）。
- 任务卡：`taskset/t023-stats-face.md`；派工单：`docs/evidence/t023/dispatch-sheet.md`
  （形态裁决 = 宿主内批跑 + 扩 `game.*` +1 方法 `game.sample_outcomes`）。
- 实现面：`host/src/rpc.rs`（新 handler `sample_outcomes_handler` + `SAMPLE_OUTCOMES_METHOD`
  + `MAX_SAMPLE_GAMES=1000` + `HostRpcPlugin` 注册行 + 方法面计数文档 6→7）、
  `host/src/main.rs`（banner `[host] methods:` 行 7 方法 + 相邻计数注释同步）。
  **`sim/` 与 `host/src/{suite.rs,presets.rs,challenges.rs}` 零改动**（红线核验见门禁节）。

## 索引

| 文件 | 内容 |
|---|---|
| `sample_smoke.sh` | D5 冒烟脚本（端口 15707 独占）：锚对拍 / 确定性逐字节 / 前缀性质 / 跨线程逐字段 / wrap 种子序列 / games=1000 上限批 + 体量实测 / HostedGame 非干扰 / error 路径 8 发 / banner+discover 7 方法 / 口径注在档 / 进程存活（CHK-00~CHK-12） |
| `sample-smoke-run.log` | 冒烟实录（REQ/RESP 原文全量 + 判定行 + SCRIPT_EXIT） |
| `matrix_example.sh` | 示例矩阵生成脚本（900 局；判定行由脚本计算生成） |
| `matrix-example.md` | 示例矩阵输出档（3 red 构型 × 3 blue 构型 × 100 局/格；口径注三件；M0 T009 同格 sanity 对照） |
| `matrix-example-run.log` | 矩阵批跑全量 REQ/RESP 原文（9 格 RESP 完整在档） |
| `api-notes.md` | 本卡 API 查证记录（sim 公共 API 行号级 + Bevy 面零新增） |
| `host-stdout.log` / `host-stderr.log` | 冒烟宿主运行留痕（banner 7 方法行 / 回环 15707 实际绑定） |
| `gate-check.log` / `gate-sim-test.log` / `gate-host-release.log` | 三件门禁实录（命令 + 原始输出 + 退出码；release 档含 90s 护栏声明与耗时） |
| `t018-smoke-rerun.log` / `t022-smoke-rerun.log` | 不回归冒烟副本（现场实录；原档为各脚本运行副产物，见文末说明） |
| `dispatch-sheet.md` | 派工单（Lead；本档只引用不复制正文） |

## 方法语义自检（派工单 §2 逐条）

| §2 条款 | 实现 | 证据 |
|---|---|---|
| red/blue 必填、复用 `parse_composition` 同解析同报错 | handler 直调 `parse_composition(&params, "red"/"blue")` | 冒烟 CHK-10c/10e（缺 red / 未知兵种消息与 deploy 同源） |
| lane 缺省 `DEFAULT_LANE_LEN_M`、max_ticks 缺省 `TICK_CAP_REDUCED`、threads 缺省 1（域同 deploy） | 缺省分支逐字沿 `deploy_handler`；域校验同式（lane ≥1 且 Q32.32 不溢出 / max_ticks 1..=14400 / threads 1..=1024） | 冒烟 CHK-03（缺省 lane/ticks 锚 1800）、CHK-10f/g/h（域错误消息） |
| `games` 域 1..=1000（`MAX_SAMPLE_GAMES`），超域消息列域 | `(1..=MAX_SAMPLE_GAMES).contains(g)` | 冒烟 CHK-10/10b（0 / 1001 → `invalid games (1..=1000 required)`）、CHK-08（1000 上限接受） |
| seed_i = `seed_base.wrapping_add(i)`（u64 wrap 显式留痕） | 循环内 `wrapping_add`（doc 注释留痕「模 2^64 回绕」） | 冒烟 CHK-07（seed_base=2^64−2 → …614/…615/0 三段直查 + 全序列链校验） |
| 每局全新 World、批间零状态残留、不触碰 HostedGame | 循环内 `World::deploy_versus`，handler 不读写 `HostedGame`（`_world` 未用） | 冒烟 CHK-04/05（重跑逐字节 + 前缀性质）、CHK-09（deploy 后 sample 批 → state_hash 零变化 → 锚②仍可达） |
| response 七字段 + outcomes 全量六字段；`win_rate_red_pp = red_wins × 10000 / games`（万分比整数，禁浮点） | 逐字段组装；纯 u64 整除 | 冒烟 integrity 模式全批校验（公式恒等）、CHK-08b（1000 局体量实测 <10MB） |
| `final_hash` `0x%016x` 与 outcome 面同格式 | `format!("0x{:016x}", …)` | 冒烟 integrity 全批正则 `0x[0-9a-f]{16}` + CHK-03g 锚值直命中 |
| 错误路径仅 INVALID_PARAMS（-32602），无 4xxx 域错误 | handler 无 HostedGame 路径 | 冒烟 CHK-10 系列 8 发全 -32602 |

## 口径注三件（D4 逐字入档；残余账 #9/#11/#12 随卡）

① 降规模口径：每方 100 共 200 单位/局、单局 ≤1800 ticks（表 6-0）——不得与全规模数据混用（残余账 #11）

② 样本量注：本卡示例档每格 100 局、冒烟批 ≤16 局（<400 场）——趋势指示、非基准（判据 ±10pp / ≥400 场/周，R5 功效注——残余账 #12）

③ 灰盒指标不作外推依据（残余账 #9）——本卡不适用：统计面无渲染指标，如实标注

## 判定行（脚本生成，人工只解读）

- **门禁三连（树根，2026-10-07）**：`cargo check --workspace -j 3` EXIT=0 且 0 警告
  （首验 1.38s 三 crate 全 Checking、无警告输出；复录 0.57s 缓存新鲜——`gate-check.log`）；
  `cargo test -p sim -j 3` EXIT=0——sim lib **44 passed / 0 failed** + arena **3 passed**
  （`gate-sim-test.log`）；`cargo build -p host --release -j 3` EXIT=0（**7.22s**，仅编 sim+host
  本地 crate——`gate-host-release.log`）。
- **不回归**：`bash docs/evidence/t018/brp_smoke.sh` → **PASS=46 FAIL=0**（副本 `t018-smoke-rerun.log`）；
  `bash docs/evidence/t022/challenge_smoke.sh` → **PASS=37 FAIL=0**（副本 `t022-smoke-rerun.log`）。
- **`sample_smoke.sh` → PASS=52 FAIL=0，SCRIPT_EXIT=0**（`sample-smoke-run.log` REQ/RESP 原文全量在档）。
  关键判定行（脚本输出原样）：
  - 锚对拍：CHK-03g `outcome[0].final_hash=0x958c5938c8682529`（T004 黄金锚经新方法命中）
    + draw@1800 + alive 30:30（默认构成 / 默认 lane 1000m / 默认 ticks 1800、seed_base=42、games=1）；
  - 确定性：`EQ: outcomes arrays byte-identical + summary equal (games=8)`；
    `PREFIX: 2-game outcomes == first 2 of 4-game batch`；
  - 跨线程：`CROSS: 16 games x 6 fields identical (threads 1 vs 12); summary equal; r/b/d=13/0/3`；
  - 种子序列：`INTEGRITY: games=3 … seeds=18446744073709551614..0`（wrap 至 0 直查）
    与 `games=1000 … seeds=42..1041` 全链校验；
  - 上限批体量：`SIZE: resp_bytes=105125 outcomes_bytes=104985`（1000 局 × 6 字段全量、<10MB）；
  - 非干扰：deploy 后 sample 批 → `game.state_hash` 零变化（CHK-09d/e）→ `run_to_tick 1800`
    仍命中锚②（CHK-09f）——不触碰 HostedGame 的方法面证据；
  - error 8 发全 `-32602`（games=0 / games=1001 / 缺 red / 缺 seed_base / 未知兵种 /
    max_ticks=0 / threads=1025 / lane_len_m=0，消息文本逐发核对）；
  - banner 7 方法行逐字 + `rpc.discover` 7 方法 + 回环 15707（CHK-01/02）；
    口径注三件在档 6 查（CHK-11，`matrix-example.md` 与 `README.md` 各 3 条）。
- **示例矩阵 900 局**（`matrix-example.md`，判定行脚本生成）：9 格 × games=100（seed_base=42、
  lane 50m、ticks 缺省、threads=1），wall_ms=**26651**；9 格全部为构造性结局（每格 100 局同 winner，
  跨 seed 稳健——单兵种 100v100 档位）；与 M0 T009 归档同口径 9 格对照 **max |Δpp| = 0**；
  汇总 900 局 r/b/d = 600/300/0（混合 9 配置——非单一命题样本）。

## 锚来源（文档引用不复制正文）

- M0 T004 黄金 `0x958c5938c8682529`：`sim/src/world.rs` 单测
  `golden_deploy_default_comp_seed42_1800ticks`（常量行 `:1279`；单测入口 `:1272`）；
  归档 `docs/evidence/t004/`。冒烟 CHK-03 经 `game.sample_outcomes`
  （默认构成 + 默认 lane/ticks + seed_base=42、games=1）复现该锚——新方法确定性
  链路与 M0 归档同源。
- T009 sanity 对照数据（只读引用）：`docs/evidence/t009/runs/matrix_per100/matrix_per100.jsonl`
  （3600 局归档；本卡示例矩阵同口径取其中 9 格对照）。

## 门禁记录（2026-10-07，树根 t023-a；逐条单独整句；`CARGO_TARGET_DIR` 指共享主仓 target）

| # | 命令 | 结果 |
|---|---|---|
| 0 | 进程门 / 内存门（每次 cargo 前两道门） | check/test 段实时复验：进程计数 = 0、CommitFree 11.6~11.7G（≥11G ✓）；release 段见下节 |
| 1 | `cargo check --workspace -j 3` | EXIT=0，**0 警告**（`gate-check.log`） |
| 2 | `cargo test -p sim -j 3` | EXIT=0：lib 44 passed / 0 failed + arena 3 passed + bench/main/doc 0（`gate-sim-test.log`） |
| 3 | `cargo build -p host --release -j 3` | EXIT=0，**7.22s**（`gate-host-release.log`）；产物 host.exe 拷贝至树内 `target/release/`（运行件，非入库） |
| 4 | `bash docs/evidence/t018/brp_smoke.sh` | PASS=46 FAIL=0（端口 15702；副本 `t018-smoke-rerun.log`） |
| 5 | `bash docs/evidence/t022/challenge_smoke.sh` | PASS=37 FAIL=0（端口 15706；副本 `t022-smoke-rerun.log`） |
| 6 | `bash docs/evidence/t023/matrix_example.sh` | EXIT=0；900 局 wall_ms=26651（端口 15707） |
| 7 | `bash docs/evidence/t023/sample_smoke.sh` | PASS=52 FAIL=0，SCRIPT_EXIT=0（端口 15707） |

### 增量授权与硬串行时序（Lead 裁决留痕）

- release 段 13G 冷建门实测 CommitFree 11.6G 不足 → 上报；Lead 裁决**附条件增量放行（同 T022 先例口径）**。
  增量事实：同仓 release 全依赖树（libbevy/libbevy_internal/libsim rlib 02:06:55–02:07:00、
  host.exe 02:07:15）六分钟前由 T019 −j1 冷编与 Lead T022 主仓门禁在共享 target 落盘，无过期；
  本单改动仅 host/src/{rpc.rs,main.rs} 两文件 ⇒ 增量秒级成立。
- **硬串行时序（裁决原因 = 共享 target 冷编在飞）**：① 起跑前进程门轮询——连续两次间隔 ≥90s
  检查均为 rustc/cargo 计数 = 0 才准起 release（实测 02:22:20 / 02:23:20 / 02:24:20 三连零、
  span 120s，达标）；② 起跑后 90s 护栏（`timeout -k 10 90`）+ 冷编签名中止条件——实测 7.22s 完成，
  日志仅 `Compiling sim` + `Compiling host` 两个本地 crate（无 bevy 依赖树在编迹象），护栏未触发。
- 冒烟与矩阵在 release 完成后串行执行；端口独占（15702/15706/15707）；全部实例收尾杀净
  （收尾复核 host 进程计数 = 0）。

## 运行副产物说明

- `docs/evidence/t018/brp-smoke-run.log` 及 `t018/host-{stdout,stderr}.log`、`docs/evidence/t022/`
  同名运行日志由各自脚本按设计重写（运行副产物，两档内文件零人工改动——沿 T020/T021/T022 先例）；
  现场实录副本 `t018-smoke-rerun.log` / `t022-smoke-rerun.log` 已归档本档自含。
- 本档 `host-stdout.log` / `host-stderr.log` 为 `sample_smoke.sh` 宿主运行留痕（矩阵脚本同名文件
  为上一轮产物，已被冒烟覆盖——同类运行件，非人工档）；`matrix-example-run.log` /
  `sample-smoke-run.log` 均为最终干净窗口实录（见上报节事件 3）。
- `dispatch-sheet.md` 为 Lead 派发档，全程零写入。
- `gate-host-release.log` 按附录 G 先例仅清理机器路径前缀（`<tree-root>`），命中行内容原样留痕。
- `target/` 为运行件目录（`.gitignore` 含 `/target`），不入库。

## 上报节（待 Lead / 审核裁决）

1. **脚本缺陷 1 项（非产品缺陷）**：首轮 `sample_smoke.sh` 51/52——CHK-10e 未知兵种消息模式
   未计 JSON 转义（log 原文 `unknown kind \"laser\"`）；模式补 `\"` 后 52/52。产品面无误：
   响应含 `-32602` + 未知兵种消息（原文在档）。
2. **示例档 threads 选择注（供 Lead 知晓；非验收项）**：`threads` 语义 = 局内池
   （派工单 §2「run_battle_with(max_ticks, pool)」）；200 单位规模下局内并行开销显著——
   探针（单格 100 局、同参数）：threads=1 → 2826ms、threads=12 → 17145ms，两档结果逐位一致。
   示例档取缺省 threads=1（wall_ms=26651 ≈ 派工单「约 ≤40s」估算；threads=12 首轮 900 局
   实测 155.6s，该轮窗口可能与其他构建资源竞争、非纯净值）。性能非本卡验收面（吞吐验收 M0 闭环）；
   若后续卡希望批内**跨局并行**（T009 matrix 口径），属方法语义变更 → 需 Lead 裁决后再动。
3. **运行纪律事件 2 件（操作层、零数据影响）**：① 首试 release 用 `cd+&` 复合后台命令触发
   沙箱权限挂起，90s 护栏误杀空转任务（无产物、无构建启动、无污染）→ 改前台
   `timeout -k 10 90` 实现护栏后正常（`gate-host-release.log` 为正常轮实录）；
   ② 单格探针宿主 kill 未生效遗留 1 进程（02:33–02:37 占 15707，同为 t023-a 二进制）——
   查明后杀净；矩阵与 `sample_smoke` 已全量干净重跑（最终档案 = 干净窗口实录；
   两轮 `sample_smoke` 关键判定行逐项一致——EQ/CROSS/体量数值相同，确定性纪律亦在档）。
4. **示例矩阵结局构造性说明（观察项，非缺陷）**：9 格单兵种 100v100 @lane50m 全部收敛为
   确定性结局（每格 100 局同 winner、跨 seed 稳健；镜像对局恒红胜，与 T009 归档一致，
   结构性归因未深查）；中间胜率构型需混编构成（T022 数值回归对象），超出本卡示例范围未做。
   与 T009 归档同格 max |Δpp| = 0——新方法（BRP 玩家面）与 M0 归档实测互证。
