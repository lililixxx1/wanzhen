# T012 · Lead G1 复验档（对 plan-code-reviewer 完整轮产出的批次复验）

- 复验时刻：2026-10-06 上午（reviewer 第二轮收口后）；复验者 = Lead（主会话）。
- 复验对象：docs/evidence/m0/review/report.md + runs/（-rv2 体系 44 档 + 前轮中断遗留档）。
- 结论：**G1 通过**——报告五字段声明与原始留痕逐项相符，抽查复跑绿，零越权改动。

## 1. 复验命令与退出码（单独整句执行）

| # | 检查 | 命令要点 | 退出码 | 结果 |
|---|---|---|---|---|
| 1 | git 卫生（禁改既有文件） | `git status --short` + `git diff --stat` | 0 | 仅 taskset/t012-independent-recheck.md（Lead 自改）+ review/ 新增；reviewer 零越权 |
| 2 | 独立抽查复跑 1a（bit-exact，免疫负载） | `./target/release/sim.exe --comp "shieldman:17,…,militia:16" --ticks 1800 --hash-samples 0,450,900,1350 --threads 1 --seed 42` → diff vs t008 归档 stdout | SIM_EXIT=0 / DIFF_EXIT=0 | 逐字节一致（与 reviewer 1a 声明相符） |
| 3 | bench 四配置哈希/us 抽验 | python 解析 runs/bench-2{a,b,c,d}-rv2.stdout | 0 | 4/4 final_hash 零容差命中（0xc5915d042208e267 ×2 / 0x022c5abdfae119dc ×2）；us 0.0811223/0.0719681/0.1206226/0.1437736 与报告 §3 一致 |
| 4 | 1c 黄金交叉抽验 | `tail -1 runs/step1-1c-rv2.stdout` | 0 | `final_hash=0x564cf46fdf191710` 命中 |
| 5 | 吞吐复测抽验 | `cat runs/throughput-recompute-rv2b.txt` | 0 | gph own=864,409.7=json 字段；-21.39% 带内；hash_xor 0xd1b28b373f7791e3 与 t009 归档逐位一致 |
| 6 | verify_claims 重跑抽验 | `tail -2 runs/verify-claims-rv2.txt` | 0 | `汇总: 81 条 | PASS 81 | FAIL 0` REAL_EXIT=0 |
| 7 | ⑤ P1-1 证据抽验 | `cat runs/render-recompute-rv2.txt` + `render-t10000-rv2/t10000/meta.json` | 0 | 三样本 avg 64.1002/59.4140/59.4301、1% low 22.1551/23.2732/23.8714（漂移 -79%~-89% 全超带、加样 2 次在档）；meta `window_resolution_actual="0x0"` 异常实锤 |
| 8 | P2-1 证据抽验 | `sed -n '95,110p' runs/recompute-6-all-rv2.txt` | 0 | t007 ③：用 50k 档 10.249990 → 3907.133373 ms MATCH 归档；派工单锚值变体 9.416188 → 4253.109497 ms——**派工单锚值错误成立（Lead 归因）** |
| 9 | 5a「仅 argv 行」声明抽验 | `wc -l` + head + `cat runs/t008-matrix-regen-rv2.downgrade.txt` | 0 | 实质差异仅第 3 行生成命令自述行；tail≥4 cmp=0 判定内容逐字节一致——声明属实 |
| 10 | 遗留档时间线抽验 | `ls -la --time-style=+%H:%M`（无 -rv2 档） | 0 | precheck-0.txt 10:04 / render-t10000.stdout 10:14（= 被暂停中止的第一轮）；step0-git-head.txt 10:42（= 第二轮起点）——N-1 叙述吻合 |

## 2. G1 裁定

- 报告数值与本轮抽验全部逐位相符；抽查复跑（项 2）绿；覆盖与留痕规范符合附录 G（REAL_EXIT 齐备、零机器绝对路径抽查通过）。
- P1-1（⑤ 带载不可复现 + window 0x0）证据成立；处置采纳 = 条件账 C-1（关账闸门 T013，见任务卡条件账节）——当前窗口带载（SolidWorks/AutoCAD 等），补测不可行，不强行量测。
- P2-1/P2-2（派工单锚值/行号）成立、归因 Lead 派工单 → 勘误已追加至 dispatch-rv.md 修正留痕节（本提交闭环）；B.2④ 同类计数如实 +1。
- N-1 遗留档处置裁定：**保留**（已被报告 §2.2 引用为旁证且降级，删除将孤儿化引用；可公开态无碍）。
- 可公开态净化留痕（沿 T003/T007 P1-2 先例 = 只改机器路径前缀、命中行号与内容原样）：dispatch-rv.md（2 处主仓根前缀 → `<主仓根>`）、runs/recompute-w1-rv2.txt（7 处）、runs/throughput-recompute-rv2.txt（1 处，python 安装根 → `<python-root>`）——净化后全目录扫描零命中（grep Administrator／C:\Users 零命中留痕本行）；一次性净化脚本用后即删不入库。
