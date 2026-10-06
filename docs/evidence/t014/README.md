# T014 · 身份决策评审材料（收官序列第 4 步）· 证据档索引

- 树：隔离树 t014-a（基线 bdffbad75dbcad1cdaad1e1cd77e3c41fc81b4b7）；生成者 worker-1；生成时刻 2026-10-06T14:26+08:00
- 任务卡：taskset/t014-identity-review.md；派工单存档：dispatches/wp-a.md（原文逐字誊写；按附录 G「只改前缀」规则替换 6 处机器路径字面量为 `<桌面>`/`<C盘Users前缀>`，映射见档首存档说明）
- 主交付：**docs/m0-identity-review.md**（单档自含立项身份决策评审材料，§0~§7）

## 文件清单

| 文件 | 说明 |
|---|---|
| verify_t014.py | 锚子串双向核对脚本（组 A~F，84 条 CLAIMS = 源档命中 ∧ 主档命中；全 PASS exit 0） |
| run_anchor_crossrepo.sh | 跨仓锚核对脚本（组 G~J，9 锚；主仓根执行版，相对路径 ../bevy-ai-workflow/；供 Lead 收获后生成 runs/anchor-crossrepo.txt） |
| runs/verify.txt | verify 正式运行留痕（命令全文 + 原始输出 + REAL_EXIT 行，自含） |
| runs/verify.stdout.txt | verify 原始 stdout/stderr 捕获（verify.txt 的来源档） |
| runs/selfcheck.md | 收尾自查四节：门禁与双盲记录 / 跨仓组 G~J 双盲表 / 三验收断言自查 / 措辞与卫生 |
| runs/hygiene.txt | 卫生检查留痕（机器路径扫描 + UTF-8 无 BOM + LF + 新建路径清单） |
| dispatches/wp-a.md | 派工单逐字存档（含本行自指；6 处机器路径字面量按附录 G 替换并登记） |

## 结论行

**T014 交付成立（worker 侧自检）**：docs/m0-identity-review.md 单档自含（§0~§6 全数字带〔源: …〕标注）+ 挂起清单 9/9 全量在列（a22 逐字引文 + 表 5-1 逐项拆解）+ 数字可溯源（verify_t014.py 组 A~F 84/84 双向 PASS + 跨仓组 G~J 9/9 人工双向双盲 PASS）——预注册预期（verify 全 PASS、REAL_EXIT=0；挂起 9/9 双命中；跨仓全命中）**未偏离**；措辞与卫生自查通过（无机器绝对路径、UTF-8 无 BOM、LF、验收级数字无「约/大概」）。本档自身新算仅三处（投入小计核对、W1 小计、③⑤ 裕度除法），算式随文给出。

## 行号偏差披露（grep -n 实测 vs 派工单标注；锚子串命中不受影响）

- a37 实际 :685（派工单 :486；:685 = 报告修订记录 V1.0 残项④；:486 为同义句，正文两处均已标注）
- b9a/b9b 实际 :158/:159（派工单 :58/:59）
- g4 实际 :134（派工单 :132）
- b14 双命中 :35/:144（正文引 :35）

## 遗留占位（Lead 收口回填，本卡禁改）

1. task-ledger.md T014 行 + taskset/README.md T014 状态列；m0-identity-review.md §2.2 表 T014 行「收口回填」（先例 = m0 README §8 T011 行）。
2. 主仓侧 runs/anchor-crossrepo.txt：Lead 收获后在主仓根执行 `bash docs/evidence/t014/run_anchor_crossrepo.sh > docs/evidence/t014/runs/anchor-crossrepo.txt 2>&1` 生成（预期 9/9 PASS、REAL_EXIT=0）。
3. 审核状态（m0-identity-review.md 文档头 + 本 README 结论行）：待审（plan-code-reviewer 完整轮）。

## 复跑指引

```
# 树根/主仓根：
python docs/evidence/t014/verify_t014.py            # 预期：汇总: 84 条 | PASS 84 | FAIL 0，exit 0
# 主仓根（收获后，与 bevy-ai-workflow 同级 checkout）：
bash docs/evidence/t014/run_anchor_crossrepo.sh     # 预期：汇总: 9 条 | PASS 9 | FAIL 0，exit 0
```
