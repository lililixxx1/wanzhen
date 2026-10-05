#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""T009 汇总判定生成器（派工单 §3；判定行全部脚本计算，防手算漂移）。

用法：
    python summarize.py --out <t009 证据目录（含 runs/ 子目录）>

读取：
    <out>/runs/matrix_per100/matrix_per100.jsonl   口径层（per_side=100, lane 50m, cap 1800）
    <out>/runs/matrix_per10/matrix_per10.jsonl     探针层（per_side=10,  lane 50m, cap 1800）
    <out>/runs/sampling/sampling.jsonl             全规模层（per_side=5000, lane 1000m, cap 14400）
    <out>/runs/throughput_t{1,3,6,12}/throughput_t{T}.json
    <out>/runs/r5-arena/sampling.jsonl + runs/r5-sim-t1.stdout + runs/r5-sim-t12.stdout（R5 交叉）

写出：<out>/summary.md

判定口径（派工单 §3 逐条，参考阈值不放松、结果如实判定）：
  - 镜像 sanity：对角格蓝胜率 pooled 二项 z = (p-0.5)/sqrt(0.25/n)；|z|<=1.96 → 50/50 一致。
  - 吞吐：games_per_hour@12t（中位）>= 10_000 → PASS（R2 阈值）。
  - 接敌实证：口径层每格 avg(alive_red+alive_blue) < 2*per_side；探针层 < 20；
    全规模层每局 alive_red+alive_blue < 10000。
  - 击溃率（v0 定义）：有胜方局中「胜方存活率 >= 0.8*per_side」局占比（分母 = 有胜方局）。
  - 分层方向：cell 方向 = 有胜方局的多数胜方；无胜方局全 Draw → "draw"；
    红蓝打平（无多数）→ "mixed"。
