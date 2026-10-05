#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""T008 对拍分析器：同种子重放确定性验证（验收④）对拍矩阵判定。

用法：
    python docs/evidence/t008/compare_matrix.py [runs_dir] [out_md]

  - runs_dir 默认 = 本脚本同级 runs/ 目录（docs/evidence/t008/runs）
  - out_md   默认 = 本脚本同级 matrix.md（docs/evidence/t008/matrix.md）；
    第二参数仅用于夹具自测重定向输出（target/t008-fixtures/...），不改变任何判定语义
  - 报告正文同时逐字打印到 stdout；诊断信息走 stderr
  - 零第三方依赖（仅 Python 标准库）

退出码（与 sim/src/bin/bench.rs 退出码风格一致）：
    0 全 PASS
    1 I/O 错（runs 目录 / T006 锚参照档 / 产物文件不可读写）
    2 输入格式错（stdout / exits.txt 不符契约、非预期文件、run_id 与内容矛盾）
    3 缺局或未完成（缺 stdout、缺 exit 行、REAL_EXIT≠0、标准采样列缺失、判定数据不足）
    5 mismatch（哈希不一致 / 锚局逐字节 diff 不一致 / 种子互异失效）
    优先级：1 > 2 > 5 > 3 > 0（1/2 表示输入本身不可信，先修数据；5 指已判定出的确定性失败；
    3 指证据不全尚不能判定）

判定语义（不可调，逐条对应派工单 WP-B D1 与任务卡 taskset/t008-replay-hash.md）：
    ① 齐全性：24 矩阵局 + 3 加样局 + 2 锚局齐备且每局 REAL_EXIT=0
    ② 断言 1：每 (scale,seed) 组合内 4 线程档 × 全部采样列 + 终局列逐位一致
    ③ 断言 2：每 scale 标准采样列组内一致
       （red200: 0/450/900/1350；full10000: 0/3600/7200/10800/12600）
    ④ 断言 3：任何 mismatch 全量列出不一致值（禁止折叠/省略）→ exit 5
    ⑤ 加样局 -r2/-r3 与基准局逐列一致（跨进程同配置）
    ⑥ 锚局 stdout 与 T006 归档逐字节 diff（期望现场读取，禁止手抄）
    ⑦ 种子互异 sanity：同 (scale,threads) 下 3 种子终局哈希两两不同
    ⑧ 覆盖披露（披露非判定）：每局 units=终局存活数；full 规模 units<10000 为战斗段已进入哈希的证据

解析契约（与 WP-A D4 同一份，出处 sim/src/main.rs:316/326-330）：
    run_id：<scale>-th<T>-s<SEED>（scale ∈ red200|full10000；加样后缀 -r2/-r3）；
            anchor1-t006-fullscale / anchor2-t006-10k300
    <run_id>.stdout：sample 行 "sample=<tick> hash=0x<16hex>"，摘要行
                     seed= / ticks= / units= / final_tick= / hash=0x<16hex>（顺序固定）
    exits.txt：<run_id> REAL_EXIT=<code> wall_ms=<ms>
    哈希值/判定全部由本脚本解析计算；报告不含手抄哈希值、不含机器绝对路径。
