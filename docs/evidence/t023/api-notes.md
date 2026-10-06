# T023 API 查证记录（`game.sample_outcomes` 实现面）

- 版本锚：本卡**零新依赖、零新 Bevy API**——注册/错误/HTTP 面沿 T018 已查证并实测的
  形态（`docs/evidence/t018/api-notes.md`，只引用不复制正文）；本档核对本卡实际调用的
  sim 公共 API（硬约束 2「禁凭记忆写 API」在本卡落点）。
- 查证方式：树内源码 `grep -n` 行号核对（基线 bb3f4ba，附录 B.2 ④）
  + T006/T018/T020/T021/T022 已归档实测先例。

## 逐条（sim 公共 API；行号 grep -n 实测）

| # | API | 行号（树内） | 本卡用法 |
|---|---|---|---|
| 1 | `World::deploy_versus(seed, red, blue, lane_q32) -> World` | `sim/src/world.rs:567` | 每局全新 World（镜像等价性单测在同函数 doc 声明：red==blue 且 lane==`LANE_LEN_Q32` 时与 `deploy` 逐位一致——本卡锚对拍依赖此等价） |
| 2 | `World::run_battle_with(max_ticks, pool) -> BattleOutcome` | `sim/src/world.rs:913` | 每局收束（幂等冻结；全灭优先于上限；`Some(&pool)` 路径与串行逐位一致——T006 跨线程纪律，T018 冒烟 CHK-10 实测） |
| 3 | `BattleOutcome { winner, end_tick, alive_red, alive_blue, final_hash }` | `sim/src/world.rs:231-238` | 响应 outcomes 逐字段序列化（哈希 `0x%016x` 口径 D5） |
| 4 | `Winner::label() -> &'static str`（"red"/"blue"/"draw"） | `sim/src/world.rs:220` | outcomes[].winner 字符串 |
| 5 | `TICK_CAP_REDUCED = 1800` / `TICK_CAP_FULL = 14400` | `sim/src/world.rs:158` / `:162` | max_ticks 缺省与域校验（与 deploy 同域） |
| 6 | `ThreadPool::new(threads)`（threads==1 不建 worker；0 饱和） | `sim/src/pool.rs:57` | 批内单池跨局复用（执行资源，不入模拟态） |
| 7 | `sim::units::kind_from_id`（未知 id → None） | `sim/src/units.rs:172-181` | 经 `parse_composition` 复用（与 `game.deploy` 同解析同报错——单一来源） |
| 8 | SPECS 表（半径/移速等；仅证据算式引用，非代码调用） | `sim/src/units.rs:78-145` | matrix-example.md 接敌可达性算式的心算来源 |

## Bevy 面（零新增）

- handler 形态 / `RemotePlugin::with_method_main` 注册 / `BrpError` 与
  `error_codes::INVALID_PARAMS` 构造：与 T018 查证一致（t018 api-notes 第 2/3 条，
  信源 `bevy_remote-0.19.1/src/lib.rs:591-599` / `:1304-1312` / `:1387`）。
  本卡只增**一个同形 handler + 一行注册**——无新 API 面，不重复查证（沿 T021/T022 先例）。
- 响应构造用 host 既有依赖 `serde_json::json!`；统计量全程 u64 整数运算
  （万分比 `red_wins × 10000 / games`——禁浮点统计，派工单 §2 字面）。

## 与实现对齐

- 逐条落在 `host/src/rpc.rs::sample_outcomes_handler`（新 handler）；
  无一条凭记忆书写。计数门禁：`game.*` 6→7（banner methods 行 / rpc.discover /
  本卡 README 三处同步留痕，任务卡 §3 D2）。
