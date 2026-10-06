# T018 M5-01 BRP 宿主与 game.* 初始方法面（席位 1 headless 形态 + 席位 2）

- 周位：W1 余~W2 初（10-07 起）
- 前置：T017（备忘录定本 v1.0——口径唯一来源）
- 类型：代码（轻量审核轮）
- 预估：6h（含 M5 全量拆卡 T018~T025 的 Lead 规划段，收口时如实分段：拆卡段 10-06 归 W1）
- 审核分级：plan-code-reviewer 轻量轮（diff + 门禁三断言核对 + 抽查复跑）

## 范围内

- workspace 新 crate（render-spike 旁；命名开工裁决，候选 `host`）——bevy 依赖**最小 feature 集 + `bevy_remote`**（`{ workspace = true }` 继承版本锁；不引渲染 feature，避开 bevy full 重编译足迹）。
- `RemotePlugin` + `RemoteHttpPlugin` 接入，**仅回环 127.0.0.1**（备忘录 §五硬约束：BRP 无鉴权、禁止绑定非回环——绑定地址代码 + 运行时输出双留痕）。
- sim 以 bevy Resource 形态挂载：宿主**只调 sim 公共 API**；sim lib 保持零 bevy 纯 lib（若需 lib 增量 API，须设计裁决留痕且不动模拟行为——黄金锚红线）。
- **game.* 初始集 6 方法**（备忘录 §五逐字）：
  - `game.deploy`（config：构成/参数/种子）→ 布阵快照哈希；
  - `game.run_to_tick`（n）→ 推进 n tick（headless 直推；观战模式按帧率节流表现随 T019）;
  - `game.state_hash` → 当前状态哈希（断言锚）；
  - `game.outcome` → 终局四元组（胜负/终局 tick/双方存活/final_hash）；
  - `game.run_tests` → 套件判定面（pass/fail + 断言清单——**判定主体**；本卡先立最小冒烟集，全量随 T020）；
  - `game.screenshot` → 观战截屏（**本卡 headless 桩**：结构化错误「观战模式未启用」，实装随 T019——方法面 6/6 在位，语义分阶段）。
- headless 模式 BRP 直调验证：脚本（curl 或等价）驱动 6 方法全调用，请求/响应原文留档 docs/evidence/t018/。
- `game.run_tests` 冒烟集：同 seed+参数 state_hash 对拍 M0 归档黄金锚（锚值清单开工定）+ 同种子重放一致。
- 环境档（沿 M0 体例）；0.19 API 查证留痕（RemotePlugin/RemoteHttpPlugin 注册形态、BRP 方法注册与响应类型、错误路径实际签名——docs.rs / 官方 examples，沿 T010 api-notes 先例；**禁凭记忆写 Bevy API**）。

## 范围外

- 观战模式（窗口/渲染插件/表现层/相机/HUD）与 `game.screenshot` 实装——T019。
- 断言面全量与断言清单任务化——T020（席位 3）。
- 配置面预设与参数化选择——T021（席位 6）。
- MCP 薄桥（T041 保持冻结、按需解冻——备忘录 §五）。
- game.* 方法扩展（初始集之后 +1 亦须同步计数门禁留痕——席位 2 说明）。

## 验收断言

1. `cargo check --workspace` 0 警告；sim crate 零 bevy 保持（依赖图证据）；宿主对 sim 只 import 公共 API。
2. game.* 6 方法经 BRP 直调全部可达且响应符合备忘录 §五签名语义（证据档留调用/响应原文；screenshot 桩 = 结构化错误不击穿进程——质量红线）。
3. 同种子+同参数序列经 BRP 重放 state_hash 逐位一致，且与 M0 归档黄金锚至少一枚对拍命中（跨线程档 ≥2 档抽查）。
4. `game.run_tests` 冒烟集 pass（pass/fail + 断言清单输出形态在位——判定面骨架）。
5. 回环硬约束留痕（监听 127.0.0.1 代码与运行时双证）。
6. 涉 bevy 的 cargo 构建按 AGENTS.md 并行上限与内存预检纪律（若触发重型冷编，-j 从严 + 前置 commit 预检）。

## 执行记录

### 设计裁决 D1~D11（2026-10-06 Lead 定稿；0.19.1 API 全部经本机 registry 源码核实，禁凭记忆条目见派工单「API 事实」节）

