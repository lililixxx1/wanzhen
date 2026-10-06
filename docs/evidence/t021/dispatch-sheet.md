# T021 派工单（worker 实现段）— 配置面：预设 + 参数化 + 入口无关性

- 发单：Lead，2026-10-06。依据：taskset/t021-config-face.md + 备忘录 §三（预设+参数化）+ T018 D5/D11。
- 你只做「实现清单」；门禁你跑；证据脚本你写你跑（端口用 15712/15713，见 D5）；**不 commit、不碰主仓**。
- **sim/ 零改动；suite.rs 零改动**（并行卡 T020 领域）；host 改动面 = main.rs / rpc.rs（deploy 段）/ 新增 presets.rs / Cargo.toml（仅当必须）。

## 已核实 API 事实（直接使用）

1. T018 既有实现（本树基线 = 主仓 4840bc5）：`host/src/rpc.rs` 的 `deploy_handler`（校验/`deploy_versus` 路径/`HostedGame` 重置/hex 格式——**语义逐字保持，本单只重构不重定义**）；`HostedGame`/`GameConfig`/`game_error_codes`/`invalid_params` 同文件。
2. `bevy::app::App::init_resource` 语义 = **已存在则不覆盖**（先 insert 的资源优先）——若你对这点不确信，自行到本机 registry `bevy_app-0.19.1/src/lib.rs` 核实后再用（禁凭记忆）。
3. CLI 解析风格先例 = `sim/src/main.rs`（std::env::args 手写、`--key value` 形态、非法值 stderr + exit 2）——**引用不誊写**，树内读该文件对齐风格。
4. sim 公共 API（沿用 T018）：`World::deploy_versus` / `ThreadPool::new` / `sim::units::kind_from_id` / `ONE_Q32_32` / `TICK_CAP_REDUCED` / `TICK_CAP_FULL`。
5. 布阵哈希锚（入口无关性对拍用，M0/T018 归档）：seed=42 默认构成（六兵种各 5×2，lane 1000）tick0 = `0xe2706f0b91a2be8e`（T018 冒烟 CHK-04/09/10 三轮一致）。

## 实现清单

### D1 部署核心抽取（单一代码路径）

rpc.rs 内抽 `pub(crate) struct ResolvedDeploy { seed: u64, red: Vec<(UnitKind, usize)>, blue: Vec<(UnitKind, usize)>, lane_len_m: i64, max_ticks: u64, threads: usize }` 与 `pub(crate) fn apply_deploy(hosted: &mut HostedGame, req: ResolvedDeploy) -> serde_json::Value`（含建 World/Pool、HostedGame 重置、响应 json 构造——deploy_handler 的校验段留 handler、执行段全进 apply_deploy）。**BRP handler 与 CLI 共用 apply_deploy——入口无关性由构造保证**。既有校验规则（每方 ≤100_000、lane ≥1、max_ticks 1..=14400、threads 1..=1024、checked_mul）语义逐字保留。

### D2 预设注册表（host/src/presets.rs 新文件）

- `pub fn names() -> &'static [&'static str]` / `pub fn get(name) -> Option<PresetDef>`；`PresetDef { red, blue, lane_len_m, max_ticks }`（**不含 seed/threads**——种子与执行细节不属预设）。
- 三预设（数据化；接敌可达性算式按附录 B.2 ② 自行代入 spec 值复核后留注释）：
  - `"default"`：六兵种各 5×2（= M0 `DEFAULT_COMPOSITION`，直接引用 sim 常量构造）、lane 1000、max_ticks 1800；
  - `"melee-brawl"`：双方各 `[swordsman:15, militia:15]`、lane 100、max_ticks 1800；
  - `"last-stand"`：红 `[shieldman:20]` vs 蓝 `[militia:60]`、lane 60、max_ticks 1800（以少胜多原型，T022 地基示例）。

### D3 BRP 预设参数

`game.deploy` params 新增可选 `"preset": str`：出现时以预设为底，**显式字段（red/blue/lane_len_m/max_ticks/seed/threads）一律覆盖**；未知 preset → INVALID_PARAMS（message 列 `presets::names()`）。其余语义不变（缺 red/blue 且无 preset 仍按缺参错误；preset + 显式 red/blue 合法 = 覆盖）。

### D4 CLI 参数化 + auto-deploy

