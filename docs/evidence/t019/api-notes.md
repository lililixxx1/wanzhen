# T019 API 查证注记（★ 清单落档，worker 实现段）

- 查证方式：本地 cargo registry 源码逐条行号核对（bevy 0.19.1 = workspace 锁版，
  registry 路径含机器哈希不入库，crate 名 + 版本 + 文件 + 行号已足以独立复核）。
- 核对日期：2026-10-07（T019 实现段）。
- 承接先例：render-spike（T010）`docs/evidence/t010/api-notes.md`——bevy_full 别名
  + derive 宏坑（Resource 等 derive 展开为 `bevy_ecs::` 绝对路径，需
  `use bevy_full::ecs as bevy_ecs;`，render-spike/src/main.rs:23-26）、
  `GlobalAmbientLight`（非 AmbientLight）、`AppExit` 是 Message、
  `Mesh3d`/`MeshMaterial3d` 包裹组件、`Capsule3d::new(radius, length)` radius 在前、
  `WindowResolution::new`/`PresentMode::AutoNoVsync`、`Transform::looking_at`。
  本卡沿用，不重复展开；以下为 T019 新用面。

## 1. 相机（D6）

- `Camera3d`：`#[derive(Component)] #[require(Camera, Projection)]`，
  `bevy_camera-0.19.1/src/components.rs:25-31`；`impl Default`（:33-40）。
  组装 = `Camera3d::default()` + 显式 `Projection::Orthographic(...)` 覆盖
  required 缺省（`Projection` 缺省为透视——enum 定义 :216）。
- `OrthographicProjection` 结构体：`bevy_camera-0.19.1/src/projection.rs:580`
  （字段 near:586 / far:592 / viewport_origin:605 / scaling_mode:625 / scale:633
  / area:633——`area` 由 camera_system 按 scaling_mode 自动重算，源注
  :633-640「In this case, `area` should not be manually modified」）。
- `OrthographicProjection::default_3d()`：同文件 :782-792（scale 1.0 / near 0.0
  / far 1000.0 / viewport_origin (0.5,0.5) / WindowSize / area Rect(-1,1)）。
- `ScalingMode::AutoMin { min_width, min_height }`：同文件 :542（定义）；
  求积实现 :696-705——两轴取「各 ≥ 最小值」的较大约束且保持窗口纵横比
  （无畸变），正是「X 覆盖 lane 全场、Y 覆盖表现散布带」的语义面。
- 俯视相机基向量验算（覆盖算式全文见 host/src/present.rs `rebuild_on_deploy`
  注释）：eye→target = (0,-1,0)、up 参数 = world −Z ⇒ 局部 X = Y×Z = (1,0,0)
  = 屏幕右 = 世界 +X。

## 2. 六 mesh 原语构造签名（D5；bevy_math-0.19.1/src/primitives/dim3.rs）

| 原语 | 构造签名 | 行号 | 本卡取值（兵种） |
| --- | --- | --- | --- |
| `Sphere` | `new(radius: f32)` | :47 | `Sphere::new(0.4)`（shieldman） |
| `Cuboid` | `new(x_length, y_length, z_length: f32)` | :710 | `Cuboid::new(0.8, 0.8, 0.8)`（heavyknight） |
| `Cylinder` | `new(radius, height: f32)` | :806 | `Cylinder::new(0.35, 0.8)`（militia） |
| `Capsule3d` | `new(radius, length: f32)` | :885 | `Capsule3d::new(0.3, 0.5)`（swordsman） |
| `Cone` | `new(radius, height: f32)` | :955 | `Cone::new(0.4, 0.8)`（pikeman） |
| `Torus` | `new(inner_radius, outer_radius: f32)` | :1162 | `Torus::new(0.25, 0.15)`（archer） |

- 全部经 `bevy::prelude::*` 可达（bevy_math primitives 随 prelude 重导出，
  T010 scene.rs `Capsule3d` 同款）；`Assets<Mesh>::add(primitive)` 沿 T010。

## 3. bevy_ui 文本最小面（D7；本仓无先例，逐条核对）

- `Text(pub String)`：`bevy_ui-0.19.1/src/widget/text.rs:105-117`；
  `#[require(Node, TextLayout, TextFont, TextColor, LineHeight, LetterSpacing,
  TextNodeFlags, ContentSize, FontHinting::Enabled)]`——`Text::new(...)` 单组件
  spawn 即合法（官方 doctest :60-91 同款）；文本更新 = `Query<&mut Text>` 后
  经 `Deref/DerefMut`（derive :109）直写内层 String。
