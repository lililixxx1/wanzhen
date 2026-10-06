#!/usr/bin/env bash
# T024 量测主脚本（M5-07 席位 9，D2/D4/D5）：干净窗口两腿一次性执行 + 三段留痕。
#
# 用法：bash docs/evidence/t024/measure_t024.sh（任意 CWD；脚本自锚 EVDIR）
#
# 前置（脚本断言）：
#   - 构建腿已完成并留空档：树内 target/release/host.exe（gate2 产物拷贝）与
#     target/release/render-spike.exe（gate3 产物拷贝）均在位；
#   - 机器空闲独占（Lead 承诺窗口内零并发负载）；跑前 rustc/cargo/host/
#     render-spike 进程计数 = 0、BRP 默认口 15702 空闲（不满足即中止，不测）。
#   窗口内本脚本不跑任何 cargo / 冒烟 / 额外 host 实例——只有两腿 exe + 抽样器。
#
# 腿 A（D2 观战档）：host --spectate --comp <10k 等比构成> --seed 42 \
#   --max-ticks 3600 --frame-capture runs/spectate-10k
#   10k 构成算式（等比缩放，源 = sim/src/bin/bench.rs:284-302 composition_for）：
#   每方 5000；q=5000/6=833、r=5000%6=2 → 表序前 r 个 +1（表序见 bench.rs:289-296）
#   ⇒ shieldman:834,heavyknight:834,pikeman:833,swordsman:833,archer:833,militia:833
#   （和 = 5000/方；host --comp 双方对称 ⇒ 总数 10,000 = 备忘录「万人常态 10k」）。
#   --max-ticks 3600 = 30Hz 下 120 s > warmup 5 + capture 65（窗内战局存活非冻结）。
#
# 腿 B（D4 C-1 关账）：render-spike --units 10000 --seed 42 --warmup-sec 5 \
#   --capture-sec 65 --res 1920x1080 --out runs/c1-render-spike/r1/out
#   （输出结构 r1/out/t10000/——t010 summarize.py 的收集器同构。）
#
# 三段留痕（D5）→ window/：pre-process.txt（全量快照+声明）/ load-samples.csv
#   （每 20 s CPU/CommitFree）/ post-process.txt（复扫）；events.txt = 时间线。
#
# 判定行生成（窗口关闭后，post-window 段自动执行）：
#   - 腿 A → summarize_t024.py --dir runs/spectate-10k（判定行 avg≥60 / 1%low≥45）
#   - 腿 B → docs/evidence/t010/summarize.py --root runs/c1-render-spike
#     （复用原脚本零改动；REAL_EXIT=3 = 缺 1k/2k/5k 档**预期**，t10000 判定行仍产出）
#
# 退出码：0 = 两腿均自成退且产物齐；非 0 = 中止（快照文件与 events.txt 保留现场）。
set -u
cd "$(dirname "$0")"
EVDIR="$PWD"
TREE_ROOT="$EVDIR/../../.."
HOST_EXE="$TREE_ROOT/target/release/host.exe"
RS_EXE="$TREE_ROOT/target/release/render-spike.exe"
RUNS="$EVDIR/runs"
WIN="$EVDIR/window"
mkdir -p "$RUNS" "$WIN"

# 10k 等比构成（算式与源行号见脚本头注）。
COMP10K='shieldman:834,heavyknight:834,pikeman:833,swordsman:833,archer:833,militia:833'

EV="$WIN/events.txt"
log() { echo "$@" | tee -a "$EV"; }

count_proc() { # name -> count
  powershell -NoProfile -Command "(Get-Process -Name '$1' -ErrorAction SilentlyContinue | Measure-Object).Count" 2>/dev/null | tr -d '\r'
}
port_free() { # port -> 0=free
  powershell -NoProfile -Command "if (Get-NetTCPConnection -State Listen -LocalPort $1 -ErrorAction SilentlyContinue) { exit 1 } else { exit 0 }" >/dev/null 2>&1
}
commit_free_g() {
  powershell -NoProfile -Command '[math]::Round(((Get-CimInstance Win32_OperatingSystem).TotalVirtualMemorySize - (Get-CimInstance Win32_OperatingSystem).FreeVirtualMemory)/1MB,1)' 2>/dev/null | tr -d '\r'
}

