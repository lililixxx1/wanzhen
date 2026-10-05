#!/usr/bin/env bash
# T010 采集跑批（断点续跑、全相对路径；从树任意 cwd 调用可定位树根）。
#
# 子命令：
#   precheck          机器空闲预检（CPU 负载/相关进程/commit 余量快照入档）
#   gate <1|2|3|4>    门禁四条（每条 cargo 前单独整句跑 commit 余量预检；失败停批）
#   smoke             R1 冒烟（--units 100 --warmup-sec 2 --capture-sec 3）
#   tier <units>      R2~R5 单档（1000/2000/5000/10000，默认参数）
#   collect           冒烟 + 四档（断点续跑：已有 REAL_EXIT 的档跳过）
#   summary           R6 summarize.py → summary.md（含 --selftest 对齐档）
#   env               R7 环境档 collect_environment.ps1
#   all               precheck + collect + summary + env
#
# 环境变量 FORCE=1 强制重跑已完成的 run。
set -u

EVID="docs/evidence/t010"
RUNS="$EVID/runs"
EXE="target/release/render-spike.exe"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# 脚本位于 <树根>/docs/evidence/t010/ ⇒ 树根 = 上溯三级。
TREE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$TREE_ROOT" || exit 1
mkdir -p "$RUNS"

now() { date "+%Y-%m-%d %H:%M:%S"; }

# commit 余量预检（附录 A）：单独整句执行；不足 sleep 120 重检（≤5 次）后失败退出。
commit_precheck() { # $1=need_G  $2=log
  local need="$1" log="$2" attempt=1 free=""
  mkdir -p "$(dirname "$log")"
  : > "$log"
  while [ "$attempt" -le 5 ]; do
    echo "[$(now)] commit 预检 attempt=$attempt (need >= ${need}G)" >> "$log"
    free=$(powershell -NoProfile -Command 'Get-CimInstance Win32_OperatingSystem | ForEach-Object { "CommitFree_G=" + [math]::Round(($_.TotalVirtualMemorySize-$_.FreeVirtualMemory)/1MB,1) + " CommitLimit_G=" + [math]::Round($_.TotalVirtualMemorySize/1MB,1) }' 2>&1 | tee -a "$log" | grep -o 'CommitFree_G=[0-9.]*' | cut -d= -f2)
    echo "[$(now)] CommitFree_G=$free (need >= $need)" | tee -a "$log" >&2
    if [ -n "$free" ] && awk -v f="$free" -v n="$need" 'BEGIN { exit !(f >= n) }'; then
      return 0
    fi
    attempt=$((attempt + 1))
    if [ "$attempt" -le 5 ]; then sleep 120; fi
  done
  echo "[$(now)] commit 预检连续 5 次不足（need >= ${need}G）—— 上报 Lead 并停止" | tee -a "$log" >&2
  return 3
}

# 单条命令落档：cmd.txt（命令全文）+ stdout.log / stderr.log + REAL_EXIT + time.txt。
# 断点续跑：已有 REAL_EXIT 则跳过（FORCE=1 重跑）。
run_cmd() { # $1=rundir  $2..=命令与参数
  local dir="$1"
  shift
  mkdir -p "$dir"
  if [ -f "$dir/REAL_EXIT" ] && [ "${FORCE:-0}" != "1" ]; then
    echo "[skip] $dir (已完成; FORCE=1 可重跑)"
    return 0
  fi
  printf '%s\n' "$*" > "$dir/cmd.txt"
  printf 'start: %s\n' "$(now)" > "$dir/time.txt"
  "$@" > "$dir/stdout.log" 2> "$dir/stderr.log"
  local rc=$?
  printf '%s\n' "$rc" > "$dir/REAL_EXIT"
  printf 'end: %s\nreal_exit: %s\n' "$(now)" "$rc" >> "$dir/time.txt"
  echo "[run] $dir -> REAL_EXIT=$rc"
  return $rc
}

gate_dir() {
  case "$1" in
    1) echo "$RUNS/r0_gate1_check" ;;
    2) echo "$RUNS/r0_gate2_test_sim" ;;
    3) echo "$RUNS/r0_gate3_test_rs" ;;
    4) echo "$RUNS/r0_gate4_build_rs" ;;
    *) echo "" ;;
  esac
}

