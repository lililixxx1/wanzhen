#!/bin/bash
# T006 证据跑批（D9 对拍矩阵 16 局 + D8 fullscale 长跑）。机器空闲独占，后台运行。
set -u
cd /c/Users/Administrator/Desktop/ccc/wanzhen
EVID=docs/evidence/t006
BIN=./target/release/sim.exe
COMP500="shieldman:84,heavyknight:83,pikeman:83,swordsman:83,archer:83,militia:84"
COMP5000="shieldman:840,heavyknight:830,pikeman:830,swordsman:830,archer:830,militia:840"
mkdir -p $EVID/runs
: > $EVID/runs/exits.txt

# ---- D9 矩阵：1k 局（每方 500，共 1000）× threads {1,3,6,12} × seeds {42,43} ----
for seed in 42 43; do
  for t in 1 3 6 12; do
    $BIN --comp "$COMP500" --ticks 14400 --hash-samples 0,1800,4000,8000,11000,14400 --threads $t --seed $seed \
      > $EVID/runs/1k-s${seed}-t${t}.stdout 2> $EVID/runs/1k-s${seed}-t${t}.stderr
    echo "1k seed=$seed threads=$t REAL_EXIT=$?" >> $EVID/runs/exits.txt
  done
done

# ---- D9 矩阵：10k 局（每方 5000，共 10000）× threads {1,3,6,12} × seeds {42,43} ----
for seed in 42 43; do
  for t in 1 3 6 12; do
    $BIN --comp "$COMP5000" --ticks 300 --hash-samples 0,100,200,300 --threads $t --seed $seed \
      > $EVID/runs/10k-s${seed}-t${t}.stdout 2> $EVID/runs/10k-s${seed}-t${t}.stderr
    echo "10k seed=$seed threads=$t REAL_EXIT=$?" >> $EVID/runs/exits.txt
  done
done

# ---- D8 fullscale 长跑：10k × 14400 ticks @ threads 12, seed 42，600s 防死锁兜底 ----
wall_start=$(date +%s%3N)
timeout 600 $BIN --comp "$COMP5000" --ticks 14400 --threads 12 --seed 42 \
  > $EVID/runs/fullscale.stdout 2> $EVID/runs/fullscale.stderr
rc=$?
wall_end=$(date +%s%3N)
echo "fullscale REAL_EXIT=$rc watchdog_wall_ms=$((wall_end - wall_start))" >> $EVID/runs/exits.txt
echo "BATCH_DONE rc=$rc"
