# T010 渲染灰盒 spike 开工前预研（2026-10-04）

> **状态：预研草稿，未实测。** 本报告为纯读调研产出（预研 agent，2026-10-04），全部 API 断言附官方出处；标注「需源码复核」的项（DeepWiki 来源）须在 T010 开工时以本地 `~/.cargo` 的 bevy 0.19.1 源码复核后方可写进结论。T010 开工时本文件转正为设计输入或作废重调研。

（预研时本仓 `Cargo.lock` 已锁定 `bevy 0.19.1`（crates.io），docs.rs 0.19.0/0.19.1 均已发布，examples 证据取自 GitHub `v0.19.0` tag。）

---

## 一、任务卡要点提炼（taskset/t010-render-spike.md）

| 项 | 内容 |
|---|---|
| 目标 | 胶囊体灰盒 **1 万单位同屏**，回答 M0 三问之二「渲染层灰盒能同屏多少单位」 |
| 验收① | 10,000 胶囊体同屏 @60fps 且 **1% low ≥ 45**（基准机 A = i5-12490F / 32GB / RTX 3050） |
| 验收③ | 降档扫描 **1k / 2k / 5k / 10k** 四档同口径数据（规模曲线） |
| 验收④ | 采样**原始帧时间数据**入档（非仅汇总值）；帧时间环形缓冲（非挂钟平均）入档 |
| 采集口径 | release 构建、固定分辨率、**窗口化独占、vsync 关闭**、记录 CPU/GPU/驱动环境、采样 ≥60s，输出 avg / 1% low / 0.1% low |
| 灰盒口径 | 无动画 / 无 LOD / 无分层渲染；单位位置由 **seed 可复现的确定性伪随机场**生成（不依赖 sim）；固定相机固定角度 |
| 架构 | 独立 crate `render-spike/`（bevy **完整 features**，与 sim 的 minimal 面隔离），入 workspace members |
| 范围外 | 分层渲染（公告板/点阵——那是渲染腿失败后 M1 关键路径动作）、战斗接入、打击感 |
| 未达标 | 只产数据与判定 + 触发 6.1 渲染腿后果记录；报告级修订归 T013 |
| 前置 | 仅 T001（不依赖 sim 战斗系统），W2-W3 可与 T007-T009 并行 |

口径提醒（AGENTS.md 表 6-0）：帧预算常态 @60fps 模拟 ≤8ms、极限 @30fps 22+11ms；灰盒指标高于 M2/M3 属预期、不作常态渲染外推依据。

---

## 二、bevy 0.19 万级单位 2D 渲染候选路线对比

**结论：「胶囊体」口径直接排除纯 sprite 贴图路线（无纹理、纯色几何），首选路线 A（Mesh2d + 共享 Capsule2d mesh + MeshMaterial2d），对照路线可选 SpriteMesh。官方 `bevymark.rs` 已内置三条路线的 A/B 切换，可直接作为 T010 骨架参考。**

### 路线 A：Mesh2d + 共享 mesh/material 句柄（推荐主线）

- API 形态（0.19 实证）：`(Mesh2d(meshes.add(Capsule2d::new(radius, half_length))), MeshMaterial2d(materials.add(Color::from(...))), Transform)`
  - `Capsule2d::new(25.0, 50.0)` 存在于官方 primitive 列表：examples/2d/2d_shapes.rs（https://github.com/bevyengine/bevy/blob/v0.19.0/examples/2d/2d_shapes.rs）
  - Mesh2d + MeshMaterial2d + `Camera2d` 一句话 spawn：examples/2d/mesh2d.rs（https://github.com/bevyengine/bevy/blob/v0.19.0/examples/2d/mesh2d.rs）
