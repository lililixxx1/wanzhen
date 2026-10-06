# T011 G1 · Lead 独立复验记录（2026-10-06，收获前后于主仓/树内执行）

- 复验者：主会话（Lead）；对象：树 t011-a 产出 18 档（基线 1d304a8）+ 收获后主仓 G1 修正。

## 1. 独立复验项（全过）

1. 脚本独立重跑（树内）：verify_claims.py → `汇总: 81 条 | PASS 81 | FAIL 0`（退出码 0）；tally_hours.py → 458/1205（与 worker 双盲值一致）。
2. 判定行锚抽查（树内 grep -n 独立命中）：t015/summary.md L42（②终态 TRIPPED）、t009/summary.md L180（吞吐 PASS）、t010/summary.md L31（渲染 PASS）、t008/matrix.md L15-16（red200 s42 终局哈希行）——与 README 引用逐字一致。
3. 抽查三档取证复看（runs/）：spot-t008-red200-th1-s42.diff 为空档（0 字节）且新跑 stdout 与归档均 208 字节；spot-bench-10k-t1.check final_hash=0xc5915d042208e267 MATCH；spot-t009-simt1.check final_hash=0x564cf46fdf191710 MATCH；REAL_EXIT 全 0。
4. 脚本源码审查（防假绿）：tally_hours.py 耗时列第 6 列解析（未转义竖线切分，兼容台账 `\|dx\|` 转义）——双盲拦截实例（首版整行搜索误抓 T010「≈92 min」，v1 输出 1157≠1205 被手工速算表拦下）证明双盲制度真实生效；verify_claims.py 双向子串核对逻辑透明无隐藏分支。
5. 纯净性：runs/changed-files.txt 18 档全部 `docs/evidence/m0/**`；主仓收获后 `git status` 仅 M taskset/t011-data-pack.md（Lead 裁决节）+ ?? docs/evidence/m0/。

## 2. G1 修正项（D3-1 补裁决，设计侧疏漏如实归因）

T007 跨预备周/W1 边界未按「跨窗口任务按实际执行日期分摊」拆分（D3 初版窗口映射整卡归 W1，派工单即错、worker 照单执行无错）。修正三处：tally_hours.py 加 SPLIT_T007_PREP=99、README §8（逐任务表/小计/判定快照行）、runs/tally.txt 追加 G1 修正段；任务卡 D3-1 留痕。快照修正：W1 1205 min/20.08 h → 1106 min/18.43 h；预备周 458 → 557 min。修正后 tally 重跑 557/1106 与手工算式（458+99；91+605+145+125+140）一致；verify 重跑 81/81 仍全过。详见 taskset/t011-data-pack.md D3-1。

## 3. worker 上报三项裁决（Lead）

1. **AGENTS.md 吞吐数字形态**（1,099,639 vs 源档 1099638.6，取整一致字面不同）：接受 worker 如实记录；处置 = T011 收口更新 AGENTS.md「当前状态」节时统一为源档全值 1,099,638.6（消除字面分歧），README §9 披露行届时一并注记。worker 禁改 AGENTS.md 正确。
2. **首条门禁误落主仓事件**（shell cwd 被宿主重置，cargo build 落主仓执行）：worker 核实零跟踪文件影响（主仓 HEAD=1d304a8 与树基线一致）属实；处置 = 事件如实留 selfcheck §5（已有）+ 台账行原因列带注，不返工不重跑（同基线构建产物无语义差异）。树内门禁仍只此一条。
3. **verify 语义边界**（只做双命中核对，不复核源档判定行 vs 原始数据再生成一致性）：确认——该层属 T012 独立复算范围，本卡规格如此，无动作。

## 4. 备注

- worker 双盲速算（预备周 150+15+18+35+110+130=458 / W1 190+605+145+125+140=1205）与 Lead 心算复核一致。
- 抽查三档选择集（D4）执行如规格，未重跑 t010/t015 重型档（复跑旁证指针已补录 README §10 末节——各档 g1-lead-recheck + 审核轮记录；审核轮 P2-2 整改，原稿误标「README §5 已注」）。
