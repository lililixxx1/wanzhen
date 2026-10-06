# -*- coding: utf-8 -*-
"""T014 · 锚子串双向核对脚本（docs/evidence/t014/verify_t014.py）

模式沿 docs/evidence/m0/verify_claims.py（T011 先例）：CLAIMS = (id, source_relpath, anchor)，
双向核对 = anchor 在源档命中 且 anchor 在本档（docs/m0-identity-review.md）命中，任一不中即 FAIL。
全 PASS exit 0；任一 FAIL exit 1。

组划分（源档 = 仓库内相对路径；本脚本在仓库根执行）：
  组 A 报告       docs/万阵-游戏前期策划报告.html      a1~a49, a50a~a50d, a52
  组 B m0 索引    docs/evidence/m0/README.md           b1~b15, b17（无 b16）
  组 C T012 报告  docs/evidence/m0/review/report.md    c1~c6
  组 D 台账       task-ledger.md                        d1~d2
  组 E AGENTS     AGENTS.md                             e1~e3
  组 F taskset    taskset/README.md                     f1
组 G~J（跨仓 bevy-ai-workflow：意向文档 / m4-game-selection.md / pre-window-plan.md / AGENTS.md）
  不入本脚本（隔离树内无兄弟仓）——核对 = docs/evidence/t014/run_anchor_crossrepo.sh
  （主仓根执行，Lead 收获后生成留痕档 runs/anchor-crossrepo.txt）+ worker 人工双盲（selfcheck.md §2）。

运行（仓库根）：python docs/evidence/t014/verify_t014.py
输出：逐条「id | PASS/FAIL」+ 尾行「汇总: N 条 | PASS n | FAIL m」。
"""
import os
import sys

DOC_RELPATH = 'docs/m0-identity-review.md'

R_REPORT = 'docs/万阵-游戏前期策划报告.html'
R_M0 = 'docs/evidence/m0/README.md'
R_T012 = 'docs/evidence/m0/review/report.md'
R_LEDGER = 'task-ledger.md'
R_AGENTS = 'AGENTS.md'
R_TASKSET = 'taskset/README.md'