gate() {
  local n="$1" d need
  d="$(gate_dir "$n")"
  [ -n "$d" ] || { echo "gate 参数须为 1..4"; exit 2; }
  case "$n" in
    1) need=10 ;;
    2) need=10 ;;
    3) need=10 ;;
    4) need=12 ;;
  esac
  commit_precheck "$need" "$d/precheck.log" || exit 3
  case "$n" in
    1) run_cmd "$d" cargo check --workspace -j 2 ;;
    2) run_cmd "$d" cargo test -p sim -j 2 ;;
    3) run_cmd "$d" cargo test -p render-spike -j 2 ;;
    4) run_cmd "$d" cargo build -p render-spike --release -j 2 ;;
  esac
  local rc
  rc="$(cat "$d/REAL_EXIT" 2>/dev/null || echo "?")"
  if [ "$rc" != "0" ]; then
    echo "[gate $n] FAILED rc=$rc（详见 $d）"
    exit 1
  fi
  echo "[gate $n] OK"
}

precheck() {
  mkdir -p "$RUNS/r0_precheck"
  {
    echo "== T010 跑批前机器空闲预检 =="
    echo "time: $(now)"
    echo "-- CPU 负载 (Win32_Processor.LoadPercentage 平均) --"
    powershell -NoProfile -Command '(Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average'
    echo "-- 相关进程（cargo/rustc/render-spike/bench/sim） --"
    tasklist 2>/dev/null | grep -iE "cargo|rustc|render-spike|bench|sim\.exe" || echo "(none)"
    echo "-- commit 余量 --"
    powershell -NoProfile -Command 'Get-CimInstance Win32_OperatingSystem | ForEach-Object { "CommitFree_G=" + [math]::Round(($_.TotalVirtualMemorySize-$_.FreeVirtualMemory)/1MB,1) + " CommitLimit_G=" + [math]::Round($_.TotalVirtualMemorySize/1MB,1) }'
  } | tee "$RUNS/r0_precheck/idle.log"
}

smoke() {
  [ -x "$EXE" ] || { echo "缺 $EXE（先跑 gate 4）"; exit 4; }
  run_cmd "$RUNS/r1_smoke" "$EXE" --units 100 --warmup-sec 2 --capture-sec 3 --out "$RUNS/r1_smoke/out"
}

tier() {
  local units="$1" d
  case "$units" in
    1000) d="$RUNS/r2_t1000" ;;
    2000) d="$RUNS/r3_t2000" ;;
    5000) d="$RUNS/r4_t5000" ;;
    10000) d="$RUNS/r5_t10000" ;;
    *) d="$RUNS/r_t$units" ;;
  esac
  [ -x "$EXE" ] || { echo "缺 $EXE（先跑 gate 4）"; exit 4; }
  run_cmd "$d" "$EXE" --units "$units" --out "$d/out"
}

collect() {
  smoke
  tier 1000
  tier 2000
  tier 5000
  tier 10000
}

summary() {
  mkdir -p "$RUNS/r6_summary"
  printf 'python %s/summarize.py --root %s --out %s\n' "$EVID" "$RUNS" "$EVID/summary.md" > "$RUNS/r6_summary/cmd.txt"
  python "$EVID/summarize.py" --root "$RUNS" --out "$EVID/summary.md" > "$RUNS/r6_summary/stdout.log" 2> "$RUNS/r6_summary/stderr.log"
  echo $? > "$RUNS/r6_summary/REAL_EXIT"
  printf 'python %s/summarize.py --selftest\n' "$EVID" > "$RUNS/r6_summary/selftest_cmd.txt"
  python "$EVID/summarize.py" --selftest > "$RUNS/r6_summary/selftest.log" 2>&1
  echo "[summary] REAL_EXIT=$(cat "$RUNS/r6_summary/REAL_EXIT")"
}

env_collect() {
  mkdir -p "$RUNS/r7_env"
  printf 'powershell -NoProfile -ExecutionPolicy Bypass -File %s/collect_environment.ps1\n' "$EVID" > "$RUNS/r7_env/cmd.txt"
  powershell -NoProfile -ExecutionPolicy Bypass -File "$EVID/collect_environment.ps1" > "$RUNS/r7_env/collect.log" 2> "$RUNS/r7_env/stderr.log"
  echo $? > "$RUNS/r7_env/REAL_EXIT"
  echo "[env] REAL_EXIT=$(cat "$RUNS/r7_env/REAL_EXIT")"
}

case "${1:-}" in
  precheck) precheck ;;
  gate) gate "${2:?gate 1..4}" ;;
  smoke) smoke ;;
  tier) tier "${2:?tier units}" ;;
  collect) collect ;;
  summary) summary ;;
  env) env_collect ;;
  all)
    precheck
    collect
    summary
    env_collect
    ;;
  *)
    echo "usage: run_t010.sh {precheck|gate <1..4>|smoke|tier <units>|collect|summary|env|all}"
    exit 2
    ;;
esac
