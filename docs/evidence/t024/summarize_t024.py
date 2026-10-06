#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""T024 汇总脚本（观战档 10k 帧采集 → summary.md；判定行脚本生成，禁止手算）。

口径**逐字沿** `docs/evidence/t010/summarize.py`（T010 先例；派工单 D3）：

    avg_fps    = N / (Σ delta_ns / 1e9)
    1% low     = 1e9 / mean(最慢 k1 帧)    k1  = max(1, ⌈N/100⌉)  = (N+99)//100
    0.1% low   = 1e9 / mean(最慢 k01 帧)   k01 = max(1, ⌈N/1000⌉) = (N+999)//1000
    p50/p95/p99= 升序第 ⌈N·p/100⌉ 个（nearest-rank；rank=(N*p+99)//100，下限 1）
    max        = 最大帧时间
    vsync 旁证 = #{delta_ns < 0.95×16.7ms} > 0 ⇒「非 vsync 锁定旁证：有」，
                 否则披露「疑似 vsync 锁定」（不影响判定口径）

判定行（阈值常量注源 = 表 6-0 验收⑤ / T010 先例；T024 任务卡验收断言 1）：
    avg_fps ≥ 60 且 1% low ≥ 45 → **PASS**，否则 **TRIPPED**（如实，不粉饰）

输入 = 观战腿落档目录（`frames.csv`（首行 `idx,delta_ns`）+ `meta.json`）。
meta.json 字段为 T024 形态（units_total/tick_at_write 等，见 host/src/
frame_capture.rs 模块注）——判定只用 frames.csv 逐帧数据。

用法：
    python summarize_t024.py --dir <落档目录> --out <summary.md>
    python summarize_t024.py --selftest        # 构造样例与 t010 同式对齐输出

退出码：0 = 判定产出；2 = 缺 frames.csv；3 = 表头/数据异常；4 = 参数错误。
"""

import argparse
import csv
import json
import os
import sys

VSYNC_PERIOD_NS = 16_700_000  # 16.7 ms 口径（逐字沿 t010/summarize.py:30）
VSYNC_EVIDENCE_LT_NS = int(0.95 * VSYNC_PERIOD_NS)  # 15_865_000 ns
JUDGE_AVG_MIN = 60.0   # 表 6-0 验收⑤（t010/summarize.py:34 同值注源）
JUDGE_LOW1_MIN = 45.0  # 表 6-0 验收⑤（t010/summarize.py:35 同值注源）
EXPECTED_HEADER = ["idx", "delta_ns"]


def k_for(n: int, num: int, den: int) -> int:
    """k = max(1, ⌈n·num/den⌉) 的整数算式（逐字沿 t010/summarize.py:39-41）。"""
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


def selftest():
    # 构造样例逐字沿 t010/summarize.py:130-140（同输入同式 → 同输出）。
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
    ap.add_argument("--dir", help="落档目录（frames.csv + meta.json）")
    ap.add_argument("--out", help="summary.md 输出路径")
    ap.add_argument("--selftest", action="store_true", help="构造样例对齐输出（t010 同式）")
    args = ap.parse_args()

    if args.selftest:
        return selftest()
    if not args.dir or not args.out:
        ap.error("--dir 与 --out 必填（或用 --selftest）")

    frames_p = os.path.join(args.dir, "frames.csv")
    meta_p = os.path.join(args.dir, "meta.json")
    if not os.path.isfile(frames_p):
        print(f"缺 frames.csv: {frames_p}", file=sys.stderr)
        return 2
    try:
        deltas = load_frames(frames_p)
    except SystemExit as e:
        print(str(e), file=sys.stderr)
        return 3
    m = metrics(deltas)
    verdict = (
        "PASS"
        if (m["avg_fps"] >= JUDGE_AVG_MIN and m["low1"] >= JUDGE_LOW1_MIN)
        else "TRIPPED"
    )

    lines = []
    lines.append("# T024 观战档汇总（帧采集判定；脚本生成，判定行禁手改）")
    lines.append("")
    lines.append(f"- 输入目录: {args.dir}")
    lines.append(
        f"- 判定口径: 表 6-0 验收⑤（T010 先例）——avg_fps ≥ {JUDGE_AVG_MIN:.0f} "
        f"且 1% low ≥ {JUDGE_LOW1_MIN:.0f}；不达标如实 TRIPPED"
    )
    lines.append(f"- vsync 旁证阈值: delta_ns < {VSYNC_EVIDENCE_LT_NS}（0.95 × 16.7 ms）")
    lines.append("")
    lines.append("## 观战 10k 指标（窗口 = warmup 5 s 丢弃 + capture 65 s）")
    lines.append("")
    lines.append(
        "| 档位 | 帧数 N | avg_fps | 1% low | 0.1% low | p50 (ms) | p95 (ms) | p99 (ms) | max (ms) | 非vsync旁证帧数 | 判定 |"
    )
    lines.append("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    lines.append(
        f"| spectate-10k | {m['n']} | {m['avg_fps']:.2f} | {m['low1']:.2f} | {m['low01']:.2f} "
        f"| {m['p50'] / 1e6:.3f} | {m['p95'] / 1e6:.3f} | {m['p99'] / 1e6:.3f} "
        f"| {m['max'] / 1e6:.3f} | {m['non_vsync_frames']} | {verdict} |"
    )
    lines.append("")
    if m["non_vsync_frames"] > 0:
        lines.append(
            f"- 非 vsync 锁定旁证：有（{m['non_vsync_frames']} 帧 < 15.865 ms）"
        )
    else:
        lines.append(
            "- 疑似 vsync 锁定（无帧 < 15.865 ms；AutoNoVsync 不生效形态，如实披露，不影响判定口径）"
        )
    lines.append("")
    lines.append("## meta.json 摘录（落档现场读数）")
    lines.append("")
    if os.path.isfile(meta_p):
        with open(meta_p, "r", encoding="utf-8") as f:
            meta = json.load(f)
        for key in (
            "mode",
            "resolution",
            "window_resolution_actual",
            "present_mode",
            "warmup_s",
            "capture_s",
            "captured_frames",
            "units_total",
            "tick_at_write",
            "seed",
            "max_ticks",
        ):
            if key in meta:
                lines.append(f"- {key}: {meta[key]}")
    else:
        lines.append(f"- （缺 meta.json: {meta_p}）")
    lines.append("")
    lines.append("## 判定行（表 6-0 验收⑤ / T010 先例）")
    lines.append("")
    lines.append(
        f"- spectate-10k: avg_fps={m['avg_fps']:.2f} (≥60 ? {'是' if m['avg_fps'] >= JUDGE_AVG_MIN else '否'})、"
        f"1% low={m['low1']:.2f} (≥45 ? {'是' if m['low1'] >= JUDGE_LOW1_MIN else '否'}) → **{verdict}**"
    )
    if verdict == "TRIPPED":
        lines.append("")
        lines.append("- 未达承诺线：如实 TRIPPED 不粉饰（T024 任务卡验收断言 1）；性能优化轮在范围外，上报裁决。")

    with open(args.out, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines) + "\n")
    print(f"summary written: {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
