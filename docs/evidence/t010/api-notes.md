# T010 API 查证记录（bevy 0.19.1，硬约束 2）

- 版本锚：`Cargo.lock` 中 `bevy = 0.19.1`（workspace 锁定 "0.19"）。
- 查证方式：docs.rs/bevy/0.19.1 逐条 WebFetch 摘录（下称「docs.rs」）+ **本地 cargo registry 解包源码行号核对**（下称「源码」；与 Cargo.lock 同版本，为最终权威）。
- 结论先行：**0.19 相对旧版的 3 处差异直接影响本卡实现**——
  1. `Time` **无 `delta_ns()` 方法**（0.19 只剩 `delta() -> Duration` / `delta_secs()` / `delta_secs_f64()`）⇒ 采集系统用 `time.delta().as_nanos()` 换算（派工单 §4「delta_ns」按 0.19 实况落地）。
  2. `AmbientLight` 在 0.19 是**相机组件**（`#[require(Camera)]`），场景级环境光是 `GlobalAmbientLight`（Resource，LightPlugin 默认插入 brightness=80）⇒ 用 `insert_resource(GlobalAmbientLight::default())`。
  3. `AppExit` 是 **Message**（0.17 起 Event/Message 拆分），发送用 `MessageWriter<AppExit>::write(AppExit::Success)`（非旧版 EventWriter::send）。
- 另注：`Mesh3d` / `MeshMaterial3d` 组件包裹（tuple struct）为 0.15+ 已知形态，0.19 保持。

## 逐条查证

### 1. Capsule3d（构造签名：半径/半高参数序）

- URL: https://docs.rs/bevy/0.19.1/bevy/math/primitives/struct.Capsule3d.html
- docs.rs 摘录：`pub struct Capsule3d { pub radius: f32, pub half_length: f32 }`；
  `pub const fn new(radius: f32, length: f32) -> Capsule3d`（**radius 在前，length 在后**；
  `new` 中 `half_length = length / 2.0`）；页面上含 `impl From<Capsule3d> for Mesh`。
- 源码核对：`bevy_math-0.19.1/src/primitives/dim3.rs:863-890`（字段与 `new` 语义）；
  `bevy_mesh-0.19.1/src/primitives/dim3/capsule.rs:432`（`impl From<Capsule3d> for Mesh`）。
- 本卡用法：`Capsule3d::new(0.5, 1.0)` ⇒ radius=0.5、half_length=0.5、**总高 = 1.0 + 2×0.5 = 2.0** ⇒ 落地 y = 1.0。

### 2. Mesh::from(Capsule3d) 与 MeshBuilder

- URL: 同上（`impl From<Capsule3d> for Mesh` 列于 Capsule3d 页）。
- 源码核对：`bevy_mesh-0.19.1/src/primitives/mod.rs:48` `impl<T: MeshBuilder> From<T> for Mesh`；
  `bevy_mesh-0.19.1/src/primitives/dim3/capsule.rs:96`（`Capsule3dMeshBuilder: MeshBuilder`）。
- 本卡用法：`Assets<Mesh>::add` 入参 `impl Into<A>`，可直接 `meshes.add(Capsule3d::new(0.5, 1.0))`。

### 3. Assets<Mesh>::add / Assets<StandardMaterial>::add

- URL: https://docs.rs/bevy/0.19.1/bevy/asset/struct.Assets.html
- docs.rs 摘录：`pub struct Assets<A> where A: Asset`；`pub fn add(&mut self, asset: impl Into<A>) -> Handle<A>`（新分配强句柄）。
- 源码核对：`bevy_asset-0.19.1/src/assets.rs:399-405`。

### 4. Mesh3d / MeshMaterial3d（组件包裹）

- URL: https://docs.rs/bevy/0.19.1/bevy/mesh/struct.Mesh3d.html
- docs.rs 摘录：`pub struct Mesh3d(pub Handle<Mesh>);`（模块 `bevy::mesh`）。
- URL: https://docs.rs/bevy/0.19.1/bevy/pbr/struct.MeshMaterial3d.html
- docs.rs 摘录：`pub struct MeshMaterial3d<M>(pub Handle<M>) where M: Material;`，实现 `Component` / `From<Handle<M>>`。
- 源码核对：`bevy_mesh-0.19.1/src/components.rs:102`；`bevy_pbr-0.19.1/src/mesh_material.rs:41`。

### 5. StandardMaterial

- URL: https://docs.rs/bevy/0.19.1/bevy/pbr/struct.StandardMaterial.html
- docs.rs 摘录：`pub base_color: Color,` / `pub perceptual_roughness: f32,` / `pub metallic: f32,` / `pub unlit: bool,`；
  含 `impl Material for StandardMaterial` 与 `impl Default`。
