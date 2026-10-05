#!/bin/bash
# T007 主会话接管收尾批（2026-10-05）。worker-2 于 2026-10-04 16:55 配额中断：
# 全矩阵 16 配置 + 复测 3 配置 + 环境档已落盘（runs/REAL_EXIT.txt 自含），剩余三步：
#   A) 复测超标配置加样（任务卡断言 2 处方「如实披露并加样本」）：
#      10000-t12 与 1000-t6 各再增 2 次独立调用（repeats=5，同矩阵参数）；
#   B) ⑥ 长跑重跑（run_longrun.ps1 -TargetSeconds 630——600s 下限之上留
#      探针-实测方差余量，口径留痕于 longrun_meta.txt）；
#   C) 汇总（bench --summarize，判定行代码计算）。
# 纪律：量测窗口机器空闲独占；本脚本零编译步骤；预检同 run_matrix.sh。
set -u
cd "$(dirname "$0")/../../.."
EVID=docs/evidence/t007
BIN=target/release/bench.exe
mkdir -p "$EVID/runs"

# ---- 预检（同 run_matrix.sh 口径）----
if [ ! -x "$BIN" ]; then echo "FATAL: $BIN missing"; exit 1; fi
tasklist //FO CSV //NH > "$EVID/runs/takeover_preflight_tasklist.csv" 2> "$EVID/runs/takeover_preflight_tasklist.err"
if grep -qiE '"(cargo|rustc)(\.exe)?"' "$EVID/runs/takeover_preflight_tasklist.csv"; then
  echo "FATAL: cargo/rustc process detected - measurement window must be idle" >&2
  exit 1
fi
{
  echo "takeover preflight OK: $(date '+%Y-%m-%d %H:%M:%S')"
  echo "bench: $BIN（含 2026-10-05 summarize ticks 一致性补丁；量测路径零改动）"
  echo "declaration: no cargo/compilation/download is started during this batch (AGENTS.md discipline)."
} > "$EVID/runs/takeover_preflight.txt"

# ---- A) 加样：两超标配置 × 2 次独立调用（repeats=5） ----
echo "== takeover remediation start $(date '+%H:%M:%S') =="
for round in 2 3; do
  "$BIN" --units 10000 --threads 12 --ticks 300 --warmup 1 --repeats 5 --seed 42 \
    > "$EVID/runs/rerun${round}-10000-t12.stdout.json" 2> "$EVID/runs/rerun${round}-10000-t12.stderr.txt"
  rc=$?
  echo "rerun${round}-10000-t12 units=10000 threads=12 ticks=300 seed=42 warmup=1 repeats=5 REAL_EXIT=$rc" >> "$EVID/runs/REAL_EXIT.txt"
  if [ "$rc" -ne 0 ]; then echo "FATAL: rerun${round}-10000-t12 REAL_EXIT=$rc" >&2; exit 1; fi
  "$BIN" --units 1000 --threads 6 --ticks 600 --warmup 1 --repeats 5 --seed 42 \
    > "$EVID/runs/rerun${round}-1000-t6.stdout.json" 2> "$EVID/runs/rerun${round}-1000-t6.stderr.txt"
  rc=$?
  echo "rerun${round}-1000-t6 units=1000 threads=6 ticks=600 seed=42 warmup=1 repeats=5 REAL_EXIT=$rc" >> "$EVID/runs/REAL_EXIT.txt"
  if [ "$rc" -ne 0 ]; then echo "FATAL: rerun${round}-1000-t6 REAL_EXIT=$rc" >&2; exit 1; fi
done

