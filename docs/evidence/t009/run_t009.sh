#!/bin/bash
# T009 跑批脚本（断点续跑；全部相对路径；机器空闲独占——每批开跑前自动执行
# precheck_load.ps1 负载预检并落档 runs/precheck-<id>.txt，processes 段非空 = 该批数据不可信）。
#
# 用法（自动 cd 到仓库根）：
#   ./docs/evidence/t009/run_t009.sh [批次前缀]...
#   例：./docs/evidence/t009/run_t009.sh r1 r2        # 快检 + 口径层
#       ./docs/evidence/t009/run_t009.sh r6           # 全规模抽样（约 20 min）
#   无参 = 全部批次。
#
# 断点续跑语义（幂等，同 t008 先例）：
#   - exits.txt 已有该批次 `REAL_EXIT=0` 行 → skip；
#   - 已有非 0 行 → 不覆盖不重跑（不静默重跑遮蔽失败记录），结尾汇总报告；
#   - 无 exit 行但有残 stdout（被中断半批）→ 覆盖重跑（半批非证据）。
#
# watchdog（防死锁兜底，非验收判据；watchdog 杀 = REAL_EXIT=124 如实入档）：
#   r2/r3 1800s；r4-t1 1800s、r4-t3/t6/t12 900s；r5 600s（sim-t1 1800s）；
#   r6 3600s（参考 ≈20 min ±50%）。r0 门禁 900s。
set -u
cd "$(dirname "$0")/../../.." || exit 1

EVID=docs/evidence/t009
RUNS="$EVID/runs"
EXITS="$RUNS/exits.txt"
mkdir -p "$RUNS"
touch "$EXITS"

precheck() { # $1 = batch id
  powershell -NoProfile -ExecutionPolicy Bypass -File "$EVID/precheck_load.ps1" \
    "$(pwd -W 2>/dev/null || pwd)/$RUNS/precheck-$1.txt" \
    > "$RUNS/precheck-$1.out" 2>&1
}

run_batch() { # $1 = id, $2 = timeout_s, $3.. = command
  local id=$1 tmo=$2
  shift 2
  local cmd="$*"
  local prev
  prev=$(grep -E "^${id} REAL_EXIT=" "$EXITS" || true)
  if [ -n "$prev" ]; then
    case "$prev" in
      *" REAL_EXIT=0"*)
        echo "SKIP        $id （exits.txt 已有 REAL_EXIT=0，幂等跳过）"
        return 0
        ;;
      *)
        echo "PRIOR-FAIL  $id （$prev —— 不覆盖不重跑）"
        return 1
        ;;
    esac
  fi
  if [ -f "$RUNS/$id.stdout" ]; then
    echo "RERUN       $id （无 exit 行但有残 stdout=被中断半批，覆盖重跑）"
  fi
  precheck "$id"
  printf '%s\n' "$cmd" > "$RUNS/$id.cmd"
  local wall_start wall_end rc
  wall_start=$(date +%s%3N)
  timeout "$tmo" bash -c "$cmd" > "$RUNS/$id.stdout" 2> "$RUNS/$id.stderr"
  rc=$?
  wall_end=$(date +%s%3N)
  echo "$id REAL_EXIT=$rc wall_ms=$((wall_end - wall_start))" >> "$EXITS"
  if [ "$rc" -eq 0 ]; then
    echo "OK          $id REAL_EXIT=0 wall_ms=$((wall_end - wall_start))"
    return 0
  else
    echo "FAIL        $id REAL_EXIT=$rc wall_ms=$((wall_end - wall_start))"
    return 1
  fi
}

# ---- 批次定义（命令全文即档存命令）----
run_batch r0-1-check        900 'cargo check --workspace -j 3'
run_batch r0-2-test         900 'cargo test -p sim -j 3'
run_batch r0-3-build-sim    900 'cargo build -p sim --release -j 3'
run_batch r0-4-build-arena  900 'cargo build -p sim --bin arena --release -j 3'
run_batch r1-smoke          300 './target/release/arena.exe --matrix --per-side 10 --per-cell 2 --threads 4 --out docs/evidence/t009/runs/smoke'
run_batch r2-matrix-per100  1800 './target/release/arena.exe --matrix --per-side 100 --per-cell 100 --threads 12 --out docs/evidence/t009/runs/matrix_per100'
run_batch r3-matrix-per10   1800 './target/release/arena.exe --matrix --per-side 10 --per-cell 100 --threads 12 --out docs/evidence/t009/runs/matrix_per10'
run_batch r4-t1             1800 './target/release/arena.exe --throughput --games 512 --threads 1 --repeats 3 --out docs/evidence/t009/runs/throughput_t1'
run_batch r4-t3              900 './target/release/arena.exe --throughput --games 512 --threads 3 --repeats 3 --out docs/evidence/t009/runs/throughput_t3'
run_batch r4-t6              900 './target/release/arena.exe --throughput --games 512 --threads 6 --repeats 3 --out docs/evidence/t009/runs/throughput_t6'
run_batch r4-t12             900 './target/release/arena.exe --throughput --games 512 --threads 12 --repeats 3 --out docs/evidence/t009/runs/throughput_t12'
run_batch r5-arena           600 './target/release/arena.exe --sampling --games 1 --threads 12 --out docs/evidence/t009/runs/r5-arena'
run_batch r5-sim-t1         1800 './target/release/sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 1'
run_batch r5-sim-t12         600 './target/release/sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 12'
run_batch r5-cross            60 'bash docs/evidence/t009/crosscheck_r5.sh'
run_batch r6-sampling       3600 './target/release/arena.exe --sampling --games 100 --threads 12 --out docs/evidence/t009/runs/sampling'
run_batch r7-env             120 'powershell -NoProfile -ExecutionPolicy Bypass -File docs/evidence/t009/collect_environment.ps1'
run_batch r8-summary         120 'python docs/evidence/t009/summarize.py --out docs/evidence/t009'

echo "----"
echo "BATCH_DONE rc=0（本次选择范围内全部批次处理完毕；逐批 REAL_EXIT 见 exits.txt）"
