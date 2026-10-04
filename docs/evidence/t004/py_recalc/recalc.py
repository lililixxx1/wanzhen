#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""T004「索敌攻击与克制结算」python 独立第二实现（双盲对拍地基）。

用途：以与 Rust 实现不同的语言/代码路径，按派工单内联设计定稿独立推导同一语义，
产出与 Rust 实现可逐位对拍的终局哈希与逐单位快照。

双盲纪律（见 README.md）：
  - 编写期禁读 sim/src/{world,units,main,hash}.rs 与 docs/evidence/t004/ 下本目录外内容；
  - 唯一共享来源为 sim/src/rng.rs（RNG 移植参考），其余语义全部来自派工单文字。

零第三方依赖，仅标准库；纯整数运算（python int 天然大整数，显式 & MASK64 模拟 u64）。

用法（完整说明见 README.md）：
  python recalc.py --mode legacy|t004 --seed N --ticks N [--comp kind:count,...] [--units N]
  python recalc.py --case 1|2|3 --seed N --ticks N [--mode ...]
  python recalc.py --selftest
"""

import argparse
import sys

MASK64 = 0xFFFFFFFFFFFFFFFF
MASK32 = 0xFFFFFFFF

ONE_Q32_32 = 1 << 32   # Q32.32 的 1.0：1 米 = 4294967296
ONE_Q16_16 = 1 << 16   # Q16.16 的 1.0：1.0 = 65536

DEPLOY_SALT = 0x6465_706C_6F79_0001  # "deploy" ASCII 前缀 + 0x0001（派工单定稿）
DEPLOY_GAP = ONE_Q32_32 // 2         # 0.5 m = 2147483648（布阵邻距附加项）
MELEE_MARGIN = 20 * ONE_Q32_32 // 100  # 0.2 m = 858993459（射程余量，闭区间）
LANE_LEN = 1000 * ONE_Q32_32         # 1000 m = 4294967296000

# ---------------------------------------------------------------------------
# 定点常数表（Q32.32 / Q16.16，全部整数表达式，截断）
# ---------------------------------------------------------------------------


def q32(num, den):
    """Q32.32 常数：num/den 米（num>=0, den>0），截断取整。"""
    assert num >= 0 and den > 0
    return num * ONE_Q32_32 // den


KIND_NAMES = ["shieldman", "heavyknight", "pikeman", "swordsman", "archer", "militia"]
KIND_INDEX = {name: i for i, name in enumerate(KIND_NAMES)}

# kind 判别值 = 下标：0 shieldman / 1 heavyknight / 2 pikeman / 3 swordsman / 4 archer / 5 militia
HP       = [120, 150, 80, 90, 60, 50]
ATTACK   = [8, 14, 10, 12, 9, 6]
INTERVAL = [30, 45, 30, 25, 60, 20]
SPEED    = [q32(5, 100), q32(20, 100), q32(10, 100), q32(12, 100), q32(8, 100), q32(9, 100)]
RADIUS   = [q32(5, 10), q32(8, 10), q32(5, 10), q32(5, 10), q32(4, 10), q32(4, 10)]
ARMOR    = [0, 0, 1, 1, 2, 2]  # 0=Heavy, 1=Light, 2=Unarmored

# 克制矩阵（行=攻方类别，列=守方类别；Q16.16）：克制 98304 / 中性 65536 / 被克 43690
COUNTER = [
    [65536, 98304, 43690],  # Heavy  攻 Heavy / Light / Unarmored
    [43690, 65536, 98304],  # Light  攻
    [98304, 43690, 65536],  # Unarmored 攻
]

DEFAULT_COMP = [(i, 5) for i in range(6)]  # 六兵种各 5 = 每方 30、共 60


# ---------------------------------------------------------------------------
# RNG：SplitMix64（seed 扩展）+ Xoshiro256**（主发生器）——逐行移植自 sim/src/rng.rs
# ---------------------------------------------------------------------------


class SplitMix64:
    def __init__(self, seed):
        self.state = seed & MASK64

    def next_u64(self):
        self.state = (self.state + 0x9E37_79B9_7F4A_7C15) & MASK64
        z = self.state
        z = ((z ^ (z >> 30)) * 0xBF58_476D_1CE4_E5B9) & MASK64
        z = ((z ^ (z >> 27)) * 0x94D0_49BB_1331_11EB) & MASK64
        return z ^ (z >> 31)


def _rotl(x, k):
    return ((x << k) | (x >> (64 - k))) & MASK64


class Xoshiro256StarStar:
    def __init__(self, seed):
        sm = SplitMix64(seed)
        self.s = [sm.next_u64() for _ in range(4)]

    def next_u64(self):
        result = (_rotl((self.s[1] * 5) & MASK64, 7) * 9) & MASK64
        t = (self.s[1] << 17) & MASK64
        self.s[2] ^= self.s[0]
        self.s[3] ^= self.s[1]
        self.s[1] ^= self.s[2]
        self.s[0] ^= self.s[3]
        self.s[2] ^= t
        self.s[3] = _rotl(self.s[3], 45)
        return result

    def state_words(self):
        return list(self.s)


# ---------------------------------------------------------------------------
# 单位
# ---------------------------------------------------------------------------


class Unit:
    __slots__ = ("side", "kind", "x", "hp", "cd", "alive")

    def __init__(self, side, kind, x, hp=None, cd=0, alive=True):
        self.side = side              # 0=红, 1=蓝
        self.kind = kind              # 0..5
        self.x = x                    # Q32.32（可负，i64 语义）
        self.hp = HP[kind] if hp is None else hp
        self.cd = cd
        self.alive = alive


# ---------------------------------------------------------------------------
# 布阵（deploy）
# ---------------------------------------------------------------------------


def deploy(seed, comp):
    """独立 RNG（seed ^ DEPLOY_SALT）Fisher-Yates 全洗 + 红方递推 + 蓝方镜像。"""
    seq = []
    for kind, count in comp:
        seq.extend([kind] * count)
    m = len(seq)
    rng = Xoshiro256StarStar(seed ^ DEPLOY_SALT)
    for i in range(m - 1, 0, -1):        # i = m-1 .. 1
        j = rng.next_u64() % (i + 1)
        seq[i], seq[j] = seq[j], seq[i]

    units = []
    x = 0
    for k in range(m):                   # 红方索引 0..m-1
        r = RADIUS[seq[k]]
        if k == 0:
            x = r
        else:
            x = x - (RADIUS[seq[k - 1]] + r + DEPLOY_GAP)
        units.append(Unit(0, seq[k], x))
    for k in range(m):                   # 蓝方索引 m..2m-1：镜像 x' = 1000m - x
        units.append(Unit(1, seq[k], LANE_LEN - units[k].x))
    return units


# ---------------------------------------------------------------------------
# 每 tick 阶段：移动 -> 战斗 -> 移除
# ---------------------------------------------------------------------------


def movement(units):
    """单 lane 移动：索引序顺序结算，用当前最新位置；最近前方者，贴身停/排队堵停。"""
    n = len(units)
    for i in range(n):
        u = units[i]
        if not u.alive:
            continue
        d = 1 if u.side == 0 else -1
        xi = u.x
        best_j = -1
        best_dist = 0
        for j in range(n):               # 从 0 向上扫，严格小于才更新（平局取最小 j）
            if j == i:
                continue
            v = units[j]
            if not v.alive:
                continue
            dist = (v.x - xi) * d
            if dist > 0 and (best_j == -1 or dist < best_dist):
                best_j = j
                best_dist = dist
        if best_j == -1:
            adv = SPEED[u.kind]
        else:
            gap = best_dist - (RADIUS[u.kind] + RADIUS[units[best_j].kind])
            adv = min(SPEED[u.kind], max(gap, 0))
        u.x = xi + d * adv


def combat(units):
    """索敌（全局最近敌方，平局索引序）+ 攻击计时 + 克制结算 + 当 tick 墓碑。"""
    n = len(units)
    for u in units:                      # 先对全部存活单位推进 cd
        if u.alive and u.cd > 0:
            u.cd -= 1
    for i in range(n):                   # 再按索引序逐单位行动（须存活）
        u = units[i]
        if not u.alive:
            continue
        xi = u.x
        best_j = -1
        best_d = 0
        for j in range(n):               # 存活敌方中 |dx| 最小者，平局取最小 j
            v = units[j]
            if not v.alive or v.side == u.side:
                continue
            d = abs(v.x - xi)
            if best_j == -1 or d < best_d:
                best_j = j
                best_d = d
        if best_j == -1:
            continue
        v = units[best_j]
        reach = RADIUS[u.kind] + RADIUS[v.kind] + MELEE_MARGIN
        if best_d <= reach and u.cd == 0:
            dmg = ATTACK[u.kind] * COUNTER[ARMOR[u.kind]][ARMOR[v.kind]] // ONE_Q16_16
            v.hp -= dmg
            if v.hp <= 0:
                v.alive = False          # 墓碑：本 tick 内不可被选中、不可行动
            u.cd = INTERVAL[u.kind]


def remove_dead(units):
    return [u for u in units if u.alive]  # 保序压缩


# ---------------------------------------------------------------------------
# 状态哈希：FNV-1a 64，两种折叠模式
# ---------------------------------------------------------------------------


def fnv1a64(init, data):
    h = init
    for b in data:
        h = ((h ^ b) * 0x100000001B3) & MASK64
    return h


def u64le(v):
    return (v & MASK64).to_bytes(8, "little")


def state_hash(units, tick, state_words, mode):
    h = 0xCBF29CE484222325
    h = fnv1a64(h, u64le(tick))
    for w in state_words:
        h = fnv1a64(h, u64le(w))
    for u in units:
        h = fnv1a64(h, bytes([1 if u.alive else 0, u.kind, u.side]))
        if mode == "t004":
            h = fnv1a64(h, (u.hp & MASK32).to_bytes(4, "little"))   # i32 补码
            h = fnv1a64(h, (u.cd & MASK32).to_bytes(4, "little"))   # u32
        h = fnv1a64(h, u64le(u.x))
    return h


# ---------------------------------------------------------------------------
# 主循环
# ---------------------------------------------------------------------------


def run_sim(units, seed, ticks, mode):
    """每 tick：tick+=1 -> RNG 消耗 1 个 u64（丢弃）-> 移动 -> 战斗 -> 移除 -> 哈希。"""
    rng = Xoshiro256StarStar(seed)
    tick = 0
    for _ in range(ticks):
        tick += 1
        rng.next_u64()                   # tick 级固定消耗，不使用
        movement(units)
        combat(units)
        units = remove_dead(units)
    h = state_hash(units, tick, rng.state_words(), mode)
    return units, tick, h


# ---------------------------------------------------------------------------
# 手算对拍三例（直接构造；hp 缺省为表值，cd=0，alive=True）
# ---------------------------------------------------------------------------


def build_case(case):
    if case == 1:
        # 红 shieldman x=0.5m、蓝 pikeman x=1.5m（恰贴身 1.0m）
        return [Unit(0, 0, q32(5, 10)), Unit(1, 2, q32(15, 10))]
    if case == 2:
        # 红 militia x=0、蓝 A x=0.8m、蓝 B x=-0.8m（B = -A 的存储值）
        return [Unit(0, 5, 0), Unit(1, 5, q32(8, 10)), Unit(1, 5, -q32(8, 10))]
    if case == 3:
        # 蓝 militia x=1.8m（hp 直填 6）、红 militia x=1.0m、红 militia x=0
        return [Unit(1, 5, q32(18, 10), hp=6), Unit(0, 5, q32(10, 10)), Unit(0, 5, 0)]
    if case == 4:
        # 真等距构造（链式贴身全静止）：红0 x=0、蓝A +0.8m、蓝B -0.8m、蓝C -1.6m、红D -2.4m。
        # 注：-1.6m/-2.4m 取 0.8m 存储值（4*ONE/5=3435973836）的精确整数倍（-2x/-3x），
        # 保证各链节间距恰为 3435973836、gap=0 全静止；floor(-8*2^32/5) 会差 1 个 Q 单位破坏静止。
        step = q32(4, 5)
        return [Unit(0, 5, 0), Unit(1, 5, step), Unit(1, 5, -step),
                Unit(1, 5, -2 * step), Unit(0, 5, -3 * step)]
    raise ValueError("case must be 1, 2, 3 or 4")


CASES = {1: "red shieldman x=0.5m vs blue pikeman x=1.5m",
         2: "red militia x=0 vs blue militia x=+0.8m / x=-0.8m",
         3: "blue militia x=1.8m hp=6 vs red militia x=1.0m / x=0",
         4: "chain contact: red0 x=0 vs blue A +0.8m / B -0.8m / C -1.6m / red D -2.4m"}


# ---------------------------------------------------------------------------
# 自检（RNG 黄金序列 + FNV 向量 + 定点常数）
# ---------------------------------------------------------------------------

# Xoshiro256** seed 42 前 8 个输出（来源：sim/src/rng.rs 黄金断言，实测固化）
GOLDEN_SEED42_FIRST8 = [
    1546998764402558742, 6990951692964543102, 12544586762248559009, 17057574109182124193,
    18295552978065317476, 14199186830065750584, 13267978908934200754, 15679888225317814407,
]


def selftest():
    ok = True

    rng = Xoshiro256StarStar(42)
    got = [rng.next_u64() for _ in range(8)]
    if got == GOLDEN_SEED42_FIRST8:
        print("selftest rng: OK (seed 42 first 8 match rng.rs golden)")
    else:
        ok = False
        print("selftest rng: FAIL")
        for i, (g, e) in enumerate(zip(got, GOLDEN_SEED42_FIRST8)):
            if g != e:
                print(f"  [{i}] got={g} expected={e}")

    a_vec = fnv1a64(0xCBF29CE484222325, b"a")
    if a_vec == 0xAF63DC4C8601EC8C:
        print("selftest fnv: OK (FNV-1a64('a') = 0xaf63dc4c8601ec8c)")
    else:
        ok = False
        print(f"selftest fnv: FAIL got=0x{a_vec:016x} expected=0xaf63dc4c8601ec8c")

    checks = [
        ("speed", SPEED, [214748364, 858993459, 429496729, 515396075, 343597383, 386547056]),
        ("radius", RADIUS, [2147483648, 3435973836, 2147483648, 2147483648, 1717986918, 1717986918]),
    ]
    const_ok = DEPLOY_GAP == 2147483648 and MELEE_MARGIN == 858993459 and LANE_LEN == 4294967296000
    for name, got_v, exp_v in checks:
        if got_v != exp_v:
            const_ok = False
            print(f"selftest const {name}: FAIL got={got_v} expected={exp_v}")
    if COUNTER != [[65536, 98304, 43690], [43690, 65536, 98304], [98304, 43690, 65536]]:
        const_ok = False
        print("selftest const counter: FAIL")
    if const_ok:
        print("selftest const: OK (speed/radius/margin/gap/counter match dispatch)")
    else:
        ok = False

    print("selftest result:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def parse_comp(s):
    parts = [p for p in s.split(",")]
    if not s.strip():
        raise ValueError("empty --comp")
    comp = []
    for part in parts:
        if ":" not in part:
            raise ValueError(f"bad --comp entry: '{part}'")
        name, cnt = part.split(":", 1)
        name = name.strip()
        if name not in KIND_INDEX:
            raise ValueError(f"unknown kind id: '{name}'")
        try:
            n = int(cnt.strip(), 10)
        except ValueError:
            raise ValueError(f"bad count: '{cnt}'")
        if n <= 0:
            raise ValueError(f"count must be >= 1: '{part}'")
        comp.append((KIND_INDEX[name], n))
    return comp


def main(argv):
    ap = argparse.ArgumentParser(prog="recalc.py", description="T004 python independent recalc")
    ap.add_argument("--mode", choices=["legacy", "t004"], default="t004")
    ap.add_argument("--seed", type=lambda s: int(s, 0), default=42)
    ap.add_argument("--ticks", type=int, default=1800)
    ap.add_argument("--comp", default=None)
    ap.add_argument("--units", type=int, default=None)
    ap.add_argument("--case", type=int, choices=[1, 2, 3, 4], default=None)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args(argv)

    if args.selftest:
        return selftest()

    if args.ticks < 0:
        print("error: --ticks must be >= 0", file=sys.stderr)
        return 2
    if args.case is not None and (args.comp is not None or args.units is not None):
        print("error: --case is mutually exclusive with --comp/--units", file=sys.stderr)
        return 2
    if args.comp is not None and args.units is not None:
        print("error: --comp and --units are mutually exclusive", file=sys.stderr)
        return 2

    if args.case is not None:
        units = build_case(args.case)
    elif args.units is not None:
        if args.units < 0:
            print("error: --units must be >= 0", file=sys.stderr)
            return 2
        units = [Unit(0, 0, 0) for _ in range(args.units)]  # T002 缺省态占位：Shieldman/Red/x=0
    elif args.comp is not None:
        try:
            comp = parse_comp(args.comp)
        except ValueError as e:
            print(f"error: {e}", file=sys.stderr)
            return 2
        units = deploy(args.seed, comp)
    else:
        units = deploy(args.seed, DEFAULT_COMP)

    units, final_tick, h = run_sim(units, args.seed, args.ticks, args.mode)

    print(f"mode={args.mode}")
    if args.case is not None:
        print(f"case={args.case}")
    print(f"seed={args.seed}")
    print(f"ticks={args.ticks}")
    print(f"units={len(units)}")
    print(f"final_tick={final_tick}")
    print(f"hash=0x{h:016x}")
    if args.case is not None:
        for idx, u in enumerate(units):
            side = "red" if u.side == 0 else "blue"
            print(f"u{idx} {side} {KIND_NAMES[u.kind]} hp={u.hp} cd={u.cd} x={u.x}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
