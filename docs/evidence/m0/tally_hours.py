#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""tally_hours.py — T011 M0 首周投入校准：台账耗时汇总（任务台账唯一数据源）。

规格（docs/evidence/m0/dispatches/wp-a.md §3 / taskset/t011-data-pack.md D3、D9）：
- 纯标准库、无参数运行；解析 task-ledger.md「## 记录」表中行首 `| T0xx |` 的行，
  取耗时列 `≈(\\d+)\\s*min` 的首个匹配（单位 min，int）。
- 窗口映射常量：PREP = T001~T006（预备周，不计 W1）；
  W1 = T007,T008,T009,T015,T010,T011（2026-10-05 ~ 10-11 窗口，按实际执行日期归属）。
- 输出（stdout）：逐任务行（ID、耗时 min、窗口归属）+ 两窗口小计（min 与 h，
  h = min/60 保留 2 位）+ 缺失任务「未登记：<ID>（收口回填）」提示行（退出码仍 0）。
- 判定语义零裁量：本脚本只做台账耗时列解析与窗口小计，不产生任何验收判定。

用法（仓库根或任意目录）：python docs/evidence/m0/tally_hours.py
（路径以脚本自身位置定位仓库根 = 脚本目录上溯三级，与运行时 cwd 无关。）
"""

import re
import sys
from pathlib import Path

LEDGER_REL = "task-ledger.md"
SECTION_HEADING = "## 记录"
ROW_RE = re.compile(r"^\|\s*(T\d+)\s*\|")
MIN_RE = re.compile(r"≈(\d+)\s*min")

# 派工单 §3 固定窗口映射（顺序即输出顺序，不得改动）
PREP = ["T001", "T002", "T003", "T004", "T005", "T006"]
W1 = ["T007", "T008", "T009", "T015", "T010", "T011"]

# 跨预备周/W1 边界拆分（D3-1 补裁决，Lead G1 修正 2026-10-06）：T007 worker-2 执行段
# ≈99 min 于 10-04（预备周日历内；t007/environment.txt 生成时刻 2026-10-04 15:28 为证），
# 主会话接管收尾 ≈65 + 审核轮 ≈25 于 10-05（takeover_preflight 2026-10-05 08:08 起）。
# 按投入口径「跨窗口任务按实际执行日期分摊」（AGENTS.md 投入口径句），99 min 归预备周、
# 余量归 W1。阶段和 99+65+25=189 ≈ 台账总 ≈190，尾差 1 min 归 W1 侧（190−99=91）。
# 其余五卡不跨预备周边界：T008 09:45 起全在 10-05；T009/T015 全在 10-05；T010 跨
# 10-05/10-06 两日均在 W1 窗口内（D9 原注）。
SPLIT_T007_PREP = 99


def repo_root() -> Path:
    # 脚本位于 docs/evidence/m0/tally_hours.py → parents[0]=m0 / [1]=evidence /
    # [2]=docs / [3]=仓库根（上溯三级目录）
    return Path(__file__).resolve().parents[3]


def parse_ledger(root: Path) -> dict:
    """返回 {任务ID: 耗时min}；只认「## 记录」节内行首 `| T0xx |` 的行。

    耗时取「耗时列」（表头：编号/任务/类型/一次通过/返工次数/耗时/原因/证据 →
    第 6 列，split 索引 6）内 `≈(\\d+)\\s*min` 首个匹配——**不得整行搜索**：
    T010 行任务列含「≈92 min 零中断」字样，整行首匹配会错抓 92（双盲对照
    拦截实例，2026-10-06）。列切分按未转义竖线 `(?<!\\\\)\\|`（台账单元格内
    存在 `\\|dx\\|` 类转义竖线，如 T004）。
    """
    text = (root / LEDGER_REL).read_text(encoding="utf-8")
    lines = text.splitlines()
    # 定位「## 记录」节：自该标题行起，至下一个「## 」标题或文件尾
    start = None
    for i, ln in enumerate(lines):
        if ln.strip() == SECTION_HEADING:
            start = i + 1
            break
    if start is None:
        raise SystemExit(f"FATAL: 未找到「{SECTION_HEADING}」节：{root / LEDGER_REL}")
    out = {}
    for ln in lines[start:]:
        if ln.startswith("## "):
            break
        m = ROW_RE.match(ln)
        if not m:
            continue
        tid = m.group(1)
        cells = re.split(r"(?<!\\)\|", ln)  # 索引：0空/1编号/2任务/3类型/4一次通过/5返工/6耗时/7原因/8证据
        if len(cells) <= 6:
            continue
        mm = MIN_RE.search(cells[6])
        if mm:
            # 耗时列内首个匹配（派工单 §3；同列内后续 ≈N min 不再消费）
            out[tid] = int(mm.group(1))
    return out


def fmt_h(total_min: int) -> str:
    return f"{total_min / 60:.2f}"


def main() -> int:
    root = repo_root()
    hours = parse_ledger(root)

    print("== 逐任务（ID | 耗时 min | 窗口归属；来源 task-ledger.md「## 记录」≈N min 首个匹配）==")
    for window_name, ids in (("预备周(不计W1)", PREP), ("W1", W1)):
        for tid in ids:
            if tid in hours:
                if tid == "T007":
                    print(f"{tid} | {hours[tid]} min | 跨预备周/W1 拆分：预备周 {SPLIT_T007_PREP}（10-04 worker-2 段）+ W1 {hours[tid] - SPLIT_T007_PREP}（10-05 接管+审核+尾差；D3-1）")
                else:
                    print(f"{tid} | {hours[tid]} min | {window_name}")
            else:
                print(f"未登记：{tid}（收口回填）| 0 min 计 | {window_name}")

    prep_total = sum(hours[t] for t in PREP if t in hours) + SPLIT_T007_PREP
    w1_total = sum(hours[t] for t in W1 if t in hours) - SPLIT_T007_PREP

    print("== 窗口小计 ==")
    print(f"预备周小计（T001~T006 + T007 的 10-04 段 {SPLIT_T007_PREP}，不计 W1）: {prep_total} min = {fmt_h(prep_total)} h")
    w1_missing = [t for t in W1 if t not in hours]
    note = ""
    if w1_missing:
        note = f"（注：{'、'.join(w1_missing)} 未登记，收口回填后更新）"
    print(f"W1 累计（T007 的 10-05 段 {hours.get('T007', 0) - SPLIT_T007_PREP} + T008、T009、T015、T010、T011；截至本脚本运行时刻{note}）: {w1_total} min = {fmt_h(w1_total)} h")
    return 0


if __name__ == "__main__":
    sys.exit(main())
