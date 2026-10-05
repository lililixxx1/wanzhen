//! 场景装配（Startup）：共享胶囊 Mesh + 灰色材质 + 万单位布阵 + 固定相机 + 光。
//!
//! 覆盖算式（派工单 §3，入档 docs/evidence/t010/README.md）：
//! 相机 C=(0,180,180)、前向 f=(0,-1,-1)/√2、right=(1,0,0)、up=(0,1,-1)/√2；
//! 对场地角点 P=(±79.45, ~1, ±79.45)（jitter 极值 + 落地高度）：
//! - 近侧角 (79.45,1,79.45)：沿视轴深度 cz = (360-1-79.45)/√2 ≈ 197.6 m，
//!   垂直偏移 |cy| = |1-79.45|/√2 ≈ 55.5 m ≤ cz·tan(22.5°) ≈ 81.9 m；
//!   水平偏移 |cx| = 79.45 m ≤ cz·tan(hfov/2)（1920/1080 ⇒ hfov ≈ 72.6°）≈ 145 m。
//! 远侧角同理更宽裕 ⇒ 全场地在视锥内、全部单位同屏（角点精确校验，粗算式见 README）。
//! 0.19 API 依据（fov 45°/near 0.1/far 1000 默认）见 api-notes.md。

use bevy_full as bevy;
use bevy::prelude::*;

use crate::grid::{
    jitter_from_u64, unit_xz, CAPSULE_LENGTH_M, CAPSULE_RADIUS_M, REST_Y_M,
};
use crate::rng::SplitMix64;
use crate::RunArgs;

/// 相机位（固定量，与布阵一致的可复现场景参数）。
pub const CAMERA_POS: Vec3 = Vec3::new(0.0, 180.0, 180.0);

/// Startup：装配共享资产、布阵 N 单位、相机、方向光。
pub fn setup_scene(
    mut commands: Commands,
    args: Res<RunArgs>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 共享单 Mesh 资产（radius=0.5、length=1.0 ⇒ 总高 2.0）+ 共享灰色材质。
    // 灰盒口径：无动画 / 无 LOD / 无分层（任务卡范围外项不引入）。
    let mesh = meshes.add(Capsule3d::new(CAPSULE_RADIUS_M, CAPSULE_LENGTH_M));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.6, 0.6),
        perceptual_roughness: 0.9,
        ..default()
    });

    // 布阵：单位 i 位置只依赖 i 与 seed（前缀性质 ⇒ 四档同 seed 前缀布局）。
    let mut rng = SplitMix64::new(args.seed);
    for i in 0..args.units {
        let jx = jitter_from_u64(rng.next_u64());
        let jz = jitter_from_u64(rng.next_u64());
        let (x, z) = unit_xz(i, jx, jz);
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x, REST_Y_M, z),
        ));
    }

    // 固定相机：默认透视 fov=45°（PI/4），looking_at 原点。
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(CAMERA_POS).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 方向光：默认无阴影（shadow_maps_enabled=false），从右上方向下照。
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(100.0, 200.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