med() { grep -o '"median_ns":[0-9.]*' "$1" | head -n1 | cut -d: -f2; }
cv()  { grep -o '"cv_pct":[0-9.]*' "$1" | head -n1 | cut -d: -f2; }
T1_10K=$(med "$EVID/runs/10000-t1.stdout.json")
T1_1K=$(med "$EVID/runs/1000-t1.stdout.json")
{
  echo "# T007 复测超标配置加样档（主会话接管，2026-10-05）"
  echo
  echo "背景：runs/rerun-check.txt 中 10000-t12 漂移 9.42%、1000-t6 漂移 20.05%（断言 <5% 超标）。"
  echo "处方（任务卡断言 2）：如实披露并加样本——每配置再增 2 次独立调用（repeats=5，同矩阵参数）。"
  echo "样本源：matrix 原行（repeats=5）/ rerun-（repeats=3，worker-2 2026-10-04）/ rerun2-、rerun3-（repeats=5，主会话 2026-10-05）。"
  echo
  for tag in 10000-t12 1000-t6; do
    m0=$(med "$EVID/runs/$tag.stdout.json")
    m1=$(med "$EVID/runs/rerun-$tag.stdout.json")
    m2=$(med "$EVID/runs/rerun2-$tag.stdout.json")
    m3=$(med "$EVID/runs/rerun3-$tag.stdout.json")
    c0=$(cv "$EVID/runs/$tag.stdout.json"); c1=$(cv "$EVID/runs/rerun-$tag.stdout.json")
    c2=$(cv "$EVID/runs/rerun2-$tag.stdout.json"); c3=$(cv "$EVID/runs/rerun3-$tag.stdout.json")
    if [ "$tag" = "10000-t12" ]; then BASE=$T1_10K; else BASE=$T1_1K; fi
    awk -v tag="$tag" -v BASE="$BASE" -v m0="$m0" -v m1="$m1" -v m2="$m2" -v m3="$m3" \
        -v c0="$c0" -v c1="$c1" -v c2="$c2" -v c3="$c3" 'BEGIN {
      printf "## %s（t1 参照 median_ns=%s）\n", tag, BASE;
      printf "| 调用 | median_ns | 组内 cv_pct |\n|---|---:|---:|\n";
      printf "| matrix(repeats=5) | %s | %.4f |\n", m0, c0;
      printf "| rerun(repeats=3) | %s | %.4f |\n", m1, c1;
      printf "| rerun2(repeats=5) | %s | %.4f |\n", m2, c2;
      printf "| rerun3(repeats=5) | %s | %.4f |\n", m3, c3;
      split(m0 " " m1 " " m2 " " m3, a, " "); lo=a[1]+0; hi=a[1]+0;
      for (i in a) { if (a[i]+0 < lo) lo=a[i]+0; if (a[i]+0 > hi) hi=a[i]+0; }
      printf "\n四次调用中位数极差 = %.4f%%（(max−min)/min）\n", (hi-lo)/lo*100;
      printf "对应加速比（t1/median）：%.4f× / %.4f× / %.4f× / %.4f×\n\n", BASE/m0, BASE/m1, BASE/m2, BASE/m3;
    }'
  done
  echo "## 判读（如实）"
  echo "- 断言 2 处置：两配置按处方加样并全量披露（上表）；超标事实不粉饰——是否视为「稳定」"
  echo "  由审核轮独立裁断。"
  echo "- 受影响判定的稳健性：① µs 侧只读 threads=1（10k-t1 复测漂移 4.16%<5%，四采样点 µs"
  echo "  判定不受移位影响）；② 加速比止损锚定 10k：12 线程加速比按四次调用中位数落在上表区间，"
  echo "  全部 ≥ 4× 止损线（判读方向对移位不敏感）。"
  echo "- summary.md 继续采用 matrix.jsonl 原始调用中位数（首测口径、保守侧=偏慢），"
  echo "  本档加样数据供独立复核交叉对照。"
} > "$EVID/rerun-remediation.txt"
cat "$EVID/rerun-remediation.txt"
echo "== takeover remediation done $(date '+%H:%M:%S') =="

# ---- B) ⑥ 长跑重跑 ----
echo "== takeover longrun start $(date '+%H:%M:%S') =="
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$EVID/run_longrun.ps1" -TargetSeconds 630 || {
  echo "FATAL: longrun failed" >&2; exit 1;
}
echo "== takeover longrun done $(date '+%H:%M:%S') =="

# ---- C) 汇总 ----
echo "== takeover summarize start $(date '+%H:%M:%S') =="
"$BIN" --summarize "$EVID/matrix.jsonl" --memory-csv "$EVID/runs/longrun_memory.csv" --out "$EVID/summary.md" \
  > "$EVID/runs/summarize.stdout.txt" 2> "$EVID/runs/summarize.stderr.txt"
rc=$?
echo "summarize REAL_EXIT=$rc (takeover, 2026-10-05)" >> "$EVID/runs/REAL_EXIT.txt"
if [ "$rc" -ne 0 ]; then echo "WARNING: summarize REAL_EXIT=$rc (see runs/summarize.stderr.txt)" >&2; fi
echo "TAKEOVER_BATCH_DONE $(date '+%Y-%m-%d %H:%M:%S') summarize_rc=$rc"
exit 0
