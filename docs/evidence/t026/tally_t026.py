# -*- coding: utf-8 -*-
"""T026 · 试金石成效量化复算脚本（docs/evidence/t026/tally_t026.py）

口径：台账 task-ledger.md 25 卡（T001~T025）逐卡转写 → 转写核对（耗时串在本卡台账行命中）
→ 窗口拆分（投入口径 = 按实际执行日期：预备周 557 / W1 3330，T007 拆 99/91 沿 T011 D3-1 补裁决）
→ 一次通过/返工/审核捕获分布合计 → 与既有归档锚值一致性断言。

既有锚（本脚本必须复现，任一不符 exit 1）：
  预备周 557 min / W1 3330 min（AGENTS.md W1 快照终值）/ 总 3887 min = 64.78 h
  / M5 段（T017~T025）1619 min（docs/evidence/m5/README.md §8「≈1474 min」为 T017~T024 八卡，
   +T025 145 = 1619）/ M0 段（T001~T016）2268 min。
审核捕获计数（轻量轮+完整轮的 P0/P1/P2；B/C-1/Important/Minor 单列如实不折算）：
  P0=4 / P1=12 / P2=16；另 B(Critical)×2（T013）+ C-1×2（T014）+ Important×1（T019）+ Minor×7。

运行（仓库根）：python docs/evidence/t026/tally_t026.py
输出：逐卡表 + 分段汇总 + 断言结果；全过 exit 0。
"""
import io
import sys

LEDGER = 'task-ledger.md'

# (卡号, 耗时min, 窗口, 一次通过严格「是」, 返工次数)
# 窗口：PREP=预备周（10-04 前）/ W1（10-05~10-11，实际执行日期口径）。
# T007 拆分：99 归 PREP / 91 归 W1（源：task-ledger.md:36 T011 行 D3-1 补裁决 +
# docs/evidence/m0/README.md §8 权威表）。T002~T006 周位标 W1 但实际执行于预备周窗口
# （源：taskset/README.md 各行「预备周窗口」标注）→ 归 PREP。
CARDS = [
    ('T001', 150, 'PREP', False, 2),
    ('T002', 15, 'PREP', False, 1),
    ('T003', 18, 'PREP', False, 2),
    ('T004', 35, 'PREP', False, 1),
    ('T005', 110, 'PREP', True, 0),
    ('T006', 130, 'PREP', False, 1),
    ('T007', 99, 'PREP', False, 3),   # 拆分前半（预备周段）
    ('T007', 91, 'W1', False, 0),     # 拆分后半（W1 段）；卡级字段以整卡计（见下）
    ('T008', 605, 'W1', True, 0),
    ('T015', 125, 'W1', False, 1),
    ('T009', 145, 'W1', False, 1),
    ('T010', 140, 'W1', False, 1),
    ('T011', 140, 'W1', False, 1),
    ('T012', 150, 'W1', False, 1),
    ('T013', 110, 'W1', False, 1),
    ('T014', 165, 'W1', False, 1),
    ('T016', 40, 'W1', True, 0),
    ('T017', 75, 'W1', True, 0),
    ('T018', 216, 'W1', False, 1),
    ('T021', 180, 'W1', False, 0),
    ('T020', 200, 'W1', False, 1),
    ('T022', 115, 'W1', False, 0),
    ('T023', 165, 'W1', False, 0),
    ('T019', 378, 'W1', False, 2),
    ('T024', 145, 'W1', False, 1),
    ('T025', 145, 'W1', False, 1),
]

# 卡级字段（一次通过/返工按整卡计，与台账行一致；T007 整卡 = 否/3）
CARD_LEVEL = {
    'T001': (False, 2), 'T002': (False, 1), 'T003': (False, 2), 'T004': (False, 1),
    'T005': (True, 0), 'T006': (False, 1), 'T007': (False, 3), 'T008': (True, 0),
    'T015': (False, 1), 'T009': (False, 1), 'T010': (False, 1), 'T011': (False, 1),
    'T012': (False, 1), 'T013': (False, 1), 'T014': (False, 1), 'T016': (True, 0),
    'T017': (True, 0), 'T018': (False, 1), 'T021': (False, 0), 'T020': (False, 1),
    'T022': (False, 0), 'T023': (False, 0), 'T019': (False, 2), 'T024': (False, 1),
    'T025': (False, 1),
}