snapshot() { # $1 = out file, $2 = phase label
  {
    echo "== T024 window process snapshot [$2] $(date '+%Y-%m-%d %H:%M:%S') =="
    echo "== full process table (Id,ProcessName,CPU,WorkingSet64,Path) =="
    powershell -NoProfile -Command 'Get-Process | Sort-Object ProcessName,Id | Select-Object Id,ProcessName,CPU,WorkingSet64,Path | Format-Table -AutoSize | Out-String -Width 220'
    echo "== toolchain/load-relevant counts =="
    echo "rustc=$(count_proc rustc) cargo=$(count_proc cargo) host=$(count_proc host) render-spike=$(count_proc render-spike)"
    echo "commit_free_G=$(commit_free_g)"
    if port_free 15702; then echo "port_15702=free"; else echo "port_15702=OCCUPIED"; fi
  } > "$1" 2>&1
}

: > "$EV"
log "== T024 measure start $(date '+%Y-%m-%d %H:%M:%S') =="
[ -f "$HOST_EXE" ] || { log "FATAL: host.exe missing ($HOST_EXE)"; exit 9; }
[ -f "$RS_EXE" ] || { log "FATAL: render-spike.exe missing ($RS_EXE)"; exit 9; }
log "== binaries =="
sha256sum "$HOST_EXE" "$RS_EXE" | tee -a "$EV"

# ---- 三段留痕 ①：跑前快照 + 无并发负载声明 ----
snapshot "$WIN/pre-process.txt" "pre"
{
  echo "== DECLARATION (D5) =="
  echo "- sole-tenant window: Lead committed zero concurrent load; this worker runs no other task during the window."
  echo "- build legs (host/render-spike release) completed BEFORE the window; no cargo/rustc inside the window."
  echo "- no rpc smoke / no additional host instances inside the window (only the two leg executables)."
  echo "- load sampled every 20 s by sampler.ps1 -> window/load-samples.csv."
} >> "$WIN/pre-process.txt"
PRE_OK=1
[ "$(count_proc rustc)" = "0" ] || PRE_OK=0
[ "$(count_proc cargo)" = "0" ] || PRE_OK=0
[ "$(count_proc host)" = "0" ] || PRE_OK=0
[ "$(count_proc render-spike)" = "0" ] || PRE_OK=0
port_free 15702 || PRE_OK=0
if [ "$PRE_OK" != "1" ]; then
  log "ABORT: pre-check failed (see window/pre-process.txt) — re-queue window"
  exit 9
fi
log "pre-check OK (rustc/cargo/host/render-spike=0, port 15702 free)"

# ---- 三段留痕 ②：量测中抽样（每 20 s） ----
powershell -NoProfile -ExecutionPolicy Bypass -File "$EVDIR/sampler.ps1" -Out "$WIN/load-samples.csv" -IntervalSec 20 \
  2> "$WIN/sampler-stderr.log" &
SAMPLER_PID=$!
log "sampler started pid=$SAMPLER_PID -> window/load-samples.csv"
sleep 2

# ---- 腿 A：观战 10k 帧采集 ----
log "== LEG A START $(date '+%Y-%m-%d %H:%M:%S') =="
TA0=$(date +%s)
"$HOST_EXE" --spectate --comp "$COMP10K" --seed 42 --max-ticks 3600 \
  --frame-capture "$RUNS/spectate-10k" \
  > "$RUNS/legA-host-stdout.log" 2> "$RUNS/legA-host-stderr.log"
RC_A=$?
TA1=$(date +%s)
printf 'REAL_EXIT=%s\nWALL_SECONDS=%s\n' "$RC_A" "$((TA1-TA0))" > "$RUNS/legA-exit.txt"
log "== LEG A END $(date '+%H:%M:%S') rc=$RC_A wall=$((TA1-TA0))s =="