"""

import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

EXIT_OK = 0
EXIT_IO = 1
EXIT_FORMAT = 2
EXIT_INCOMPLETE = 3
EXIT_MISMATCH = 5

SCALES = ("red200", "full10000")
THREADS = (1, 3, 6, 12)
SEEDS = (42, 43, 44)
STANDARD_SAMPLES = {
    "red200": (0, 450, 900, 1350),
    "full10000": (0, 3600, 7200, 10800, 12600),
}
SPIKE_RUN_IDS = ("red200-th12-s42-r2", "red200-th12-s42-r3", "full10000-th12-s42-r2")
ANCHOR_REFERENCES = {
    "anchor1-t006-fullscale": "docs/evidence/t006/runs/fullscale.stdout",
    "anchor2-t006-10k300": "docs/evidence/t006/runs/10k-s42-t12.stdout",
}

RUN_ID_RE = re.compile(r"^(red200|full10000)-th(\d+)-s(\d+)(?:-r([23]))?$")
SAMPLE_RE = re.compile(r"^sample=(\d+) hash=0x([0-9a-f]{16})$")
KEY_RE = re.compile(r"^(seed|ticks|units|final_tick)=(\d+)$")
HASH_RE = re.compile(r"^hash=0x([0-9a-f]{16})$")
EXITS_RE = re.compile(r"^(\S+) REAL_EXIT=(\d+) wall_ms=(\d+)$")
KEY_ORDER = ("seed", "ticks", "units", "final_tick")

ANCHOR_DIFF_LIST_CAP = 512  # 逐字节差异列出上限（超出时如实给出剩余计数，不静默省略）


class FatalIO(Exception):
    """不可继续的 I/O 错误（runs 目录 / 锚参照档不可读）。"""


@dataclass
class RunData:
    samples: dict = field(default_factory=dict)  # tick(int) -> "0x<16hex>"
    seed: object = None
    ticks: object = None
    units: object = None
    final_tick: object = None
    final_hash: object = None


@dataclass
class RunRecord:
    run_id: str
    kind: str  # matrix / spike / anchor
    meta: object = None
    exit_code: object = None  # None = exits.txt 无该局行
    wall_ms: object = None
    stdout_path: object = None  # Path 或 None
    status: str = "missing"  # ok / failed / missing / parse_error
    data: object = None  # RunData（ok；failed 为尽力解析结果）
    errors: list = field(default_factory=list)


@dataclass
class ColumnVerdict:
    tick: object  # int；None = 终局列
    label: str
    verdict: str  # PASS / FAIL / INCOMP
    values: dict  # run_id -> 值或 None（缺列/缺局）
    note: str = ""


@dataclass
class GroupAnalysis:
    scale: str
    seed: int
    records: list
    columns: list  # [(tick, label)] 含终局列（tick=None）
    verdicts: list  # [ColumnVerdict]
    overall: str  # PASS / FAIL / INCOMP
    ok_count: int


# ---------------------------------------------------------------- 基础工具


def eprint(*args):
    print(*args, file=sys.stderr)


def rel_display(path, root):
    """输出用路径：仓库内相对路径；仓库外退化为系统相对路径（禁止机器绝对路径进入产出）。"""
    p = Path(path)
    try:
        return p.resolve().relative_to(Path(root).resolve()).as_posix()
    except ValueError:
        try:
            return os.path.relpath(str(p), str(root)).replace("\\", "/")
        except ValueError:
            return p.name


def matrix_run_ids():
    return tuple(
        f"{scale}-th{t}-s{seed}" for scale in SCALES for seed in SEEDS for t in THREADS
    )


def expected_run_ids():
    return matrix_run_ids() + SPIKE_RUN_IDS + tuple(ANCHOR_REFERENCES)


def run_id_meta(run_id):
    m = RUN_ID_RE.match(run_id)
    if not m:
        return None
    return {
        "scale": m.group(1),
        "threads": int(m.group(2)),
        "seed": int(m.group(3)),
        "retry": m.group(4),
    }


def column_value(data, tick):
    """列取值：tick=None 取终局哈希，否则取该采样点哈希。"""
    if data is None:
        return None
    if tick is None:
        return data.final_hash
    return data.samples.get(tick)


# ---------------------------------------------------------------- 解析


def parse_stdout_text(text, strict):
    """解析 <run_id>.stdout。

    返回 (RunData|None, errors)。errors 为契约违背列表；strict=False 时仍返回
    尽力解析的数据（用于 REAL_EXIT≠0 的失败局披露，不参与判定）。
    """
    errors = []
    data = RunData()
    lines = text.split("\n")
    while lines and lines[-1] == "":
        lines.pop()
    saw_summary = False
    next_key_idx = 0  # 下一个期望的摘要键（KEY_ORDER 顺序，其后为 hash= 行）
    hash_seen = False
    for lineno, raw in enumerate(lines, 1):
        line = raw[:-1] if raw.endswith("\r") else raw
        if line == "":
            errors.append(f"第 {lineno} 行: 空行（契约无空行）")
            continue
        m = SAMPLE_RE.match(line)
        if m:
            if saw_summary:
                errors.append(f"第 {lineno} 行: sample 行出现在摘要行之后")
                continue
            tick = int(m.group(1))
            h = "0x" + m.group(2)
            if tick in data.samples:
                errors.append(f"第 {lineno} 行: 采样点 t={tick} 重复")
                continue
            data.samples[tick] = h
            continue
        m = KEY_RE.match(line)
        if m:
            key, val = m.group(1), int(m.group(2))
            saw_summary = True
            if hash_seen:
                errors.append(f"第 {lineno} 行: {key}= 出现在 hash= 行之后")
                continue
            if next_key_idx < len(KEY_ORDER) and key == KEY_ORDER[next_key_idx]:
                next_key_idx += 1
            else:
                expect = KEY_ORDER[next_key_idx] if next_key_idx < len(KEY_ORDER) else "hash"
                errors.append(f"第 {lineno} 行: 摘要键顺序不符（期望 {expect}=，实际 {key}=）")
            setattr(data, key, val)
            continue
        m = HASH_RE.match(line)
        if m:
            saw_summary = True
            if data.final_hash is not None:
                errors.append(f"第 {lineno} 行: 终局 hash= 行重复")
                continue
            if next_key_idx != len(KEY_ORDER):
                errors.append(
                    f"第 {lineno} 行: hash= 行前缺少摘要键 {KEY_ORDER[next_key_idx:]}"
                )
            data.final_hash = "0x" + m.group(1)
            hash_seen = True
            continue
        errors.append(f"第 {lineno} 行: 不符合契约语法: {line[:100]!r}")
    if hash_seen:
        if next_key_idx < len(KEY_ORDER):
            errors.append("缺少摘要行: " + ", ".join(f"{k}=" for k in KEY_ORDER[next_key_idx:]))
    else:
        missing = [k for k in KEY_ORDER if getattr(data, k) is None]
        if missing:
            errors.append("缺少摘要行: " + ", ".join(f"{k}=" for k in missing))
        if data.final_hash is None:
            errors.append("缺少终局 hash= 行")
    ticks = list(data.samples)
    for i in range(len(ticks) - 1):
        if ticks[i] >= ticks[i + 1]:
            errors.append("采样点未严格升序: " + ",".join(str(t) for t in ticks))
            break
    if data.final_tick is not None:
        bad = [t for t in ticks if t > data.final_tick]
        if bad:
            errors.append(f"采样点超过 final_tick={data.final_tick}: {bad}")
    if strict and errors:
        return None, errors
    return data, errors


def load_exits(path, expected_set, format_errors):
    """读 exits.txt → {run_id: (exit_code, wall_ms)}；返回 (entries, present)。"""
    entries = {}
    if not path.is_file():
        return entries, False
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as e:
        raise FatalIO(f"exits.txt 读取失败: {e}")
    for lineno, raw in enumerate(text.split("\n"), 1):
        line = raw[:-1] if raw.endswith("\r") else raw
        if line == "":
            continue
        m = EXITS_RE.match(line)
        if not m:
            format_errors.append(
                f"exits.txt 第 {lineno} 行不符契约 '<run_id> REAL_EXIT=<code> wall_ms=<ms>': {line[:100]!r}"
            )
            continue
        rid = m.group(1)
        if rid not in expected_set:
            format_errors.append(f"exits.txt 第 {lineno} 行: run_id 不在预期 29 局清单内: {rid}")
            continue
        if rid in entries:
            format_errors.append(f"exits.txt 第 {lineno} 行: run_id 重复: {rid}")
            continue
        entries[rid] = (int(m.group(2)), int(m.group(3)))
    return entries, True


def scan_stdout_files(runs_dir, expected_set, format_errors):
    """扫描 runs 目录 *.stdout；非预期命名一律 input format error（防漏名/错名遮蔽）。"""
    found = {}
    try:
        names = sorted(os.listdir(runs_dir))
    except OSError as e:
        raise FatalIO(f"runs 目录不可读: {e}")
    for name in names:
        if not name.endswith(".stdout"):
            continue
        p = runs_dir / name
        if not p.is_file():
            format_errors.append(f"runs 目录存在非文件条目（.stdout 后缀）: {name}")
            continue
        stem = name[: -len(".stdout")]
        if stem not in expected_set:
            format_errors.append(f"runs 目录存在非预期 run_id 的 stdout 档: {name}")
            continue
        found[stem] = p
    return found


def build_records(expected, exits, stdout_files, format_errors):
    matrix_ids = set(matrix_run_ids())
    records = {}
    for rid in expected:
        if rid in matrix_ids:
            kind = "matrix"
        elif rid in SPIKE_RUN_IDS:
            kind = "spike"
        else:
            kind = "anchor"
        rec = RunRecord(run_id=rid, kind=kind, meta=run_id_meta(rid))
        if rid in exits:
            rec.exit_code, rec.wall_ms = exits[rid]
        rec.stdout_path = stdout_files.get(rid)
        if rec.exit_code is None:
            rec.status = "missing"
            rec.errors.append(
                "有 stdout 档但 exits.txt 无该局行（半局残档，非证据）"
                if rec.stdout_path
                else "无 stdout 档、exits.txt 亦无该局行"
            )
        elif rec.stdout_path is None:
            rec.status = "missing"
            rec.errors.append(f"有 exits 行（REAL_EXIT={rec.exit_code}）但无 stdout 档")
        else:
            try:
                text = rec.stdout_path.read_text(encoding="utf-8")
            except OSError as e:
                raise FatalIO(f"stdout 档不可读: {rec.stdout_path.name}: {e}")
            if rec.exit_code == 0:
                data, errs = parse_stdout_text(text, strict=True)
                if rec.meta and data is not None and data.seed is not None and data.seed != rec.meta["seed"]:
                    errs = list(errs) + [
                        f"run_id 种子 s{rec.meta['seed']} 与 stdout seed={data.seed} 不一致"
                    ]
                if errs:
                    rec.status = "parse_error"
                    rec.errors = errs
                    format_errors.append(f"{rid}.stdout 解析错误: " + "; ".join(errs))
                else:
                    rec.status = "ok"
                    rec.data = data
            else:
                rec.status = "failed"
                data, _ = parse_stdout_text(text, strict=False)
                rec.data = data
                rec.errors.append(
                    f"REAL_EXIT={rec.exit_code}≠0（失败局；stdout 仅作披露，不参与判定）"
                )
        records[rid] = rec
    return records


# ---------------------------------------------------------------- 判定


def group_columns(records):
    """scale 级列集合：全部 ok 矩阵局的采样 tick 并集（升序）+ 终局列。"""
    ticks = set()
    for r in records:
        if r.status == "ok" and r.data:
            ticks.update(r.data.samples)
    return [(t, f"t={t}") for t in sorted(ticks)] + [(None, "final")]


def analyse_group(scale, seed, group, columns):
    ok = [r for r in group if r.status == "ok"]
    verdicts = []
    for tick, label in columns:
        values = {}
        for r in group:
            values[r.run_id] = column_value(r.data, tick) if r.status == "ok" else None
        present = {k: v for k, v in values.items() if v is not None}
        absent_ok = [r.run_id for r in ok if column_value(r.data, tick) is None]
        note = ""
        if absent_ok:
            verdict = "FAIL"
            note = "同组合内出现列缺失分歧（" + ",".join(absent_ok) + " 缺该列）"
        elif not present:
            verdict = "INCOMP"
            note = "该列无任何可判定数据"
        elif len(set(present.values())) > 1:
            verdict = "FAIL"
        elif len(ok) < len(group):
            verdict = "INCOMP"
            note = f"仅 {len(ok)}/{len(group)} 档可判定"
        else:
            verdict = "PASS"
        verdicts.append(ColumnVerdict(tick, label, verdict, values, note))
    if any(v.verdict == "FAIL" for v in verdicts):
        overall = "FAIL"
    elif any(v.verdict == "INCOMP" for v in verdicts):
        overall = "INCOMP"
    else:
        overall = "PASS"
    return GroupAnalysis(scale, seed, group, columns, verdicts, overall, len(ok))


def final_column_label(scale_records, default="final"):
    """终局列标签：全部 ok 局 final_tick 一致时写 final(t=..)，否则仅 final。"""
    fts = {r.data.final_tick for r in scale_records if r.status == "ok" and r.data}
    if len(fts) == 1:
        return f"final(t={fts.pop()})"
    return default


def analyse_assertion2(records):
    """断言 2：每 scale 标准采样列组内一致。返回 {scale: (verdict, detail)}。"""
    out = {}
    for scale in SCALES:
        std = STANDARD_SAMPLES[scale]
        n_pass = n_fail = 0
        no_data = []
        fail_detail = []
        for seed in SEEDS:
            group = [
                records[f"{scale}-th{t}-s{seed}"] for t in THREADS
            ]
            ok = [r for r in group if r.status == "ok"]
            for tick in std:
                with_tick = [r for r in ok if tick in r.data.samples]
                without = [r for r in ok if tick not in r.data.samples]
                if not with_tick:
                    no_data.append(f"t={tick}(s{seed})")
                    continue
                vals = {r.run_id: r.data.samples[tick] for r in with_tick}
                if without:
                    n_fail += 1
                    fail_detail.append(
                        f"t={tick}(s{seed}): 组合内列缺失分歧（{','.join(r.run_id for r in without)} 缺列）"
                    )
                elif len(set(vals.values())) > 1:
                    n_fail += 1
                    fail_detail.append(f"t={tick}(s{seed}): 哈希不一致")
                elif len(ok) < len(group):
                    no_data.append(f"t={tick}(s{seed})（仅 {len(ok)}/4 档）")
                else:
                    n_pass += 1
        total = len(std) * len(SEEDS)
        if n_fail:
            verdict = "FAIL"
            detail = f"FAIL（{n_pass}/{total} 判定点 PASS；{n_fail} 处不一致）: " + "; ".join(fail_detail)
        elif n_pass == total:
            verdict = "PASS"
            detail = f"PASS（{n_pass}/{total} 判定点 = {len(SEEDS)} 组合 × {len(std)} 中间列组内一致）"
        else:
            verdict = "INCOMP"
            detail = (
                f"INCOMPLETE（{n_pass}/{total} 判定点 PASS；未判定 {total - n_pass} 处: "
                + "; ".join(no_data[:8])
                + ("…" if len(no_data) > 8 else "")
                + "）"
            )
        out[scale] = (verdict, detail)
    return out


def analyse_spike(records):
    """加样局 vs 基准局逐列一致。返回 [(spike_id, base_id, verdict, note, columns_detail)]。"""
    rows = []
    for spike_id in SPIKE_RUN_IDS:
        base_id = spike_id.rsplit("-r", 1)[0]
        rec_s, rec_b = records[spike_id], records[base_id]
        if rec_s.status != "ok" or rec_b.status != "ok":
            note_parts = []
            if rec_b.status != "ok":
                note_parts.append(f"基准局状态={rec_b.status}")
            if rec_s.status != "ok":
                note_parts.append(f"加样局状态={rec_s.status}")
            rows.append((spike_id, base_id, "INCOMP", "；".join(note_parts) + "（无法逐列比对）", []))
            continue
        cols = sorted(set(rec_s.data.samples) | set(rec_b.data.samples))
        bad = []
        for tick in cols:
            vs, vb = rec_s.data.samples.get(tick), rec_b.data.samples.get(tick)
            if vs != vb:
                bad.append((tick, vs, vb))
        if rec_s.data.final_hash != rec_b.data.final_hash:
            bad.append((None, rec_s.data.final_hash, rec_b.data.final_hash))
        if bad:
            detail = "; ".join(
                f"t={t if t is not None else 'final'}: 加样={vs} 基准={vb}" for t, vs, vb in bad
            )
            rows.append((spike_id, base_id, "FAIL", detail, bad))
        else:
            n = len(cols) + 1
            rows.append((spike_id, base_id, "PASS", f"{n}/{n} 列一致（采样 {len(cols)} + 终局 1）", []))
    return rows


def analyse_anchors(records, ref_bytes):
    """锚局逐字节 diff（参照档现场读取）。返回 [(rid, ref_rel, verdict, detail, sizes)]。"""
    rows = []
    for rid, ref_rel in ANCHOR_REFERENCES.items():
        rec = records[rid]
        ref = ref_bytes[rid]
        if rec.stdout_path is None:
            rows.append((rid, ref_rel, "INCOMP", "缺 stdout 档（无法逐字节 diff）", (len(ref), None), []))
            continue
        try:
            actual = rec.stdout_path.read_bytes()
        except OSError as e:
            raise FatalIO(f"锚局 stdout 不可读: {rec.stdout_path.name}: {e}")
        diffs = [(i, a, b) for i, (a, b) in enumerate(zip(actual, ref)) if a != b]
        diffs += [(i, actual[i], None) for i in range(len(ref), len(actual))] if len(actual) > len(ref) else []
        diffs += [(i, None, ref[i]) for i in range(len(actual), len(ref))] if len(ref) > len(actual) else []
        equal = not diffs and len(actual) == len(ref)
        if equal and rec.status == "ok":
            verdict = "PASS"
            detail = f"逐字节一致（{len(actual)} bytes）"
        elif equal and rec.status != "ok":
            verdict = "INCOMP"
            detail = f"逐字节一致但局未成功（REAL_EXIT={rec.exit_code}），证据不完整"
        elif rec.status == "ok":
            verdict = "FAIL"
            detail = _diff_detail(actual, ref, diffs)
        else:
            verdict = "INCOMP"
            detail = "局未成功（REAL_EXIT=%s）；diff 亦不一致：" % rec.exit_code + _diff_detail(actual, ref, diffs)
        rows.append((rid, ref_rel, verdict, detail, (len(ref), len(actual)), diffs))
    return rows


def _diff_detail(actual, ref, diffs):
    head = f"不一致（参照 {len(ref)} bytes / 实测 {len(actual)} bytes，共 {len(diffs)} 处字节差异）"
    listed = diffs[:ANCHOR_DIFF_LIST_CAP]
    parts = [
        f"offset {i}: 实测 {('0x%02x' % a) if a is not None else 'EOF'} vs 参照 {('0x%02x' % b) if b is not None else 'EOF'}"
        for i, a, b in listed
    ]
    if len(diffs) > ANCHOR_DIFF_LIST_CAP:
        parts.append(f"（超出列出上限 {ANCHOR_DIFF_LIST_CAP} 的剩余 {len(diffs) - ANCHOR_DIFF_LIST_CAP} 处以计数如实披露）")
    return head + "；" + "；".join(parts)


def analyse_seed_divergence(records):
    """种子互异 sanity：同 (scale,threads) 下 3 种子终局哈希两两不同。"""
    rows = []
    for scale in SCALES:
        for t in THREADS:
            vals, missing = {}, []
            for seed in SEEDS:
                rec = records[f"{scale}-th{t}-s{seed}"]
                if rec.status == "ok":
                    vals[seed] = rec.data.final_hash
                else:
                    missing.append(seed)
            if len(vals) == 3:
                collision = len(set(vals.values())) != 3
                rows.append((scale, t, collision, vals, missing))
            else:
                rows.append((scale, t, None, vals, missing))
    return rows


# ---------------------------------------------------------------- 报告渲染


VERDICT_WORD = {"PASS": "PASS", "FAIL": "FAIL", "INCOMP": "INCOMPLETE"}


def cell(v):
    return "—" if v is None else str(v)


def render_group_table(an, out):
    columns = an.columns  # [(tick, label)]
    header = ["run_id", "REAL_EXIT"] + [label for _tick, label in columns] + ["units"]
    out.append("| " + " | ".join(header) + " |")
    out.append("|" + "---|" * len(header))
    for r in an.records:
        row = [r.run_id]
        if r.status == "missing":
            row.append("缺局")
            row += ["（缺）"] * len(columns)
            row.append("（缺）")
        else:
            row.append(str(r.exit_code))
            for tick, _label in columns:
                row.append(cell(column_value(r.data, tick) if r.data else None))
            row.append(cell(r.data.units if r.data else None))
        out.append("| " + " | ".join(row) + " |")
    jrow = [f"判定 {an.scale}-s{an.seed}（{an.ok_count}/4 档）→ {VERDICT_WORD[an.overall]}", "—"]
    jrow += [VERDICT_WORD[c.verdict] for c in an.verdicts]
    jrow.append("—")
    out.append("| " + " | ".join(jrow) + " |")
    notes = [f"{c.label}: {c.note}" for c in an.verdicts if c.note]
    if an.ok_count == 0 and notes:
        notes = ["组合内无任何可判定局（4 档全缺）"]
    elif len({c.note for c in an.verdicts if c.note}) == 1 and len(notes) > 3:
        notes = [f"全部列: {notes[0].split(': ', 1)[1]}"]
    if notes:
        out.append("")
        out.append("（备注：" + "；".join(notes) + "）")
    out.append("")


def build_report(records, ref_bytes, format_errors, exits_present, n_exits, n_stdout, runs_rel, cmd_rel, exit_code):
    out = []
    out.append("# T008 对拍矩阵（同种子重放确定性验证，验收④）")
    out.append("")
    out.append(f"- 生成命令: {cmd_rel}")
    out.append(f"- runs 目录: {runs_rel}")
    out.append(
        "- 预期清单: 29 局 = 矩阵 24（red200 ×12 + full10000 ×12；threads 1/3/6/12 × seeds 42/43/44）"
        " + 加样 3（red200-th12-s42-r2 / -r3、full10000-th12-s42-r2）+ 锚 2（anchor1-t006-fullscale / anchor2-t006-10k300）"
    )
    out.append(
        f"- 实收: exits.txt {'%d 行' % n_exits if exits_present else '缺失（视为 0 行）'}；stdout 档 {n_stdout} 个"
    )
    out.append("- 退出码语义: 0 全 PASS / 1 I/O 错 / 2 输入格式错 / 3 缺局或未完成 / 5 mismatch（本次运行退出码见 §8）")
    out.append("")

    # ---- 矩阵表（每 seed 组合 4 线程档 + 判定行）
    matrix_meta = {}
    group_analyses = []
    for scale in SCALES:
        matrix_recs = [records[f"{scale}-th{t}-s{seed}"] for seed in SEEDS for t in THREADS]
        columns = group_columns(matrix_recs)
        final_label = final_column_label(matrix_recs)
        if columns and columns[-1][0] is None:
            columns = columns[:-1] + [(None, final_label)]
        out.append(f"## 1. 矩阵对拍表 — {scale}" if scale == "red200" else f"## 2. 矩阵对拍表 — {scale}")
        out.append("")
        out.append(
            f"标准采样列（派工单 D3）: {','.join(str(t) for t in STANDARD_SAMPLES[scale])}；"
            f"终局 = 五行摘要 hash= 行；采样列按实际 stdout 并集动态呈现"
        )
        out.append("")
        for seed in SEEDS:
            group = [records[f"{scale}-th{t}-s{seed}"] for t in THREADS]
            an = analyse_group(scale, seed, group, columns)
            group_analyses.append(an)
            matrix_meta[(scale, seed)] = an
            render_group_table(an, out)

    # ---- 加样局
    spike_rows = analyse_spike(records)
    out.append("## 3. 加样局（跨进程同配置逐列一致）")
    out.append("")
    out.append("| 加样局 | 基准局 | 判定 | 明细 |")
    out.append("|---|---|---|---|")
    for spike_id, base_id, verdict, detail, _bad in spike_rows:
        out.append(f"| {spike_id} | {base_id} | {VERDICT_WORD[verdict]} | {detail} |")
    out.append("")

    # ---- 锚局
    anchor_rows = analyse_anchors(records, ref_bytes)
    out.append("## 4. 锚局（跨卡黄金锚，逐字节 diff；期望现场读取自 T006 归档）")
    out.append("")
    out.append("| 锚局 | 参照档（现场读取） | 参照 bytes | 实测 bytes | 逐字节 diff 判定 | REAL_EXIT | 实测 hash（解析自 stdout） | 参照 hash（解析自参照档） |")
    out.append("|---|---|---|---|---|---|---|---|")
    for rid, ref_rel, verdict, detail, sizes, _diffs in anchor_rows:
        rec = records[rid]
        actual_hash = rec.data.final_hash if rec.data else None
        ref_text = ref_bytes[rid].decode("utf-8", errors="replace")
        ref_data, _ = parse_stdout_text(ref_text, strict=False)
        out.append(
            f"| {rid} | {ref_rel} | {sizes[0] if sizes else '—'} | {cell(sizes[1])} | {VERDICT_WORD[verdict]} — {detail} | "
            f"{cell(rec.exit_code)} | {cell(actual_hash)} | {cell(ref_data.final_hash)} |"
        )
    out.append("")

    # ---- mismatch 全量明细
    mismatch_blocks = []
    for an in group_analyses:
        for c in an.verdicts:
            if c.verdict == "FAIL":
                block = [f"### 组合 {an.scale}-s{an.seed}：列 {c.label} 不一致"]
                if c.note:
                    block.append(f"备注：{c.note}")
                block.append("4 档全量值（禁止折叠/省略）：")
                for r in an.records:
                    block.append(f"  - {r.run_id} = {cell(c.values.get(r.run_id))}")
                mismatch_blocks.append("\n".join(block))
    for spike_id, base_id, verdict, detail, bad in spike_rows:
        if verdict == "FAIL":
            block = [f"### 加样局不一致: {spike_id} vs 基准 {base_id}"]
            block.append(detail)
            block.append(
                f"  加样局终局 = {cell(records[spike_id].data.final_hash)}；基准局终局 = {cell(records[base_id].data.final_hash)}"
            )
            mismatch_blocks.append("\n".join(block))
    for rid, ref_rel, verdict, detail, _sizes, _diffs in anchor_rows:
        if verdict == "FAIL":
            mismatch_blocks.append(f"### 锚局逐字节 diff 不一致: {rid} vs {ref_rel}\n{detail}")
    seed_rows = analyse_seed_divergence(records)
    for scale, t, collision, vals, _missing in seed_rows:
        if collision:
            block = [f"### 种子互异失效: {scale} th={t}"]
            for seed in SEEDS:
                block.append(f"  - seed={seed} final = {cell(vals.get(seed))}")
            mismatch_blocks.append("\n".join(block))
    out.append("## 5. 不一致全量明细（禁止折叠/省略）")
    out.append("")
    if mismatch_blocks:
        out.append("\n\n".join(mismatch_blocks))
    else:
        out.append("无 mismatch（无哈希不一致、无锚局逐字节差异、无种子互异失效）。")
    out.append("")

    # ---- 缺局/失败局
    out.append("## 6. 缺局 / 失败局清单")
    out.append("")
    bad_rows = [records[rid] for rid in expected_run_ids() if records[rid].status in ("missing", "failed")]
    if bad_rows:
        out.append("| run_id | 状态 | 说明 |")
        out.append("|---|---|---|")
        for r in bad_rows:
            label = "缺局" if r.status == "missing" else f"失败局（REAL_EXIT={r.exit_code}）"
            out.append(f"| {r.run_id} | {label} | {'；'.join(r.errors)} |")
    else:
        out.append("无缺局、无失败局（29 局齐备且 REAL_EXIT=0）。")
    out.append("")

    # ---- 覆盖披露
    out.append("## 7. 覆盖披露（披露非判定）")
    out.append("")
    out.append("### 7.1 每局 units（终局存活数）与参数")
    out.append("")
    out.append("| run_id | 类别 | seed | ticks | final_tick | units | REAL_EXIT |")
    out.append("|---|---|---|---|---|---|---|")
    for rid in expected_run_ids():
        r = records[rid]
        if r.data is None:
            continue
        meta = r.meta
        out.append(
            f"| {rid} | {r.kind} | {cell(r.data.seed)} | {cell(r.data.ticks)} | "
            f"{cell(r.data.final_tick)} | {cell(r.data.units)} | {cell(r.exit_code)} |"
        )
    out.append("")
    full_runs = [
        records[rid]
        for rid in expected_run_ids()
        if (
            (records[rid].meta and records[rid].meta["scale"] == "full10000")
            or rid == "anchor1-t006-fullscale"
        )
        and records[rid].data is not None
        and records[rid].data.units is not None
    ]
    battle = [r for r in full_runs if r.data.units < 10000]
    out.append("### 7.2 full 规模战斗段证据（units<10000 ⇒ 战斗段已进入哈希）")
    out.append("")
    if battle:
        out.append(
            f"观察到 {len(battle)}/{len(full_runs)} 个 full 规模局终局 units<10000（战斗段已进入哈希）："
            + "；".join(f"{r.run_id} units={r.data.units}" for r in battle)
        )
    else:
        out.append("未观察到 full 规模局 units<10000（战斗段覆盖未获证据）。")
    out.append("")
    out.append("### 7.3 标准采样列覆盖（按 scale，ok 矩阵/加样局中该列出现数）")
    out.append("")
    for scale in SCALES:
        std = STANDARD_SAMPLES[scale]
        scale_runs = [
            records[rid]
            for rid in expected_run_ids()
            if records[rid].kind in ("matrix", "spike")
            and records[rid].meta
            and records[rid].meta["scale"] == scale
            and records[rid].status == "ok"
        ]
        if not scale_runs:
            out.append(f"- {scale}: 无可判定局（缺局），标准列覆盖无法统计")
            continue
        parts = []
        for tick in std:
            have = sum(1 for r in scale_runs if tick in r.data.samples)
            parts.append(f"t={tick}: {have}/{len(scale_runs)}")
        out.append(f"- {scale}: " + "；".join(parts))
    out.append("")

    # ---- 汇总判定（末尾，对应任务卡验收断言 1/2/3）
    out.append("## 8. 汇总判定（任务卡验收断言 1/2/3）")
    out.append("")
    # 断言 1
    combos = [(an.scale, an.seed, an) for an in group_analyses]
    n_c1_pass = sum(1 for _, _, an in combos if an.overall == "PASS")
    n_c1_fail = sum(1 for _, _, an in combos if an.overall == "FAIL")
    n_c1_inc = len(combos) - n_c1_pass - n_c1_fail
    if n_c1_fail:
        a1 = "FAIL"
    elif n_c1_pass == len(combos):
        a1 = "PASS"
    else:
        a1 = "INCOMPLETE"
    detail1 = f"{n_c1_pass}/{len(combos)} 组合逐位一致"
    if n_c1_fail:
        detail1 += f"，{n_c1_fail} 组合 FAIL（见 §5）"
    if n_c1_inc:
        detail1 += f"，{n_c1_inc} 组合缺局/数据不足未判定"
    combo_bits = "；".join(
        f"{an.scale}-s{an.seed} {VERDICT_WORD[an.overall]}" for an in group_analyses
    )
    out.append(f"- 断言 1（零 mismatch：每 (scale,seed) 组合 4 线程档 × 全部采样列 + 终局列逐位一致）: {a1} — {detail1}")
    out.append(f"  - 组合明细: {combo_bits}")
    # 断言 2
    a2 = analyse_assertion2(records)
    verdict2 = "PASS"
    for scale in SCALES:
        v, _d = a2[scale]
        if v == "FAIL":
            verdict2 = "FAIL"
        elif v == "INCOMP" and verdict2 == "PASS":
            verdict2 = "INCOMPLETE"
    out.append(f"- 断言 2（中间哈希：每 scale 标准采样列组内一致）: {verdict2}")
    for scale in SCALES:
        _v, d = a2[scale]
        out.append(f"  - {scale}: {d}")
    # 断言 3
    if mismatch_blocks:
        out.append(f"- 断言 3（mismatch 全量如实列出）: 触发 — 共 {len(mismatch_blocks)} 处不一致，全部值见 §5（禁止折叠/省略）")
    else:
        out.append("- 断言 3（mismatch 全量如实列出）: 未触发（无 mismatch）")
    # 加样
    spike_bits = []
    for spike_id, _base, verdict, detail, _bad in spike_rows:
        spike_bits.append(f"{spike_id} {VERDICT_WORD[verdict]}")
    if any(v == "FAIL" for _s, _b, v, _d, _x in spike_rows):
        spike_overall = "FAIL"
    elif all(v == "PASS" for _s, _b, v, _d, _x in spike_rows):
        spike_overall = "PASS"
    else:
        spike_overall = "INCOMPLETE"
    out.append(f"- 加样局（跨进程同配置逐列一致）: {spike_overall} — " + "；".join(spike_bits))
    # 锚
    anchor_bits = []
    for rid, _ref, v, _d, _s, _x in anchor_rows:
        anchor_bits.append(f"{rid} {VERDICT_WORD[v]}")
    if any(v == "FAIL" for _r, _ref, v, _d, _s, _x in anchor_rows):
        anchor_overall = "FAIL"
    elif all(v == "PASS" for _r, _ref, v, _d, _s, _x in anchor_rows):
        anchor_overall = "PASS"
    else:
        anchor_overall = "INCOMPLETE"
    out.append(f"- 锚局（vs T006 归档逐字节 diff）: {anchor_overall} — " + "；".join(anchor_bits))
    # 种子互异
    n_seed_eval = sum(1 for _s, _t, c, _v, _m in seed_rows if c is not None)
    n_seed_coll = sum(1 for _s, _t, c, _v, _m in seed_rows if c)
    if n_seed_coll:
        seed_overall = f"FAIL — {n_seed_coll} 个 (scale,threads) 出现种子哈希碰撞（见 §5）"
    elif n_seed_eval == len(seed_rows):
        seed_overall = f"PASS — {n_seed_eval}/{len(seed_rows)} 个 (scale,threads) 下 3 种子终局哈希两两不同"
    else:
        seed_overall = f"数据不足 — 可判定 {n_seed_eval}/{len(seed_rows)} 个 (scale,threads)（缺局/失败局所致）"
    out.append(f"- 种子互异 sanity（同 (scale,threads) 下 3 种子终局哈希两两不同）: {seed_overall}")
    out.append(f"- 退出码: {exit_code}")
    out.append("")
    return "\n".join(out), mismatch_blocks, anchor_rows, spike_rows, group_analyses, seed_rows


# ---------------------------------------------------------------- 主流程


def decide_exit(group_analyses, spike_rows, anchor_rows, seed_rows, records):
    hard_mismatch = False
    incomplete = False
    for an in group_analyses:
        if an.overall == "FAIL":
            hard_mismatch = True
        elif an.overall == "INCOMP":
            incomplete = True
    for _s, _b, v, _d, _x in spike_rows:
        if v == "FAIL":
            hard_mismatch = True
        elif v == "INCOMP":
            incomplete = True
    for _r, _ref, v, _d, _s, _x in anchor_rows:
        if v == "FAIL":
            hard_mismatch = True
        elif v == "INCOMP":
            incomplete = True
    for _scale, _t, c, _v, _m in seed_rows:
        if c:
            hard_mismatch = True
    for rid in expected_run_ids():
        rec = records[rid]
        if rec.status in ("missing", "failed"):
            incomplete = True
    # 断言 2 的标准列缺失/未判定也计入 incomplete
    a2 = analyse_assertion2(records)
    for scale in SCALES:
        if a2[scale][0] == "INCOMP":
            incomplete = True
    if hard_mismatch:
        return EXIT_MISMATCH
    if incomplete:
        return EXIT_INCOMPLETE
    return EXIT_OK


def run(runs_dir, out_md, script_dir, repo_root):
    if not runs_dir.is_dir():
        raise FatalIO(f"runs 目录不存在或不是目录: {rel_display(runs_dir, repo_root)}")
    expected = expected_run_ids()
    expected_set = set(expected)
    ref_bytes = {}
    for rid, ref_rel in ANCHOR_REFERENCES.items():
        p = repo_root / ref_rel
        try:
            ref_bytes[rid] = p.read_bytes()
        except OSError as e:
            raise FatalIO(f"锚参照档不可读（现场读取失败）: {ref_rel} ({e})")

    format_errors = []
    exits, exits_present = load_exits(runs_dir / "exits.txt", expected_set, format_errors)
    stdout_files = scan_stdout_files(runs_dir, expected_set, format_errors)
    records = build_records(expected, exits, stdout_files, format_errors)

    runs_rel = rel_display(runs_dir, repo_root)
    out_rel = rel_display(out_md, repo_root)
    cmd_rel = f"python docs/evidence/t008/compare_matrix.py {runs_rel}" + (
        f" {out_rel}" if out_md != script_dir / "matrix.md" else ""
    )

    if format_errors:
        lines = [
            "# T008 对拍矩阵（输入格式错，判定未执行）",
            "",
            f"- runs 目录: {runs_rel}",
            "- 退出码: 2（输入格式错；判定未执行，修正输入后重跑）",
            "",
            "## 输入格式错误全量清单（禁止折叠/省略）",
            "",
        ]
        for i, err in enumerate(format_errors, 1):
            lines.append(f"{i}. {err}")
        text = "\n".join(lines) + "\n"
        _write_and_print(text, out_md, repo_root)
        return EXIT_FORMAT

    # 判定
    group_analyses = []
    for scale in SCALES:
        columns = group_columns([records[f"{scale}-th{t}-s{seed}"] for seed in SEEDS for t in THREADS])
        for seed in SEEDS:
            group = [records[f"{scale}-th{t}-s{seed}"] for t in THREADS]
            group_analyses.append(analyse_group(scale, seed, group, columns))
    spike_rows = analyse_spike(records)
    anchor_rows = analyse_anchors(records, ref_bytes)
    seed_rows = analyse_seed_divergence(records)
    exit_code = decide_exit(group_analyses, spike_rows, anchor_rows, seed_rows, records)

    n_exits = len(exits)
    n_stdout = sum(1 for rid in expected if records[rid].stdout_path is not None)
    text, _mm, _ar, _sr, _ga, _sg = build_report(
        records, ref_bytes, format_errors, exits_present, n_exits, n_stdout, runs_rel, cmd_rel, exit_code
    )
    _write_and_print(text + "\n", out_md, repo_root)
    return exit_code


def _write_and_print(text, out_md, repo_root):
    try:
        out_md.parent.mkdir(parents=True, exist_ok=True)
        out_md.write_text(text, encoding="utf-8", newline="\n")
    except OSError as e:
        raise FatalIO(f"输出文件不可写: {rel_display(out_md, repo_root)} ({e})")
    sys.stdout.write(text)


def _setup_stdio():
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", newline="\n")
        except (AttributeError, ValueError, OSError):
            pass


def main(argv):
    _setup_stdio()
    args = argv[1:]
    if len(args) > 2:
        eprint("用法: python docs/evidence/t008/compare_matrix.py [runs_dir] [out_md]")
        return EXIT_FORMAT
    script_dir = Path(__file__).resolve().parent
    repo_root = script_dir.parents[2]
    runs_dir = Path(args[0]) if len(args) >= 1 else script_dir / "runs"
    out_md = Path(args[1]) if len(args) >= 2 else script_dir / "matrix.md"
    try:
        return run(runs_dir, out_md, script_dir, repo_root)
    except FatalIO as e:
        eprint(f"compare_matrix: I/O 错误: {e}")
        return EXIT_IO
    except OSError as e:
        eprint(f"compare_matrix: I/O 错误: {e}")
        return EXIT_IO


if __name__ == "__main__":
    sys.exit(main(sys.argv))