"""

import argparse
import json
import math
import re
from pathlib import Path

KINDS = ["shieldman", "heavyknight", "pikeman", "swordsman", "archer", "militia"]
CELLS = 36
R2_GPH_THRESHOLD = 10_000.0


def load_jsonl(path: Path):
    rows = []
    with path.open("r", encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return rows


def pct(x: float) -> str:
    return f"{x * 100:.1f}%"


class Layer:
    def __init__(self, name: str, rows):
        self.name = name
        self.rows = rows
        self.per_side = rows[0]["per_side"] if rows else None
        self.cells = {}  # cell -> list[rows]
        for r in rows:
            self.cells.setdefault(r["cell"], []).append(r)

    def cell_stats(self, cell: int):
        rows = self.cells.get(cell, [])
        n = len(rows)
        if n == 0:
            return None
        red = sum(1 for r in rows if r["winner"] == "red")
        blue = sum(1 for r in rows if r["winner"] == "blue")
        draw = sum(1 for r in rows if r["winner"] == "draw")
        resolved = sum(1 for r in rows if r["resolved"])
        avg_end = sum(r["end_tick"] for r in rows) / n
        avg_alive = sum(r["alive_red"] + r["alive_blue"] for r in rows) / n
        return {
            "n": n, "red": red, "blue": blue, "draw": draw,
            "resolved": resolved, "avg_end": avg_end, "avg_alive": avg_alive,
        }

    def direction(self, cell: int) -> str:
        rows = self.cells.get(cell, [])
        red = sum(1 for r in rows if r["winner"] == "red")
        blue = sum(1 for r in rows if r["winner"] == "blue")
        if red == 0 and blue == 0:
            return "draw"
        if red > blue:
            return "red"
        if blue > red:
            return "blue"
        return "mixed"  # 红蓝胜局数打平（无多数）

    def rout_rate(self):
        """击溃率（v0）：有胜方局中 胜方存活 >= 0.8*per_side 局占比。"""
        decided = [r for r in self.rows if r["winner"] in ("red", "blue")]
        if not decided:
            return None, 0
        rout = sum(
            1
            for r in decided
            if (r["alive_red"] if r["winner"] == "red" else r["alive_blue"])
            >= 0.8 * r["per_side"]
        )
        return rout / len(decided), len(decided)


def matrix_table(out, layer: Layer, title: str):
    out.append(f"### {title}（per_side={layer.per_side}，lane={layer.rows[0]['lane_m'] if layer.rows else '-'}m）\n")
    out.append("| cell（红\\蓝） | 红兵种 | 蓝兵种 | n | 红胜% | 蓝胜% | Draw% | resolved% | avg_end_tick | avg 存活(和) |")
    out.append("|---|---|---|---:|---:|---:|---:|---:|---:|---:|")
    for i in range(6):
        for j in range(6):
            cell = i * 6 + j
            st = layer.cell_stats(cell)
            if st is None:
                out.append(f"| {cell} | {KINDS[i]} | {KINDS[j]} | 0 | - | - | - | - | - | - |")
                continue
            out.append(
                f"| {cell} | {KINDS[i]} | {KINDS[j]} | {st['n']} "
                f"| {pct(st['red'] / st['n'])} | {pct(st['blue'] / st['n'])} "
                f"| {pct(st['draw'] / st['n'])} | {pct(st['resolved'] / st['n'])} "
                f"| {st['avg_end']:.1f} | {st['avg_alive']:.2f} |"
            )
    out.append("")


def mirror_sanity(out, layer: Layer, tag: str):
    """派工单 §3-2：对角 6 格蓝胜率 pooled 二项 z 检验 + 95% CI。"""
    out.append(f"### 镜像 sanity（{tag}，对角 6 格：镜像局蓝胜率应 50/50）\n")
    out.append("| cell | 兵种 | n | 红胜 | 蓝胜 | Draw | p_blue | 95% CI | z | 判定 |")
    out.append("|---|---|---:|---:|---:|---:|---:|---|---:|---|")
    h1_ok = True
    for i in range(6):
        cell = i * 6 + i
        st = layer.cell_stats(cell)
        if st is None:
            out.append(f"| {cell} | {KINDS[i]} | 0 | - | - | - | - | - | - | 数据缺失 |")
            h1_ok = False
            continue
        n = st["n"]
        p = st["blue"] / n
        se = math.sqrt(0.25 / n)
        z = (p - 0.5) / se
        ci = 1.96 * math.sqrt(p * (1 - p) / n)
        verdict = (
            "50/50 一致"
            if abs(z) <= 1.96
            else "H2 结构性偏差形态"
        )
        if abs(z) > 1.96:
            h1_ok = False
        out.append(
            f"| {cell} | {KINDS[i]} | {n} | {st['red']} | {st['blue']} | {st['draw']} "
            f"| {p:.4f} | ±{ci:.4f} | {z:+.3f} | {verdict} |"
        )
        if abs(z) > 1.96:
            out.append(
                f"  - [H2 披露] cell={cell}（{KINDS[i]}）三态分布：红 {pct(st['red'] / n)} / "
                f"蓝 {pct(st['blue'] / n)} / Draw {pct(st['draw'] / n)}"
            )
    out.append(
        f"\n- [镜像 sanity 判定行·{tag}] "
        + ("全部对角格 |z| <= 1.96 → 与 50/50 一致（H1）" if h1_ok else "存在 |z| > 1.96 的对角格 → H2 结构性偏差形态（三态分布已逐格披露）")
    )
    out.append("")
    return h1_ok


def throughput_section(out, runs: Path):
    out.append("## 吞吐判定（派工单 §3-3；R2 阈值 games_per_hour@12t >= 10_000）\n")
    tiers = [1, 3, 6, 12]
    data = {}
    for t in tiers:
        p = runs / f"throughput_t{t}" / f"throughput_t{t}.json"
        if p.exists():
            data[t] = json.loads(p.read_text(encoding="utf-8"))
        else:
            out.append(f"- 缺数据：{p.as_posix()}")
    if 12 not in data:
        out.append("\n- [吞吐判定行] 数据缺失，无法判定\n")
        return None
    gph12 = data[12]["games_per_hour"]
    verdict = "PASS" if gph12 >= R2_GPH_THRESHOLD else "FAIL"
    out.append("| threads | games | wall_s(中位) | games_per_hour | per_game_ms |")
    out.append("|---:|---:|---:|---:|---:|")
    for t in tiers:
        if t in data:
            d = data[t]
            out.append(
                f"| {t} | {d['games']} | {d['wall_s_median']:.4f} "
                f"| {d['games_per_hour']:.1f} | {d['per_game_ms']:.3f} |"
            )
    out.append(f"\n- [吞吐判定行] games_per_hour@12t = {gph12:.1f} （阈值 >= {R2_GPH_THRESHOLD:.0f}）→ {verdict}")

    # 16 线程外推：elapsed(T) = c1 + c2/T，四档中位双参数最小二乘（2x2 正规方程）。
    pts = [(float(t), data[t]["wall_s_median"]) for t in tiers if t in data]
    if len(pts) == 4:
        n = 4.0
        sx = sum(1.0 / t for t, _ in pts)
        sx2 = sum((1.0 / t) ** 2 for t, _ in pts)
        sy = sum(y for _, y in pts)
        sxy = sum((1.0 / t) * y for t, y in pts)
        det = n * sx2 - sx * sx
        if abs(det) < 1e-12 * max(n * sx2, 1.0):
            out.append("- [16 线程外推] 拟合退化（行列式≈0），判定不可得\n")
        else:
            c1 = (sy * sx2 - sx * sxy) / det
            c2 = (n * sxy - sx * sy) / det
            e16 = c1 + c2 / 16.0
            g16 = data[12]["games"] * 3600.0 / e16
            out.append(
                f"- [16 线程外推] elapsed(T)=c1+c2/T OLS（T∈{{1,3,6,12}} 中位）："
                f"c1={c1:.4f}s c2={c2:.4f}s·T → elapsed(16)={e16:.4f}s → 吞吐16 = {g16:.1f} 场/h"
            )
            # 残差留痕
            res = ", ".join(f"T={t:.0f}:{(c1 + c2 / t - y):+.4f}" for t, y in pts)
            out.append(f"  - 拟合残差（s）：{res}")
    out.append("")
    return gph12


def engagement_section(out, layers: dict):
    out.append("## 接敌实证判定（派工单 §3-4）\n")
    specs = [
        ("matrix_per100", "口径层", 200.0, "cell"),
        ("matrix_per10", "探针层", 20.0, "cell"),
    ]
    all_ok = True
    for key, cname, thresh, _ in specs:
        layer = layers.get(key)
        if layer is None or not layer.rows:
            out.append(f"- [{cname}] 数据缺失，无法判定")
            all_ok = False
            continue
        bad = []
        for cell in range(CELLS):
            st = layer.cell_stats(cell)
            if st is None or not (st["avg_alive"] < thresh):
                bad.append(cell)
        ok = not bad
        all_ok = all_ok and ok
        worst = max(
            (layer.cell_stats(c)["avg_alive"] for c in range(CELLS) if layer.cell_stats(c)),
            default=float("nan"),
        )
        out.append(
            f"- [接敌实证判定行·{cname}] 36 格全部 avg(alive_red+alive_blue) < {thresh:.0f}"
            f"（2*per_side）→ {'PASS' if ok else 'FAIL（格 ' + ','.join(map(str, bad)) + '）'}"
            f"；全场最大 avg 存活(和) = {worst:.2f}"
        )
    full = layers.get("sampling")
    if full is None or not full.rows:
        out.append("- [全规模层] 数据缺失，无法判定")
        all_ok = False
    else:
        bad = [r["seed"] for r in full.rows if not (r["alive_red"] + r["alive_blue"] < 10000)]
        ok = not bad
        all_ok = all_ok and ok
        mx = max(r["alive_red"] + r["alive_blue"] for r in full.rows)
        out.append(
            f"- [接敌实证判定行·全规模层] {len(full.rows)} 局全部 alive_red+alive_blue < 10000"
            f" → {'PASS' if ok else 'FAIL（seed ' + ','.join(map(str, bad)) + '）'}"
            f"；全场最大存活(和) = {mx}"
        )
    out.append("")
    return all_ok


def layered_section(out, layers: dict):
    out.append("## 分层比对（派工单 §3-5）\n")
    agree = 0
    comparable = 0
    rows_out = []
    for cell in range(CELLS):
        d100 = layers["matrix_per100"].direction(cell) if "matrix_per100" in layers else "?"
        d10 = layers["matrix_per10"].direction(cell) if "matrix_per10" in layers else "?"
        dfull = layers["sampling"].direction(cell) if "sampling" in layers else "?"
        nfull = len(layers["sampling"].cells.get(cell, [])) if "sampling" in layers else 0
        consistent = d100 == d10 == dfull and "?" not in (d100, d10, dfull)
        if "?" not in (d100, d10, dfull):
            comparable += 1
            if consistent:
                agree += 1
        note = f"小样本 n={nfull}" if nfull <= 3 else ""
        rows_out.append(f"| {cell} | {KINDS[cell // 6]} | {KINDS[cell % 6]} | {d100} | {d10} | {dfull}{'（' + note + '）' if note else ''} | {'一致' if consistent else '不一致'} |")
    out.append("### 逐 cell 胜方方向（多数胜方；无多数 = mixed；全 Draw = draw）\n")
    out.append("| cell | 红 | 蓝 | 口径层 | 探针层 | 全规模层 | 三层一致 |")
    out.append("|---:|---|---|---|---|---|---|")
    out.extend(rows_out)
    out.append(
        f"\n- [分层方向一致判定行] {agree}/{comparable} cell 三层方向一致"
        f"（比例 {agree / comparable * 100:.1f}%）\n" if comparable else ""
    )

    out.append("### 击溃率（v0：有胜方局中 胜方存活 >= 0.8*per_side 局占比）与 avg_end_tick\n")
    out.append("| 层 | 击溃率 | 有胜方局数 | avg_end_tick |")
    out.append("|---|---:|---:|---:|")
    for key, cname in [("matrix_per100", "口径层"), ("matrix_per10", "探针层"), ("sampling", "全规模层")]:
        layer = layers.get(key)
        if layer is None or not layer.rows:
            out.append(f"| {cname} | 数据缺失 | - | - |")
            continue
        rate, decided = layer.rout_rate()
        avg_end = sum(r["end_tick"] for r in layer.rows) / len(layer.rows)
        out.append(
            f"| {cname} | {pct(rate) if rate is not None else '-'} | {decided} | {avg_end:.1f} |"
        )
    out.append("")


def crosscheck_section(out, runs: Path):
    out.append("## CLI 黄金交叉（派工单 §3-6 / R5）\n")
    arena_p = runs / "r5-arena" / "sampling.jsonl"
    t1_p = runs / "r5-sim-t1.stdout"
    t12_p = runs / "r5-sim-t12.stdout"
    vals = {}
    if arena_p.exists():
        rows = load_jsonl(arena_p)
        if rows:
            vals["arena"] = rows[0]["final_hash"]
    for tag, p in [("sim_t1", t1_p), ("sim_t12", t12_p)]:
        if p.exists():
            m = re.search(r"final_hash=(0x[0-9a-f]+)", p.read_text(encoding="utf-8"))
            if m:
                vals[tag] = m.group(1)
    if len(vals) == 3:
        h = vals["arena"]
        ok = h == vals["sim_t1"] == vals["sim_t12"]
        out.append(f"- arena={h}")
        out.append(f"- sim_t1={vals['sim_t1']}")
        out.append(f"- sim_t12={vals['sim_t12']}")
        out.append(f"- [CLI 交叉判定行] 三方 final_hash 逐位一致 → {'PASS' if ok else 'FAIL'}")
    else:
        out.append(f"- [CLI 交叉判定行] 数据缺失（取到 {sorted(vals)}），无法判定")
    out.append("")


def preregistration_section(out, layers: dict, gph12, runs: Path):
    """派工单[时间盒·预期登记]对照（预注册 → 实测，如实披露、禁止事后改预期）。"""
    out.append("## 预注册预期对照（派工单登记；仅供披露，判定线见上各节）\n")
    if gph12 is not None:
        out.append(
            f"- 吞吐@12t 预期 >= 100_000 场/h 量级（判定线 10_000）→ 实测 {gph12:.1f} 场/h"
        )
    sp = runs / "sampling"
    full = layers.get("sampling")
    if full and full.rows:
        # 单局均值从 arena stdout 摘要取不到时以 JSONL 行数与 runs 文件为准：
        # 这里用 wall_total/局数不可得（JSONL 无 wall），改由 stdout 摘要档读取。
        stdout_p = runs / "r6-sampling.stdout"
        m = None
        if stdout_p.exists():
            m = re.search(r'"wall_s_median":([0-9.eE+-]+)', stdout_p.read_text(encoding="utf-8"))
        if m:
            med = float(m.group(1))
            lo, hi = 11.6 - 11.6, 11.6 + 11.6  # 偏离 ±100% → [0, 23.2]
            out.append(
                f"- 采样单局 ≈11.6s（T015 同规模同档参考）→ 实测中位 {med:.2f}s"
                f"（偏离 ±100% 区间 [{lo:.1f}, {hi:.1f}] → {'区间内' if lo <= med <= hi else '偏离区间（上报项）'}）"
            )
        else:
            out.append("- 采样单局参考对照：runs/r6-sampling.stdout 无 wall_s_median，跳过")
        n_resolved = sum(1 for r in full.rows if r["resolved"])
        out.append(
            f"- 口径层多数格 resolved=false（预注册设计预期）→ 实测口径层 resolved 局数 "
            f"{sum(1 for r in layers['matrix_per100'].rows if r['resolved'])}/3600"
            + (f"；全规模层 resolved {n_resolved}/{len(full.rows)}" if full.rows else "")
        )
    out.append("")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, help="t009 证据目录（含 runs/）")
    args = ap.parse_args()
    root = Path(args.out)
    runs = root / "runs"

    layers = {}
    for key, sub in [("matrix_per100", "matrix_per100/matrix_per100.jsonl"),
                     ("matrix_per10", "matrix_per10/matrix_per10.jsonl"),
                     ("sampling", "sampling/sampling.jsonl")]:
        p = runs / sub
        layers[key] = Layer(key, load_jsonl(p)) if p.exists() else Layer(key, [])

    out = []
    out.append("# T009 汇总判定（summarize.py 自动生成，禁止手改）\n")
    out.append("- 数据源：runs/ 下 matrix_per100 / matrix_per10 / sampling JSONL + throughput_t{1,3,6,12}.json + R5 交叉原始档")
    out.append("- 判定行全部由本脚本计算（防手算漂移）；阈值逐字取自派工单 §3（R2 阈值 10_000 场/h；|z|<=1.96；接敌实证阈值 2*per_side / 20 / 10000）\n")

    # 1. 矩阵表（两层）
    out.append("## 胜率矩阵（派工单 §3-1）\n")
    for key, cname in [("matrix_per100", "口径层 matrix_per100"), ("matrix_per10", "探针层 matrix_per10")]:
        layer = layers[key]
        if layer.rows:
            matrix_table(out, layer, cname)
        else:
            out.append(f"### {cname}：数据缺失\n")
    full = layers["sampling"]
    if full.rows:
        out.append(f"### 全规模层 sampling（per_side=5000，lane=1000m，{len(full.rows)} 局）逐局另见 sampling.jsonl；按 cell 汇总：\n")
        out.append("| cell | 红 | 蓝 | n | 红胜 | 蓝胜 | Draw | resolved | avg_end_tick | avg 存活(和) |")
        out.append("|---:|---|---|---:|---:|---:|---:|---:|---:|---:|")
        for cell in range(CELLS):
            st = full.cell_stats(cell)
            if st is None:
                continue
            out.append(
                f"| {cell} | {KINDS[cell // 6]} | {KINDS[cell % 6]} | {st['n']} "
                f"| {st['red']} | {st['blue']} | {st['draw']} | {st['resolved']} "
                f"| {st['avg_end']:.1f} | {st['avg_alive']:.1f} |"
            )
        out.append("")

    # 2. 镜像 sanity
    out.append("## 镜像 sanity（派工单 §3-2）\n")
    for key, tag in [("matrix_per100", "口径层"), ("matrix_per10", "探针层")]:
        if layers[key].rows:
            mirror_sanity(out, layers[key], tag)

    # 3. 吞吐
    gph12 = throughput_section(out, runs)

    # 4. 接敌实证
    engagement_section(out, layers)

    # 5. 分层比对
    layered_section(out, layers)

    # 6. CLI 交叉
    crosscheck_section(out, runs)

    # 预注册对照
    preregistration_section(out, layers, gph12, runs)

    out.append("\n（end；由 summarize.py 从 runs/ 原始档复算生成）")
    (root / "summary.md").write_text("\n".join(out), encoding="utf-8")
    print(f"summary written: {(root / 'summary.md').as_posix()}")


if __name__ == "__main__":
    main()