- 参数：`--seed <u64>` / `--comp <kind:count,...>`（双方对称构成，sim CLI 同语法）/ `--threads <N>` / `--max-ticks <N>` / `--lane-len-m <m>` / `--preset <name>` / `--port <u16>`（D5）。
- **任一配置参数（seed/comp/threads/max-ticks/lane-len-m/preset）出现 → 启动即 auto-deploy**（解析 → 覆盖 preset 底座 → `apply_deploy` → 构造好的 `HostedGame` 在 `add_plugins(HostRpcPlugin)` 之前 `insert_resource`——init_resource 不覆盖既有资源）；`--port` 单独出现不触发 deploy。无任何参数 = T018 纯服务形态（行为不变）。
- CLI 非法值：stderr 一行错误 + exit 2（对齐 sim CLI）；未知 `--preset` 同样 exit 2（消息列预设清单）。
- banner 增一行（仅 auto-deploy 时）：`[host] auto-deployed <preset名或"custom"> seed=<n> units=<n> tick=0`。

### D5 `--port`（T018 D11 修正案，本单落地）

- 可选 `--port <u16>`，缺省 **15702 不变**；`RemoteHttpPlugin` 组装改为 `with_port(port)`，**地址恒为 Ipv4Addr::LOCALHOST（回环硬约束不变）**；banner 的监听行打印实际端口。理由留痕：并行波次验证隔离（多树多实例同机）与后续多实例实验需要；「写死」精确化为「地址写死回环、端口默认 15702 可选覆盖」。

### D6 证据面（docs/evidence/t021/，你写你跑）

`preset_smoke.sh`（沿 T018 brp_smoke.sh 体例：REQ/RESP 走 stderr `>&2` 落档、判定行、SUMMARY、SCRIPT_EXIT、收尾杀进程）：
1. 起 host `--port 15712`；`game.deploy {"preset":"default","seed":42}` → deploy_hash == `0xe2706f0b91a2be8e`（预设路径 == T018 显式构成路径，入口无关锚）。
2. `{"preset":"melee-brawl","seed":7}` → units 60；`run_to_tick 1800` → 记录哈希 H1；重新 deploy 同参再跑 → H1 逐位一致。
3. `{"preset":"last-stand","seed":7}` → units 80；`run_to_tick 1800` → 冻结 tick < 1800（灭绝）；`game.outcome` → winner ∈ {red,blue}、一方存活 0。
4. 覆盖语义：`{"preset":"default","seed":42,"red":[militia×5],"blue":[militia×5],"lane_len_m":10}` 的 deploy_hash == 纯显式同参调用（两次调用对拍）。
5. CLI 入口无关性：另起 host `--seed 42 --preset default --port 15713` → 该实例 BRP `game.state_hash` == `0xe2706f0b91a2be8e`。
6. CLI 非法三发：`--comp laser:5` / `--preset nope` / `--threads 0` → 各自 exit 2 + stderr 留档。
7. BRP 非法两发：`{"preset":"nope"}` → -32602；`{"preset":"default","red":[{kind:"laser",count:1}]}` → -32602。
8. 进程存活收尾。
- `README.md`：文件清单 + 判定行汇总（沿 t018 体例）。

## 门禁（你跑；注意 ZCode Bash 单命令 10 分钟硬超时，超时拆步重跑）

1. `cargo check --workspace -j 2`（树根执行）→ 0 警告 0 错误。
2. `cargo build -p host --release -j 2`（树根）→ 成功。
3. `bash docs/evidence/t018/brp_smoke.sh`（树根；默认端口 15702，跑前确认无其他 host 进程）→ **46/46 PASS SCRIPT_EXIT=0**（T018 回归原绿——deploy 重构零漂移的判定门禁）。
4. `bash docs/evidence/t021/preset_smoke.sh` → 全判定 PASS、SCRIPT_EXIT=0。
5. 回报：文件清单 + 四门禁尾部输出原文 + 偏离清单（应为零；有疑虑先上报不拍板）。

## 红线

- sim/、suite.rs、docs/evidence/t018/、taskset/、台账零改动；不碰主仓；不 commit。
- deploy 语义重构零漂移（门禁 3 即判）；黄金锚字面量逐字使用。
- 附录 B.2 五类笔误检查单过一遍（数值三查/接敌可达性算式/行号 grep -n 核对再引用）。
