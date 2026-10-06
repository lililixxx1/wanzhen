#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""T024 C-1 漂移表（加样披露用；附录 G「复测超标即加样复测、全档披露」+ T012 先例）。

- 归档基准（常数，源 = docs/evidence/t010/summary.md 四档表 t10000 行，逐字）：
    N=21656  avg_fps=333.15  1% low=202.48
- 复测样本（本档，判定行已由脚本生成于各 summary；此处独立重算指标做漂移表）：
    r1（主样本）runs/c1-render-spike/r1/out/t10000/frames.csv
    r2（加样 #1）runs/c1-supp-r2/r2/out/t10000/frames.csv
    r3（加样 #2）runs/c1-supp-r3/r3/out/t10000/frames.csv
- 漂移 = 样本/归档 − 1；±25% 带 = T012 报告（docs/evidence/m0/review/report.md:251）口径。

用法：python drift_c1.py        （从本档目录运行；stdout = markdown 表）
"""

import csv

ARCHIVE = {"n": 21656, "avg_fps": 333.15, "low1": 202.48}  # t010/summary.md L14/L31 逐字
SAMPLES = [
    ("r1 (主样本)", "runs/c1-render-spike/r1/out/t10000/frames.csv"),
    ("r2 (加样 #1)", "runs/c1-supp-r2/r2/out/t10000/frames.csv"),
    ("r3 (加样 #2)", "runs/c1-supp-r3/r3/out/t10000/frames.csv"),
]


def avg_fps(deltas):
    return len(deltas) / (sum(deltas) / 1e9)


def low_fps(deltas, num, den):
    n = len(deltas)
    k = max(1, (n * num + den - 1) // den)
    slowest = sorted(deltas)[-k:]
    return 1e9 / (sum(slowest) / k)


def load(path):
    with open(path, "r", newline="", encoding="utf-8") as f:
        reader = csv.reader(f)
        header = next(reader)
        assert [c.strip() for c in header] == ["idx", "delta_ns"], header
        return [int(row[1]) for row in reader]


def main():
    print("| 样本 | N | avg_fps | avg 漂移 vs 归档 | 1% low | 1% low 漂移 vs 归档 | ±25% 带 |")
    print("| --- | --- | --- | --- | --- | --- | --- |")
    print(
        f"| 归档（t010/summary.md） | {ARCHIVE['n']} | {ARCHIVE['avg_fps']:.2f} | — | "
        f"{ARCHIVE['low1']:.2f} | — | — |"
    )
    for name, path in SAMPLES:
        d = load(path)
        a = avg_fps(d)
        l1 = low_fps(d, 1, 100)
        da = (a / ARCHIVE["avg_fps"] - 1) * 100
        dl = (l1 / ARCHIVE["low1"] - 1) * 100
        band = "带内" if abs(da) <= 25 else "超带"
        bandl = "带内" if abs(dl) <= 25 else ("超带（向上）" if dl > 0 else "超带（向下）")
        print(
            f"| {name} | {len(d)} | {a:.2f} | {da:+.2f}%（{band}） | {l1:.2f} | "
            f"{dl:+.2f}%（{bandl}） | ±25% |"
        )


if __name__ == "__main__":
    main()
