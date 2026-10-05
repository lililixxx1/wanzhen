#!/bin/bash
# T015 量测矩阵 runner（任务卡 D9，复用 T007 bench 套件口径）：全矩阵 + 复测 +
# 跨版本哈希一致性断言 + 汇总。不含 ⑥ 长跑（D10：无内存行为变更不重跑，
# summarize 披露「未提供 --memory-csv，验收⑥ 判定跳过」）。
#
# - 前提：主仓已收获 T015 优化代码并完成 release 构建（cargo build -p sim --bin
#   bench --release -j 3）；量测窗口机器空闲独占（AGENTS.md 纪律），启动前
#   tasklist 查无 cargo/rustc，预检快照入档 runs/preflight_tasklist.csv。
# - 中断韧性：matrix.jsonl 逐配置成功即追加；重跑设 T015_RESUME=1 跳过已有
#   （默认非空即拒绝启动，防误覆盖）。
# - 每配置：stdout → runs/<tag>.stdout.json，stderr → runs/<tag>.stderr.txt，
#   REAL_EXIT → runs/REAL_EXIT.txt；成功行追加 matrix.jsonl。
# - 复测 3 配置（10k-t1 / 10k-t12 / 1k-t6，repeats=3）：跨调用中位数漂移 <5%
#   断言写 rerun-check.txt（PASS/FAIL 如实，超标按卡面「披露+加样」上报）。
# - 跨版本等价断言：16 配置 final_hash 与 docs/evidence/t007/matrix.jsonl
#   同配置**动态比对**（零硬编码哈希；基准 = T007 归档）。不一致 → 汇总非零退出。
# - 预注册预期（任务卡 D10）：① 四点 µs 预期 0.08~0.4 PASS；② @10k 加速比预期
#   ≈1.5~2.5× TRIPPED（结构性，Amdahl 留痕于 README）。
# - 脚本零机器绝对路径（相对仓库根解析）；不使用任何编译命令。
set -u
cd "$(dirname "$0")/../../.."
EVID=docs/evidence/t015
BIN=target/release/bench.exe
REF=docs/evidence/t007/matrix.jsonl
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

# ---- 2) 全矩阵（16 配置 = 1k×600t / 5k×300t / 10k×300t / 50k×30t × threads{1,3,6,12}） ----
if [ -s "$EVID/matrix.jsonl" ]; then
  if [ "${T015_RESUME:-0}" = "1" ]; then
    echo "resume mode: matrix.jsonl non-empty, skipping already present configs"
    FRESH=0
  else
    echo "FATAL: $EVID/matrix.jsonl non-empty (set T015_RESUME=1 to resume; or archive/remove it)"
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
      echo "FATAL: $tag REAL_EXIT=$rc - investigate; resume with T015_RESUME=1 after fix" >&2
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

# ---- 4) 跨版本 final_hash 一致性断言（vs T007 归档，动态比对零硬编码） ----
# 行格式（bench stdout JSON，固定键序）：{"config":{"units":N,"per_side":P,"threads":T,...
: > "$EVID/hash-crosscheck.txt"
hfail=0
get_hash() { # $1=jsonl $2=units $3=per_side $4=threads -> stdout = 0x... （空=未找到）
  grep -oF "\"units\":$2,\"per_side\":$3,\"threads\":$4," "$1" | head -n1 >/dev/null || return 0
  grep -F "\"units\":$2,\"per_side\":$3,\"threads\":$4," "$1" | head -n1 \
    | grep -oE '"final_hash":"0x[0-9a-f]+"' | cut -d'"' -f4
}
for spec in "1000 600" "5000 300" "10000 300" "50000 30"; do
  units=${spec% *}; ticks=${spec#* }; per_side=$((units / 2))
  for t in 1 3 6 12; do
    h_new=$(get_hash "$EVID/matrix.jsonl" "$units" "$per_side" "$t")
    h_old=$(get_hash "$REF" "$units" "$per_side" "$t")
    if [ -n "$h_new" ] && [ "$h_new" = "$h_old" ]; then
      echo "units=$units t=$t final_hash=$h_new MATCH" >> "$EVID/hash-crosscheck.txt"
    else
      echo "units=$units t=$t t015=$h_new t007=$h_old MISMATCH" >> "$EVID/hash-crosscheck.txt"
      hfail=$((hfail + 1))
    fi
  done
done
echo "hash crosscheck: fail=$hfail (16 configs vs t007 archive)" >> "$EVID/hash-crosscheck.txt"
if [ "$hfail" -ne 0 ]; then
  echo "FATAL: final_hash cross-version MISMATCH ($hfail) - equivalence red line violated" >&2
  exit 1
fi

# ---- 5) 汇总（判定行代码计算；无 --memory-csv → ⑥ 跳过披露行） ----
echo "== summarize start $(date '+%H:%M:%S') =="
"$BIN" --summarize "$EVID/matrix.jsonl" --out "$EVID/summary.md" \
  > "$EVID/runs/summarize.stdout.txt" 2> "$EVID/runs/summarize.stderr.txt"
rc=$?
echo "summarize REAL_EXIT=$rc" >> "$EVID/runs/REAL_EXIT.txt"
if [ "$rc" -ne 0 ]; then
  echo "WARNING: summarize REAL_EXIT=$rc (see runs/summarize.stderr.txt)" >&2
fi
echo "BATCH_DONE $(date '+%Y-%m-%d %H:%M:%S')"