- 批处理机制：Mesh2d 实例进 `BatchedInstanceBuffer<Mesh2dUniform>`，`RenderMesh2dInstance` 带 `automatic_batching` 标志，`GetBatchData` 按 mesh_asset_id + material_bind_group_index 合批（来源：DeepWiki bevy 仓库问答，AI 生成、需开工时以 crates/bevy_sprite_render 源码复核 → https://deepwiki.com/bevyengine/bevy）。**同一 mesh 句柄 + 同一 material 句柄是合批前提**——官方 bevymark 的 Mesh2d 模式正是全实体共享一个 `quad: meshes.add(Rectangle::from_size(...))` 句柄（见下）。
- 规模参照：`bevymark.rs` Mesh2d 模式可跑到数十万 quad（per_wave × waves 由 CLI 控制）；`many_cubes.rs` 系列为 3D 对照。
- 对 T010 的适配度：**最高**——胶囊体 = 一个共享 Capsule2d mesh 句柄 × 10k 实例；若全单位同色（单 material 句柄）可最大化合批；若需阵营两色，用 2 个 material 句柄（≈2 批）。

### 路线 B：独立 Sprite（`Sprite` 组件，无 Bundle）

- API 形态（0.19 实证）：`(Sprite { image: handle.clone(), custom_size: Some(tile), color, ..default() }, Transform)`，`commands.spawn_batch(...)`：examples/stress_tests/many_sprites.rs（https://github.com/bevyengine/bevy/blob/v0.19.0/examples/stress_tests/many_sprites.rs）
- 规模参照：该 example 默认 **320×320 = 102,400 个 sprite** 同屏并移动相机（含视锥剔除测试）；注释明确「--colored 参数导致多批渲染、降低性能」——**同 image 才能合批（SpriteBatch），色彩 tint 叹变化会破坏合批**。
- ECS 开销：每 sprite 一个实体 + Transform；spawn 阶段官方用 `spawn_batch` 一次提交。运行期主要成本在 extract/prepare 的 CPU 侧实例缓冲（`SpriteMeta::sprite_instance_buffer`，来源同 DeepWiki，需源码复核）。
- 对 T010 的适配度：**不贴合**——Sprite 必须持 `image: Handle<Image>`（贴图），而灰盒胶囊体是纯色几何。仅当用程序生成 1×1 白图 + color tint 才能套用，且 tint 多色会碎批。列为对照项即可。

### 路线 B'：`SpriteMesh`（0.19 新组件，mesh 后端的 Sprite API）

- 定位（docs.rs 官方文档原文）："This is a carbon copy of Sprite that uses the Mesh backend instead of the Sprite backend"，唯一 API 差异是新增 `alpha_mode` 字段（默认 `Mask(0.5)`；`Blend`「significantly worse for performance」）：https://docs.rs/bevy/0.19.0/bevy/sprite/struct.SpriteMesh.html
- 官方压测：examples/stress_tests/many_sprite_meshes.rs（many_sprites 的 SpriteMesh 版，同为 102,400 实例）：https://github.com/bevyengine/bevy/blob/v0.19.0/examples/stress_tests/many_sprite_meshes.rs
- 0.19 发布讨论确认其为 sprite 后端向 mesh 后端统一的第一步（官方发布帖 https://bevy.org/news/bevy-0-19 将「统一 2D/3D 渲染内部」列为后续方向，说明该路线仍在演进）。
- 对 T010 的适配度：中等——同样要求贴图（`image` 字段）；作为「sprite 规模上界」的对照测量项有价值。

### 路线 C：GPU instancing（官方「automatic instancing」）

