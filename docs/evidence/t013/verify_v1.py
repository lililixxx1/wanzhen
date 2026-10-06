# -*- coding: utf-8 -*-
"""T013 · 报告 V1.0 实测修订校验脚本（零第三方依赖）。

五组断言（派工单 T013-WP-A 第四步）：
  1. 版本一致性（V1.0 >=5 处；V0.9.1 仅限修订记录历史行）
  2. 实测数字双向核对（HTML <-> 源档，千分位逗号归一后子串匹配）
  3. 残项处置锚（HTML 含 12 个处置关键词）
  4. 旧初值零残留（负向；范围排除修订记录表 V0.x 历史行）
  5. HTML 工程（锚点 id 存在、href->id 差集空、标签平衡、UTF-8 可解码）

用法：python verify_v1.py   （在任意目录可跑，路径相对本脚本定位）
退出码：全 PASS = 0，任一 FAIL = 1。
"""
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent          # docs/evidence/t013
DOCS = HERE.parents[1]                          # docs
ROOT = HERE.parents[2]                          # 仓库根

HTML_PATH = DOCS / "万阵-游戏前期策划报告.html"
README_PATH = DOCS / "evidence" / "m0" / "README.md"
AGENTS_PATH = ROOT / "AGENTS.md"

results = []  # (name, ok, detail)


def record(name, ok, detail=""):
    results.append((name, bool(ok), detail))
    print(("PASS" if ok else "FAIL") + " | " + name + ((" | " + detail) if detail else ""))


def norm(text):
    """千分位逗号归一。"""
    return text.replace(",", "")


