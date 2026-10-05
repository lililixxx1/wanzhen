#!/bin/bash
# T008 跑批脚本（D5 断点续跑版；WP-A/WP-C 共用，按 run_id 前缀选择）。
#
# 机器空闲独占声明：本脚本执行窗口内机器须空闲独占，不得并行其他重负载任务；
# cargo 门禁与构建已在跑批前完成（留痕 docs/evidence/t008/check.txt）。
#
# 用法（自动 cd 到仓库根，全部相对路径）：
#   ./docs/evidence/t008/run_t008.sh <run_id 前缀>...
#   例：./docs/evidence/t008/run_t008.sh anchor2 red200      # WP-A 选择集（15 局）
#       ./docs/evidence/t008/run_t008.sh anchor1 full10000   # WP-C 选择集（14 局）
#       ./docs/evidence/t008/run_t008.sh all                 # 全部 29 局
#   无参 = all。前缀按 run_id 开头匹配（如 red200 匹配 red200-th12-s42-r2 等）。
#
# 断点续跑语义（幂等）：
#   - runs/exits.txt 已有该 run_id 的 `REAL_EXIT=0` 行 → skip（不重跑，幂等）；
#   - 已有非 0 行 → 不覆盖不重跑（断言 3：不得静默重跑遮蔽失败记录），结尾汇总报告；
#   - 无 exit 行但有残 stdout（被中断的半局）→ 覆盖重跑（半局非证据）。
#
# watchdog（防死锁兜底，非验收判据；watchdog 杀 = REAL_EXIT=124 如实入档）：
#   red 局 1800s；full th12 3600s / th6 7200s / th3 14400s / th1 28800s。
#   估值换算留痕（执行者按归档数据独立复算，与派工单给定值同量级，沿用派工单值）：
#   T006 fullscale 归档 elapsed_ms=1017022 @th12；T007 12 线程加速比 7.22× @10k 锚点
#   → 串行外推 ≈ 1017.0s × 7.22 ≈ 7343s；th12 3600s = 1017s × 3.5；
#   th6 7200s = (7343/6 ≈ 1224s) × 5.9；th3 14400s = (7343/3 ≈ 2448s) × 5.9；
#   th1 28800s = 7343s × 3.9。各档余量 ≥3.5×，与派工单「约 4× 余量」一致，无显著偏离。
#
# 结尾打印 BATCH_DONE rc=<n>（0 = 本次选择范围内全部计划局 REAL_EXIT=0）。
set -u
# 脚本位于 docs/evidence/t008/ 下 → 回仓库根需上溯 3 级（2026-10-05 修正：原 ../.. 只到 docs/）。
cd "$(dirname "$0")/../../.." || exit 1

EVID=docs/evidence/t008
RUNS="$EVID/runs"
PLAN="$RUNS/plan.txt"
EXITS="$RUNS/exits.txt"
mkdir -p "$RUNS"
touch "$EXITS"

if [ ! -f "$PLAN" ]; then
  echo "run_t008: plan.txt not found: $PLAN" >&2
  exit 1
fi

# 断言 3 汇总与失败计数
prior_fail=""
fail_count=0
skip_count=0
run_count=0

while IFS= read -r line; do
  case "$line" in
    '#'*|'') continue ;;
  esac
  run_id=${line%% *}
  cmd=${line#* }
  [ -z "$cmd" ] || [ "$cmd" = "$line" ] && { echo "run_t008: malformed plan line: $line" >&2; exit 1; }

  # 前缀选择：无参或 all = 全部
  selected=0
  if [ $# -eq 0 ] || [ "$1" = "all" ]; then
    selected=1
  else
    for p in "$@"; do
      case "$run_id" in "$p"*) selected=1; break ;; esac
    done
  fi
  [ "$selected" -eq 1 ] || continue

  # 查既有 exit 行（幂等 / 断言 3）
  prev=$(grep -E "^${run_id} REAL_EXIT=" "$EXITS" || true)
  if [ -n "$prev" ]; then
    case "$prev" in
      *" REAL_EXIT=0"*)
        echo "SKIP        $run_id （exits.txt 已有 REAL_EXIT=0，幂等跳过）"
        skip_count=$((skip_count + 1))
        continue
        ;;
      *)
        echo "PRIOR-FAIL  $run_id （$prev —— 不覆盖不重跑，断言 3）"
        prior_fail="$prior_fail $run_id"
        fail_count=$((fail_count + 1))
        continue
        ;;
    esac
  fi

  # watchdog 档位
  case "$run_id" in
    full10000-th12-*|anchor1-*) tmo=3600 ;;
    full10000-th6-*)  tmo=7200 ;;
    full10000-th3-*)  tmo=14400 ;;
    full10000-th1-*)  tmo=28800 ;;
    *)                tmo=1800 ;;  # red200* 与 anchor2 等降规模/秒级局
  esac

  if [ -f "$RUNS/$run_id.stdout" ]; then
    echo "RERUN       $run_id （无 exit 行但有残 stdout=被中断半局，覆盖重跑）"
  fi

  printf '%s\n' "$cmd" > "$RUNS/$run_id.cmd"
  wall_start=$(date +%s%3N)
  timeout "$tmo" bash -c "$cmd" > "$RUNS/$run_id.stdout" 2> "$RUNS/$run_id.stderr"
  rc_run=$?
  wall_end=$(date +%s%3N)
  echo "$run_id REAL_EXIT=$rc_run wall_ms=$((wall_end - wall_start))" >> "$EXITS"

  if [ "$rc_run" -eq 0 ]; then
    echo "OK          $run_id REAL_EXIT=0 wall_ms=$((wall_end - wall_start))"
    run_count=$((run_count + 1))
  else
    echo "FAIL        $run_id REAL_EXIT=$rc_run wall_ms=$((wall_end - wall_start))"
    fail_count=$((fail_count + 1))
  fi
done < "$PLAN"

echo "----"
echo "summary: executed_ok=$run_count skipped=$skip_count failed_total=$fail_count"
if [ -n "$prior_fail" ]; then
  echo "prior_nonzero_exit_lines（断言 3 汇总，未重跑）:$prior_fail"
fi
echo "BATCH_DONE rc=$fail_count"
exit 0