# 腿间空档（窗口内无构建/无其他负载；仅等待，保持采样连续）。
log "== GAP 30s =="
sleep 30

# ---- 腿 B：C-1 render-spike t10000 干净复测 ----
log "== LEG B START $(date '+%Y-%m-%d %H:%M:%S') =="
TB0=$(date +%s)
"$RS_EXE" --units 10000 --seed 42 --warmup-sec 5 --capture-sec 65 --res 1920x1080 \
  --out "$RUNS/c1-render-spike/r1/out" \
  > "$RUNS/legB-rs-stdout.log" 2> "$RUNS/legB-rs-stderr.log"
RC_B=$?
TB1=$(date +%s)
printf 'REAL_EXIT=%s\nWALL_SECONDS=%s\n' "$RC_B" "$((TB1-TB0))" > "$RUNS/legB-rs-exit.txt"
log "== LEG B END $(date '+%H:%M:%S') rc=$RC_B wall=$((TB1-TB0))s =="

# 停抽样器
kill "$SAMPLER_PID" 2>/dev/null
sleep 1
kill -9 "$SAMPLER_PID" 2>/dev/null
log "sampler stopped pid=$SAMPLER_PID"

# 抽样覆盖校验（D5：量测中每 20 s 一次；窗口 ≈ 200 s 量测段 ⇒ 期望 ≥ 8 行 = 表头 + ≥7 样本）
SAMPLE_LINES=$(wc -l < "$WIN/load-samples.csv" 2>/dev/null | tr -d ' ')
if [ "${SAMPLE_LINES:-0}" -ge 8 ]; then
  SAMPLE_OK=1
  log "sampler coverage OK: $SAMPLE_LINES lines (header + $((SAMPLE_LINES-1)) samples)"
else
  SAMPLE_OK=0
  log "SAMPLER INSUFFICIENT: ${SAMPLE_LINES:-0} lines (<8) -- window INVALID; see window/sampler-stderr.log"
fi

# ---- 三段留痕 ③：结束复扫 ----
snapshot "$WIN/post-process.txt" "post"
POST_OK=1
[ "$(count_proc rustc)" = "0" ] || POST_OK=0
[ "$(count_proc cargo)" = "0" ] || POST_OK=0
[ "$(count_proc host)" = "0" ] || POST_OK=0
[ "$(count_proc render-spike)" = "0" ] || POST_OK=0
[ "$SAMPLE_OK" = "1" ] || POST_OK=0
log "post-check rustc/cargo/host/render-spike zero: $([ "$POST_OK" = 1 ] && echo OK || echo 'NOT-OK (see window/post-process.txt / load-samples.csv)')"

# 产物存在性
for f in "$RUNS/spectate-10k/frames.csv" "$RUNS/spectate-10k/meta.json" \
         "$RUNS/c1-render-spike/r1/out/t10000/frames.csv" "$RUNS/c1-render-spike/r1/out/t10000/meta.json"; do
  if [ -s "$f" ]; then log "artifact OK: $f ($(wc -c < "$f") bytes)"; else log "artifact MISSING/EMPTY: $f"; fi
done

log "== WINDOW CLOSED $(date '+%Y-%m-%d %H:%M:%S') =="

# ---- post-window：判定行生成（不属窗口内负载） ----
log "== POST-WINDOW summaries =="
python "$EVDIR/summarize_t024.py" --dir "$RUNS/spectate-10k" --out "$EVDIR/summary-spectate.md" >> "$EV" 2>&1
log "summarize_t024.py REAL_EXIT=$?"
python "$TREE_ROOT/docs/evidence/t010/summarize.py" --root "$RUNS/c1-render-spike" --out "$EVDIR/summary-c1-render-spike.md" >> "$EV" 2>&1
log "t010/summarize.py REAL_EXIT=$? (3 = 缺 1k/2k/5k 档预期，t10000 判定行仍产出)"

log "== T024 measure done rc_A=$RC_A rc_B=$RC_B post_check=$POST_OK =="
[ "$RC_A" = "0" ] && [ "$RC_B" = "0" ] && [ "$POST_OK" = "1" ] || exit 1
exit 0
