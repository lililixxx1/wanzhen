//! `game.run_tests` 冒烟套件（T018/D8）：进程内行使、无净副作用——全新
//! World、不碰 [`crate::rpc::HostedGame`]，纯读 sim 公共 API。host 不加
//! `#[cfg(test)]`（套件经 BRP 在 release 路径行使，免 bevy_remote debug
//! 构建足迹）；全量断言面随 T020（席位 3）。
//!
//! 黄金锚 = M0 归档值（D7 锚清单）：十六进制字面量逐字取自派工单，勿手抄
//! 换算。detail 口径：pass 写实际哈希值（hex），fail 写「expected … got …」。

use sim::world::{DEFAULT_COMPOSITION, LANE_LEN_Q32};

/// 当前唯一套件名（非此值 → 参数错误；全量套件随 T020）。
pub const SMOKE_SUITE: &str = "t018-smoke";

/// 单条断言结果（pass=false 是正常返回值——断言失败 ≠ 协议错误，D5）。
pub struct AssertionResult {
    pub name: &'static str,
    pub pass: bool,
    pub detail: String,
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