def main():
    html_raw = HTML_PATH.read_bytes()
    try:
        html = html_raw.decode("utf-8")
        record("5.6 HTML 可 UTF-8 无异常解码", True, "%d bytes" % len(html_raw))
    except UnicodeDecodeError as exc:
        record("5.6 HTML 可 UTF-8 无异常解码", False, str(exc))
        html = html_raw.decode("utf-8", errors="replace")

    readme = README_PATH.read_text(encoding="utf-8")
    agents = AGENTS_PATH.read_text(encoding="utf-8")

    # ---------- 组 1：版本一致性 ----------
    v10_count = html.count("V1.0")
    record("1a V1.0 出现 >=5 处", v10_count >= 5, "count=%d" % v10_count)

    v091_bad = []
    for m in re.finditer(re.escape("V0.9.1"), html):
        ctx = html[max(0, m.start() - 200): m.start() + 200]
        if ("2026-09-29" not in ctx) and ("2026-09-30" not in ctx):
            v091_bad.append(m.start())
    record("1b V0.9.1 仅出现在修订记录历史行（上下文 200 字符含 2026-09-29/30）",
           len(v091_bad) == 0,
           ("occurrences=%d, violations=%s" % (html.count("V0.9.1"), v091_bad)) if v091_bad
           else "occurrences=%d (全部为历史行)" % html.count("V0.9.1"))

    # ---------- 组 2：实测数字双向核对 ----------
    html_n = norm(html)
    readme_n = norm(readme)
    agents_n = norm(agents)
    numbers = [
        ("0.060651", "m0"), ("0.072196", "m0"), ("0.070617", "m0"), ("0.108681", "m0"),
        ("1.001913", "m0"), ("1.112480", "m0"), ("17.532997", "m0"),
        ("1099638.6", "m0"), ("1405967.6", "m0"),
        ("333.15", "m0"), ("202.48", "m0"),
        ("6.8 MiB", "m0"), ("670.6", "m0"),
        ("0.088271", "m0"), ("0.032099", "m0"), ("0.679255", "m0"), ("0.247002", "m0"),
        ("329.33", "m0"), ("177.95", "m0"),
        ("23.27", "agents"),
    ]
    g2_fail = []
    for value, source in numbers:
        in_html = norm(value) in html_n
        src_text = agents_n if source == "agents" else readme_n
        src_name = "AGENTS.md" if source == "agents" else "m0/README.md"
        in_src = norm(value) in src_text
        if not (in_html and in_src):
            g2_fail.append("%s(html=%s,%s=%s)" % (value, in_html, src_name, in_src))
    record("2 实测数字双向核对（20 值，HTML 与源档均命中，逗号归一）",
           len(g2_fail) == 0,
           "20/20 命中" if not g2_fail else "未命中: " + "; ".join(g2_fail))

    # ---------- 组 3：残项处置锚 ----------
    anchors = [
        "达承诺线 1µs", "5,400 ticks", "±10pp", "≥400 场/周", "约 76%",
        "26 周排期为 Q5 候选方案", "316.57", "6.54 亿", "红蓝互换",
        "最终判 FAIL 时", "条件性披露", "docs/evidence/m0/",
    ]
    g3_missing = [a for a in anchors if a not in html]
    record("3 残项处置锚（12 词全部在 HTML）", not g3_missing,
           "12/12 命中" if not g3_missing else "缺失: " + "; ".join(g3_missing))

    # ---------- 组 4：旧初值零残留（排除修订记录 V0.x 历史行） ----------
    history_spans = []
    for m in re.finditer(r"<tr>.*?</tr>", html, flags=re.S):
        if re.search(r'<td class="mono">V0\.\d', m.group(0)):
            history_spans.append((m.start(), m.end()))

    def outside_history(sub):
        """返回 sub 所有出现在「历史行之外」的位置。"""
        hits = []
        start = 0
        while True:
            i = html.find(sub, start)
            if i < 0:
                break
            if not any(s <= i < e for s, e in history_spans):
                hits.append(i)
            start = i + 1
        return hits

    forbidden = [
        "@ 2µs 需 ≥ 2.5×",
        "9.1×（22ms 预算）",
        "@ 1µs 需 ≥ 4.6×",
        "微秒成本与扩展曲线达标",
        "初值，M0 首周可调",
        "立项预验证 · M0 先行，身份未定",
        "PRE-VALIDATION PHASE",
        "性能指标均为初值，M0 profiling 后修订",
        "立项身份待 M0 数据与独立复算后定",
    ]
    g4_hits = []
    for sub in forbidden:
        pos = outside_history(sub)
        if pos:
            g4_hits.append("%s@%s" % (sub, pos))
    record("4 旧初值零残留（9 串 0 命中，历史行已排除；排除行数=%d）" % len(history_spans),
           not g4_hits,
           "9/9 零命中" if not g4_hits else "命中: " + "; ".join(g4_hits))

    # ---------- 组 5：HTML 工程 ----------
    ids = set(re.findall(r'id="([^"]+)"', html))
    want_ids = ["s%d" % i for i in range(1, 10)] + ["fn"] + ["fn%d" % i for i in range(1, 16)]
    missing_ids = [i for i in want_ids if i not in ids]
    record("5a 锚点 id=s1~s9, fn, fn1~fn15 全存在", not missing_ids,
           "26/26 存在" if not missing_ids else "缺失: " + "; ".join(missing_ids))

    hrefs = set(re.findall(r'href="#([^"]+)"', html))
    dangling = sorted(h for h in hrefs if h not in ids)
    record("5b 正文引用 href=#X -> id 差集为空（href 数=%d）" % len(hrefs),
           not dangling,
           "差集空" if not dangling else "悬空: " + "; ".join(dangling))

    balance_fail = []
    for tag in ["section", "table", "div", "ul", "ol", "p"]:
        opens = len(re.findall(r"<%s\b" % tag, html))
        closes = len(re.findall(r"</%s>" % tag, html))
        if opens != closes:
            balance_fail.append("%s(%d/%d)" % (tag, opens, closes))
    record("5c 标签平衡 section/table/div/ul/ol/p 开闭计数相等", not balance_fail,
           "全部相等" if not balance_fail else "不等: " + "; ".join(balance_fail))

    # ---------- 汇总 ----------
    total = len(results)
    failed = sum(1 for _, ok, _ in results if not ok)
    print("-" * 60)
    print("SUMMARY: %d/%d PASS, %d FAIL" % (total - failed, total, failed))
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
