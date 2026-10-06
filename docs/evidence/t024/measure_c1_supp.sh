#!/usr/bin/env bash
# T024 C-1 加样复测脚本（window-2；附录 G「复测超标即加样复测、全档披露」+ T012 先例）：
# 首样本（window-1 leg B）avg 漂移 +1.9%（带内）/ 1% low 漂移 +28.1%（超 ±25% 带，向上）
# → 加样 2 次（r2/r3），同干净窗口纪律（三段留痕 window-2）。
#
# 用法：bash docs/evidence/t024/measure_c1_supp.sh（前置同 measure_t024.sh：机器空闲独占、
# 无 cargo/冒烟在跑；本脚本同样只跑 render-spike 腿 + 抽样器）。
#
# 产物：
#   runs/c1-supp-r2/r2/out/t10000/{frames.csv,meta.json}   加样 #1
#   runs/c1-supp-r3/r3/out/t10000/{frames.csv,meta.json}   加样 #2
#   window/{pre-process-supp.txt,load-samples-supp.csv,post-process-supp.txt,events-supp.txt}
#   runs/legB-r2-*.log / legB-r3-*.log（stdout/stderr/退出码/墙钟）
#
# 退出码：0 = 两加样均自成退且抽样覆盖合规；非 0 = 中止（现场保留）。
set -u
cd "$(dirname "$0")"
EVDIR="$PWD"
TREE_ROOT="$EVDIR/../../.."
RS_EXE="$TREE_ROOT/target/release/render-spike.exe"
RUNS="$EVDIR/runs"
WIN="$EVDIR/window"
mkdir -p "$RUNS" "$WIN"

EV="$WIN/events-supp.txt"
log() { echo "$@" | tee -a "$EV"; }

count_proc() { powershell -NoProfile -Command "(Get-Process -Name '$1' -ErrorAction SilentlyContinue | Measure-Object).Count" 2>/dev/null | tr -d '\r'; }
port_free() { powershell -NoProfile -Command "if (Get-NetTCPConnection -State Listen -LocalPort $1 -ErrorAction SilentlyContinue) { exit 1 } else { exit 0 }" >/dev/null 2>&1; }
commit_free_g() {
  powershell -NoProfile -Command '[math]::Round(((Get-CimInstance Win32_OperatingSystem).TotalVirtualMemorySize - (Get-CimInstance Win32_OperatingSystem).FreeVirtualMemory)/1MB,1)' 2>/dev/null | tr -d '\r'
}
snapshot() { # $1 = out file, $2 = phase label
  {
    echo "== T024 window-2 process snapshot [$2] $(date '+%Y-%m-%d %H:%M:%S') =="
    echo "== full process table (Id,ProcessName,CPU,WorkingSet64,Path) =="
    powershell -NoProfile -Command 'Get-Process | Sort-Object ProcessName,Id | Select-Object Id,ProcessName,CPU,WorkingSet64,Path | Format-Table -AutoSize | Out-String -Width 220'
    echo "== toolchain/load-relevant counts =="
    echo "rustc=$(count_proc rustc) cargo=$(count_proc cargo) host=$(count_proc host) render-spike=$(count_proc render-spike)"
    echo "commit_free_G=$(commit_free_g)"
  } > "$1" 2>&1
}

: > "$EV"
log "== T024 C-1 supplement start $(date '+%Y-%m-%d %H:%M:%S') =="
[ -f "$RS_EXE" ] || { log "FATAL: render-spike.exe missing ($RS_EXE)"; exit 9; }
sha256sum "$RS_EXE" | tee -a "$EV"

snapshot "$WIN/pre-process-supp.txt" "pre"
{
  echo "== DECLARATION (D5, window-2) =="
  echo "- sole-tenant window: Lead committed zero concurrent load; worker runs no other task during the window."
  echo "- no cargo/rustc/smoke inside the window (only render-spike legs + sampler)."
} >> "$WIN/pre-process-supp.txt"
PRE_OK=1
[ "$(count_proc rustc)" = "0" ] || PRE_OK=0
[ "$(count_proc cargo)" = "0" ] || PRE_OK=0
[ "$(count_proc host)" = "0" ] || PRE_OK=0
[ "$(count_proc render-spike)" = "0" ] || PRE_OK=0
if [ "$PRE_OK" != "1" ]; then log "ABORT: window-2 pre-check failed"; exit 9; fi
log "window-2 pre-check OK"

