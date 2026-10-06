//! 最小 HUD（T019，M5 席位 5；派工单 D7）：三行文本——
//! `tick=N` / `alive red=R blue=B` / 终局行（frozen 时
//! `WINNER=<label> end_tick=<n> hash=0x…`，未冻结 `—`）。
//!
//! 数据面**只读** `Res<HostedGame>`（单向只读红线，同 [`crate::present`]）；
//! 存活计数复用 [`crate::rpc::alive_counts`]（单一来源，与 sim 内部计数同
//! 口径——按索引序单遍）；终局行读 `outcome()`（`&self` 纯读，sim/src/
//! world.rs:943）——与 `game.outcome` BRP 响应同源，一致性由构造保证。
//!
//! 0.19 UI 文本 API（registry 行号另见 docs/evidence/t019/api-notes.md）：
//! `Text(pub String)` + required components（Node/TextLayout/TextFont/TextColor/…，
//! bevy_ui-0.19.1/src/widget/text.rs:105-117，`Text::new` 官方 doctest 同款
//! 最小面）；`TextFont` 缺省 `font: FontSource::Handle(default)` → default_font
//! feature（bevy_full 缺省开）内置 FiraMono-subset.ttf（bevy_text-0.19.1/src/
//! text.rs:282-290 FontSource 文档字面）；`TextColor(pub Color)`（:1066）；
//! 定位用 `Node` + `PositionType::Absolute`（bevy::prelude，ui_node/ui_transform）。

use bevy_full as bevy;
// derive 宏（下方 Component）展开为 `bevy_ecs::` 绝对路径；依赖名为别名
// bevy_full 时宏无法经清单解析命中，按 bevy_macro_utils 0.19.1 内置说明补
// 别名（render-spike/src/main.rs:23-26 + T010 api-notes 先例）。
use bevy_full::ecs as bevy_ecs;
use bevy::prelude::*;

use crate::rpc::{alive_counts, HostedGame};

/// HUD 字号（px，1920×1080 固定窗口下的可读基线，cosmetic）。
const FONT_SIZE: f32 = 30.0;
/// 行距（px）。
const LINE_STEP_PX: f32 = 40.0;
/// 左/上边距（px）。
const MARGIN_PX: f32 = 8.0;

/// tick 行标记。
#[derive(Component)]
pub(crate) struct HudTickLine;
/// 存活行标记。
#[derive(Component)]
pub(crate) struct HudAliveLine;
/// 终局行标记。
#[derive(Component)]
pub(crate) struct HudEndLine;

/// Startup：三行文本实体（绝对定位左上角纵排；默认字体缺省 Handle）。
pub(crate) fn spawn_hud(mut commands: Commands) {
    let node = |top: f32| Node {
        position_type: PositionType::Absolute,
        left: Val::Px(MARGIN_PX),
        top: Val::Px(top),
        ..default()
    };
    let font = || TextFont {
        // 0.19.1 font_size 为 FontSize 枚举（bevy_text-0.19.1/src/text.rs，
        // FontSize::Px 官方 doctest 同款——bevy_ui widget/text.rs:79）。
        font_size: FontSize::Px(FONT_SIZE),
        ..default()
    };
    commands.spawn((
        HudTickLine,
        node(MARGIN_PX),
        Text::new("tick=0"),
        font(),
        TextColor(Color::WHITE),
    ));
    commands.spawn((
        HudAliveLine,
        node(MARGIN_PX + LINE_STEP_PX),
        Text::new("alive red=0 blue=0"),
        font(),
        TextColor(Color::WHITE),
    ));
    commands.spawn((
        HudEndLine,
        node(MARGIN_PX + 2.0 * LINE_STEP_PX),
        Text::new("—"),
        font(),
        TextColor(Color::WHITE),
    ));
}

/// 逐帧刷新（D7）：tick / 双方存活 / 终局行。签名只取 `Res<HostedGame>` +
/// HUD 自有组件——单向只读证明。
pub(crate) fn update_hud(
    hosted: Res<HostedGame>,
    mut tick_line: Query<
        &'static mut Text,
        (With<HudTickLine>, Without<HudAliveLine>, Without<HudEndLine>),
    >,
    mut alive_line: Query<
        &'static mut Text,
        (With<HudAliveLine>, Without<HudTickLine>, Without<HudEndLine>),
    >,
    mut end_line: Query<
        &'static mut Text,
        (With<HudEndLine>, Without<HudTickLine>, Without<HudAliveLine>),
    >,
) {
    let (tick, alive_text, end_text) = match hosted.world.as_ref() {
        Some(game) => {
            let (red, blue) = alive_counts(game.units());
            // 终局行：frozen → WINNER=<label> end_tick=<n> hash=0x…（与
            // game.outcome 响应同源同格式）；未冻结 → —（D7 字面）。
            let end = match game.outcome() {
                Some(outcome) => format!(
                    "WINNER={} end_tick={} hash=0x{:016x}",
                    outcome.winner.label(),
                    outcome.end_tick,
                    outcome.final_hash
                ),
                None => "—".to_string(),
            };
            (
                game.tick,
                format!("alive red={red} blue={blue}"),
                end,
            )
        }
        None => (0, "alive red=0 blue=0".to_string(), "—".to_string()),
    };

    if let Ok(mut text) = tick_line.single_mut() {
        **text = format!("tick={tick}");
    }
    if let Ok(mut text) = alive_line.single_mut() {
        **text = alive_text;
    }
    if let Ok(mut text) = end_line.single_mut() {
        **text = end_text;
    }
}