CLAIMS = [
    # ── 组 A · 报告 V1.0 ──
    ('a1', R_REPORT, '0.060651 / 0.072196 / 0.070617 / 0.108681 µs（1k/5k/10k/50k 模拟单位）'),
    ('a2', R_REPORT, '12 线程实测 1.001913×、16 线程外推 1.112480×'),
    ('a3', R_REPORT, 'C(100000) ÷ speedup16（50k 档）= 17.532997 ms / tick'),
    ('a4', R_REPORT, '保留（≤22ms 极限预算；成本拟合 O(N²) 项已消除、近线性）'),
    ('a5', R_REPORT, '降规模口径 1,099,638.6 场/小时 @12 线程；16 线程外推 1,405,967.6 场/小时'),
    ('a6', R_REPORT, '24 独立进程矩阵（2 规模 × 3 种子 × 4 线程档）+ 3 跨进程加样 + 2 跨卡锚局 = 29 局'),
    ('a7', R_REPORT, '线程间 / 进程间 / 优化前后逐位一致（含战斗段中间采样点）'),
    ('a8', R_REPORT, '胶囊体灰盒 1 万同屏：avg_fps=333.15、1% low=202.48（物理窗口 2400×1350，保守方向）'),
    ('a9', R_REPORT, '50k 单位 670.6s（≈11.2min ≥10min）长跑稳态工作集 6.8 MiB'),
    ('a10', R_REPORT, 'Amdahl 拟合可并行占比仅 ≈8.9%'),
    ('a11', R_REPORT, 'avg 329.33 / 1% low 177.95 PASS 佐证'),
    ('a12', R_REPORT, '截至 2026-10-06 T012 收口累计 23.27 h'),
    ('a13', R_REPORT, '第 1 周第 2 日即完成全部六项验收实测与独立复算，无进度落后'),
    ('a14', R_REPORT, 'M0 数据与独立复算已产出，身份评审输入就绪（7.2 Q5）'),
    ('a15', R_REPORT, '所需加速比 = 单位数 × 单线程每单位每 tick 成本 ÷ 每帧模拟预算'),
    ('a16', R_REPORT, '则「万人常态」目标整体下调一档并在下一版重写 4.1'),
    ('a17', R_REPORT, '极限十万 17.532997ms ≤ 22ms 保留'),
    ('a18', R_REPORT, '渲染腿后果：⑤ 灰盒 1 万同屏 1% low'),
    ('a19', R_REPORT, '即降档为 20h/周、周期拉长一倍，验收阈值不变只改日历'),
    ('a20', R_REPORT, '等效产能约 1.5–2 人力'),
    ('a21', R_REPORT, '商业产品（本报告 26 周全案）/ 新试金石（按既有工作流选品标准收缩范围与规模）/ 终止'),
    ('a22', R_REPORT, '商业目标与成功标准、EA 前预算（V0.7 移出的成本预算表届时恢复）、开源边界细则、Steam 主体与法务、外部评审与试玩人选、Bevy 0.20 处置（5.3）、基准机 B 获取方式与费用（表 6-0）、UGC/Mod 内容审核与授权条款（5.4 收录流水线与 5.5 方案上传的版权归属与审核口径）、每日同阵的用户身份基座（Steam 凭据 vs 自建账号，5.5）'),
    ('a23', R_REPORT, '复议触发 = M1 生死题不达标（先按 R1 玩法级收窄重测一轮、仍败则重开本条'),
    ('a24', R_REPORT, 'M2 可读性盲测不达标、R5 联机判据触发、0.20 窗口开启'),
    ('a25', R_REPORT, '同题解谜练习模式（每日同阵的教学形态，不设排名，见 3.3）'),
    ('a26', R_REPORT, '同屏 2,000 渲染单位 @60fps、1% low ≥ 45（基准机 A，画质预设 M）；内部试玩「想再来一局」比例 ≥50%（n≥8）；打击感试玩评分 ≥3.5/5（量表见 08.3）'),
    ('a27', R_REPORT, '同屏 5,000 渲染单位 @60fps、1% low ≥ 45（基准机 B）；常规对局回放码 ≤ 2 KB'),
    ('a28', R_REPORT, '外包 1,500–4,000 元/个；M1 先试产 2 个样件校准单件工时，M2 批量完形'),
    ('a29', R_REPORT, '外包套件 1–3.5 万；天候为纯参数变化，不计新资产'),
    ('a30', R_REPORT, '外包 0.5–1.5 万或引擎内置程序化'),
    ('a31', R_REPORT, '随机 5 帧截图 × ≥ 3 名判卷人，可辨兵种 ≥ 80%'),
    ('a32', R_REPORT, 'M1「想再来一局」≥50% + 打击感试玩评分 ≥3.5/5'),
    ('a33', R_REPORT, 'M2 可读性盲测 ≥70%（未接触者看 3 分钟战局复述战况）'),
    ('a34', R_REPORT, '身份=终止时，模拟核心与实验场仍按开源实验向沉淀发布，M0 仓转归档、不并入工作流仓历史'),
    ('a35', R_REPORT, '悬挂期支出上限 = M0 三周工时 + 文档工时'),
    ('a36', R_REPORT, 'n=2,000 对真实发生率 0.2% 的检出率约 76%'),
    ('a37', R_REPORT, '胜率基准需 ≥400 场/周'),
    ('a38', R_REPORT, '16 GB 下限档须在 M2 前补一次实测'),
    ('a39', R_REPORT, '跨机档自 M1 起纳入持续 CI'),
    ('a40', R_REPORT, '灰盒无动画/LOD/分层渲染，指标高于 M2/M3 属预期，不作外推依据'),
    ('a41', R_REPORT, '以宽松许可（MIT / Apache-2.0）独立开源'),
    ('a42', R_REPORT, '获取方式与费用归属（自购/租用/云端）为 Q5 挂起项，M2 前锁定'),
    ('a43', R_REPORT, 'AI 池扩档后须重测'),
    ('a44', R_REPORT, '每方 100 模拟单位（共 200）、单局 ≤ 60s（≤ 1,800 ticks @30Hz）'),
    ('a45', R_REPORT, '316.57 亿元（2026 上半年，同比 +36.01%；检索日 2026-10-06）'),
    ('a46', R_REPORT, '月活超 5 亿保持稳定、开发者超 50 万'),
    ('a47', R_REPORT, '6.54 亿、同比 +32.7%'),
    ('a48', R_REPORT, '以 Q5 拍板为准'),
    ('a49', R_REPORT, '降规模口径 1,099,638.6 场/小时 @12 线程、16 线程外推 1,405,967.6 场/小时'),
    ('a50a', R_REPORT, '第 4–8 周'),
    ('a50b', R_REPORT, '第 9–14 周'),
    ('a50c', R_REPORT, '第 15–20 周'),
    ('a50d', R_REPORT, '第 21–26 周'),
    ('a52', R_REPORT, 'M1/M2/M3 各含一次模拟成本与吞吐复测'),
    # ── 组 B · m0 索引 ──
    ('b1', R_M0, 'PASS（T015 后；T007 曾 4/4 TRIPPED——R2 轮一响应）'),
    ('b2', R_M0, '| 50000 | 0.108681 | 0.679255 | 承诺线内（≤2×） | 0.247002 | 承诺线内（≤2×） |'),
    ('b3', R_M0, '| 10000 | 0.070617 | 0.088271 | 承诺线内（≤2×） | 0.032099 | 承诺线内（≤2×） |'),
    ('b4', R_M0, 'us_per_unit_tick=4.012138'),
    ('b5', R_M0, '12 线程实测加速比=7.221411 | 止损判定（<4×）: OK'),
    ('b6', R_M0, '12 线程实测加速比=1.001913 | 止损判定（<4×）: TRIPPED'),
    ('b7', R_M0, 'C(100000) ÷ speedup16 = 17.532997 ms vs 22 ms 极限预算 → [保留]'),
    ('b8', R_M0, 'C(100000) ÷ speedup16 = 3907.133373 ms vs 22 ms 极限预算 → [超界：极限十万目标不保留]'),
    ('b9a', R_M0, 'b=3.997072319'),
    ('b9b', R_M0, 'b=0.000918316'),
    ('b10a', R_M0, 'T007 基线 4.012/19.847/41.010/200.626'),
    ('b10b', R_M0, '距止损线 2µs 裕度 ≥18×'),
    ('b11', R_M0, 'W1 终判未到期（窗口 2026-10-05~10-11，截至 T011 收口 2026-10-06 累计 20.77 h < 30h）——不可终判'),
    ('b12', R_M0, '预备周小计（T001~T006 共 458 + T007 的 10-04 段 99，不计 W1）：458+99 = 557 min = 9.28 h'),
    ('b13', R_M0, '91+605+145+125+140+140 = 1246 min = 20.77 h'),
    ('b14', R_M0, '按字面呈现 TRIPPED，不放行不粉饰'),
    ('b15', R_M0, 'W1 收口（2026-10-11 窗口结束）后由 Lead 回写终判行'),
    ('b17', R_M0, '对角 6 格 z=−10.000、红 100%/蓝 0%/Draw 0%——apply 索引序先手语义'),
    # ── 组 C · T012 独立复算报告 ──
    ('c1', R_T012, 'M0 六验收归档数据与判定独立复算全部逐位通过、零 P0'),
    ('c2', R_T012, '验收⑤ 的新鲜复测在当前桌面环境不可复现'),
    ('c3', R_T012, '验收⑤ 新鲜复测不可复现且判定行 FAIL，预注册「PASS」期望偏离'),
    ('c4', R_T012, '漂移 -79%~-89%'),
    ('c5', R_T012, '归档 333.15/202.48 三实现复算逐位 PASS（自洽）'),
    ('c6', R_T012, '557 min（预备周）/ 1246 min = 20.77h（W1）双 MATCH'),
    # ── 组 D · 任务台账 ──
    ('d1', R_LEDGER, '≈150 min（主会话前置/裁决 D1~D12'),
    ('d2', R_LEDGER, '≈110 min（主会话侦察/owner 三裁决问询'),
    # ── 组 E · AGENTS.md ──
    ('e1', R_AGENTS, 'W1 累计 1506 min = 25.10 h'),
    ('e2', R_AGENTS, '台账挂钟全计（主会话+worker+审核轮，含等待）'),
    ('e3', R_AGENTS, '降档若触发即如实接受；推进按依赖链自然节奏，不为凑时数灌水'),
    # ── 组 F · taskset 索引 ──
    ('f1', R_TASKSET, '裕度 5.55×/4.5×'),
]


def read_text(relpath):
    with open(relpath, 'r', encoding='utf-8', newline='') as f:
        return f.read()


def main():
    try:
        sys.stdout.reconfigure(encoding='utf-8', newline='\n')
    except Exception:
        pass

    doc_text = read_text(DOC_RELPATH)
    source_cache = {}
    n_pass = 0
    n_fail = 0
    for cid, src, anchor in CLAIMS:
        if src not in source_cache:
            source_cache[src] = read_text(src)
        ok_src = anchor in source_cache[src]
        ok_doc = anchor in doc_text
        ok = ok_src and ok_doc
        if ok:
            n_pass += 1
        else:
            n_fail += 1
        if not ok:
            detail = []
            if not ok_src:
                detail.append('源档未命中')
            if not ok_doc:
                detail.append('本档未命中')
            print('%s | FAIL | %s | %s' % (cid, src, '+'.join(detail)))
        else:
            print('%s | PASS' % cid)
    print('汇总: %d 条 | PASS %d | FAIL %d' % (len(CLAIMS), n_pass, n_fail))
    if n_fail > 0:
        sys.exit(1)
    sys.exit(0)


if __name__ == '__main__':
    main()
