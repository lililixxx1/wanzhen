#!/bin/bash
# T007 证据跑批 runner（M0-06，任务卡 D8/D10）：全矩阵 + 复测 + ⑥ 长跑 + 汇总。
#
# - 顺序执行；全部直跑 target/release/bench.exe（不用 cargo run）。
# - 量测窗口机器空闲独占：启动前 tasklist 查无 cargo/rustc 进程（AGENTS.md 纪律），
#   预检快照入档 runs/preflight_tasklist.csv + runs/preflight.txt。
# - 中断韧性：matrix.jsonl 逐配置成功即追加；重跑时设 T007_RESUME=1 跳过已有配置
#   （默认 matrix.jsonl 非空即拒绝启动，防误覆盖/重复）。
# - 每配置：stdout → runs/<tag>.stdout.json，stderr → runs/<tag>.stderr.txt，
#   REAL_EXIT+wall → runs/REAL_EXIT.txt；成功后 stdout 行追加进 matrix.jsonl。
# - 复测 3 配置（10k-t1 / 10k-t12 / 1k-t6，repeats=3）：|median2-median1|/median1
#   < 5% 断言写入 rerun-check.txt（PASS/FAIL 如实）。
# - ⑥ 长跑：powershell -File run_longrun.ps1（探针校准 K6 → 单局 → 5s 采样 CSV）。
# - 汇总：bench --summarize matrix.jsonl --memory-csv runs/longrun_memory.csv
#   --out summary.md（stdout/stderr/REAL_EXIT 留档）。
# - 脚本零机器绝对路径（相对仓库根解析）；不使用任何编译命令。
set -u
cd "$(dirname "$0")/../../.."
EVID=docs/evidence/t007
BIN=target/release/bench.exe
mkdir -p "$EVID/runs"

# ---- 0) 预检 ----
if [ ! -x "$BIN" ]; then
  echo "FATAL: $BIN missing - build first: cargo build -p sim --bin bench --release -j 3"
  exit 1
fi
tasklist //FO CSV //NH > "$EVID/runs/preflight_tasklist.csv" 2> "$EVID/runs/preflight_tasklist.err"
if grep -qiE '"(cargo|rustc)(\.exe)?"' "$EVID/runs/preflight_tasklist.csv"; then
  {
    echo "FATAL: cargo/rustc process detected - measurement window must be idle"
    echo "matching rows:"
    grep -iE '"(cargo|rustc)(\.exe)?"' "$EVID/runs/preflight_tasklist.csv"
  } > "$EVID/runs/preflight.txt"
  exit 1
fi
{
  echo "preflight OK: $(date '+%Y-%m-%d %H:%M:%S')"
  echo "bench: $BIN"
  echo "tasklist snapshot (no cargo/rustc detected): runs/preflight_tasklist.csv"
  echo "declaration: no cargo/compilation/download is started between this point and batch end;"
  echo "             measurement window is otherwise idle (AGENTS.md discipline)."
} > "$EVID/runs/preflight.txt"

# ---- 1) 环境档 ----
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$EVID/collect_environment.ps1" || {
  echo "FATAL: environment collection failed"; exit 1;
}

# ---- 2) 全矩阵（D8：1k×600t / 5k×300t / 10k×300t / 50k×30t × threads{1,3,6,12}） ----
if [ -s "$EVID/matrix.jsonl" ]; then
  if [ "${T007_RESUME:-0}" = "1" ]; then
    echo "resume mode: matrix.jsonl non-empty, skipping already present configs"
    FRESH=0
  else
    echo "FATAL: $EVID/matrix.jsonl non-empty (set T007_RESUME=1 to resume; or archive/remove it)"
    exit 1
  fi
else
  : > "$EVID/matrix.jsonl"
  : > "$EVID/runs/REAL_EXIT.txt"
  FRESH=1
fi