- `TextFont`：`bevy_text-0.19.1/src/text.rs:376`；`font: FontSource`（:392 附近
  字段区）+ `font_size: FontSize`；缺省构造 `..default()` 即用缺省字体面；
  `font_size` 为 **FontSize 枚举**（`FontSize::Px(f32)`——编译首轮 E0308 实锚，
  官方 doctest bevy_ui widget/text.rs:79 同款），非裸 f32。
- 默认字体机制：`FontSource` 定义 `bevy_text-0.19.1/src/text.rs:282`——
  `#[default] Handle(Handle<Font>)` 变体文档字面（:284-290）：「If the default
  font handle is used, then if `default_font` feature is enabled (enabled by
  default in `bevy` crate), `FiraMono-subset.ttf` compiled into the library is
  used」——bevy_full（default features）下零配置即有内置字体，无需
  `Font::default()` 手工装载。
- `TextColor(pub Color)`：同文件 :1066；`Node`/`Val`/`PositionType`：
  bevy_ui prelude（bevy_ui-0.19.1/src/lib.rs:63-77 prelude 区
  `ui_node::*`/`ui_transform::*` glob 重导出）。

## 4. Window 字段（D3；bevy_window-0.19.1/src/window.rs）

- `present_mode: PresentMode` :166 / `resolution: WindowResolution` :172 /
  `title: String` :174 / `focused: bool` :237；`focused` 缺省 `true`
  （Default impl :481-498 内 `focused: true`）——main.rs 显式钉 `focused: true`
  防上游漂移（语义 = 缺省不变）。
- `WindowResolution::new(physical_width: u32, physical_height: u32)`：
  bevy_window-0.19.1/src/window.rs:923——**u32 传参**（编译首轮 E0308 实锚；
  T010 render-spike 同款，勿写浮点）。
- `WindowPlugin { primary_window, .. }` 组装沿 render-spike/src/main.rs:182-185。

## 5. 截图（D8；bevy_render-0.19.1/src/view/window/screenshot.rs）

- `Screenshot(pub RenderTarget)` :80 + `primary_window()` :98；
  `ScreenshotCaptured { entity, image }` :49-52；`save_to_disk`（同步写 PNG）
  :134——与工作流仓 `game/src/rpc/screenshot.rs` 0.19.1 核实注记一致（该仓
  注记：capture 异步、`ScreenshotCaptured` 为 EntityEvent、`EntityWorldMut::
  observe` 挂双 observer（save_to_disk + 自记录）、两 observer 相对次序无契约）。
- **ScreenshotPlugin 无需手工 add**：`bevy_render-0.19.1/src/view/window/mod.rs:26,32`
  ——bevy_render 装配时自动 `app.add_plugins(ScreenshotPlugin)`（DefaultPlugins
  路径下生效）；`clear_screenshots` 挂 First 调度（screenshot.rs:406-425）。
- spawn+observe 形态照工作流仓先例（受理 → 轮询 → 文件核验两段式契约），
  响应面按本卡 D8 收敛为 `{status, id}` / `{status:"captured", path, bytes}`
  （方法面固定 6 不加 screenshot_log——派工单事实 3「形态参照勿整抄」）。

## 6. 其它

- `error_codes::INVALID_PARAMS = -32602` / `INTERNAL_ERROR = -32603`：
  `bevy_remote-0.19.1/src/lib.rs:1401/:1404`（截图轮询文件核验失败走
  -32603，不静默 pending——防轮询死等，见 host/src/spectate.rs）。
- `Time::delta() -> Duration`：0.19 无 `delta_ns()`（T010 api-notes 已核），
  驱动预算用 `delta().as_secs_f64()`。
- sim 侧既有公共 API 复核（本卡直接引用）：`World::units() -> &[Unit]`（
  sim/src/world.rs:662）、`Unit { alive, kind, side, x }`（:422-433）、
  `outcome(&self)`（:943）、`run_with(n, pool)` = n 次 `step_with`
  （:895-899——分帧切块推进与单次直推终态逐位一致的确证）、
  `ONE_Q32_32 = 1 << 32`（sim/src/units.rs:23）、`UnitKind` 六变体序
  （sim/src/units.rs:47 附近，Shieldman=0…Militia=5）。
- 确定性相关：melee-brawl seed7 无灭绝、run_to_tick 1800 收于上限 tick=1800
  （T021 归档 preset-smoke-run.log RESP 实锚 `"state_hash":"0xb82a248ff23515e2",
  "tick":1800`）→ spectate autorun 的上限收束语义（run_battle_with(max_ticks)
  resolve_by_hp 不再推 tick）成立依据；Lead 批复 2026-10-07 属 D4「终局 =
  outcome 存在」语义内。
