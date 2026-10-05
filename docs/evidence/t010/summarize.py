#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""T010 汇总脚本：四档帧数据 → summary.md（判定行脚本计算，禁止手算）。

口径 = 派工单 §5 预注册公式；与 render-spike/src/stats.rs 同式
（k / rank 用整数算式，避免浮点 ceil 误差）：

    avg_fps    = N / (Σ delta_ns / 1e9)
    1% low     = 1e9 / mean(最慢 k1 帧)    k1  = max(1, ⌈N/100⌉)  = (N+99)//100
    0.1% low   = 1e9 / mean(最慢 k01 帧)   k01 = max(1, ⌈N/1000⌉) = (N+999)//1000
    p50/p95/p99= 升序第 ⌈N·p/100⌉ 个（nearest-rank；rank=(N*p+99)//100，下限 1）
    max        = 最大帧时间
    vsync 旁证 = #{delta_ns < 0.95×16.7ms} > 0 ⇒「非 vsync 锁定旁证：有」，
                 否则披露「疑似 vsync 锁定」（不影响判定口径）
    判定行     = t10000: avg_fps ≥ 60 且 1% low ≥ 45 → PASS/FAIL（如实）

用法：
    python summarize.py --root <runs 目录> --out <summary.md>
    python summarize.py --selftest        # W4 构造样例跨语言对齐输出