echo "== matrix start $(date '+%H:%M:%S') =="
for spec in "1000 600" "5000 300" "10000 300" "50000 30"; do
  units=${spec% *}
  ticks=${spec#* }
  per_side=$((units / 2))
  for t in 1 3 6 12; do
    tag="${units}-t${t}"
    if [ "$FRESH" = "0" ] && grep -qF '"units":'"${units}"',"per_side":'"${per_side}"',"threads":'"${t}"',' "$EVID/matrix.jsonl"; then
      echo "resume skip: $tag"
      continue
    fi
    start_ms=$(date +%s%3N)
    echo "== run $tag units=$units threads=$t ticks=$ticks seed=42 warmup=1 repeats=5 =="
    "$BIN" --units "$units" --threads "$t" --ticks "$ticks" --warmup 1 --repeats 5 --seed 42 \
      > "$EVID/runs/$tag.stdout.json" 2> "$EVID/runs/$tag.stderr.txt"
    rc=$?
    end_ms=$(date +%s%3N)
    echo "$tag units=$units threads=$t ticks=$ticks seed=42 warmup=1 repeats=5 REAL_EXIT=$rc wall_ms=$((end_ms - start_ms))" >> "$EVID/runs/REAL_EXIT.txt"
    if [ "$rc" -ne 0 ]; then
      echo "FATAL: $tag REAL_EXIT=$rc - investigate; fix bench if it is its own bug, then resume with T007_RESUME=1" >&2
      exit 1
    fi
    cat "$EVID/runs/$tag.stdout.json" >> "$EVID/matrix.jsonl"
  done
done
echo "== matrix done $(date '+%H:%M:%S') =="

# ---- 3) 复测 3 配置（同配置跨调用中位数漂移 <5%；repeats=3） ----
: > "$EVID/rerun-check.txt"
rerun() {
  tag=$1; units=$2; t=$3; ticks=$4
  "$BIN" --units "$units" --threads "$t" --ticks "$ticks" --warmup 1 --repeats 3 --seed 42 \
    > "$EVID/runs/rerun-$tag.stdout.json" 2> "$EVID/runs/rerun-$tag.stderr.txt"
  rc=$?
  echo "rerun-$tag units=$units threads=$t ticks=$ticks seed=42 warmup=1 repeats=3 REAL_EXIT=$rc" >> "$EVID/runs/REAL_EXIT.txt"
  if [ "$rc" -ne 0 ]; then
    echo "FATAL: rerun-$tag REAL_EXIT=$rc" >&2
    return 1
  fi
  m1=$(grep -o '"median_ns":[0-9.]*' "$EVID/runs/$tag.stdout.json" | head -n1 | cut -d: -f2)
  m2=$(grep -o '"median_ns":[0-9.]*' "$EVID/runs/rerun-$tag.stdout.json" | head -n1 | cut -d: -f2)
  line=$(awk -v tag="$tag" -v m1="$m1" -v m2="$m2" 'BEGIN {
    d = m1 - m2; if (d < 0) d = -d;
    rel = d / m1 * 100.0;
    verdict = (rel < 5.0) ? "PASS" : "FAIL";
    printf "%s median_1=%s median_2=%s drift_pct=%.4f assertion(<5%%)=%s\n", tag, m1, m2, rel, verdict;
    exit (rel < 5.0) ? 0 : 1;
  }')
  rcawk=$?
  echo "$line" | tee -a "$EVID/rerun-check.txt"
  if [ "$rcawk" -ne 0 ]; then
    echo "WARNING: rerun drift assertion FAILED for $tag (disclose and add samples per card)" >&2
  fi
  return 0
}
echo "== rerun checks start $(date '+%H:%M:%S') =="
rerun "10000-t1" 10000 1 300 || exit 1
rerun "10000-t12" 10000 12 300 || exit 1
rerun "1000-t6" 1000 6 600 || exit 1
echo "== rerun checks done $(date '+%H:%M:%S') =="

# ---- 4) ⑥ 长跑（探针校准 + 5s 采样） ----
echo "== longrun start $(date '+%H:%M:%S') =="
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$EVID/run_longrun.ps1" || {
  echo "FATAL: longrun failed"; exit 1;
}
echo "== longrun done $(date '+%H:%M:%S') =="

# ---- 5) 汇总（判定行代码计算） ----
echo "== summarize start $(date '+%H:%M:%S') =="
"$BIN" --summarize "$EVID/matrix.jsonl" --memory-csv "$EVID/runs/longrun_memory.csv" --out "$EVID/summary.md" \
  > "$EVID/runs/summarize.stdout.txt" 2> "$EVID/runs/summarize.stderr.txt"
rc=$?
echo "summarize REAL_EXIT=$rc" >> "$EVID/runs/REAL_EXIT.txt"
if [ "$rc" -ne 0 ]; then
  echo "WARNING: summarize REAL_EXIT=$rc (see runs/summarize.stderr.txt)" >&2
fi
echo "BATCH_DONE $(date '+%Y-%m-%d %H:%M:%S')"
