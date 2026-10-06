# T014 · selfcheck（收尾自查四节）

- 卡：T014 身份决策评审材料（派工单 WP-A）；树：隔离树 t014-a（基线 bdffbad75dbcad1cdaad1e1cd77e3c41fc81b4b7）
- 生成时刻：2026-10-06T14:4x+08:00；生成者 worker-1

## §1 门禁与双盲记录（G0~G2 + 行号全量核对）

| 项 | 结果 |
|---|---|
| G0 附录 B.1/B.2/C/G + 派工单全文已读 | 完成（主仓 team-prompt/PROJECT-APPENDIX.md，只读） |
| G1 树内 `ls docs/evidence` | m0/ t002/ t003/ t004/ t005/ t006/ t007/ t008/ t009/ t010/ t013/ t015/ + 两份 2026-09-30 cargo-check 留痕在档——基线导出完整 |
| G2 双盲对拍（12 锚，A~J 组各 ≥1） | a1/a22/b2/b11/c1/d1/e1/f1/g1/h1/i1/j1 全 PASS，行号与派工单一致（a1=532 a22=572 b2=80 b11=413 c1=11 d1=38 e1=12 f1=20 g1=13 h1=41 i1=6 j1=12） |
| 行号全量核对（一次性 sweep，非交付物） | 组 A~F 84 锚源档侧全命中、0 MISS；三处与派工单标注偏差如实披露：a37 实际 ：685（派工单 ：486；:685 = 报告修订记录 V1.0 残项④ 句「胜率基准需 ≥400 场/周」；:486 为同义句「承担胜率基准结论需 ≥400 场/周（±5pp）」，正文两处均已标注）、b9a/b9b 实际 ：158/:159（派工单 ：58/:59）、b14 双命中 :35/:144（引用 :35） |
| verify_t014.py 正式运行 | 「汇总: 84 条 | PASS 84 | FAIL 0」、REAL_EXIT=0（runs/verify.txt；预注册预期全部 PASS、REAL_EXIT=0——未偏离） |

## §2 跨仓组 G~J 逐条人工双盲结果（9/9 PASS；源档侧 grep -n + 本档侧 grep -c 双向闭合）

| id | 源档（bevy-ai-workflow 仓） | 源档侧实际行 | 源档侧 | 本档侧 | 备注 |
|---|---|---|---|---|---|
| g1 | Bevy-AI开发意向文档.md | 13 | PASS | PASS | 与派工单标注一致；`**` 为源档 markdown 加粗符号，本档逐字保留 |
| g2 | Bevy-AI开发意向文档.md | 132 | PASS | PASS | 与派工单标注一致 |
| g3 | Bevy-AI开发意向文档.md | 137 | PASS | PASS | 与派工单标注一致 |
| g4 | Bevy-AI开发意向文档.md | 134 | PASS | PASS | **行号偏差**：派工单标 ：132，实际命中 ：134——锚子串命中不受影响，正文按实际行标注 |
| g5 | Bevy-AI开发意向文档.md | 140 | PASS | PASS | 与派工单标注一致 |
| h1 | docs/m4-game-selection.md | 41 | PASS | PASS | 与派工单标注一致 |
| i1 | docs/pre-window-plan.md | 6 | PASS | PASS | 与派工单标注一致 |
| j1 | AGENTS.md（wf 仓） | 12 | PASS | PASS | 与派工单标注一致 |
| j2 | AGENTS.md（wf 仓） | 12 | PASS | PASS | 与派工单标注一致 |

主仓 G1 留痕档 = runs/anchor-crossrepo.txt，由 Lead 收获后在主仓根执行 run_anchor_crossrepo.sh 生成（预期 9/9 PASS、REAL_EXIT=0）。

## §3 三条验收断言自查（任务卡「验收断言」节）

1. **单档自含**：PASS——§0~§6 全部数字带〔源: …〕标注（仓库相对路径:行号；跨仓 = bevy-ai-workflow 仓 · 文件:行号）；三选项量化依据全部落在 §1/§4 已溯源数字内；评审讨论不需打开其他档案即可进行，证据档仅为深挖指针。
2. **挂起清单全量**：PASS——表 5-1 九行 = Q5 原文（a22）逐项拆解；a22 全文在 §5 逐字引出（组 A verify 双命中），九个「Q5 原文片段」均为该引文之子串；核对关键词（商业目标与成功标准 / EA 前预算 / 开源边界细则 / Steam 主体与法务 / 外部评审与试玩人选 / Bevy 0.20 处置 / 基准机 B 获取 / UGC-Mod 审核授权 / 用户身份基座）9/9 在列。
3. **数字可溯源**：PASS——verify_t014.py 组 A~F 84 条双向子串核对全 PASS（REAL_EXIT=0）+ 跨仓组 G~J 9 条人工双向双盲全 PASS；正文全部「源: 路径:行号」经 grep -n 实测（§1 行号全量核对）。

## §4 措辞与卫生

- **无机器绝对路径**：对 docs/m0-identity-review.md 与 docs/evidence/t014/ 全树做 Windows 用户目录前缀固定串扫描（模式 = 盘符冒号紧跟反斜杠 Users 形，bash 单引号拼接写法 `'C:''\Users'`，等价于派工单 §5 所记扫描命令），零命中（留痕 = runs/hygiene.txt，扫描 grep REAL_EXIT=1 = 无命中语义）；跨仓引用一律相对表述（「bevy-ai-workflow 仓 · 文件:行号」「../bevy-ai-workflow/」）。
- **派工单存档的机器路径前缀处置**：dispatches/wp-a.md 原文含 6 处机器路径字面量（5 处完整路径前缀「盘符至 Desktop」+ 1 处 §5 卫生扫描命令内嵌的「盘符冒号+反斜杠 Users」模式字面量——原文逐字存档将使本卡预注册的零命中扫描自体命中），按附录 G 可公开态「只改前缀、命中行内容原样」规则分别替换为 `<桌面>` 与 `<C盘Users前缀>`（T013 build-sim.txt `<repo-root>` 先例），替换映射、处数与等价可执行命令已在 wp-a.md 存档说明登记；本项扫描在该替换后执行。
- **行尾与编码**：docs/m0-identity-review.md + docs/evidence/t014/** 全部 UTF-8 无 BOM、LF 行尾（python open(..., encoding='utf-8', newline='\n') 规范 + 字节级复检，留痕 = runs/hygiene.txt）。
- **措辞**：验收级数字无「约/大概」；「≈」仅出现在台账投入耗时口径（§2）与引用原文内（源档自带字符）；本档自身新算仅三处（预备周小计核对、W1 小计、③⑤ 裕度除法），均给出算式与源。
- **范围纪律**：仅新建 7 项交付物路径（docs/m0-identity-review.md + docs/evidence/t014/ 下 verify_t014.py / run_anchor_crossrepo.sh / runs/verify.txt / runs/verify.stdout.txt / runs/selfcheck.md / README.md / dispatches/wp-a.md / runs/hygiene.txt）；树内既有文件零改动；主仓零写入；wf 仓零写入。
- **遗留占位**：task-ledger.md T014 行与 taskset/README.md T014 状态 = Lead 收口回填（本卡禁改既有文件）；m0-identity-review.md §2.2 表 T014 行「收口回填」同。
