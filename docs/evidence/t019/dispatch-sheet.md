# T019 派工单（worker 实现段）— 观战模式与表现层（席位 1 观战形态 + 4 + 5）

- 发单：Lead，2026-10-06。依据：taskset/t019-spectate-view.md + 备忘录 §三占位策略/§五 + T010 已验形态。
- 你只做「实现清单」；门禁你跑；**不 commit、不碰主仓**（树内工作）。
- host 改动面：Cargo.toml（D2）+ main.rs（D3 双形态组装）+ rpc.rs（D4 advance_ticks 抽取 + D8 screenshot 实装）+ 新文件 spectate.rs / present.rs / hud.rs + `docs/evidence/t019/`。sim/、suite.rs、presets.rs 零改动（并行卡领域）。

## 已核实事实（直接使用；标 ★ 的仍须你到 registry 源码复核行号后落码——禁凭记忆）

1. **bevy_full 别名模式**（T010 验证）：`bevy_full = { workspace = true, features = ["bevy_remote"] }` + 源文件 `use bevy_full as bevy;`（既有 `bevy::` 引用零改动；裸 `bevy` 依赖项同现移除——同一 package 不容双依赖项）。**derive 宏坑**：Resource 等 derive 展开为 `bevy_ecs::` 绝对路径，别名下需 `use bevy_full::ecs as bevy_ecs;`（render-spike/src/main.rs:23-26 先例 + T010 api-notes）。
2. 窗口/相机/光先例（render-spike/src/main.rs + scene.rs，全部 0.19 已验）：WindowPlugin + `WindowResolution::new(1920.0, 1080.0)` + `PresentMode::AutoNoVsync`；`GlobalAmbientLight`（非 AmbientLight）；`AppExit` 是 Message；Mesh3d/MeshMaterial3d 包裹组件 + 共享 Assets；`Capsule3d::new(radius, length)`（radius 在前）；固定相机 `Transform::from_translation(..).looking_at(Vec3::ZERO, Vec3::Y)`。
3. 截图先例（工作流仓 `<workflow-repo>\game\src\rpc\screenshot.rs`——0.19 源码级核实注记齐全，**形态参照勿整抄**：本仓方法面固定 6 不加 screenshot_log）：`bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured}`（bevy_render-0.19.1/src/view/window/screenshot.rs:68-101）+ `EntityWorldMut::observe` + ScreenshotPlugin 的 `clear_screenshots`（First 调度）——两段式契约（受理 → 轮询至 captured → PNG 魔数核对）。
4. sim 公共 API：沿 T018/T021（HostedGame/apply_deploy/advance 路径现状；`world.units() -> &[Unit]`、`unit.x` Q32.32、`UnitKind` 判别、`LANE_LEN_Q32`/`ONE_Q32_32`）。
5. ★ 需你复核后使用的 API（每条 registry 行号写进代码注释或 api-notes）：`Camera3d` + `OrthographicProjection`（0.19 正交相机组装面与 scaling 字段）；六个 Mesh 原语的 0.19 构造签名（Capsule3d 已验；Sphere/Cuboid/Cone/Torus/Cylinder 逐个）；bevy_ui 文本最小面（`Text`/`TextFont`/默认字体 `Font::default()` 或 default_fonts feature 的实际取用形态——本仓无先例，官方 examples `ui/` 目录找 0.19.1 版样例）。
6. 黄金锚：melee-brawl seed7 1800t = `0xb82a248ff23515e2`（T021 归档，跨模式对拍用）；民兵 5v5 lane10 seed42 = `0xfdbc4995554ee691`@878。

## 实现清单

### D1 模式启用 = CLI `--spectate` 开关（单二双形态）

无 cargo feature 分裂；`--spectate` 与 T021 配置参数自由组合（`--spectate --preset melee-brawl --seed 7` = 即看即打形态）。

### D2 依赖切换（见事实 1）

### D3 app 双形态组装

- headless（无 `--spectate`）：现路径逐字不变（MinimalPlugins + 60Hz run_loop）。
- spectate：`DefaultPlugins.set(WindowPlugin { focused, resolution: 1920×1080 固定（范围预裁剪「多分辨率不做」）, title: "万阵观战", present_mode: AutoNoVsync, ..default() })` + **无 ScheduleRunnerPlugin**（winit 循环驱动）+ HostRpcPlugin{port} 不变 + `SpectatePlugin`（D4~D7：状态资源 + 驱动系统 + 表现系统 + HUD + 截图 observer）。banner 增 `[host] mode=spectate ...` 行。

### D4 观战节流与驱动（确定性红线核心）

- rpc.rs 抽 `pub(crate) fn advance_ticks(game: &mut sim::world::World, pool: Option<&ThreadPool>, ticks: u64) -> ()`：**headless run_to_tick 与 spectate 驱动系统共用同一推进函数**（逐 tick 灭绝检查 + run_battle_with(当前tick) 冻结——现 handler 循环体逐字迁移，语义零漂移）。
- `SpectateState { pending: u64, autorun: bool }` Resource。
- spectate 模式 `game.run_to_tick {ticks:n}`：入 pending 队列**立即返回** `{tick: <当前>, state_hash: <当前>, queued: n}`（响应多 `queued` 字段 = 模式差异留痕；headless 响应形态不变）；已冻结仍 4002。
- 驱动系统（Update）：Time 累积 30Hz 预算（表 6-0 tick 口径；首帧起 accumulator），每帧推 `min(预算, pending)` 或 autorun 模式下恒推预算量至冻结；推进走 advance_ticks；冻结时 pending=0、autorun 停。
- autorun 语义：CLI auto-deploy 且 --spectate → autorun=true（核心循环观战腿——部署即自走至终局）；BRP deploy 恒 autorun=false（调用方驱动）；run_to_tick 入队时 autorun 置 false（手动接管优先）。

