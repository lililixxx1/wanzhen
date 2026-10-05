#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""T008 WP-A D2 构成映射重算（零第三方依赖，标准库 only）。

算法逐字口径：sim/src/bin/bench.rs:284-301 `composition_for`
    per_side = units / 2;
    q = per_side / 6; r = per_side % 6;
    表序（bench.rs:288-295）= Shieldman/HeavyKnight/Pikeman/Swordsman/Archer/Militia，
    按表序前 r 个兵种各 +1（comp = q + usize::from(i < r)）。
kind 小写 id 口径：UnitKind::id()（--comp 解析用，与 docs/evidence/t006/batch.sh 同款）。

产出：docs/evidence/t008/comp.txt（两串 + 断言结果）。
退出码非 0 = 任一断言失败（含与主会话速算参考值双盲比对不一致）→ 上报停止。
"""
import sys

# ---- D2 算法（bench.rs:284-301 逐字转写）----
KINDS = ["shieldman", "heavyknight", "pikeman", "swordsman", "archer", "militia"]


def composition_for(units: int):
    per_side = units // 2          # bench.rs:285  units / 2
    q = per_side // 6              # bench.rs:286  per_side / 6
    r = per_side % 6               # bench.rs:287  per_side % 6
    # bench.rs:296-300  comp = kinds.map(|(i, k)| (k, q + (i < r)))
    comp = [(k, q + (1 if i < r else 0)) for i, k in enumerate(KINDS)]
    return per_side, q, r, comp


def fmt(comp):
    return ",".join("%s:%d" % (k, c) for k, c in comp)


# ---- 主会话速算参考值（派工单 D2，仅双盲核对用；不一致即退出非 0）----
REF_RED = "shieldman:17,heavyknight:17,pikeman:17,swordsman:17,archer:16,militia:16"
REF_FULL = "shieldman:834,heavyknight:834,pikeman:833,swordsman:833,archer:833,militia:833"

failures = []
lines = []
lines.append("# T008 D2 构成映射生成记录（gen_comp.py 自动产出，2026-10-05）")
lines.append("#")
lines.append("# 算法逐字引用：sim/src/bin/bench.rs:284-301 composition_for")
lines.append("#   per_side = units / 2; q = per_side / 6; r = per_side % 6;")
lines.append("#   表序（bench.rs:288-295）= Shieldman/HeavyKnight/Pikeman/Swordsman/Archer/Militia，")
lines.append("#   按表序前 r 个兵种各 +1（comp = q + usize::from(i < r)）。")
lines.append("#")


def check(tag, units, expect_per_side, ref):
    per_side, q, r, comp = composition_for(units)
    s = fmt(comp)
    total = sum(c for _, c in comp)
    # 断言 1：每方总和 == per_side == units/2
    ok_total = (total == per_side == expect_per_side)
    # 断言 2：逐 kind 数值 ∈ {q, q+1}，且恰前 r 个为 q+1
    ok_kinds = all(c == q + (1 if i < r else 0) for i, (_, c) in enumerate(comp))
    # 断言 3：与主会话速算参考值逐字一致（双盲）
    ok_ref = (s == ref)
    lines.append("## %s（units=%d → per_side=%d, q=%d, r=%d）" % (tag, units, per_side, q, r))
    lines.append("comp=%s" % s)
    lines.append("assert_per_side_sum: sum=%d expect=%d -> %s"
                 % (total, expect_per_side, "PASS" if ok_total else "FAIL"))
    lines.append("assert_kind_q_or_q_plus_1: -> %s" % ("PASS" if ok_kinds else "FAIL"))
    lines.append("assert_blind_ref_match: -> %s" % ("PASS" if ok_ref else "FAIL"))
    lines.append("#")
    if not ok_total:
        failures.append("%s per-side sum" % tag)
    if not ok_kinds:
        failures.append("%s kind distribution" % tag)
    if not ok_ref:
        failures.append("%s blind ref mismatch: script=%r ref=%r" % (tag, s, ref))


check("red200 降规模", 200, 100, REF_RED)
check("full10000 全规模", 10000, 5000, REF_FULL)

if failures:
    lines.append("RESULT: FAIL（%s）——按派工单纪律上报停止，不得静默改数" % "; ".join(failures))
    out = "\n".join(lines) + "\n"
    with open("docs/evidence/t008/comp.txt", "w", encoding="utf-8", newline="\n") as f:
        f.write(out)
    print(out)
    print("gen_comp.py: FAIL", file=sys.stderr)
    sys.exit(1)

lines.append("RESULT: PASS（两串全部断言通过，与主会话速算参考值逐字一致）")
out = "\n".join(lines) + "\n"
with open("docs/evidence/t008/comp.txt", "w", encoding="utf-8", newline="\n") as f:
    f.write(out)
print(out)
print("gen_comp.py: PASS")