- **官方 example 实际名称为 `examples/shader/automatic_instancing.rs`**（v0.19.0 tag 实测存在于 shader 目录；`examples/3d/gpu_instancing.rs` 在 0.15/0.16/0.19 三个 tag 的 3d 目录均**不存在**——凡训练记忆中的 `gpu_instancing` example 名一律作废）：https://github.com/bevyengine/bevy/blob/v0.19.0/examples/shader/automatic_instancing.rs
- 该 example 文档注释："Shows that multiple instances of a cube are automatically instanced in one draw call ... use the same mesh handle and material handle for each instance"，并演示 `MeshTag` 组件把每实例索引传入自定义材质。
- **关键限制：automatic instancing 演示的是 Mesh3d；其 GPU compute 扩展（MeshInputUniform → MeshUniform）未实现于 2D 管线**。2D 的 Mesh2d/Sprite 走 CPU 侧批处理（BatchedInstanceBuffer / sprite_instance_buffer）（来源：DeepWiki 问答，AI 生成、开工时以源码复核）。0.19 发布帖也确认 2D 不在本轮 GPU-driven 收益范围（https://bevy.org/news/bevy-0-19）。
- 对 T010：**不能作为 2D 的独立路线宣称**——万级 2D 胶囊体的合批上限由 CPU 侧 2D 管线决定。报告中如实写「2D 无 GPU-driven instancing，量出的就是 2D 管线真实上限」。

### 路线 D：自定义 render feature / 直接 wgpu 通道（兜底）

- 官方 0.19 参考代码：examples/2d/mesh2d_manual.rs——「manually render 2d items using "mid level render apis" with a custom pipeline」，涉及 `SpecializedRenderPipeline`、wrap `Mesh2dPipeline`、`Transparent2d`、`ViewSortedRenderPhases`、`AddRenderCommand`、自管 `RenderColoredMesh2dInstances(MainEntityHashMap)` 实例存储、手写 WGSL（https://github.com/bevyengine/bevy/blob/v0.19.0/examples/2d/mesh2d_manual.rs）
- 成本评估：**高**。0.19 渲染架构大改（RenderGraph → ECS schedules，见风险清单），网上旧教程（0.15/0.16 时代 RenderGraph Node 写法）基本失效，只能以 0.19 自带 example + 迁移指南为据；再叠加 wgpu 原生通道等于第二套渲染栈维护。M0 灰盒**不应进入此路线**——仅当 A/B' 四档扫描全部未达标、且 owner 决定渲染腿续命时才评估（属 6.1 失败后 M1 分层渲染的动作空间）。

### bevymark.rs：T010 的最佳骨架参考（三路线 A/B 切换 + 官方确定性量测口径）

https://github.com/bevyengine/bevy/blob/v0.19.0/examples/stress_tests/bevymark.rs，0.19 版实证要点：

- 三模式 CLI 参数 `--mode sprite|spritemesh|mesh2d`，对应组件组合：
  - Sprite：`(Sprite { image, color, ..default() }, transform, Bird{velocity})`
  - SpriteMesh：`(SpriteMesh { image, color, alpha_mode, ..default() }, transform, Bird{velocity})`
  - Mesh2d：`(Mesh2d(bird_resources.quad.clone()), MeshMaterial2d(material), transform, Bird{velocity})`——单一共享 quad 句柄
- `--benchmark` 开关：**全量 wave 预生成即刻满载 + movement 用固定 `FIXED_DELTA_TIME = 1.0/60.0` 步进保证跨次运行一致**（官方确定性压测口径，与 T010 的 seed 复现口径同构）
- 所有 RNG 用 `ChaCha8Rng::seed_from_u64(42)`（rand crate；T010 的确定性伪随机场可直接借用此模式——注意 sim 态禁超越函数与此无关，渲染 crate 无此约束）
- `ColorMaterial { color, texture, alpha_mode }` 是 0.19 的 ColorMaterial 形态

---

## 三、30Hz 模拟与 60fps 渲染的解耦模式建议

**M0 事实：T010 不接 sim**（前置仅 T001，位置由 seed 伪随机场自生成）——本节为 M0 收尾后若走向 M1 的预置建议，M0 内零额外工作量。

