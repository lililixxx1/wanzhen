# cargo check --workspace -j 3（T001 修正档：workspace 级 default-features=false 修 manifest 警告，sim 降 headless 最小面；并行 ≤3 owner 2026-09-30 指令）
$ cargo check --workspace -j 3
   Compiling serde_core v1.0.229
   Compiling syn v3.0.6
   Compiling serde v1.0.229
    Checking windows-sys v0.61.2
   Compiling crossbeam-utils v0.8.23
    Checking spin v0.10.1
    Checking hashbrown v0.16.1
    Checking foldhash v0.2.0
   Compiling bytemuck_derive v1.12.1
   Compiling thiserror v2.0.21
    Checking bevy_platform v0.19.1
   Compiling erased-serde v0.4.10
    Checking hashbrown v0.17.1
    Checking derive_more v2.1.1
    Checking indexmap v2.14.2
    Checking bytemuck v1.25.2
   Compiling thiserror-impl v2.0.21
    Checking disqualified v1.1.0
   Compiling slotmap v1.1.1
    Checking bevy_utils v0.19.1
    Checking glam v0.32.1
   Compiling bevy_reflect_derive v0.19.1
    Checking futures-core v0.3.34
    Checking downcast-rs v2.0.2
    Checking smallvec v1.16.2
    Checking futures-lite v2.6.1
    Checking crossbeam-queue v0.3.14
    Checking async-task v4.7.1
    Checking arrayvec v0.7.8
    Checking bevy_tasks v0.19.1
    Checking concurrent-queue v2.5.0
    Checking bitflags v2.13.2
    Checking log v0.4.34
    Checking fixedbitset v0.5.7
    Checking nonmax v0.5.5
    Checking either v1.18.0
    Checking itertools v0.14.0
    Checking bevy_reflect v0.19.1
    Checking bevy_ecs v0.19.1
    Checking bevy_math v0.19.1
    Checking bevy_app v0.19.1
    Checking bevy_time v0.19.1
    Checking bevy_transform v0.19.1
    Checking bevy_input v0.19.1
    Checking bevy_diagnostic v0.19.1
    Checking bevy_internal v0.19.1
    Checking bevy v0.19.1
    Checking sim v0.1.0 (<repo>\sim)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.21s
REAL_EXIT=0