退出码：0 = 四档齐全且已出判定；2 = 未找到任何档；3 = 缺档（仍写出部分 summary）。
"""

import argparse
import csv
import json
import os
import sys

VSYNC_PERIOD_NS = 16_700_000  # 16.7 ms 口径（派工单 §5）
VSYNC_EVIDENCE_LT_NS = int(0.95 * VSYNC_PERIOD_NS)  # 15_865_000 ns
REQUIRED_TIERS = [1000, 2000, 5000, 10000]
JUDGE_TIER = 10000
JUDGE_AVG_MIN = 60.0
JUDGE_LOW1_MIN = 45.0
EXPECTED_HEADER = ["idx", "delta_ns"]


def k_for(n: int, num: int, den: int) -> int:
    """k = max(1, ⌈n·num/den⌉) 的整数算式。"""
    return max(1, (n * num + den - 1) // den)


def avg_fps(deltas):
    return len(deltas) / (sum(deltas) / 1e9)


def low_fps(deltas, num: int, den: int):
    n = len(deltas)
    k = k_for(n, num, den)
    slowest = sorted(deltas)[-k:]
    mean = sum(slowest) / k
    return 1e9 / mean


def percentile(deltas, p: int) -> int:
    n = len(deltas)
    rank = max(1, (n * p + 99) // 100)
    return sorted(deltas)[rank - 1]


def load_frames(path: str):
    with open(path, "r", newline="", encoding="utf-8") as f:
        reader = csv.reader(f)
        try:
            header = next(reader)
        except StopIteration:
            raise SystemExit(f"{path}: 空文件")
        if [c.strip() for c in header] != EXPECTED_HEADER:
            raise SystemExit(f"{path}: 表头异常 {header}（期望 {EXPECTED_HEADER}）")
        deltas = []
        for row in reader:
            if len(row) != 2 or row[0].strip() == "":
                raise SystemExit(f"{path}: 行格式异常 {row}")
            deltas.append(int(row[1]))
    if not deltas:
        raise SystemExit(f"{path}: 无帧数据")
    return deltas


def collect(root: str):
    """扫描 root/*/out/t*/meta.json（+frames.csv）；返回 {units: record} 与重复告警。"""
    found = {}
    dup_notes = []
    for run_dir in sorted(os.listdir(root)):
        outp = os.path.join(root, run_dir, "out")
        if not os.path.isdir(outp):
            continue
        for tdir in sorted(os.listdir(outp)):
            if not tdir.startswith("t"):
                continue
            meta_p = os.path.join(outp, tdir, "meta.json")
            frames_p = os.path.join(outp, tdir, "frames.csv")
            if not (os.path.isfile(meta_p) and os.path.isfile(frames_p)):
                continue
            with open(meta_p, "r", encoding="utf-8") as f:
                meta = json.load(f)
            units = int(meta["units"])
            rel = os.path.join(root, run_dir, "out", tdir).replace("\\", "/")
            rec = {"meta": meta, "frames_path": frames_p, "rel": rel}
            if units in found:
                dup_notes.append(f"{units}: {found[units]['rel']} 与 {rel} 重复（后列计入）")
            found[units] = rec
    return found, dup_notes


def metrics(deltas):
    n = len(deltas)
    return {
        "n": n,
        "avg_fps": avg_fps(deltas),
        "low1": low_fps(deltas, 1, 100),
        "low01": low_fps(deltas, 1, 1000),
        "p50": percentile(deltas, 50),
        "p95": percentile(deltas, 95),
        "p99": percentile(deltas, 99),
        "max": max(deltas),
        "non_vsync_frames": sum(1 for d in deltas if d < VSYNC_EVIDENCE_LT_NS),
    }


def fmt_row(units, m, verdict=""):
    return (
        f"| t{units} | {m['n']} | {m['avg_fps']:.2f} | {m['low1']:.2f} | {m['low01']:.2f} "
        f"| {m['p50'] / 1e6:.3f} | {m['p95'] / 1e6:.3f} | {m['p99'] / 1e6:.3f} "
        f"| {m['max'] / 1e6:.3f} | {m['non_vsync_frames']} | {verdict} |"
    )


def selftest():
    const_ms = [8, 8, 8, 16, 32, 64]
    deltas = [v * 1_000_000 for v in const_ms]
    print(f"selftest sample={const_ms} ms (N={len(deltas)})")
    print(f"avg_fps={repr(avg_fps(deltas))}")
    print(f"1%low={repr(low_fps(deltas, 1, 100))}")
    print(f"0.1%low={repr(low_fps(deltas, 1, 1000))}")
    for p in (50, 95, 99):
        print(f"p{p}_ms={repr(percentile(deltas, p) / 1e6)}")
    print(f"max_ms={repr(max(deltas) / 1e6)}")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", help="runs 目录（扫描其下 */out/t*/）")
    ap.add_argument("--out", help="summary.md 输出路径")
    ap.add_argument("--selftest", action="store_true", help="W4 构造样例跨语言对齐输出")
    args = ap.parse_args()

    if args.selftest:
        return selftest()
    if not args.root or not args.out:
        ap.error("--root 与 --out 必填（或用 --selftest）")

    found, dup_notes = collect(args.root)
    if not found:
        print(f"未在 {args.root}/*/out/t* 找到任何档位", file=sys.stderr)
        return 2

    lines = []
    lines.append("# T010 汇总（渲染 spike 四档帧率；脚本生成，判定行禁手改）")
    lines.append("")
    lines.append(f"- 输入根目录: {args.root}")
    lines.append(f"- 判定口径: 表 6-0 验收⑤——t10000 avg_fps ≥ {JUDGE_AVG_MIN:.0f} 且 1% low ≥ {JUDGE_LOW1_MIN:.0f}")
    lines.append(f"- vsync 旁证阈值: delta_ns < {VSYNC_EVIDENCE_LT_NS}（0.95 × 16.7 ms）")
    lines.append("")
    if dup_notes:
        lines.append("## 重复档位告警")
        for n in dup_notes:
            lines.append(f"- {n}")
        lines.append("")

    lines.append("## 四档表（降档扫描 1000/2000/5000/10000）")
    lines.append("")
    lines.append(
        "| 档位 | 帧数 N | avg_fps | 1% low | 0.1% low | p50 (ms) | p95 (ms) | p99 (ms) | max (ms) | 非vsync旁证帧数 | 判定 |"
    )
    lines.append("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")

    missing = [u for u in REQUIRED_TIERS if u not in found]
    tier_results = {}
    for units in REQUIRED_TIERS:
        if units not in found:
            lines.append(f"| t{units} | - | - | - | - | - | - | - | - | - | 缺档 |")
            continue
        rec = found[units]
        m = metrics(load_frames(rec["frames_path"]))
        tier_results[units] = m
        verdict = ""
        if units == JUDGE_TIER:
            verdict = (
                "PASS"
                if (m["avg_fps"] >= JUDGE_AVG_MIN and m["low1"] >= JUDGE_LOW1_MIN)
                else "FAIL"
            )
        lines.append(fmt_row(units, m, verdict))
    lines.append("")

    extras = sorted(u for u in found if u not in REQUIRED_TIERS)
    if extras:
        lines.append("## 非判定档（冒烟等，参考）")
        lines.append("")
        lines.append("| 档位 | 帧数 N | avg_fps | 1% low | 0.1% low | p50 (ms) | p95 (ms) | p99 (ms) | max (ms) | 非vsync旁证帧数 | 判定 |")
        lines.append("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
        for units in extras:
            rec = found[units]
            m = metrics(load_frames(rec["frames_path"]))
            lines.append(fmt_row(units, m))
        lines.append("")

    lines.append("## vsync 旁证行")
    lines.append("")
    for units in REQUIRED_TIERS:
        if units not in tier_results:
            continue
        m = tier_results[units]
        if m["non_vsync_frames"] > 0:
            lines.append(
                f"- t{units}: 非 vsync 锁定旁证：有（{m['non_vsync_frames']} 帧 < 15.865 ms）"
            )
        else:
            lines.append(
                f"- t{units}: 疑似 vsync 锁定（无帧 < 15.865 ms；AutoNoVsync 不生效形态，如实披露，不影响判定口径）"
            )
    lines.append("")

    lines.append("## 判定行（表 6-0 验收⑤）")
    lines.append("")
    if JUDGE_TIER in tier_results:
        m = tier_results[JUDGE_TIER]
        verdict = "PASS" if (m["avg_fps"] >= JUDGE_AVG_MIN and m["low1"] >= JUDGE_LOW1_MIN) else "FAIL"
        lines.append(
            f"- t10000: avg_fps={m['avg_fps']:.2f} (≥60 ? {'是' if m['avg_fps'] >= JUDGE_AVG_MIN else '否'})、"
            f"1% low={m['low1']:.2f} (≥45 ? {'是' if m['low1'] >= JUDGE_LOW1_MIN else '否'}) → **{verdict}**"
        )
        if verdict == "FAIL":
            lines.append("")
            lines.append("## 渲染腿后果记录行（报告 6.1 原文语义，仅 FAIL 输出）")
            lines.append("")
            lines.append(
                "- 分层渲染提前进 M1 关键路径；常态规模承诺减半。"
                "本卡只产数据与判定，报告级修订归 T013；不粉饰、不重判。"
            )
    else:
        lines.append("- t10000 缺档，判定不可（exit 3）。")

    if missing:
        lines.append("")
        lines.append(f"- 缺档告警：{missing} → exit 3")

    with open(args.out, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    print(f"summary written: {args.out}")
    return 3 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
