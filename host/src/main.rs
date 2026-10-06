//! M5-01（T018）BRP/观战宿主入口——headless 形态（席位 1）+ game.* 初始方法面
//! （席位 2，实装见 [`rpc`]）。观战模式（窗口/表现层/HUD/`game.screenshot`
//! 实装）随 T019，本卡为 headless 桩。
//!
//! 组装（D3/D11）：`MinimalPlugins` + `ScheduleRunnerPlugin::run_loop`(60Hz)
//! + [`rpc::HostRpcPlugin`]（内含 `RemotePlugin` 6 方法注册 + `RemoteHttpPlugin`
//! 显式回环绑定 127.0.0.1:15702——BRP 无鉴权，禁止绑定非回环地址；显式绑定
//! 防上游默认值漂移）。BRP 请求由调度内系统排空，60Hz 足够且免满速空转。
//!
//! banner 走 `eprintln!`（stderr 元信息，零 bevy_log feature 依赖——D3），
//! 运行时留痕与 [`rpc`] 注册的方法面/绑定地址一一对应。

mod rpc;
mod suite;

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;

fn main() {
    // 三行运行时留痕（stderr）：headless 形态 / 回环绑定 / 方法面计数（6/6）。
    eprintln!("[host] mode=headless (spectate arrives with T019)");
    eprintln!("[host] BRP listening on 127.0.0.1:15702 (explicit loopback bind; non-loopback forbidden)");
    eprintln!("[host] methods: game.deploy, game.run_to_tick, game.state_hash, game.outcome, game.run_tests, game.screenshot(stub->T019)");

    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
            // headless 常驻循环（官方文档例原样）：BRP 请求经调度排空。
            Duration::from_secs_f64(1.0 / 60.0),
        )))
        .add_plugins(rpc::HostRpcPlugin)
        .run();
}
