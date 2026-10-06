# T018 派工单（worker-1 实现段）— host crate + game.* 初始集 6 方法

- 发单：Lead（主会话），2026-10-06。依据：taskset/t018-brp-host.md（范围 + 验收断言 + 设计裁决 D1~D11）+ 备忘录定本 v1.0 §五。
- 你（worker-1）只做本单「实现清单」节；门禁由你跑（cargo check + release build）；**证据留档 / 台账 / commit 由 Lead 收口**，你不要 commit。
- sim crate 零改动（任何发现 sim 需要改的地方 → 停下上报，不拍板）。

## API 事实（已由 Lead 于 2026-10-06 对照本机 registry 源码核实，直接使用，勿再凭记忆改写）

1. `bevy` feature 名：`bevy_remote`（bevy-0.19.1/Cargo.toml:2695 `bevy_remote = ["bevy_internal/bevy_remote"]`）。
2. 方法注册：`RemotePlugin::default().with_method_main(name: impl Into<String>, handler: impl IntoSystem<In<Option<Value>>, BrpResult, M>)`（bevy_remote-0.19.1/src/lib.rs:591-599）。
3. handler 形态：`fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult`（bevy 的 `World`；经 `world.run_system_with` 独占执行，handler 内可安全 `world.get_resource_mut::<T>()`）。
4. `BrpError` 公开字段直填：`BrpError { code: i16, message: String, data: Option<Value> }`（lib.rs:1304-1312）。参数错误码 `bevy::remote::error_codes::INVALID_PARAMS: i16 = -32602`。
5. HTTP 绑定：`RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST)`（http.rs:167-170 `with_address(impl Into<IpAddr>)`）；默认地址 127.0.0.1、端口 15702（http.rs:47-60），无 CLI、写死。
6. headless 常驻循环：`MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)))`（bevy_internal-0.19.1/src/default_plugins.rs:163-190 官方文档例原样）。BRP 请求由调度内系统排空，60Hz 足够。
7. 导入形态（沿工作流仓 game/src/brp.rs 先例）：
   ```rust
   use bevy::remote::http::RemoteHttpPlugin;
   use bevy::remote::RemotePlugin;
   ```
8. sim 公共 API（全部已存在，勿改 sim）：
   - `sim::world::World::deploy(seed: u64, composition: &[(UnitKind, usize)]) -> World`
   - `sim::world::World::deploy_versus(seed, red: &[(UnitKind, usize)], blue: &[..], lane_q32: i64) -> World`（red==blue 且 lane==LANE_LEN_Q32 时与 deploy 逐位一致——既有单测锚）
   - `world.run(ticks: u64)` / `world.run_with(ticks, Option<&ThreadPool>)` / `world.run_battle_with(max_ticks, Option<&ThreadPool>) -> BattleOutcome`（幂等冻结）
   - `world.outcome() -> Option<&BattleOutcome>`；`BattleOutcome { winner: Winner, end_tick: u64, alive_red: u32, alive_blue: u32, final_hash: u64 }`；`Winner::{Red,Blue,Draw}` + `.label() -> "red"/"blue"/"draw"`
   - `world.tick: u64` / `world.last_hash: u64` / `world.units() -> &[Unit]`（`Unit { alive, kind, side, hp, cd, x }`，`side: Side::{Red,Blue}`）
   - `sim::units::UnitKind::{Shieldman,HeavyKnight,Pikeman,Swordsman,Archer,Militia}` + `.id() -> "shieldman"/"heavyknight"/"pikeman"/"swordsman"/"archer"/"militia"`；`sim::units::ONE_Q32_32`（米→Q32.32 定点乘数）
   - `sim::world::{DEFAULT_COMPOSITION, LANE_LEN_Q32, TICK_CAP_REDUCED}`（1800）
   - `sim::pool::ThreadPool::new(threads: usize) -> ThreadPool`
9. serde_json：`use serde_json::{json, Value};`，响应 `Ok(json!({...}))`。

## 实现清单（精确到文件）

### 1. 根 `Cargo.toml`：members 增加 `"host"`（保持字母序不必，沿用现有顺序风格追加即可，与 render-spike 并列）。