| 模式 | 形态 | 0.19 依据 | 复杂度评估 |
|---|---|---|---|
| **M0 现状（推荐维持）** | `render-spike` 独立 crate，渲染 App 自产位置；sim crate 保持 headless 最小面（当前 `default-features = false`） | 仓库 Cargo.toml 既有分层 | 最低；符合任务卡「不依赖 sim」 |
| **进程内·固定步进（M1 合流首选）** | 单渲染 App，`RunFixedMainLoop` 以 30Hz 驱动 sim 库函数（sim 作纯库被调用，每 Fixed tick 步进一次），渲染帧间可做位置插值（可选） | `RunFixedMainLoop` / `FixedMain` / `FixedUpdate` / `FixedMainScheduleOrder` 均在 0.19 `bevy::app` 模块（https://docs.rs/bevy/0.19.0/bevy/app/index.html）；bevymark 的 scheduled spawner 即跑在 `FixedUpdate` | 低-中；单 World 单事件循环，无 IPC；sim 核心逻辑保持在纯函数层即可复用于 headless 量测 |
| 进程内·双 SubApp | sim 独立 `SubApp`（0.19 app 模块存在 SubApp/SubApps）或双 App + channel | 同上 | 中；边界清晰但状态镜像代码量大 |
| 进程间 | sim headless 进程（`ScheduleRunnerPlugin` + `RunMode::Loop(Duration)`，docs.rs app 模块实证存在）+ 渲染进程 IPC/共享内存 | 同上 | 高；M0 明确不取——双份世界状态 + 确定性纪律（AGENTS 硬约束 4）复杂化，且 M0 三问不需要 |

**建议**：M0 维持零耦合；报告 V1.0 若需描述合流路径，写「进程内 RunFixedMainLoop + sim 纯库」为默认取向。注意 sim 是否引入 bevy_app 依赖是 T002 的决定，本预研不预设。

---

## 四、帧预算推算与量测方法建议

### 预算推算（表 6-0 口径）

- 常态 @60fps：帧总预算 16.67ms = 模拟 ≤8ms + 渲染/其他 ≤8.67ms → **渲染腿常态预算 ≈ 8ms 量级（含提取/排队/GPU 提交）**
- 极限 @30fps：22ms（模拟）+ 11ms（渲染）→ **渲染腿极限预算 11ms**
- T010 为独立渲染 crate（无模拟负载），量出的帧时间即渲染腿净开销；10k 档判定口径 @60fps + 1% low ≥45（即 **1% low 帧时间 ≤22.2ms**）
- 0.1% low 在 60s@60fps ≈ 3600 帧样本下只有 ~3-4 帧入样，**统计噪声大**——入档时注明样本数与分位定义（建议：将帧时间降序排序后取 top 0.1% 的均值，环形缓冲容量 ≥8192 以覆盖 60s 余量）

### 量测方法（全部有 0.19 官方实证）

1. **vsync 关闭 + 固定分辨率**（官方 benchmark 标配，many_sprites.rs 原文）：
   ```rust
   DefaultPlugins.set(WindowPlugin {
       primary_window: Some(Window {
           present_mode: PresentMode::AutoNoVsync,
           resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
           ..default()
       }), ..default()
   })
   ```
   （import 路径 `bevy::window::{PresentMode, WindowResolution}`；来源 many_sprites.rs）