- 源码核对：`bevy_pbr-0.19.1/src/pbr_material.rs:34/133/144/670`。
- 本卡用法：`StandardMaterial { base_color: Color::srgb(0.6,0.6,0.6), perceptual_roughness: 0.9, ..default() }`；
  `Color::srgb` 见 `bevy_color-0.19.1/src/color.rs:116`（`pub const fn srgb(r,g,b)`）。

### 6. AmbientLight / GlobalAmbientLight（0.19 语义变更）

- URL: https://docs.rs/bevy/0.19.1/bevy/light/struct.AmbientLight.html
- docs.rs 摘录：`pub struct AmbientLight { pub color: Color, pub brightness: f32, pub affects_lightmapped_meshes: bool }`；
  页注 required component = `Camera`（即**相机组件**，非全局 Resource）。
- 源码核对：`bevy_light-0.19.1/src/ambient_light.rs:12-30`（`#[derive(Component)] #[require(Camera)]`）；
  同文件 `:62-90` `GlobalAmbientLight`（`#[derive(Resource)]`，默认 brightness=80，由 LightPlugin 插入）。
- 本卡用法：全局环境光 = `insert_resource(GlobalAmbientLight::default())`。

### 7. DirectionalLight

- URL: https://docs.rs/bevy/0.19.1/bevy/light/struct.DirectionalLight.html
- docs.rs 摘录：字段含 `pub color: Color,` / `pub illuminance: f32,`（默认 `lux::AMBIENT_DAYLIGHT`=10,000）/ `pub shadow_maps_enabled: bool,`（默认 false，需显式开启）；
  `Component`、`Default`。
- 源码核对：`bevy_light-0.19.1/src/directional_light.rs:73-160`（Default：`shadow_maps_enabled: false`）。
- 本卡用法：`DirectionalLight::default()` + Transform（朝向由 looking_at 决定；不启用阴影，灰盒口径）。

### 8. Camera3d + Transform::looking_at

- URL: https://docs.rs/bevy/0.19.1/bevy/camera/struct.Camera3d.html
- docs.rs 摘录：`pub struct Camera3d { pub depth_load_op: ..., pub depth_texture_usages: ... }`；required `Camera, Projection`；`impl Default for Camera3d`。
- 源码核对：`bevy_camera-0.19.1/src/components.rs:25-40`；
  默认投影：`bevy_camera-0.19.1/src/projection.rs:417-427`（`fov: PI/4.0` = 45°，`near: 0.1`，`far: 1000.0`）。
- URL: https://docs.rs/bevy/0.19.1/bevy/transform/struct.Transform.html（方法页）
- 源码核对：`bevy_transform-0.19.1/src/components/transform.rs:187` `pub fn looking_at(mut self, target: Vec3, up: impl TryInto<Dir3>) -> Self`。
- 本卡用法：`Transform::from_translation(Vec3::new(0,180,180)).looking_at(Vec3::ZERO, Vec3::Y)`。

### 9. Window / WindowResolution / PresentMode / WindowPlugin

- URL: https://docs.rs/bevy/0.19.1/bevy/window/struct.Window.html
- docs.rs 摘录：`pub present_mode: PresentMode,` / `pub resolution: WindowResolution,` / `pub title: String,`；含 `impl Default for Window`。
- URL: https://docs.rs/bevy/0.19.1/bevy/window/struct.WindowResolution.html
- docs.rs 摘录：`pub fn new(physical_width: u32, physical_height: u32) -> WindowResolution`（**0.19 为 u32 物理像素**）。
- URL: https://docs.rs/bevy/0.19.1/bevy/window/enum.PresentMode.html
- docs.rs 摘录：变体 `AutoVsync = 0 / AutoNoVsync = 1 / Fifo = 2 / FifoRelaxed = 3 / Immediate = 4 / Mailbox = 5`；
  `AutoNoVsync`：按可用性取 Immediate → Mailbox → Fifo(web)。
- URL: https://docs.rs/bevy/0.19.1/bevy/window/struct.WindowPlugin.html
- docs.rs 摘录：`pub primary_window: Option<Window>,` / `pub exit_condition: ExitCondition,` / `pub close_when_requested: bool,`；`impl Default` + `impl Plugin`。
- 源码核对：`bevy_window-0.19.1/src/window.rs:164-172`（字段）、`:923`（`WindowResolution::new(u32,u32)`）、`:1219-1227`（PresentMode 变体）、`lib.rs:64-94`（WindowPlugin）。
- 本卡用法：`DefaultPlugins.set(WindowPlugin { primary_window: Some(Window { resolution: WindowResolution::new(1920,1080), present_mode: PresentMode::AutoNoVsync, ..default() }), ..default() })`；
  实际窗口物理尺寸经 `PrimaryWindow` 查询写 meta.json（`bevy_window-0.19.1/src/window.rs:56` `pub struct PrimaryWindow`）。