- **D1 crate 命名/布局**：workspace 新成员 `host/`（bin 名 `host`，render-spike 旁）。三文件：`src/main.rs`（app 组装 + banner）、`src/rpc.rs`（6 方法 handler + 参数解析 + 错误码 + HostedGame 资源）、`src/suite.rs`（run_tests 冒烟套件 4 断言）。
- **D2 依赖形态**：`bevy = { workspace = true, features = ["bevy_remote"] }`（不引渲染 feature）+ `serde_json = "1"`（Cargo.lock 已含）+ `sim = { path = "../sim" }`。**sim crate 零改动**（备忘录 §五「零 bevy 纯 lib 不动」+ 黄金锚红线）。注：sim/Cargo.toml 骨架期遗留的未用 bevy 声明（T002 前工具链验证遗留）本卡不动，evidence 如实注记。
- **D3 app 组装**：`MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0/60.0)))` + `RemotePlugin::default().with_method_main(...)` ×6 + `RemoteHttpPlugin::default().with_address(Ipv4Addr::LOCALHOST)`（显式回环防默认值漂移）。BRP 请求由调度内系统排空（bevy_remote lib.rs），60Hz 循环足够且免满速空转。banner 用 eprintln!（stderr 元信息、零 bevy_log feature 依赖）。
- **D4 宿主状态**：Resource `HostedGame { world: Option<sim::world::World>, pool: Option<ThreadPool>, config: Option<GameConfig> }`；`GameConfig { seed, max_ticks, threads, lane_len_m }`（outcome 驱动语义需 max_ticks）。命名注意：bevy `World` 与 `sim::world::World` 同名——sim 侧一律全限定 `sim::world::World`。
- **D5 方法签名**（JSON；哈希一律 `"0x" + {:016x}` hex 字符串，与 M0 CLI 口径一致）：
  - `game.deploy` params `{seed: u64, red: [{kind, count}...], blue: [...], lane_len_m: i64=1000, max_ticks: u64=1800, threads: 1..=1024=1}` → `{deploy_hash, tick: 0, units, alive_red, alive_blue}`。kind 表 = UnitKind::id 六串，**解析直接调 sim 公共 API `sim::units::kind_from_id`**（审核轮 P1-1 勘误：D5 原文「无 sim from_id 公共 API」系派工单查证 grep 模式漏列（完整串 `"pub fn from_id\|fn from_str"` 对实际签名 `pub fn kind_from_id` 不命中——API 实存于 sim/src/units.rs:171-182，整改后 id 表单一来源在 sim；教训：存在性查证搜裸词勿预设签名前缀）；count ≥0、每方单位总数 ≤100,000（防误配 OOM）；lane_len_m ≥1。实现走 `deploy_versus`（镜像情形与 deploy 逐位一致——sim 既有单测等价锚）。red/blue 允许空（tick 0 立即终局语义）。threads 建 `ThreadPool::new(n)` 随 deploy 重建。
  - `game.run_to_tick` params `{ticks: u64}` → `{tick, state_hash}`。语义：**逐 tick 推进、每步前查双方存活**（host 侧数公共 units() 切片，O(N) 可忽略）；任一方归零即停于该 tick 并经 `run_battle_with(current_tick)` 冻结终局（灭绝分支初检即收束、end_tick 真值）；越过灭绝继续推会使 outcome 的 end_tick 失真——逐 tick 检查即为此语义代价。已冻结再调 → 结构化错误 BATTLE_RESOLVED。
  - `game.state_hash` 无 params → `{tick, state_hash}`（读 last_hash）。
  - `game.outcome` 无 params → 已收束返回冻结缓存；未收束 `run_battle_with(config.max_ticks)` 驱动至终局（灭绝或上限 hp 判定）→ `{winner: red|blue|draw, end_tick, alive_red, alive_blue, final_hash}`。幂等。
  - `game.run_tests` params `{suite: str="t018-smoke"}` → `{suite, total, passed, failed, results: [{name, pass, detail}]}`。断言失败 ≠ 协议错误（正常返回 pass=false——判定主体语义）；套件无净副作用（全新 World，不碰 HostedGame）。
  - `game.screenshot` 任意 params → 结构化错误（D6 错误码 4101 + data `{mode: "headless", planned_task: "T019"}`），不击穿进程。