2. **防后台节流**：`.insert_resource(WinitSettings::continuous())`（many_sprites.rs 同款；窗口失焦时 winit 默认暂停会污染 1% low）
3. **帧时间诊断**：`FrameTimeDiagnosticsPlugin`（注册 frame_time/fps/frame_count 三条诊断）+ `LogDiagnosticsPlugin`（周期性日志输出）——两者均在 0.19 `bevy::diagnostic` 模块，无弃用重命名（https://docs.rs/bevy/0.19.0/bevy/diagnostic/index.html）
4. **1%/0.1% low 自算**：bevy 无内建分位诊断——用 `Res<DiagnosticsStore>` 读取 frame_time 历史（`DEFAULT_MAX_HISTORY_LENGTH` 常量存在）或自维护环形缓冲（任务卡明确要求环形缓冲方法入档：建议自定义系统在 `Last` schedule 记 `time.delta()`，Real 时间源，避免 Virtual 时间 manipulations 干扰）；退出时（`AppExit`/固定 60s 计时器）dump 原始数组到 docs/evidence/t010/
5. **环境记录**：`SystemInformationDiagnosticsPlugin`（0.19 诊断模块实证存在）+ 手工登记 GPU/驱动版本，满足任务卡「记录环境」要求
6. **量测纪律**：跑采集时机器空闲独占（AGENTS 常用命令节的既有经验）；四档各跑 ≥60s

---

## 五、风险清单（0.19 相对 0.15~0.18 的 API 漂移与已知问题）

按对 T010 的杀伤力排序：

1. **【已知 bug】Sprite 与 Mesh2d 同实体不渲染**：github.com/bevyengine/bevy/issues/19459。T010 若做路线对照，绝不在同一实体混用两个后端组件。
2. **【架构级】0.19 RenderGraph → ECS schedules**：render pass 变为 render world 上的普通系统（Core2d/Core3d 等 schedules），旧 trait Node 写法废除（官方发布帖 https://bevy.org/news/bevy-0-19；迁移指南 0-18-to-0-19 条目「Render Graph as Systems」https://bevy.org/learn/migration-guides/0-18-to-0-19）。**影响**：任何 0.15~0.18 时代的自定义渲染教程/记忆均不可用；路线 D 成本因此上调；T010 主线不受影响（只用高层 Mesh2d/Sprite API）。
3. **【记忆陷阱】旧 Bundle API 全灭**：`SpriteBundle` / `AtlasSpriteBundle` / `Mesh2dHandle` / `Material2d`（类型名）均不存在于 0.19。现行写法：`Sprite{image: Handle<Image>, ..}` 直接当组件（0.16 起 TextureAtlas 不再是 Component，atlas 用 `Sprite::from_atlas_image(image, TextureAtlasSources::handle(...))`，见 examples/2d/texture_atlas.rs 与 sprite_sheet.rs https://github.com/bevyengine/bevy/blob/v0.19.0/examples/2d/texture_atlas.rs）；`Mesh2d` + `MeshMaterial2d<ColorMaterial>`；`spawn(Camera2d)` 单组件即可（required components 自动补全，0.17「Required components refactor」https://bevy.org/learn/migration-guides/0-16-to-0-17）。
4. **【模块漂移】bevy_render 大重组（0.17）**：`Mesh2d` 移入 bevy_mesh、2D 渲染类型移入 bevy_sprite_render（迁移条目「bevy_render reorganization」https://bevy.org/learn/migration-guides/0-16-to-0-17）；0.19 docs.rs 顶层已见 `sprite` / `sprite_render`、`gizmos` / `gizmos_render`、`ui` / `ui_render` 成对模块（https://docs.rs/bevy/0.19.0/bevy/）。T010 用 prelude 导入基本无感，但写绝对路径 import 时易踩。
5. **【渲染资源初始化时点】RenderStartup（0.17+）**：MeshPipeline 等 render 资源改在 RenderStartup 系统创建（0-18-to-0-19 指南条目「Resources MeshPipeline... are now created in RenderStartup systems」；0-16-to-0-17 条目「Many render resources now initialized in RenderStartup」）。影响路线 D；主线无感。
6. **【SpriteMesh 半透明性能】**：`alpha_mode` 默认 `Mask(0.5)`，切 `Blend` 官方文档明示「significantly worse for performance」（docs.rs SpriteMesh 页）。灰盒胶囊体不贴图不受影响，但对照测量若用 SpriteMesh 需固定 alpha_mode 口径入档。
7. **【合批脆弱性】**：sprite 路线同 image 才合批、颜色 tint 碎批（many_sprites 注释原文）；Mesh2d 路线同 mesh+同 material 句柄才合批（automatic_instancing.rs 注释 + DeepWiki 的 GetBatchData 依据）。**T010 设计须保证：全 10k 实体共享 1 个（或按阵营 2 个）Capsule2d mesh 句柄与 material 句柄**，且材质数量作为口径入档。
8. **【2D 无 GPU instancing】**：automatic instancing 的 GPU compute 扩展只覆盖 Mesh3d（DeepWiki 结论，需源码复核）；若 10k 档未达标，不能指望「开个 instancing 开关」补救——直接进 6.1 后果路径。
9. **【采集假红】**：vsync 未关 → 1% low 恒为帧周期整数倍；窗口失焦节流；DPI 缩放非 1.0 导致实际渲染分辨率≠名义分辨率（用 `with_scale_factor_override(1.0)`）；后台编译/杀毒扫描（机器空闲独占纪律）。
10. **【语义提示】**0.17→0.18 起 sprites/meshes 自动更新 AABB（迁移条目「Automatic Aabb updates for sprites and meshes」https://bevy.org/learn/migration-guides/0-17-to-0-18）——固定相机下无移动时影响小，但 T010 若做相机扫描类附加测试需知悉剔除依赖 AABB。