### 2. `host/Cargo.toml`（新建）：

```toml
# M5-01（T018）BRP/观战宿主（席位 1 headless 形态 + 席位 2）。
# 纪律：bevy 依赖最小 feature 集 + bevy_remote（备忘录 §五）；不引渲染 feature。
# sim 以纯 lib 挂载（零 bevy API），宿主只调其公共 API。
[package]
name = "host"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
bevy = { workspace = true, features = ["bevy_remote"] }
serde_json = "1"
sim = { path = "../sim" }
```

### 3. `host/src/main.rs`：组装 + banner。要点：
- `App::new()` + MinimalPlugins.set(run_loop 60Hz) + `rpc::HostRpcPlugin`（rpc.rs 内定义，内含 RemotePlugin 6 方法注册 + RemoteHttpPlugin 回环绑定 + `app.init_resource::<rpc::HostedGame>()()`）。
- banner（eprintln!，三行）：`[host] mode=headless (spectate arrives with T019)` / `[host] BRP listening on 127.0.0.1:15702 (explicit loopback bind; non-loopback forbidden)` / `[host] methods: game.deploy, game.run_to_tick, game.state_hash, game.outcome, game.run_tests, game.screenshot(stub->T019)`。
- `.run()` 收尾。

### 4. `host/src/rpc.rs`：核心。内容要求：

- 常量：6 个方法名 `pub const DEPLOY_METHOD: &str = "game.deploy";` 等。
- 域错误码（正码 4xxx，避开 JSON-RPC 保留段 -32768..-32000）：
  ```rust
  pub mod game_error_codes {
      pub const NOT_DEPLOYED: i16 = 4001;
      pub const BATTLE_RESOLVED: i16 = 4002;
      pub const SPECTATE_NOT_ENABLED: i16 = 4101;
  }
  ```
- `fn invalid_params(message: &str) -> BrpError`（INVALID_PARAMS -32602，沿工作流仓先例）。
- `pub struct GameConfig { pub seed: u64, pub max_ticks: u64, pub threads: usize, pub lane_len_m: i64 }`。
- `#[derive(Resource, Default)] pub struct HostedGame { pub world: Option<sim::world::World>, pub pool: Option<sim::pool::ThreadPool>, pub config: Option<GameConfig> }`。
- `fn kind_from_id(id: &str) -> Option<UnitKind>`（六串 match，与 UnitKind::id 同表）。
- `fn parse_composition(params: &Value, field: &str) -> Result<Vec<(UnitKind, usize)>, BrpError>`：字段缺失/非数组 → invalid_params；每项 `{kind, count}`：kind 未知 → invalid_params（message 附六串清单）、count 经 as_u64 → usize（负/非整数拒绝）；每方总数累计 ≤ 100_000 超限拒绝（防误配 OOM）。
- **deploy handler**：解析 seed（as_u64 必填）/ red / blue（必填，允许空数组）/ lane_len_m（可选默认 1000，as_i64 ≥1）/ max_ticks（可选默认 `TICK_CAP_REDUCED`，1..=14400 即 `sim::world::TICK_CAP_FULL`）/ threads（可选默认 1，1..=1024）。构 `deploy_versus(seed, &red, &blue, lane_len_m * ONE_Q32_32)`；`ThreadPool::new(threads)`；HostedGame 整体重置。响应 `{deploy_hash, tick: 0, units, alive_red, alive_blue}`（deploy_hash = world.last_hash 的 `format!("0x{h:016x}")`；alive 计数 = 数 units() 切片）。
- **run_to_tick handler**：ticks 必填 as_u64。HostedGame 未 deploy → NOT_DEPLOYED 错误（data: `{"hint": "call game.deploy first"}`）。已 resolved（`world.outcome().is_some()`）→ BATTLE_RESOLVED 错误（data 附 outcome 摘要）。否则循环：每步先数双方存活，任一为 0 → `run_battle_with(world.tick)`（灭绝初检即收束、end_tick 真值）后 break；tick 达 target（start+ticks，saturating_add）break；否则 `run_with(1, pool.as_ref())`。响应 `{tick, state_hash}`。
- **state_hash handler**：未 deploy → NOT_DEPLOYED。响应 `{tick, state_hash}`。
- **outcome handler**：未 deploy → NOT_DEPLOYED。已收束取缓存；未收束 `run_battle_with(config.max_ticks, pool.as_ref())` 驱动至终局。响应 `{winner, end_tick, alive_red, alive_blue, final_hash}`（winner = label()；final_hash hex 格式）。
- **run_tests handler**：params 可空；suite 字段可选默认 `"t018-smoke"`；非 "t018-smoke" → invalid_params（message 说明当前仅此套件，全量随 T020）。调 `crate::suite::run_smoke()` 得 `Vec<AssertionResult{name, pass, detail}>`。响应 `{suite, total, passed, failed, results}`（断言失败 ≠ 协议错误——pass=false 正常返回）。
- **screenshot handler**：恒返回 `Err(BrpError { code: SPECTATE_NOT_ENABLED, message: "spectate mode not enabled (headless host, T018); arrives with T019".into(), data: Some(json!({"mode": "headless", "planned_task": "T019"})) })`。
- `pub struct HostRpcPlugin;` impl Plugin：init_resource::<HostedGame>() + add_plugins(RemotePlugin::default().with_method_main ×6) + add_plugins(RemoteHttpPlugin::default().with_address(Ipv4Addr::LOCALHOST))。
- 注：bevy `World` 与 `sim::world::World` 同名——sim 侧类型一律写全限定 `sim::world::World`，勿 `use sim::world::World`。