- **D6 错误码命名空间**：参数错误 = JSON-RPC `INVALID_PARAMS(-32602)`（沿工作流仓先例）；域错误（协议合法、业务状态拒绝）用**正码** 4xxx（避开 JSON-RPC 保留段 -32768..-32000）：`NOT_DEPLOYED=4001` / `BATTLE_RESOLVED=4002` / `SPECTATE_NOT_ENABLED=4101`。BrpError 公开字段直填（code/message/data）。
- **D7 黄金锚清单**（对拍 M0 归档）：① seed=42 双方空构成 1800 ticks → `0xd3b6408fd46c2008`（T002 锚，World::new(42,0) 等价路径）；② seed=42 默认构成（六兵种各 5 对称）1800 ticks → `0x958c5938c8682529`（T004 锚，deploy_versus 镜像路径）。跨线程抽查：锚② threads=1 与 threads=12 各跑一遍逐位一致（≥2 档）。
- **D8 冒烟套件断言集**（suite.rs，进程内全新 World）：`golden_units0_seed42_1800`（锚①）/ `golden_default_comp_seed42_1800`（锚②）/ `replay_pairwise_checkpoints`（同参两 World 独立跑 tick 0/900/1800 哈希逐位一致）/ `deploy_versus_mirror_equivalence`（对称构成 versus 与 deploy tick0 哈希逐位一致）。host 不加 #[cfg(test)]（套件经 BRP 在 release 路径行使，免 bevy_remote debug 构建足迹）。
- **D9 直调验证形态**：`docs/evidence/t018/brp_smoke.sh`（Git Bash + curl）：起 host（release 后台）→ rpc.discover + game.* ×6 全调用 + 错误路径三发（未 deploy 先 state_hash / screenshot 桩 / 冻结后 run_to_tick）→ 黄金锚对拍 + 跨线程对拍 → 原文与判定写 `brp-smoke-run.log`；环境档 `env.md` 沿 M0 体例。
- **D10 构建纪律**：workspace check 因 bevy feature 统一（render-spike bevy_full ∪ host bevy_remote）触发 bevy facade 重编——涉重型冷编纪律适用：内存预检（10-06 实测 free 24.8G ≥12G ✓）+ cargo 一律 -j 2。
- **D11 端口/地址**：写死 127.0.0.1:15702（= bevy_remote DEFAULT_ADDR/DEFAULT_PORT 常量值），无 CLI 参数（沿工作流仓 brp.rs「写死不暴露」先例）；回环约束代码 + banner 运行时双留痕。

### 审核与收口记录

- **执行链**：worker-1 按派工单实现（`docs/evidence/t018/dispatch-sheet.md`；五处实现层偏离经 Lead 复核接受——GameConfig `allow(dead_code)` / lane `checked_mul` / JSON null=缺省 / 手工 `json!` 构造 / banner 先于组装）→ Lead 门禁（check 复跑 0 警告 + BRP 冒烟首版 33/33）→ **plan-code-reviewer 轻量轮 = 有条件通过（P0×1 / P1×1 / P2×3 / S×3，报告 `review-plan-code-reviewer.md`）→ Lead 同日整改 → 复验关账**。
- **整改闭环（2026-10-06 同日）**：P0-1 冒烟 REQ/RESP 原文未落档（脚本 `$( )` 捕获缺陷）→ `brp()` echo 改 `>&2` 重跑，46/46 PASS + SCRIPT_EXIT=0 + 原文全量在档；P1-1 本地重复 kind 表 → 改调 `sim::units::kind_from_id`（D5 勘误入档）；P2-1 suite 非字符串静默回落 → 显式 INVALID_PARAMS（CHK-13b 验证）；P2-3 灭绝冻结分支零覆盖 → 补 CHK-15/16 两路径（空侧 tick0 冻结 + 短 lane 交战灭绝@tick878，含 4002 双发）；S-2 api-notes 转写笔误修正；S-3 state_hash 纯读路径改 `get_resource`（outcome/run_to_tick 驱动模拟仍需 mut——S-3 半采纳如实注）。整改后 check 0 警告 + release 重建 5.31s + 冒烟 46/46。
- **P2-2 移交 T020（规格缺口非缺陷）**：run_to_tick 不受 max_ticks 约束——超限推进后再 outcome，上限 hp 判定发生在超限 tick（调用序可影响 winner）。T020 断言面定稿时钉死语义（run_to_tick 截断/拒绝超限，或 outcome 文档明示按当前 tick 判定）。
- **黄金锚双盲核对**：锚① 0xd3b6408fd46c2008 / 锚② 0x958c5938c8682529 与 M0 归档原文（t002 双跑 / t004 三档）逐字一致（Lead grep -F + 审核轮独立复算十进制双证）。
- **复验关账（2026-10-06 同日）**：plan-code-reviewer 复验轮 **最终裁决 = 通过（可合并）**——六项整改逐字属实、无新缺陷引入、check 独立复跑 0 警告；确定性脚注 = deploy_hash `0xe2706f0b91a2be8e` 三轮运行逐字一致。复验非阻断备注 4 条处置：env 产物指纹双版本时点刷新 + 冒烟窗口行更新（随收口）、grep 模式表述精确化（裸词教训）、S-1 log 自截断保持（归档版 = 21:48 整改版，策略注记于 env.md）、审核存档改题「节录存档」（即 review 档注）。
- **耗时（挂钟全计，终填）**：≈216 min 全计 W1 10-06（Lead 规划段 ≈80 含 M5 拆卡 ≈30 + API 查证/设计裁决/派工单 ≈50；worker-1 实现 ≈40（agent 2426s，含首轮编译 14m10s）；Lead 门禁+证据 ≈25；审核轻量轮 ≈26；整改+复验 ≈30；收口回写 ≈15）。W1 快照滚动 1786+216 = 2002 min = 33.37 h（终判未到期，10-11 例行回写）。