**查证不到、需开工时实测/源码复核的项**：
- 2D 批处理内部细节（BatchedInstanceBuffer / SpriteBatch / automatic_batching 标志）来自 DeepWiki（AI 生成），开工写对照结论前应以 `crates/bevy_sprite_render`（0.19.1，本地 `~/.cargo` 源码可查）复核；
- many_sprites 102,400 实例在本机（RTX 3050）的实际帧率量级——未有公开的 0.19 基准数据，T010 本身就是量测任务；
- `PresentMode::AutoNoVsync` 在 Windows 窗口模式下的实际行为（独占全屏 vs 窗口化无边框）需开工实测确认与任务卡「窗口化独占」口径一致。

---

## 六、给 T010 开工的落地建议（设计起点，非决定）

1. 骨架直接改自 bevymark.rs 的 Mesh2d 模式：`(Mesh2d(共享胶囊 mesh), MeshMaterial2d(1-2 个 ColorMaterial), Transform)` ×N，`spawn_batch` 生成；相机 `spawn(Camera2d)` 固定。
2. CLI 参数化四档规模（1k/2k/5k/10k）+ 固定 seed（ChaCha8Rng）；位置场一次 Startup 生成后**不动**（固定相机口径，排除每帧 Transform 写入成本对渲染量测的污染——如需「动态位」加测另开一档并在档注明）。
3. 采集层：AutoNoVsync + 1920×1080 + scale_factor 1.0 + WinitSettings::continuous() + 自定义环形缓冲系统（Last schedule）+ 60s 定时退出 dump 原始帧时间。
4. 对照项（可选、时间盒内做）：SpriteMesh 模式同规模一档，给「sprite 后端 vs mesh 后端」留一组对照数据，供报告 V1.0 渲染腿论述引用。

**主要出处索引**：docs.rs/bevy/0.19.0（模块页、diagnostic、app、sprite::SpriteMesh）· github.com/bevyengine/bevy tag v0.19.0 的 examples/2d/{mesh2d,2d_shapes,mesh2d_manual,texture_atlas,sprite_sheet}.rs 与 examples/stress_tests/{many_sprites,many_sprite_meshes,bevymark}.rs 与 examples/shader/automatic_instancing.rs · bevy.org/learn/migration-guides/{0-15-to-0-16, 0-16-to-0-17, 0-17-to-0-18, 0-18-to-0-19} · bevy.org/news/bevy-0-19 · github.com/bevyengine/bevy/issues/19459 · deepwiki.com/bevyengine/bevy（标注为待复核渠道）。
