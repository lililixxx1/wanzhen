# T018 API 查证记录（bevy_remote 0.19.1，硬约束 2「禁凭记忆写 Bevy API」）

- 版本锚：`Cargo.lock` bevy = 0.19.1（workspace 锁 "0.19"）。
- 查证方式：本地 cargo registry 解包源码行号核对（`~/.cargo/registry/src/.../bevy_remote-0.19.1` 等，
  与 Cargo.lock 同版本，最终权威）+ 工作流仓已核实先例（`bevy-ai-workflow` 仓 `docs/brp-smoke.md`
  2026-09-26 实测 + `game/src/rpc/mod.rs` 核实注记）。沿 T010 api-notes 先例，本卡全部条目源码级核对。
- 结论先行：自定义 BRP 方法注册与错误构造的 0.19 形态全部确认，无旧版差异坑。

## 逐条查证

### 1. feature 名与依赖入口

- 源码：`bevy-0.19.1/Cargo.toml:2695` `bevy_remote = ["bevy_internal/bevy_remote"]`。
- 本卡用法：`bevy = { workspace = true, features = ["bevy_remote"] }`（host 成员层追加 feature；
  workspace 级 default-features = false 声明保持）。

### 2. 自定义方法注册（RemotePlugin::with_method_main）

- 源码：`bevy_remote-0.19.1/src/lib.rs:591-599`
  `pub fn with_method_main<M>(self, name: impl Into<String>, handler: impl IntoSystem<In<Option<Value>>, BrpResult, M>) -> Self`。
- handler 形态（先例核实注记 + 内置 handler 同形态 `builtin_methods.rs:1058-1061`）：
  `fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult`；经
  `world.run_system_with(id, message.params)` 独占执行（`lib.rs:1501`）——handler 内可安全
  `world.get_resource_mut::<T>()`。
- 本卡用法：`RemotePlugin::default().with_method_main("game.deploy", deploy_handler)` ×6 链式。

### 3. 错误构造（BrpError 公开字段 + error_codes）

- 源码：`lib.rs:1304-1312` `pub struct BrpError { pub code: i16, pub message: String, pub data: Option<Value> }`
  （data `skip_serializing_if = "Option::is_none"`）；`lib.rs:1387` 起 `error_codes::INVALID_PARAMS = -32602`。
- 本卡用法：参数错误 = INVALID_PARAMS(-32602)；域错误用正码 4xxx（D6——JSON-RPC 保留段
  -32768..-32000 避开，bevy_remote 源码注释同款口径）：NOT_DEPLOYED=4001 / BATTLE_RESOLVED=4002 /
  SPECTATE_NOT_ENABLED=4101。

### 4. HTTP 回环绑定（RemoteHttpPlugin）

- 源码：`bevy_remote-0.19.1/src/http.rs:167-170` `pub fn with_address(self, address: impl Into<IpAddr>) -> Self`；
  `http.rs:47-60` `DEFAULT_PORT = 15702` / `DEFAULT_ADDR = 127.0.0.1`。
- 本卡用法：`RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST)`（显式回环防默认值
  漂移，D11——备忘录 §五「仅回环 127.0.0.1」硬约束）。

### 5. headless 常驻循环（MinimalPlugins + ScheduleRunnerPlugin）

- 源码：`bevy_internal-0.19.1/src/default_plugins.rs:163-190`——MinimalPlugins 组成 =
  TaskPoolPlugin + FrameCountPlugin + TimePlugin + ScheduleRunnerPlugin；官方文档例原样
  `MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0/60.0)))`。
- BRP 请求排空：bevy_remote 在调度内以系统处理请求（`lib.rs:675` 起内置方法系统注册形态），宿主循环
  必须常驻；60Hz 既覆盖请求延迟（≤1 帧排队）又免满速空转 CPU。
- 本卡实测：banner 后 `rpc.discover` 即刻可达（brp-smoke-run.log CHK-00）。

### 6. 导入路径

- `use bevy::remote::RemotePlugin;` / `use bevy::remote::http::RemoteHttpPlugin;` /
  `use bevy::remote::{BrpError, BrpResult};` / `use bevy::remote::error_codes;`（先例
  `bevy-ai-workflow/game/src/brp.rs:18-20` 同款；lib.rs 导出核实）。

## 与实现的对齐

以上 6 条全部落在 `host/src/rpc.rs` / `host/src/main.rs`（见文件头注释 D 号引用）；无一条凭记忆书写。
BRP 直调实测形态（JSON-RPC over HTTP POST、响应包装 `{"jsonrpc":"2.0","result":...}` /
错误 `{"error":{"code":...,"message":...,"data":...}}`）见 `brp-smoke-run.log` 请求/响应原文。

## 审核轮勘误（2026-10-06 轻量轮）

- 派工单「API 事实 8」漏列 `sim::units::kind_from_id`（`sim/src/units.rs:171-182`，pub，CLI 报错路径
  既有公共 API）——首轮 host 侧据此误建本地重复表；整改改调 sim 公共实现（P1-1 闭环）。Lead 漏列
  原因 = 查证 grep 模式为完整串 `"pub fn from_id\|fn from_str"`（前缀锚定形态），对实际签名
  `pub fn kind_from_id` 不命中——**教训：API 存在性查证应搜裸词（如 `from_id`）而非预设签名前缀**。

