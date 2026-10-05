#!/bin/bash
# T009 交叉验证比对（R5）：arena sampling(row0) vs sim --threads 1 vs sim --threads 12
# 三方 final_hash 逐位一致判定。输入三个原始档，输出 runs/r5-crosscheck.txt。
# 退出码：0 = PASS；1 = FAIL（三方不一致或取值失败）。
set -u
cd "$(dirname "$0")/../../.." || exit 1
RUNS=docs/evidence/t009/runs

h_arena=$(grep -o '"final_hash":"0x[0-9a-f]*"' "$RUNS/r5-arena/sampling.jsonl" | head -1 | grep -o '0x[0-9a-f]*')
h_t1=$(grep -o 'final_hash=0x[0-9a-f]*' "$RUNS/r5-sim-t1.stdout" | head -1 | grep -o '0x[0-9a-f]*')
h_t12=$(grep -o 'final_hash=0x[0-9a-f]*' "$RUNS/r5-sim-t12.stdout" | head -1 | grep -o '0x[0-9a-f]*')

{
  echo "# R5 CLI 黄金交叉（arena --sampling --games 1 vs sim --comp shieldman:5000 --seed 3000000 --battle --ticks 14400）"
  echo "arena=$h_arena"
  echo "sim_t1=$h_t1"
  echo "sim_t12=$h_t12"
  if [ -n "$h_arena" ] && [ "$h_arena" = "$h_t1" ] && [ "$h_arena" = "$h_t12" ]; then
    echo "match=PASS (三方 final_hash 逐位一致)"
    exit_code=0
  else
    echo "match=FAIL (三方不一致或取值失败——上报项)"
    exit_code=1
  fi
} > "$RUNS/r5-crosscheck.txt"
cat "$RUNS/r5-crosscheck.txt"
exit $exit_code