# 整卡耗时（转写核对用：台账行须命中「≈N min」）
MINUTES_WHOLE = {
    'T001': 150, 'T002': 15, 'T003': 18, 'T004': 35, 'T005': 110, 'T006': 130,
    'T007': 190, 'T008': 605, 'T015': 125, 'T009': 145, 'T010': 140, 'T011': 140,
    'T012': 150, 'T013': 110, 'T014': 165, 'T016': 40, 'T017': 75, 'T018': 216,
    'T021': 180, 'T020': 200, 'T022': 115, 'T023': 165, 'T019': 378, 'T024': 145,
    'T025': 145,
}

# 审核轮捕获（轻量轮/完整轮的分级缺陷 + 行级转写核对锚串）
# (卡, 轮级, P0, P1, P2, 台账行内锚串列表)
REVIEW = [
    ('T007', '完整轮', 0, 3, 0, ['P1×3']),
    ('T008', '完整轮', 0, 1, 5, ['P1×1', 'P2×5']),
    ('T009', '完整轮', 0, 0, 0, ['Minor×4']),
    ('T010', '完整轮', 0, 0, 0, ['完整轮通过（覆盖率 100%']),
    ('T011', '完整轮', 0, 0, 2, ['P2×2']),
    ('T012', '完整轮', 0, 1, 2, ['P0=0/P1×1/P2×2']),
    ('T013', '完整轮', 0, 0, 0, ['B1×1 + B2×1']),
    ('T014', '完整轮', 0, 0, 0, ['C-1×2', 'Minor×1']),
    ('T015', '完整轮', 0, 0, 1, ['P1=0；P2×1']),
    ('T018', '轻量轮', 1, 1, 3, ['P0×1/P1×1/P2×3']),
    ('T019', '轻量轮', 1, 1, 0, ['P0（六兵种', 'P1 截图旧文件']),
    ('T020', '完整轮', 0, 2, 1, ['P0=0/P1×2/P2×1']),
    ('T021', '轻量轮', 0, 0, 2, ['P0=0/P1=0/P2×2']),
    ('T022', '轻量轮', 0, 0, 0, ['零代码缺陷']),
    ('T023', '轻量轮', 0, 0, 0, ['通过零 P0/P1/P2']),
    ('T024', '完整轮', 0, 1, 0, ['P1×1 = 路径清洗漏网']),
    ('T025', '完整轮', 2, 2, 0, ['不通过 P0×2+P1×2']),
]
# 单列等级（不折算入 P 计数）：T013 B×2 Critical + E21 派工单笔误×1；
# T014 C-1×2；T019 Important×1（pid 终处置）；Minor 合计 = T009×4 + T011×2 + T014×1 = 7。
SINGLE_NOTE = 'B(Critical)×2[T013] + C-1×2[T014] + Important×1[T019] + Minor×7[T009×4/T011×2/T014×1]'
# 复验/复审跟进轮（审核事件超出首跑的追加轮）
RE_REVIEW = ['T018 复验轮（整改后复验通过）', 'T019 修后复审（+ Important×1 终处置）', 'T025 修后复审（P0=0 P1=0）']
# 无 plan-code-reviewer 轮的卡（如实）：T001~T006（三层分级 2026-10-05 定案前，主会话门禁复核）
# + T016/T017（owner 决策卡，决策不可代评故无审核轮——台账行原文）。
NO_REVIEW = ['T001~T006（主会话门禁复核，分级制度定案前）', 'T016/T017（owner 决策卡免审，台账行原文）']


