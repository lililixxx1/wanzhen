//! `game.run_tests` 断言套件（T018/D8 冒烟集 `t018-smoke`；T020/D1 全量
//! `m5-core` 九断言）：进程内行使、无净副作用——全新 World、不碰
//! [`crate::rpc::HostedGame`]，纯读 sim 公共 API。host 不加 `#[cfg(test)]`
//! （套件经 BRP 在 release 路径行使，免 bevy_remote debug 构建足迹）。
//!
//! 套件注册单一来源（T020 预备重构）：[`run`] 按名分发 + [`names`] 清单 +
//! [`DEFAULT_SUITE`] 缺省名——rpc.rs 的 run_tests handler 只委托本模块，
//! 扩充断言面不动 rpc.rs。`m5-core` 前 4 条复用冒烟集原函数（同一实现，
//! T020/D1 字面；勿复制粘贴）。
//!
//! 黄金锚 = M0/T018 归档值（D7 锚清单 + docs/evidence/t018 归档）：十六进制
//! 字面量逐字取自派工单/归档，勿手抄换算；本卡新增锚按 PIT-M-002 纪律先
//! 占位 0 首测、与派工单锚值核对一致后回填（断言 6/7；矩阵 F 侧见
//! replay_matrix.sh）。detail 口径：pass 写实际哈希值（hex），fail 写
//! 「expected … got …」。

use sim::pool::ThreadPool;
use sim::units::{UnitKind, ONE_Q32_32};
use sim::world::{DEFAULT_COMPOSITION, LANE_LEN_Q32, TICK_CAP_REDUCED};

/// 冒烟套件名（T018 落位）。
pub const SMOKE_SUITE: &str = "t018-smoke";

/// 全量套件名（T020/D1 落位）：九断言判定主体（席位 3）。
pub const M5_CORE_SUITE: &str = "m5-core";

/// 缺省套件名（run_tests 不带 suite 参数时行使）。
pub const DEFAULT_SUITE: &str = SMOKE_SUITE;

/// 单条断言结果（pass=false 是正常返回值——断言失败 ≠ 协议错误，D5）。
pub struct AssertionResult {
    pub name: &'static str,
    pub pass: bool,
    pub detail: String,
}

/// 按名分发套件（未知名字返回 None——handler 侧转 INVALID_PARAMS）。
pub fn run(name: &str) -> Option<Vec<AssertionResult>> {
    match name {
        SMOKE_SUITE => Some(run_smoke()),
        M5_CORE_SUITE => Some(run_m5_core()),
        _ => None,
    }
}

/// 可用套件清单（错误消息用；与 [`run`] 分发同步）。
pub fn names() -> Vec<&'static str> {
    vec![SMOKE_SUITE, M5_CORE_SUITE]
}

/// 冒烟四断言（D8）：两枚 M0 黄金锚 + 同种子重放逐位一致 +
/// deploy_versus 镜像等价。
pub fn run_smoke() -> Vec<AssertionResult> {
    vec![
        golden_units0_seed42_1800(),
        golden_default_comp_seed42_1800(),
        replay_pairwise_checkpoints(),
        deploy_versus_mirror_equivalence(),
    ]
}

/// 锚①（M0 T002）：seed=42 双方空构成（裸容器路径）1800 ticks 终局哈希。
fn golden_units0_seed42_1800() -> AssertionResult {
    // M0 归档锚（十进制 15255451774252490760），逐字使用勿换算。
    const GOLDEN: u64 = 0xd3b6408fd46c2008;
    let mut world = sim::world::World::new(42, 0);
    world.run(1800);
    hash_assert("golden_units0_seed42_1800", world.last_hash, GOLDEN)
}

/// 锚②（M0 T004）：seed=42 默认构成（六兵种各 5 对称）1800 ticks 终局哈希。
fn golden_default_comp_seed42_1800() -> AssertionResult {
    // M0 归档锚（十进制 10776086108806063401），逐字使用勿换算。
    const GOLDEN: u64 = 0x958c5938c8682529;
    let mut world = sim::world::World::deploy(42, &DEFAULT_COMPOSITION);
    world.run(1800);
    hash_assert("golden_default_comp_seed42_1800", world.last_hash, GOLDEN)
}