### 10. App / add_systems / Update

- 源码核对：`bevy_app-0.19.1/src/lib.rs:59-70`（prelude 导出 `App` / `Update` / `Startup` 等）；
  `App::add_systems(schedule, systems)` 见 `bevy_app-0.19.1/src/app.rs`。
- 本卡用法：`App::new().add_plugins(...).add_systems(Startup, setup_scene).add_systems(Update, capture_system).run()`（零 third-party 依赖，无 bevy 扩展 crate）。

### 11. AppExit（Message，0.19）

- URL: https://docs.rs/bevy/0.19.1/bevy/app/enum.AppExit.html
- docs.rs 摘录：`pub enum AppExit { Success, Error(NonZero<u8>) }`；**"A Message that indicates the App should exit."**；实现 `Message`（非 `Event`）。
- 源码核对：`bevy_app-0.19.1/src/app.rs:130`（`app.add_message::<AppExit>()`）、`:1560`（Message derive）、`:1971-1975`（测试用 `MessageWriter<AppExit>` + `.write(AppExit::Success)`）。
- URL: https://docs.rs/bevy/0.19.1/bevy/ecs/message/struct.MessageWriter.html
- docs.rs 摘录：`pub struct MessageWriter<'w, M> where M: Message`；`pub fn write(&mut self, message: M) -> MessageId<M>`；系统参数写法 `mut writer: MessageWriter<MyMessage>`。
- 源码核对：`bevy_ecs-0.19.1/src/message/message_writer.rs:62`；prelude 导出见 `bevy_ecs-0.19.1/src/lib.rs:81-84`。
- 本卡用法：`mut exit: MessageWriter<AppExit>` → `exit.write(AppExit::Success)`。

### 12. Res<Time> 的帧间隔（0.19：delta()，无 delta_ns）

- URL: https://docs.rs/bevy/0.19.1/bevy/time/struct.Time.html
- docs.rs 摘录：`pub struct Time<T = ()> where T: Default`；`pub fn delta(&self) -> Duration` / `delta_secs()` / `delta_secs_f64()`；**页面无 `delta_ns`**。
- 源码核对：`bevy_time-0.19.1/src/time.rs:196-291`（字段与 `delta`；无 `delta_ns` 全仓 grep 零命中）；
  prelude 导出 `Time`（`bevy_time-0.19.1/src/lib.rs:36-39`）。
- 本卡用法：`delta_ns = u64::try_from(time.delta().as_nanos()).unwrap_or(u64::MAX)`（帧时长 < 1 s 量级，无截断风险）。

### 13. Commands::spawn

- 源码核对：`bevy_ecs-0.19.1/src/system/commands/mod.rs:398` `pub fn spawn<T: Bundle>(&mut self, bundle: T) -> EntityCommands<'_>`。
- 本卡用法：每单位 `commands.spawn((Mesh3d(..), MeshMaterial3d(..), Transform::from_xyz(..)))`（Bundle 元组）。

## 14. derive 宏 × 重命名依赖（bevy_full 别名）——开工首编实测发现

- 现象（门禁 1 首败，原始摘录见 `runs/r0_gate1_first_attempt/compile_errors_excerpt.txt`）：
  `#[derive(Resource)]` 展开报 `E0433: cannot find module or crate bevy_ecs in this scope`
  ——宏生成代码使用 `bevy_ecs::...` 绝对路径，而本 crate 依赖名是别名 `bevy_full`。
- 机制（源码核对）：`bevy_macro_utils-0.19.1/src/bevy_manifest.rs:82-115`
  `maybe_get_path`：deps 命中 `bevy_ecs` → 用之；命中 `bevy` → `::bevy::ecs`；
  否则回退为**裸路径 `bevy_ecs`**（在调用处模块作用域解析）。源码注释明言：
  “supporting remapped crate names in derive macros is not worth that compile time ...
  As a workaround, people aliasing bevy crate names can use `use REMAPPED as bevy_X` or
  `use REMAPPED::x as bevy_x`”（L93-97）。
- 落地：在含 derive 的模块补 `use bevy_full::ecs as bevy_ecs;`（main.rs / capture.rs），
  即官方内置 workaround；`bevy_ecs` 再导出存在性见 `bevy_internal-0.19.1/src/lib.rs:43`
  `pub use bevy_ecs as ecs;`。
- 影响面：仅 derive 宏展开的 crate 路径；非 derive 的类型/函数路径不受影响
  （`use bevy_full as bevy;` 的惯用路径照常）。

## 未使用/明确排除

- `Msaa` / 阴影 / LOD / 分层：不在本卡范围（任务卡 D2 灰盒口径，保持默认管线）。
- `Time<Fixed>` / FixedUpdate：本卡采集真实帧时长（D4），不走固定步。