powershell -NoProfile -ExecutionPolicy Bypass -File "$EVDIR/sampler.ps1" -Out "$WIN/load-samples-supp.csv" -IntervalSec 20 \
  2> "$WIN/sampler-supp-stderr.log" &
SAMPLER_PID=$!
log "sampler started pid=$SAMPLER_PID -> window/load-samples-supp.csv"
sleep 2

RC2=1; RC3=1
for R in r2 r3; do
  log "== SUPP $R START $(date '+%H:%M:%S') =="
  T0=$(date +%s)
  "$RS_EXE" --units 10000 --seed 42 --warmup-sec 5 --capture-sec 65 --res 1920x1080 \
    --out "$RUNS/c1-supp-$R/$R/out" \
    > "$RUNS/legB-$R-stdout.log" 2> "$RUNS/legB-$R-stderr.log"
  RC=$?
  T1=$(date +%s)
  printf 'REAL_EXIT=%s\nWALL_SECONDS=%s\n' "$RC" "$((T1-T0))" > "$RUNS/legB-$R-exit.txt"
  log "== SUPP $R END rc=$RC wall=$((T1-T0))s =="
  [ "$R" = "r2" ] && RC2=$RC || RC3=$RC
  [ "$R" = "r2" ] && { log "== GAP 30s =="; sleep 30; }
done

kill "$SAMPLER_PID" 2>/dev/null
sleep 1
kill -9 "$SAMPLER_PID" 2>/dev/null
log "sampler stopped pid=$SAMPLER_PID"
SAMPLE_LINES=$(wc -l < "$WIN/load-samples-supp.csv" 2>/dev/null | tr -d ' ')
if [ "${SAMPLE_LINES:-0}" -ge 8 ]; then
  SAMPLE_OK=1; log "sampler coverage OK: $SAMPLE_LINES lines"
else
  SAMPLE_OK=0; log "SAMPLER INSUFFICIENT: ${SAMPLE_LINES:-0} lines (<8) -- window-2 INVALID"
fi

snapshot "$WIN/post-process-supp.txt" "post"
POST_OK=1
[ "$(count_proc rustc)" = "0" ] || POST_OK=0
[ "$(count_proc cargo)" = "0" ] || POST_OK=0
[ "$(count_proc host)" = "0" ] || POST_OK=0
[ "$(count_proc render-spike)" = "0" ] || POST_OK=0
[ "$SAMPLE_OK" = "1" ] || POST_OK=0
log "post-check rustc/cargo/host/render-spike zero: $([ "$POST_OK" = 1 ] && echo OK || echo 'NOT-OK')"

for f in "$RUNS/c1-supp-r2/r2/out/t10000/frames.csv" "$RUNS/c1-supp-r3/r3/out/t10000/frames.csv"; do
  if [ -s "$f" ]; then log "artifact OK: $f ($(wc -c < "$f") bytes)"; else log "artifact MISSING/EMPTY: $f"; fi
done
log "== WINDOW-2 CLOSED $(date '+%Y-%m-%d %H:%M:%S') =="

log "== POST-WINDOW summaries (supp) =="
python "$TREE_ROOT/docs/evidence/t010/summarize.py" --root "$RUNS/c1-supp-r2" --out "$EVDIR/summary-c1-supp-r2.md" >> "$EV" 2>&1
log "summarize r2 REAL_EXIT=$? (3 = 缺 1k/2k/5k 档预期)"
python "$TREE_ROOT/docs/evidence/t010/summarize.py" --root "$RUNS/c1-supp-r3" --out "$EVDIR/summary-c1-supp-r3.md" >> "$EV" 2>&1
log "summarize r3 REAL_EXIT=$? (3 = 缺 1k/2k/5k 档预期)"

log "== C-1 supplement done rc_r2=$RC2 rc_r3=$RC3 post_check=$POST_OK =="
[ "$RC2" = "0" ] && [ "$RC3" = "0" ] && [ "$POST_OK" = "1" ] || exit 1
exit 0
