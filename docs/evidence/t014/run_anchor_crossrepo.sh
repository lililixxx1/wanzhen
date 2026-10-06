#!/usr/bin/env bash
# T014 · 跨仓锚核对（组 G~J，共 9 锚）——主仓根执行版（Lead 收获后 G1 留痕用）
#
# 用法（主仓根 = 与 bevy-ai-workflow 同级的 wanzhen 仓根）：
#   cd <主仓根> && bash docs/evidence/t014/run_anchor_crossrepo.sh > docs/evidence/t014/runs/anchor-crossrepo.txt 2>&1
#   echo "REAL_EXIT=$?"
# 前置：bevy-ai-workflow 仓与本仓同级 checkout（相对路径 ../bevy-ai-workflow/）；
#       docs/evidence/t014/ 已收获入主仓。
# 预注册预期（派工单 §3）：跨仓组全命中（9/9 PASS、总退出码 0）。
# 锚行号实测参考（隔离树 t014-a 侧 grep -n）：
#   g1 :13 / g2 :132 / g3 :137 / g4 :134（派工单标 :132，偏差已记 selfcheck）/ g5 :140 /
#   h1 :41 / i1 :6 / j1 :12 / j2 :12
set -u

WF=../bevy-ai-workflow
pass=0
fail=0

chk() {
  local id="$1" file="$2" anchor="$3" c
  if [ ! -f "$file" ]; then
    echo "$id | FAIL(file-missing) | $file"
    fail=$((fail+1))
    return
  fi
  c=$(grep -c -F -- "$anchor" "$file")
  if [ "$c" -gt 0 ]; then
    echo "$id | PASS"
    pass=$((pass+1))
  else
    echo "$id | FAIL | $file"
    fail=$((fail+1))
  fi
}

# 组 G · Bevy-AI开发意向文档.md
chk g1 "$WF/Bevy-AI开发意向文档.md" '游戏是**试金石**而非产品野心'
chk g2 "$WF/Bevy-AI开发意向文档.md" '什么品类最能锻炼并验证这套工作流'
chk g3 "$WF/Bevy-AI开发意向文档.md" '中小体量——它是试金石，不是产品野心'
chk g4 "$WF/Bevy-AI开发意向文档.md" '离散/回合制逻辑优先，确定性模拟（可种子重放）'
chk g5 "$WF/Bevy-AI开发意向文档.md" '（周数或系统数，防失控）'
# 组 H · docs/m4-game-selection.md
chk h1 "$WF/docs/m4-game-selection.md" '系统 ≤10、日历周 ≤4、taskset 新增任务 ≤14'
# 组 I · docs/pre-window-plan.md
chk i1 "$WF/docs/pre-window-plan.md" '意向文档无 M5 定义'
# 组 J · AGENTS.md
chk j1 "$WF/AGENTS.md" '万阵 M0 预验证优先'
chk j2 "$WF/AGENTS.md" 'T041 MCP 薄桥已冻结'

echo "汇总: 9 条 | PASS $pass | FAIL $fail"
if [ "$fail" -gt 0 ]; then
  exit 1
fi
exit 0
