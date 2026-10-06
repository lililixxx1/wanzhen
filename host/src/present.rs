//! 表现层映射（T019，M5 席位 4）+ 相机与光（席位 5）。
//!
//! **单向只读红线（编译期证明）**：本模块全部系统对模拟态只取
//! `Res<HostedGame>`（无 `ResMut`/`Mut`/`&mut`）——sim 状态零回写路径由系统
//! 签名构造保证（模拟推进唯一入口 = [`crate::spectate`] 驱动系统，经
//! [`crate::rpc::advance_ticks`]）。本模块的资源/组件写操作（Assets、
//! Transform、Visibility）全部是表现层自有状态，与模拟态无交集。
//!
//! 资产策略（派工单 D5）：6 共享 Mesh（兵种→形状映射，~0.8 m 量级）× 2 共享
//! StandardMaterial（阵营色），程序化零外部资产；N 实体一次建池（死亡
//! `Visibility::Hidden` 不 despawn——防 10k 级实体 churn）；重部署以
//! `HostedGame.generation` 世代替换检测重建（D5）。
//!
//! 坐标映射：sim 单位 x（Q32.32 定点）→ 米 → 世界 X 直映（Y/Z 表现层自由）。
//! 同 x 队列错开（D5 算式 `Y = (排队位序 % 5) * 1.2 - 2.4`）落在**表现散布带**
//! 轴 = 世界 Z（俯视相机下屏幕纵轴，见 [`rebuild_on_deploy`] 覆盖算式注）；
//! 世界 Y 恒 [`REST_Y_M`]（落地高度，cosmetic）。
//!
//! 0.19 API 依据（registry 行号另见 docs/evidence/t019/api-notes.md）：
//! `Camera3d #[require(Camera, Projection)]`（bevy_camera-0.19.1/src/
//! components.rs:25-31）、`Projection::Orthographic(OrthographicProjection)`
//! + `ScalingMode::AutoMin`（bevy_camera-0.19.1/src/projection.rs:216/523/
//! 696-705：两轴 ≥ 最小值且保持纵横比）、正交缺省 `default_3d()`（:782-792，
//! near 0/far 1000）、六 mesh 原语构造（bevy_math-0.19.1/src/primitives/
//! dim3.rs）、`GlobalAmbientLight`（0.19 场景级环境光资源名，T010 先例）、
//! `Mesh3d`/`MeshMaterial3d` 包裹组件 + `Transform::looking_at`（T010
//! scene.rs 先例）。

use bevy_full as bevy;
// derive 宏（下方 Resource/Component）展开为 `bevy_ecs::` 绝对路径；依赖名
// 为别名 bevy_full 时宏无法经清单解析命中，按 bevy_macro_utils 0.19.1 内置
// 说明补别名（render-spike/src/main.rs:23-26 + T010 api-notes 先例）。
use bevy_full::ecs as bevy_ecs;
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use sim::units::{UnitKind, ONE_Q32_32};
use sim::world::Side;

use crate::rpc::HostedGame;

/// 相机覆盖 X 轴外扩边距（米，D6 margin）。
const MARGIN_X: f32 = 5.0;
/// 表现散布带外扩边距（米；带半宽见 [`BAND_HALF`]）。
const MARGIN_Z: f32 = 1.0;
/// 同 x 队列错开带半宽（米）——D5 算式 `(排队位序 % 5) * 1.2 - 2.4` 的值域
/// 上界 |z| ≤ 2.4。
const BAND_HALF: f32 = 2.4;
/// 单位落地高度（米，cosmetic——形状 ~0.8 m 量级居中于此高度，俯视相机下
/// 仅影响遮挡序）。
const REST_Y_M: f32 = 0.8;
/// 相机高度（米；正交投影下仅影响 near/far 裁剪窗——视距 ≈ 199 m ∈ (0, 1000)）。
const CAMERA_HEIGHT: f32 = 200.0;

/// 表现层共享资产（Startup 一次建齐；重部署复用——mesh/材质与布阵无关）。
#[derive(Resource)]
pub struct PresentationAssets {
    /// 兵种形状槽（索引 = [`mesh_slot`]）。
    meshes: [Handle<Mesh>; 6],
    /// 阵营色材质槽（索引 = [`material_slot`]）。
    materials: [Handle<StandardMaterial>; 2],
}

/// 表现实体池根（generation 世代 + 相机实体 + 单位实体清单）。
#[derive(Resource, Default)]
pub(crate) struct PresentationRoot {
    /// 已建池对应的布阵世代（0 = 未建）。
    generation: u64,
    /// 相机实体（重部署 despawn 重建——lane 变化 → 覆盖窗变化）。
    camera: Option<Entity>,
    /// 单位实体池（与 `units()` 索引一一对应，经 [`PresentUnit::index`]）。
    units: Vec<Entity>,
}

/// 单位表现实体标记：`index` = `HostedGame.world.units()` 索引（查询迭代序
/// 无保证，映射必须经显式索引——不依赖实体序）。
#[derive(Component)]
pub(crate) struct PresentUnit {
    index: usize,
}