def main() -> int:
    out = io.StringIO()
    fails = []

    with open(LEDGER, encoding='utf-8') as f:
        lines = f.read().splitlines()
    row_of = {}
    for i, ln in enumerate(lines, 1):
        for cid in MINUTES_WHOLE:
            if ln.startswith('| ' + cid + ' |'):
                row_of[cid] = (i, ln)

    # 转写核对：整卡耗时串 + 审核锚串须在本卡台账行命中
    for cid, minutes in sorted(MINUTES_WHOLE.items()):
        if cid not in row_of:
            fails.append('台账行缺失: %s' % cid)
            continue
        _, ln = row_of[cid]
        if ('≈%d min' % minutes) not in ln:
            fails.append('耗时转写不符: %s 期望 ≈%d min' % (cid, minutes))
    for cid, _, _, _, _, anchors in REVIEW:
        if cid not in row_of:
            fails.append('台账行缺失(审核): %s' % cid)
            continue
        _, ln = row_of[cid]
        for a in anchors:
            if a not in ln:
                fails.append('审核锚串未命中: %s ← %r' % (cid, a))

    # 窗口/阶段汇总
    prep = sum(m for _, m, w, _, _ in CARDS if w == 'PREP')
    w1 = sum(m for _, m, w, _, _ in CARDS if w == 'W1')
    total = prep + w1
    m5 = sum(MINUTES_WHOLE[c] for c in MINUTES_WHOLE if c >= 'T017')
    m0 = total - m5

    def expect(name, got, want):
        if got != want:
            fails.append('%s = %r ≠ 锚值 %r' % (name, got, want))

    expect('预备周(min)', prep, 557)
    expect('W1(min)', w1, 3330)
    expect('总投入(min)', total, 3887)
    expect('M5段(min)', m5, 1619)
    expect('M0段(min)', m0, 2268)

    # 卡级统计
    n = len(CARD_LEVEL)
    strict_pass = sum(1 for p, _ in CARD_LEVEL.values() if p)
    zero_rework = sum(1 for _, r in CARD_LEVEL.values() if r == 0)
    rework_sum = sum(r for _, r in CARD_LEVEL.values())
    expect('卡数', n, 25)
    expect('一次通过严格是', strict_pass, 4)
    expect('返工=0 卡数', zero_rework, 7)
    expect('返工轮合计', rework_sum, 23)

    # 审核捕获合计
    p0 = sum(r[2] for r in REVIEW)
    p1 = sum(r[3] for r in REVIEW)
    p2 = sum(r[4] for r in REVIEW)
    n_full = sum(1 for r in REVIEW if r[1] == '完整轮')
    n_light = sum(1 for r in REVIEW if r[1] == '轻量轮')
    expect('P0 合计', p0, 4)
    expect('P1 合计', p1, 12)
    expect('P2 合计', p2, 16)
    expect('完整轮卡数', n_full, 12)
    expect('轻量轮卡数', n_light, 5)

    # 输出
    out.write('T026 量化复算（源 = task-ledger.md 逐卡转写 + 转写核对）\n')
    out.write('=' * 72 + '\n')
    out.write('%-6s %8s %-6s %-8s %s\n' % ('卡', '耗时min', '窗口', '一次通过', '返工'))
    for cid, m, w, _, _ in CARDS:
        flag = '是' if CARD_LEVEL[cid][0] else '否'
        if cid == 'T007':
            out.write('%-6s %8d %-6s %-8s %d  （99/91 拆分：本行 %s 段）\n'
                      % (cid, m, w, '—', 0, 'PREP' if w == 'PREP' else 'W1'))
        else:
            out.write('%-6s %8d %-6s %-8s %d\n' % (cid, m, w, flag, CARD_LEVEL[cid][1]))
    out.write('-' * 72 + '\n')
    out.write('窗口：预备周 %d min / W1 %d min / 总 %d min = %.2f h\n' % (prep, w1, total, total / 60.0))
    out.write('阶段：M0 段（T001~T016）%d min = %.2f h / M5 段（T017~T025）%d min = %.2f h\n'
              % (m0, m0 / 60.0, m5, m5 / 60.0))
    out.write('卡级：%d 卡 / 一次通过（严格「是」）%d / 返工=0 %d / 返工轮合计 %d\n'
              % (n, strict_pass, zero_rework, rework_sum))
    out.write('审核：%d 轮首跑（完整轮 %d + 轻量轮 %d）+ 复验/复审跟进 %d 轮\n'
              % (n_full + n_light, n_full, n_light, len(RE_REVIEW)))
    out.write('捕获：P0=%d / P1=%d / P2=%d；单列等级：\n  %s\n' % (p0, p1, p2, SINGLE_NOTE))
    for r in RE_REVIEW:
        out.write('  · %s\n' % r)
    out.write('无审核轮卡（如实）：\n')
    for r in NO_REVIEW:
        out.write('  · %s\n' % r)
    out.write('=' * 72 + '\n')

    if fails:
        out.write('断言失败 %d 项：\n' % len(fails))
        for x in fails:
            out.write('  FAIL %s\n' % x)
        out.write('RESULT: FAIL\n')
    else:
        out.write('全部断言通过（预备周557/W1 3330/总3887/M5 1619/M0 2268/25卡/4严格一次通过/23返工轮/P0=4/P1=12/P2=16/完整轮12/轻量轮5）\n')
        out.write('RESULT: PASS\n')

    text = out.getvalue()
    sys.stdout.write(text)
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main())