### D5 表现层映射（席位 4：单向只读，编译期可证）

- 资产：6 共享 Mesh（兵种→形状：shieldman=Sphere、heavyknight=Cuboid、pikeman=Cone、swordsman=Capsule3d、archer=Torus、militia=Cylinder——尺寸统一 ~0.8m 量级）× 2 共享 StandardMaterial（阵营色：红 `Color::srgb(0.85,0.2,0.2)` / 蓝 `Color::srgb(0.2,0.35,0.9)`）。
- 实体：deploy 后一次性创建 N 实体（Mesh3d+MeshMaterial3d+Transform+Visibility）；每帧表现系统**只读 `HostedGame.world`**：`unit.x`（Q32.32→米）映射世界 X∈[-lane/2, +lane/2]；同 x 队列错开 Y（`Y = (排队位序 % 5) * 1.2 - 2.4`——表现层 cosmetic，不入模拟态）；死亡单位 `Visibility::Hidden`（不 despawn——防 10k 级实体 churn）。表现系统签名只取 `Res<HostedGame>`（无 &mut——对模拟态零回写的编译期证明）。
- 重部署：新 World 单位数不同 → 表现层检测 deploy 世代替换（HostedGame 增 `generation: u64`，apply_deploy 递增；rpc.rs 允许触碰此行——T020 树领域外）→ 重建实体池。

### D6 相机与光（席位 5）

正交俯视全场地同屏：`Camera3d` + OrthographicProjection（★复核 0.19 形态），X 覆盖 [-lane/2-margin, +lane/2+margin]、Y 覆盖表现散布带；覆盖算式写注释（沿 T010 scene.rs 角点校验体例）。方向光沿 T010。lane 全长与 margin 从 HostedGame config 读（默认形态一次设定，重部署随 generation 重建）。

### D7 HUD（席位 5，bevy 默认字体）

三行文本：`tick=N` / `alive red=R blue=B` / 终局行（frozen 时 `WINNER=<label> end_tick=<n> hash=0x…`，未冻结显示 `—`）。★文本 API 全查证；HUD 数据只读 HostedGame。

### D8 game.screenshot 实装（两段式单方法——计数门禁 6 不变）

- spectate 模式：`{}` → 受理（对相机实体 observe ScreenshotCaptured + save_to_disk 到 `screenshot-{id}.png`（CWD 相对，id 递增））→ `{status:"requested", id}`；`{"id":n}` → `{status:"captured", path, bytes}`（PNG 魔数 \x89PNG 核对后置 captured）/ `{status:"pending"}` / 未知 id → -32602。
- headless 模式：4101 桩**逐字不变**（语义分阶段留痕沿 T018）。

### D9 证据面（docs/evidence/t019/）

`spectate_smoke.sh`（端口 15715；REQ/RESP `>&2` 落档体例）：
1. CLI `--spectate --preset melee-brawl --seed 7 --port 15715` 起（autorun 自走）→ 轮询 `game.state_hash` 至冻结 → 终局四元组 + final_hash == `0xb82a248ff23515e2`（**跨模式逐位一致主断言**——T021 headless 归档锚）；全窗无 panic。
2. 第二种子：`--spectate --seed 43 --comp swordsman:10 --lane-len-m 50 --port 15716` → 走到冻结（winner/end_tick 记录，无 panic；不作跨模式对拍——新参数实测记录 H 并 headless 对拍一轮：起 headless `--port 15717` 同参 BRP run_to_tick 至终局 → final_hash 逐位相等）。
3. 手动驱动路径：spectate 起 + BRP deploy + `run_to_tick {ticks:300}` → 立即返回（queued=300 断言）→ 轮询 tick==300 → state_hash == headless 同参 300t 哈希（对拍）。
4. screenshot 两段：`{}` → id → 轮询 `{"id"}` 至 captured → path/bytes/PNG 魔数断言。
5. 进程存活收尾；六兵种形状×阵营色可见性 = 截图 PNG 人证注记（README 记录文件名；性能/帧率零宣称——T024 领域）。
`README.md` 对账表（任务卡验收断言 1~5 逐条）。

## 门禁（你跑）

1. `cargo check --workspace -j 2` → 0 警告。
2. `cargo build -p host --release -j 2` → 成功（bevy_internal release 重编 ~10m 量级属预期，耐心；ZCode Bash 单命令 10 分钟硬超时 → 用 `run_in_background` 或拆步重跑取证）。
3. `bash docs/evidence/t018/brp_smoke.sh` → 46/46（headless 回归原绿）。
4. `bash docs/evidence/t021/preset_smoke.sh` → 30/30（T021 回归）。
5. `bash docs/evidence/t019/spectate_smoke.sh` → 全 PASS SCRIPT_EXIT=0。

## 红线

- 表现层零回写模拟态（编译期只读证明 = 系统签名）；headless 全行为逐字不变（门禁 3/4 即判）。
- 0.19 未知 API 全查证（★清单）；sim/ 与并行卡领域文件零改动；主仓只读；不 git；入库内容零机器路径。
- 窗口开启期间勿跑其他量测类任务（机器窗口独占——本卡不是量测卡但窗口化运行同理避让）。