/// 兵种 → mesh 槽位（D5 映射：shieldman=Sphere / heavyknight=Cuboid /
/// pikeman=Cone / swordsman=Capsule3d / archer=Torus / militia=Cylinder）。
fn mesh_slot(kind: UnitKind) -> usize {
    match kind {
        UnitKind::Shieldman => 0,
        UnitKind::HeavyKnight => 1,
        UnitKind::Pikeman => 2,
        UnitKind::Swordsman => 3,
        UnitKind::Archer => 4,
        UnitKind::Militia => 5,
    }
}

/// 阵营 → 材质槽位（红 0 / 蓝 1）。
fn material_slot(side: Side) -> usize {
    match side {
        Side::Red => 0,
        Side::Blue => 1,
    }
}

/// sim 定点 x（Q32.32）→ 米（f64 中转，单值转换无归约序问题——确定性纪律
/// 只约束归约，此处为逐单位独立换算）。
fn unit_x_meters(x_q32: i64) -> f32 {
    (x_q32 as f64 / ONE_Q32_32 as f64) as f32
}

/// Startup：共享资产建齐 + 环境光 + 方向光（与布阵无关，全程一次）。
pub(crate) fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 六兵种形状（D5 映射表；尺寸 ~0.8 m 量级——与 sim 半径域一致）。
    // 构造签名（bevy_math-0.19.1/src/primitives/dim3.rs，registry 行号）：
    // Sphere::new(radius)=:47 / Cuboid::new(x,y,z)=:710 / Cylinder::new(radius,
    // height)=:806 / Capsule3d::new(radius,length)=:885 / Cone::new(radius,
    // height)=:955 / Torus::new(inner,outer)=:1162。
    let mesh_handles = [
        meshes.add(Sphere::new(0.4)),           // shieldman 剑盾兵
        meshes.add(Cuboid::new(0.8, 0.8, 0.8)), // heavyknight 重骑兵
        meshes.add(Cone::new(0.4, 0.8)),        // pikeman 长矛兵
        meshes.add(Capsule3d::new(0.3, 0.5)),   // swordsman 剑士
        meshes.add(Torus::new(0.25, 0.15)),     // archer 弓箭手
        meshes.add(Cylinder::new(0.35, 0.8)),   // militia 民兵
    ];
    // 阵营色（D5 字面数值）：红 srgb(0.85,0.2,0.2) / 蓝 srgb(0.2,0.35,0.9)。
    let material_handles = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.2, 0.2),
            perceptual_roughness: 0.9,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.35, 0.9),
            perceptual_roughness: 0.9,
            ..default()
        }),
    ];
    commands.insert_resource(PresentationAssets {
        meshes: mesh_handles,
        materials: material_handles,
    });

    // 0.19 场景级环境光资源名 = GlobalAmbientLight（T010 先例：AmbientLight 已
    // 是相机组件）；显式插入缺省值，防上游 LightPlugin 缺省漂移。
    commands.insert_resource(GlobalAmbientLight::default());

    // 方向光（沿 T010 scene.rs 先例：默认无阴影），自上略前照——俯视场景主光。
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 100.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// 重部署重建（D5/D6）：generation 世代替换检测 → 旧池 despawn → 相机 +
/// N 单位实体一次建池。对 `HostedGame` 只读（`Res`）——写面仅表现层自有
/// 资源/命令。
pub(crate) fn rebuild_on_deploy(
    hosted: Res<HostedGame>,
    assets: Res<PresentationAssets>,
    mut root: ResMut<PresentationRoot>,
    mut commands: Commands,
) {
    if root.generation == hosted.generation {
        return; // 无世代替换
    }
    let Some(game) = hosted.world.as_ref() else {
        return; // 未布阵（generation 0）不建池
    };
    let Some(config) = hosted.config.as_ref() else {
        return; // world 与 config 同步写入（apply_deploy），防御分支
    };

    // 清旧池（实体 + 相机；材质/网格资产共享复用不清理）。
    for entity in root.units.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(camera) = root.camera.take() {
        commands.entity(camera).despawn();
    }

    // ── D6 覆盖算式注（沿 T010 scene.rs 角点校验体例）────────────────────
    // 相机：正交俯视（world (center_x, 200, 0) → 看向 (center_x, 0, 0)，up =
    // world −Z ⇒ 屏幕右 = 世界 +X、屏幕上 = 世界 −Z；局部基叉积验算：
    // f=(0,-1,0)、Y=(0,0,-1)、Z=(0,1,0)、X=Y×Z=(1,0,0) ✓）。
    // 覆盖窗 ScalingMode::AutoMin（bevy_camera-0.19.1/src/projection.rs:696-705
    // 实测语义：两轴取「各 ≥ 最小值」的较大约束且保持窗口纵横比——无畸变）：
    // - 屏幕横轴（世界 X）：最小宽度 = max(lane_m, spread) + 2×MARGIN_X。
    //   sim 坐标系对齐注：sim x ∈ [0, lane]（deploy_versus 红队头 x=radius_0、
    //   蓝队镜像 lane−x，sim/src/world.rs:601-622）——表现层世界 X = 单位 x
    //   米值直映；派工单 D6「X 覆盖 [-lane/2-margin, +lane/2+margin]」的意图
    //   （lane 全场 + margin 同屏）由本式覆盖：相机中心 = bounds 中点，
    //   spread = 部署时点单位 x 实测散布（布阵后单位只向对侧闭合、x 范围单调
    //   收窄 ⇒ 部署界恒有效，队列可伸出 [0, lane] 界外——max(spread, lane)
    //   保界外队列同屏）。
    // - 屏幕纵轴（表现散布带 = 世界 Z）：最小高度 = 2×BAND_HALF + 2×MARGIN_Z
    //   = 5.8 m；单位 z ∈ [−2.4, +2.4]（D5 错开算式值域）⊂ [−3.4, +3.4] ✓。
    // - 相机光轴（世界 Y）：单位 y = REST_Y_M = 0.8，视距 ≈ 199.2 m ∈
    //   (near 0, far 1000) ✓（default_3d，projection.rs:782-792）。
    let units = game.units();
    let lane_m = config.lane_len_m as f32;
    let (min_x_m, max_x_m) = units
        .iter()
        .map(|u| unit_x_meters(u.x))
        .fold((f32::MAX, f32::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)));
    let (min_x_m, max_x_m) = if units.is_empty() {
        (0.0, lane_m) // 空场（双空侧 deploy，t018 CHK-15 同族边界）：按 lane 覆盖
    } else {
        (min_x_m, max_x_m)
    };
    let center_x = (min_x_m + max_x_m) * 0.5;
    let min_width = (max_x_m - min_x_m).max(lane_m) + 2.0 * MARGIN_X;
    let min_height = 2.0 * BAND_HALF + 2.0 * MARGIN_Z;

    let camera = commands
        .spawn((
            Camera3d::default(),
            Projection::Orthographic(OrthographicProjection {
                near: 0.0,
                far: 1000.0,
                scaling_mode: ScalingMode::AutoMin {
                    min_width,
                    min_height,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(Vec3::new(center_x, CAMERA_HEIGHT, 0.0))
                .looking_at(Vec3::new(center_x, 0.0, 0.0), Vec3::NEG_Z),
        ))
        .id();

    // 单位实体池：一次 N 实体（Mesh3d/MeshMaterial3d 共享 handle——D5）。
    // 初始 Transform 给布阵位；逐帧更新在 [`update_presentation`]（chain 次序
    // 保证本系统后同帧生效）。
    let mut entity_pool = Vec::with_capacity(units.len());
    for (index, unit) in units.iter().enumerate() {
        let entity = commands
            .spawn((
                PresentUnit { index },
                Mesh3d(assets.meshes[mesh_slot(unit.kind)].clone()),
                MeshMaterial3d(assets.materials[material_slot(unit.side)].clone()),
                Transform::from_xyz(unit_x_meters(unit.x), REST_Y_M, 0.0),
                Visibility::default(),
            ))
            .id();
        entity_pool.push(entity);
    }

    root.camera = Some(camera);
    root.units = entity_pool;
    root.generation = hosted.generation;
}

/// 逐帧表现映射（D5 核心）：**只读** `Res<HostedGame>`，写面仅表现组件
/// （Transform / Visibility）——编译期单向只读证明。
pub(crate) fn update_presentation(
    hosted: Res<HostedGame>,
    root: Res<PresentationRoot>,
    mut query: Query<(&PresentUnit, &mut Transform, &mut Visibility)>,
    mut queue: Local<std::collections::HashMap<i64, u32>>,
) {
    if root.generation == 0 || root.generation != hosted.generation {
        return; // 未建池 / 世代不符（重建帧由 rebuild_on_deploy 先行处理）
    }
    let Some(game) = hosted.world.as_ref() else {
        return;
    };
    let units = game.units();

    // 同 x 排队位序：每帧按 units 固定索引序计数（HashMap 仅 entry 计数、
    // **从不迭代**——AGENTS.md 确定性纪律 R4 针对「迭代序进模拟态」；此处为
    // 表现层 cosmetic 且消费序 = 固定索引序，计数结果与哈希内部分桶无关）。
    queue.clear();
    for (marker, mut transform, mut visibility) in query.iter_mut() {
        let Some(unit) = units.get(marker.index) else {
            continue; // 防御：池/世界错配（generation 一致则不可达）
        };
        let ordinal = {
            let counter = queue.entry(unit.x).or_insert(0u32);
            let value = *counter;
            *counter += 1;
            value
        };
        // D5 错开算式（值域 {-2.4, -1.2, 0, 1.2, 2.4}）→ 表现散布带（世界 Z）。
        let band_z = (ordinal % 5) as f32 * 1.2 - 2.4;
        transform.translation = Vec3::new(unit_x_meters(unit.x), REST_Y_M, band_z);
        // 死亡单位隐藏不 despawn（D5：防 10k 级实体 churn）。
        *visibility = if unit.alive {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