/// 同种子重放：两个独立 World 同 seed 同参数，tick 0（布阵快照）/ 900 /
/// 1800 三点 last_hash 逐位对拍。
fn replay_pairwise_checkpoints() -> AssertionResult {
    let mut a = sim::world::World::deploy(42, &DEFAULT_COMPOSITION);
    let mut b = sim::world::World::deploy(42, &DEFAULT_COMPOSITION);

    let mut mismatch: Option<(&'static str, u64, u64)> = None;
    if a.last_hash != b.last_hash {
        mismatch = Some(("tick 0", a.last_hash, b.last_hash));
    }
    if mismatch.is_none() {
        a.run(900);
        b.run(900);
        if a.last_hash != b.last_hash {
            mismatch = Some(("tick 900", a.last_hash, b.last_hash));
        }
    }
    if mismatch.is_none() {
        a.run(900);
        b.run(900);
        if a.last_hash != b.last_hash {
            mismatch = Some(("tick 1800", a.last_hash, b.last_hash));
        }
    }

    match mismatch {
        None => AssertionResult {
            name: "replay_pairwise_checkpoints",
            pass: true,
            detail: format!(
                "tick 0/900/1800 pairwise identical; final 0x{:016x}",
                a.last_hash
            ),
        },
        // expected = 先跑的 a，got = 重放的 b。
        Some((tick, expected, got)) => AssertionResult {
            name: "replay_pairwise_checkpoints",
            pass: false,
            detail: format!("{tick} mismatch: expected 0x{expected:016x} got 0x{got:016x}"),
        },
    }
}

/// 镜像等价（D5/sim 既有单测锚的外部行使）：`deploy_versus` 镜像情形
/// （red == blue 且 lane == `LANE_LEN_Q32`）与 `World::deploy` 的 tick 0
/// 布阵快照哈希逐位一致。
fn deploy_versus_mirror_equivalence() -> AssertionResult {
    let mirror = sim::world::World::deploy_versus(
        42,
        &DEFAULT_COMPOSITION,
        &DEFAULT_COMPOSITION,
        LANE_LEN_Q32,
    );
    let plain = sim::world::World::deploy(42, &DEFAULT_COMPOSITION);
    let (mirror_hash, deploy_hash) = (mirror.last_hash, plain.last_hash);
    AssertionResult {
        name: "deploy_versus_mirror_equivalence",
        pass: mirror_hash == deploy_hash,
        detail: if mirror_hash == deploy_hash {
            format!("tick 0 last_hash identical: 0x{mirror_hash:016x}")
        } else {
            format!(
                "expected 0x{deploy_hash:016x} got 0x{mirror_hash:016x}"
            )
        },
    }
}

/// 全量套件（T020/D1 九断言）：前 4 条复用冒烟集原函数（同一实现，勿复制
/// 粘贴——D1 字面）；5~9 为 M5 判定面新增：跨线程进程内复证 / 短 lane 战斗
/// 黄金锚（PIT-M-002）/ seed 敏感性（T004 伴随锚）/ 冻结幂等 / 越过上限的
/// outcome 判定时点钉死（T018 P2-2 移交项）。
pub fn run_m5_core() -> Vec<AssertionResult> {
    vec![
        golden_units0_seed42_1800(),
        golden_default_comp_seed42_1800(),
        replay_pairwise_checkpoints(),
        deploy_versus_mirror_equivalence(),
        cross_thread_pool_equivalence_900t(),
        battle_golden_shortlane_seed42(),
        seed_sensitivity_42_43(),
        outcome_freeze_idempotence_shortlane(),
        outcome_overrun_semantics(),
    ]
}

/// 跨线程等价（T020/D1-5）：同 seed 同构成，一路 12 线程池
/// `run_with(900, Some(&pool))`、一路串行 `run(900)` → `last_hash` 与 tick
/// 逐位一致（sim 5.2 跨线程纪律的进程内复证；与 T018 CHK-10 BRP 跨线程同族）。
fn cross_thread_pool_equivalence_900t() -> AssertionResult {
    let mut parallel = sim::world::World::deploy(42, &DEFAULT_COMPOSITION);
    let mut serial = sim::world::World::deploy(42, &DEFAULT_COMPOSITION);
    let pool = ThreadPool::new(12);
    parallel.run_with(900, Some(&pool));
    serial.run(900);
    let pass = parallel.last_hash == serial.last_hash && parallel.tick == serial.tick;
    AssertionResult {
        name: "cross_thread_pool_equivalence_900t",
        pass,
        detail: if pass {
            format!(
                "t12 pool vs serial identical @tick 900; last_hash 0x{:016x}",
                parallel.last_hash
            )
        } else {
            format!(
                "mismatch @tick 900: t12 0x{:016x} (tick {}) vs serial 0x{:016x} (tick {})",
                parallel.last_hash, parallel.tick, serial.last_hash, serial.tick
            )
        },
    }
}

/// 短 lane 战斗黄金锚（T020/D1-6；PIT-M-002 先实测后回填）：seed42 民兵 5v5、
/// lane 10 m → 灭绝@tick878、red 胜、alive 1/0、final_hash
/// `0xfdbc4995554ee691`。锚源 = T018 BRP 冒烟 CHK-16 归档
/// （docs/evidence/t018/brp-smoke-run.log:90-99）。本断言走进程内直跑
/// （run_battle_with）——与 BRP 路径的语义分歧属 P0 级发现（派工单 PIT 条款），
/// 实测不符即停手上报、不得静默改锚。
fn battle_golden_shortlane_seed42() -> AssertionResult {
    // M0/T018 归档锚，十六进制字面量逐字使用勿换算（PIT-M-002：占位首测实测
    // winner red/end_tick 878/alive 1-0/0xfdbc4995554ee691，与派工单锚核对一致
    // 后回填——首测留痕 docs/evidence/t020/m5core-suite-measure.log）。
    const GOLDEN_HASH: u64 = 0xfdbc4995554ee691;
    const GOLDEN_END_TICK: u64 = 878;
    let mut world = sim::world::World::deploy_versus(
        42,
        &[(UnitKind::Militia, 5)],
        &[(UnitKind::Militia, 5)],
        10 * ONE_Q32_32,
    );
    let o = world.run_battle_with(TICK_CAP_REDUCED, None);
    let pass = o.winner.label() == "red"
        && o.end_tick == GOLDEN_END_TICK
        && o.alive_red == 1
        && o.alive_blue == 0
        && o.final_hash == GOLDEN_HASH;
    AssertionResult {
        name: "battle_golden_shortlane_seed42",
        pass,
        detail: if pass {
            format!("red/{GOLDEN_END_TICK}/1/0 final_hash 0x{GOLDEN_HASH:016x} == golden")
        } else {
            format!(
                "expected red/{GOLDEN_END_TICK}/1/0 0x{GOLDEN_HASH:016x}; got winner {} end_tick {} alive {}/{} final_hash 0x{:016x}",
                o.winner.label(),
                o.end_tick,
                o.alive_red,
                o.alive_blue,
                o.final_hash
            )
        },
    }
}

/// seed 敏感性（T020/D1-7；PIT-M-002 先实测后回填）：默认构成 seed42 vs seed43
/// 各 1800 tick → 哈希不同；seed43 侧 == T004 归档伴随值
/// `0x54611ed6ded02540`（锚源 docs/evidence/t004/README.md:90）。
fn seed_sensitivity_42_43() -> AssertionResult {
    // T004 归档伴随锚（PIT-M-002：占位首测实测 0x54611ed6ded02540，与派工单
    // 锚核对一致后回填——首测留痕 docs/evidence/t020/m5core-suite-measure.log）。
    const GOLDEN_43: u64 = 0x54611ed6ded02540;
    let mut a = sim::world::World::deploy(42, &DEFAULT_COMPOSITION);
    let mut b = sim::world::World::deploy(43, &DEFAULT_COMPOSITION);
    a.run(1800);
    b.run(1800);
    let pass = a.last_hash != b.last_hash && b.last_hash == GOLDEN_43;
    AssertionResult {
        name: "seed_sensitivity_42_43",
        pass,
        detail: if pass {
            format!(
                "seed42 0x{:016x} != seed43 0x{:016x}; seed43 == T004 golden",
                a.last_hash, b.last_hash
            )
        } else {
            format!(
                "seed42 0x{:016x}; seed43 0x{:016x} (expected seed43 0x{GOLDEN_43:016x} and != seed42)",
                a.last_hash, b.last_hash
            )
        },
    }
}

/// 终局冻结幂等（T020/D1-8）：短 lane 同构断言 6。`run_battle_with` 连调
/// （含更小 max_ticks 的重入）四元组全等且 world.tick 不推进（收束路径幂等
/// 冻结，sim/src/world.rs:913-940 缓存分支）；随后直接 `run(100)` 属纯原语
/// ——`resolved` 不影响 step/run（sim/src/world.rs:32-34 模块注），world.tick
/// 前移，但冻结缓存不被重算/清除：再读 `outcome()` 仍返回原收束点四元组。
fn outcome_freeze_idempotence_shortlane() -> AssertionResult {
    let mut world = sim::world::World::deploy_versus(
        42,
        &[(UnitKind::Militia, 5)],
        &[(UnitKind::Militia, 5)],
        10 * ONE_Q32_32,
    );
    let o1 = world.run_battle_with(TICK_CAP_REDUCED, None);
    let tick_frozen = world.tick;
    let o2 = world.run_battle_with(TICK_CAP_REDUCED, None);
    let o3 = world.run_battle_with(100, None); // 重入收束路径：幂等返回缓存
    let tick_reentered = world.tick;
    world.run(100); // 纯原语推进（模块注：resolved 不拦 run/step）
    let tick_after_run = world.tick;
    let o4 = world.outcome().cloned();

    let pass = o1 == o2
        && o2 == o3
        && tick_frozen == tick_reentered
        && o4.as_ref() == Some(&o1);
    AssertionResult {
        name: "outcome_freeze_idempotence_shortlane",
        pass,
        detail: format!(
            "freeze: run_battle_with x3 identical (end_tick {} hash 0x{:016x}), tick stays {}; \
after run(100): world.tick {} -> {} (run/step 纯原语语义), frozen cache unchanged (end_tick {})",
            o1.end_tick, o1.final_hash, tick_frozen, tick_frozen, tick_after_run, o1.end_tick
        ),
    }
}

/// 越过上限的 outcome 判定时点（T020/D1-9，T018 P2-2 移交项钉死）：先
/// `run(2000)` 越过默认 max_ticks（1800；对称构成 lane 1000 m 结构性不接敌，
/// 最快闭合对需 >2000 tick），再 `run_battle_with(1800)` → while 循环条件
/// `tick < max_ticks` 立即不成立、无灭绝 → 上限 hp 判定落在**首次求值时点**
/// （sim/src/world.rs:913-940）：winner=draw、end_tick=2000、final_hash ==
/// `run(2000)` 后的 last_hash。语义留痕：outcome 判定时点 = 收束点（灭绝）或
/// 首次求值时点——调用序影响判定，故为本套件锚定对象。
fn outcome_overrun_semantics() -> AssertionResult {
    let mut world = sim::world::World::deploy(42, &DEFAULT_COMPOSITION);
    world.run(2000);
    let hash_after_run = world.last_hash;
    let o = world.run_battle_with(TICK_CAP_REDUCED, None);
    let pass = o.winner.label() == "draw"
        && o.end_tick == 2000
        && o.final_hash == hash_after_run
        && o.alive_red == 30
        && o.alive_blue == 30;
    AssertionResult {
        name: "outcome_overrun_semantics",
        pass,
        detail: if pass {
            format!(
                "draw@2000 final_hash 0x{hash_after_run:016x} == run(2000) last_hash; alive 30/30"
            )
        } else {
            format!(
                "expected draw/2000 == run(2000) last_hash 0x{hash_after_run:016x}; got winner {} end_tick {} alive {}/{} final_hash 0x{:016x}",
                o.winner.label(),
                o.end_tick,
                o.alive_red,
                o.alive_blue,
                o.final_hash
            )
        },
    }
}

/// 黄金锚断言通用判定（detail 口径见模块注释）。
fn hash_assert(name: &'static str, actual: u64, expected: u64) -> AssertionResult {
    if actual == expected {
        AssertionResult {
            name,
            pass: true,
            detail: format!("last_hash 0x{actual:016x} == golden"),
        }
    } else {
        AssertionResult {
            name,
            pass: false,
            detail: format!("expected 0x{expected:016x} got 0x{actual:016x}"),
        }
    }
}