### 5. `host/src/suite.rs`：冒烟套件（进程内、无净副作用、全新 World、不碰 HostedGame）：

```rust
pub struct AssertionResult { pub name: &'static str, pub pass: bool, pub detail: String }
pub fn run_smoke() -> Vec<AssertionResult>
```

四断言（黄金锚 = M0 归档值，逐字使用勿手抄换算）：
1. `golden_units0_seed42_1800`：`sim::world::World::new(42, 0)` → `run(1800)` → last_hash == `0xd3b6408fd46c2008`（十进制 15255451774252490760；用十六进制字面量）。
2. `golden_default_comp_seed42_1800`：`World::deploy(42, &DEFAULT_COMPOSITION)` → run(1800) → last_hash == `0x958c5938c8682529`（十进制 10776086108806063401）。
3. `replay_pairwise_checkpoints`：两个独立 `deploy(42, &DEFAULT_COMPOSITION)`，各 run(900)，哈希一致；再各 run(900)，哈希一致（tick 0 布阵哈希亦先对一次）。
4. `deploy_versus_mirror_equivalence`：`deploy_versus(42, &DEFAULT_COMPOSITION, &DEFAULT_COMPOSITION, LANE_LEN_Q32)` 的 last_hash == `deploy(42, &DEFAULT_COMPOSITION)` 的 last_hash（tick 0）。

detail 字段：pass 时写实际哈希值（hex），fail 时写「expected … got …」。

### 6. 文档注释：每文件头 `//!` 一段（沿仓内体例：说依据、留设计裁决号 D1~D11 对应关系）。代码内注释只写约束性说明，勿写「我做了什么」。

## 门禁（你跑，全部留 stdout 证据在你的回复里）

1. `cargo check --workspace -j 2 2>&1 | tail -5`（必须 0 警告 0 错误；bevy facade 因 feature 统一会重编，预期耗时数分钟，耐心等）。
2. `cargo build -p host --release -j 2 2>&1 | tail -3`（成功即可，勿运行 host——BRP 冒烟由 Lead 执行）。
3. 回报：改动文件清单 + 两个门禁的尾部输出原文 + 若有任何偏离本单的地方（必须为零偏离，有疑虑先问）。

## 红线

- sim/ 目录零改动；docs/、taskset/、台账零改动；不 commit。
- 禁凭记忆写 Bevy API——本单 API 事实节已全部核实，若需用到清单外 API：停下上报。
- 数字勿速算：黄金锚直接用上面字面量。
