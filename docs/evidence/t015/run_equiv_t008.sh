#!/bin/bash
# T015 等价性复跑器（Lead G1 证据，任务卡 D8-④）：T008 对拍矩阵全 29 局逐字节复现。
#
# - 数据源：docs/evidence/t008/runs/plan.txt（29 局命令清单，逐字复现 T008 归档局）。
# - 判定：每局 stdout 与 docs/evidence/t008/runs/<run_id>.stdout **逐字节 diff**
#   （期望 identical；stderr 不比——elapsed_ms 计时行天然漂移）。
# - 断点续跑：已完成且 IDENTICAL 的局重跑时跳过（equiv-exits.txt 幂等标记）；
#   非 0 退出码或 diff 局如实记录、不中断批、结尾汇总非零退出。
# - watchdog 预算（预登记）：red*/anchor* 300s；full* 900s——估计依据：优化后
#   10k×14400 单线程预估 ≲30s（T007 实测 300.9s/30t×300t 外推 + 优化目标量级），
#   预算 = 预估 ×20+ 余量；watchdog 杀 = REAL_EXIT=124 如实入档。
# - 本脚本为验证跑批（非计时量测），不占用空闲独占纪律窗口；仍避免与其他
#   cargo 编译并发。零机器绝对路径。
set -u
cd "$(dirname "$0")/../../.."
PLAN=docs/evidence/t008/runs/plan.txt
ARCH=docs/evidence/t008/runs
OUT=docs/evidence/t015/equiv
mkdir -p "$OUT"

fail=0; n=0
while read -r rid cmd; do
  [ -z "${rid:-}" ] && continue
  case "$rid" in '#'*) continue ;; esac
  # 幂等：已 IDENTICAL 的局跳过
  if grep -qE "^$rid REAL_EXIT=0 .* verdict=IDENTICAL$" "$OUT/equiv-exits.txt" 2>/dev/null; then
    echo "skip (done): $rid"
    continue
  fi
  n=$((n + 1))
  wd=300; case "$rid" in full*) wd=900 ;; esac
  start=$(date +%s%3N)
  timeout "$wd" bash -c "$cmd" > "$OUT/$rid.stdout" 2> "$OUT/$rid.stderr"
  rc=$?
  end=$(date +%s%3N)
  if [ "$rc" -eq 0 ] && cmp -s "$OUT/$rid.stdout" "$ARCH/$rid.stdout"; then
    v=IDENTICAL
  else
    v=DIFF_OR_NONZERO
    fail=$((fail + 1))
    # 差异首行留痕（诊断用，不折叠）
    diff "$ARCH/$rid.stdout" "$OUT/$rid.stdout" | head -5 > "$OUT/$rid.diff-head" 2>&1 || true
  fi
  echo "$rid REAL_EXIT=$rc wall_ms=$((end - start)) verdict=$v" | tee -a "$OUT/equiv-exits.txt"
done < "$PLAN"

echo "TOTAL=$n FAIL=$fail (fail 含非零退出与字节差异)" | tee -a "$OUT/equiv-exits.txt"
if [ "$fail" -eq 0 ]; then
  echo "EQUIV_ALL_PASS"
  exit 0
else
  echo "EQUIV_HAS_FAIL - see equiv-exits.txt"
  exit 1
fi
