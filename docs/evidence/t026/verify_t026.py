# -*- coding: utf-8 -*-
"""T026 · 锚子串双向核对脚本（docs/evidence/t026/verify_t026.py）

模式沿 T014 verify_t014.py（T011 verify_claims.py 先例）：CLAIMS = (id, source_relpath, anchor)，
双向核对 = anchor 在源档命中 且 anchor 在主档（docs/m5-identity-review.md）命中，任一不中即 FAIL。
全 PASS exit 0；任一 FAIL exit 1。

组划分（仓库根执行；跨仓组以相对路径 ../bevy-ai-workflow/ 直核——主根有兄弟仓，无需沿 T014 拆双脚本）：
  组 A 台账      task-ledger.md
  组 B M5 判定档 docs/evidence/m5/README.md
  组 C T016 记录 docs/m0-identity-decision.md
  组 D Q5 底本   docs/m0-identity-review.md
  组 E AGENTS    AGENTS.md
  组 F taskset   taskset/README.md
  组 G 跨仓      ../bevy-ai-workflow/{AGENTS.md, Bevy-AI开发意向文档.md, docs/m5-game-selection.md}
"""
import io
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
DOC = 'docs/m5-identity-review.md'

R_LEDGER = 'task-ledger.md'
R_M5 = 'docs/evidence/m5/README.md'
R_DEC = 'docs/m0-identity-decision.md'
R_REV = 'docs/m0-identity-review.md'
R_AGENTS = 'AGENTS.md'
R_TASKSET = 'taskset/README.md'
R_WF_AGENTS = '../bevy-ai-workflow/AGENTS.md'
R_INTENT = '../bevy-ai-workflow/Bevy-AI开发意向文档.md'
R_MEMO = '../bevy-ai-workflow/docs/m5-game-selection.md'

CLAIMS = [
    # ── 组 A · 台账 ──
    ('a1', R_LEDGER, '0x958c5938c8682529'),
    ('a2', R_LEDGER, '0xb82a248ff23515e2'),
    ('a3', R_LEDGER, '≈2px'),
    ('a4', R_LEDGER, 'tools/sweep_paths.py'),
    ('a5', R_LEDGER, '附录 E'),
    # ── 组 B · M5 判定档 ──
    ('b1', R_M5, '900 局矩阵'),
    ('b2', R_M5, '304.10'),
    ('b3', R_M5, '197.22'),
    ('b4', R_M5, '残余账 12 项'),
    ('b5', R_M5, '预裁剪八条'),
    # ── 组 C · T016 决策记录 ──
    ('c1', R_DEC, '未接入 M0'),
    ('c2', R_DEC, '外部评审意见'),
    ('c3', R_DEC, 'M1 生死题不达标'),
    # ── 组 D · Q5 可信底本 ──
    ('d1', R_REV, '不得与「试金石」定位含糊并存'),
    ('d2', R_REV, '开源实验向'),
    ('d3', R_REV, '挂起 ≠ 终止'),
    ('d4', R_REV, '26 周候选基线'),
    ('d5', R_REV, '先迁移后恢复'),
    # ── 组 E · AGENTS ──
    ('e1', R_AGENTS, '3330 min'),
    ('e2', R_AGENTS, '55.50 h'),
    ('e3', R_AGENTS, '预备周 557 min'),
    ('e4', R_AGENTS, '10-11 终判行'),
    ('e5', R_AGENTS, '保守基线'),
    # ── 组 F · taskset ──
    ('f1', R_TASKSET, '≈39h'),
    ('f2', R_TASKSET, '推荐采纳 8/8'),
    # ── 组 G · 跨仓 ──
    ('g1', R_WF_AGENTS, '解冻 = 主动决策非自动触发'),
    ('g2', R_WF_AGENTS, 'M1–M4 已完成'),
    ('g3', R_WF_AGENTS, '先迁移后恢复'),
    ('g4', R_INTENT, '它是试金石，不是产品野心'),
    ('g5', R_MEMO, '系统 ≤10、日历周 ≤4、taskset 新增任务 ≤14'),
]


def read(rel: str) -> str:
    with open(os.path.join(ROOT, rel), encoding='utf-8') as f:
        return f.read()


def main() -> int:
    doc = read(DOC)
    cache = {}
    fails = []
    for cid, src, anchor in CLAIMS:
        if src not in cache:
            cache[src] = read(src)
        ok_src = anchor in cache[src]
        ok_doc = anchor in doc
        if not (ok_src and ok_doc):
            fails.append((cid, src, anchor, ok_src, ok_doc))
    print('T026 锚子串双向核对：%d 条' % len(CLAIMS))
    for cid, src, anchor, ok_src, ok_doc in fails:
        print('FAIL %-4s 源%-50s 命中%s 本档命中%s 锚=%r' % (cid, src, ok_src, ok_doc, anchor))
    print('汇总: %d 条 | PASS %d | FAIL %d' % (len(CLAIMS), len(CLAIMS) - len(fails), len(fails)))
    print('RESULT: %s' % ('PASS' if not fails else 'FAIL'))
    return 0 if not fails else 1


if __name__ == '__main__':
    sys.exit(main())
