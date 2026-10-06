#!/usr/bin/env bash
# T020 D2 BRP 参数序列重放矩阵（七配置 A~G，席位 3 判定面）：每配置 = 完整
# deploy→run→采 hash（E/G 另采 outcome）序列**整段重放两遍**、逐位对拍；
# 含跨线程对（A/B、C/D）与 T018 P2-2 移交项的 BRP 侧钉死（G）。
# 体例沿 t018 整改版：REQ/RESP 原文走 stderr（>&2）落 replay-matrix-run.log；
# 判定行 / SUMMARY / SCRIPT_EXIT 收尾；观测值逐配置落 replay-matrix.jsonl。
# 用法：bash replay_matrix.sh（树根执行；端口 15714；非帧敏量测，机器基本空闲）
#
# 锚值字面量逐字使用勿换算（T020 派工单「已核实事实」3 + 归档）：
#   ANCHOR2   = 0x958c5938c8682529  seed42 默认构成 1800t（A/B；M0 T004 归档）
#   SHORTLANE = 0xfdbc4995554ee691  seed42 民兵5v5 lane10m 灭绝@878（E；T018
#               归档 docs/evidence/t018/brp-smoke-run.log:90-99）
#   H_C = melee-brawl seed7 threads1 1800t —— 本卡新记录值（PIT-M-002：占位
#         首测→实测回填→复跑全绿；回填后为跨版本回归常量）
#   H_F = 混编 seed2026 lane100 threads3 1800t —— 本卡新锚（同 PIT-M-002）
#
# F 的 lane_len_m=100 选定算式（附录 B.2 ②「接敌可达性」，带 sim 源行号）：
#   算式：(队头初始距离 − 射程阈值) ÷ 合闭合速度 ≤ 判定窗口 tick（本单窗口
#   上限 1500）。F 红 {archer×8, militia×2} 蓝 {shieldman×6, pikeman×4}：
#   最慢闭合对 = archer 0.08 m/tick（sim/src/units.rs:129）+ shieldman 0.05
#   （units.rs:85）⇒ 合闭合 0.13 m/tick；半径 0.4（units.rs:130）+ 0.5
#   （units.rs:86）⇒ 射程阈值 = 0.4+0.5+0.2（MELEE_MARGIN_Q32，units.rs:34）
#   = 1.1 m；队头初始距离 = lane − (0.4+0.5)（布阵队头 x=radius_0 + 蓝镜像，
#   sim/src/world.rs:601-622）⇒ lane 100：(100 − 0.9 − 1.1) ÷ 0.13 =
#   98 ÷ 0.13 ≈ 753.85 → ≤ 754 tick ≤ 1500 ✓（最慢对口径覆盖任意 seed 之
#   队头兵种组合——其余组合合闭合 ≥ 0.14）。100 与 melee-brawl 同尺度，
#   1800 cap 内留 ≥ 1046 tick 战斗段。
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../target/release/host.exe"
LOG="replay-matrix-run.log"
JSONL="replay-matrix.jsonl"
PORT=15714

ANCHOR2="0x958c5938c8682529"
SHORTLANE_HASH="0xfdbc4995554ee691"
# H_C/H_F：占位首测（留痕 replay-matrix-measure.log）→ 实测回填（PIT-M-002）。
# H_C 与 T021 预设冒烟（t021/preset-smoke-run.log 同参）实测值逐位一致 = 跨树旁证。
H_C="0xb82a248ff23515e2" # melee-brawl seed7 t1 1800t（回归常量）
H_F="0x185fe8c22adeec36" # 混编 seed2026 lane100 t3 1800t（本卡新锚）

PASS=0; FAIL=0
: > "$LOG"
: > "$JSONL"
exec > >(tee -a "$LOG") 2>&1

check() { # name condition(0=pass)
  if [ "$2" -eq 0 ]; then PASS=$((PASS+1)); echo "[$1] PASS"; else FAIL=$((FAIL+1)); echo "[$1] FAIL"; fi
}

ID=0
brp() { # method params-json -> echoes response body（REQ/RESP 原文走 stderr 落档）
  ID=$((ID+1))
  local body="{\"jsonrpc\":\"2.0\",\"id\":$ID,\"method\":\"$1\",\"params\":$2}"
  echo "--- REQ #$ID: $body" >&2
  local resp
  resp=$(curl -s -m 120 -H 'Content-Type: application/json' -d "$body" "http://127.0.0.1:$PORT/")
  echo "--- RESP #$ID: $resp" >&2
  echo "$resp"
}

# 归一化：取 JSON-RPC result 对象本体（扁平对象；去掉随 ID 变化的信封，两遍对拍用）
res_only() { echo "$1" | grep -o '"result":{[^{}]*}' | head -1; }
hex_key() { echo "$1" | grep -o "\"$2\":\"0x[0-9a-f]*\"" | head -1 | grep -o '0x[0-9a-f]*'; }
# 尾取：观测串内同名数字键（如 deploy 的 tick:0 与 run 的 tick:N）取最后一次出现
int_key() { echo "$1" | grep -o "\"$2\":[0-9]*" | tail -1 | grep -o '[0-9]*'; }
str_key() { echo "$1" | grep -o "\"$2\":\"[a-z]*\"" | head -1 | grep -o '[a-z]*"$' | tr -d '"'; }

# 配置序列（整段重放对象）：deploy → run_to_tick [→ outcome]，输出归一化观测串
run_seq() { # $1=deploy params $2=ticks $3="outcome"（可选）
  local d r o
  d=$(brp game.deploy "$1")
  r=$(brp game.run_to_tick "{\"ticks\":$2}")
  local out="$(res_only "$d") | $(res_only "$r")"
  if [ "${3:-}" = "outcome" ]; then
    o=$(brp game.outcome '{}')
    out="$out | $(res_only "$o")"
  fi
  echo "$out"
}

echo "== T020 replay matrix $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE =="

# 预检：端口已被占（残留 host）即失败退出——防对拍错对象（脚本自起宿主为唯一被测）
if curl -s -m 2 -H 'Content-Type: application/json' \
     -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
     "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then
  echo "== PREFLIGHT FAIL: port $PORT already serving (残留 host?) =="
  echo "== SCRIPT_EXIT=2 =="
  exit 2
fi

"$HOST_EXE" --port $PORT > host-replay-stdout.log 2> host-replay-stderr.log &
HOST_PID=$!
echo "== host pid $HOST_PID =="

READY=0
for i in $(seq 1 150); do
  if curl -s -m 2 -H 'Content-Type: application/json' \
       -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
       "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then READY=1; break; fi
  sleep 0.2
done
check CHK-00-port-ready $([ "$READY" -eq 1 ]; echo $?)

# 七配置 deploy 参数（preset 路径 / 显式覆盖路径 / 短 lane 战斗路径）
PA='{"preset":"default","seed":42,"threads":1}'
PB='{"preset":"default","seed":42,"threads":12}'
PC='{"preset":"melee-brawl","seed":7,"threads":1}'
PD='{"preset":"melee-brawl","seed":7,"threads":12}'
PE='{"seed":42,"red":[{"kind":"militia","count":5}],"blue":[{"kind":"militia","count":5}],"lane_len_m":10,"threads":1}'
PF='{"seed":2026,"red":[{"kind":"archer","count":8},{"kind":"militia","count":2}],"blue":[{"kind":"shieldman","count":6},{"kind":"pikeman","count":4}],"lane_len_m":100,"threads":3}'
PG='{"preset":"default","seed":42,"threads":1}'

# ── A：default seed42 t1，1800t；两遍整段对拍 + 锚②命中 ──
A1=$(run_seq "$PA" 1800); A2=$(run_seq "$PA" 1800)
AH1=$(hex_key "$A1" state_hash); AH2=$(hex_key "$A2" state_hash)
echo "A: run1=$AH1 run2=$AH2"
check CHK-A-replay-bitexact $([ "$A1" = "$A2" ]; echo $?)
check CHK-A-anchor2-hit $([ "$AH1" = "$ANCHOR2" ]; echo $?)
printf '{"id":"A","deploy":%s,"ticks":1800,"run1_hash":"%s","run2_hash":"%s","tick":%s,"replay_bitexact":%s,"anchor":"%s","anchor_hit":%s}\n' \
  "$PA" "$AH1" "$AH2" "$(int_key "$A1" tick)" \
  "$([ "$A1" = "$A2" ] && echo true || echo false)" "$ANCHOR2" \
  "$([ "$AH1" = "$ANCHOR2" ] && echo true || echo false)" >> "$JSONL"

# ── B：default seed42 **threads12**，1800t；跨线程对（B hash == 锚② == A hash） ──
B1=$(run_seq "$PB" 1800); B2=$(run_seq "$PB" 1800)
BH1=$(hex_key "$B1" state_hash); BH2=$(hex_key "$B2" state_hash)
echo "B: run1=$BH1 run2=$BH2"
check CHK-B-replay-bitexact $([ "$B1" = "$B2" ]; echo $?)
check CHK-B-cross-thread-anchor2 $([ "$BH1" = "$ANCHOR2" ]; echo $?)
printf '{"id":"B","deploy":%s,"ticks":1800,"run1_hash":"%s","run2_hash":"%s","tick":%s,"replay_bitexact":%s,"anchor":"%s","anchor_hit":%s}\n' \
  "$PB" "$BH1" "$BH2" "$(int_key "$B1" tick)" \
  "$([ "$B1" = "$B2" ] && echo true || echo false)" "$ANCHOR2" \
  "$([ "$BH1" = "$ANCHOR2" ] && echo true || echo false)" >> "$JSONL"

# ── C：melee-brawl seed7 t1，1800t；记录 H_C（新记录值）+ 两遍对拍 ──
C1=$(run_seq "$PC" 1800); C2=$(run_seq "$PC" 1800)
CH1=$(hex_key "$C1" state_hash); CH2=$(hex_key "$C2" state_hash)
echo "C: run1=$CH1 run2=$CH2 (expected H_C=$H_C)"
check CHK-C-replay-bitexact $([ "$C1" = "$C2" ]; echo $?)
check CHK-C-recorded-anchor-hit $([ "$CH1" = "$H_C" ]; echo $?)
printf '{"id":"C","deploy":%s,"ticks":1800,"run1_hash":"%s","run2_hash":"%s","tick":%s,"replay_bitexact":%s,"recorded":"%s","recorded_hit":%s}\n' \
  "$PC" "$CH1" "$CH2" "$(int_key "$C1" tick)" \
  "$([ "$C1" = "$C2" ] && echo true || echo false)" "$H_C" \
  "$([ "$CH1" = "$H_C" ] && echo true || echo false)" >> "$JSONL"

# ── D：melee-brawl seed7 **threads12**，1800t；跨线程对（D hash == H_C == C hash） ──
D1=$(run_seq "$PD" 1800); D2=$(run_seq "$PD" 1800)
DH1=$(hex_key "$D1" state_hash); DH2=$(hex_key "$D2" state_hash)
echo "D: run1=$DH1 run2=$DH2"
check CHK-D-replay-bitexact $([ "$D1" = "$D2" ]; echo $?)
check CHK-D-cross-thread-HC $([ "$DH1" = "$H_C" ]; echo $?)
printf '{"id":"D","deploy":%s,"ticks":1800,"run1_hash":"%s","run2_hash":"%s","tick":%s,"replay_bitexact":%s,"recorded":"%s","recorded_hit":%s}\n' \
  "$PD" "$DH1" "$DH2" "$(int_key "$D1" tick)" \
  "$([ "$D1" = "$D2" ] && echo true || echo false)" "$H_C" \
  "$([ "$DH1" = "$H_C" ] && echo true || echo false)" >> "$JSONL"

# ── E：短 lane 真交战（民兵 5v5 lane 10m）；冻结@878 + 终局四元组锚（T018 CHK-16） ──
E1=$(run_seq "$PE" 1800 outcome); E2=$(run_seq "$PE" 1800 outcome)
ET1=$(int_key "$E1" tick); EH1=$(hex_key "$E1" state_hash)
EW1=$(str_key "$E1" winner); EE1=$(int_key "$E1" end_tick)
EF1=$(hex_key "$E1" final_hash)
ER1=$(echo "$E1" | grep -o '"alive_red":[0-9]*' | tail -1 | grep -o '[0-9]*')
EB1=$(echo "$E1" | grep -o '"alive_blue":[0-9]*' | tail -1 | grep -o '[0-9]*')
echo "E: run1 tick=$ET1 hash=$EH1 outcome=winner:$EW1 end:$EE1 alive:$ER1/$EB1 final:$EF1"
check CHK-E-replay-bitexact $([ "$E1" = "$E2" ]; echo $?)
check CHK-E-frozen-tick878 $([ "$ET1" = "878" ]; echo $?)
check CHK-E-shortlane-hash $([ "$EH1" = "$SHORTLANE_HASH" ]; echo $?)
check CHK-E-outcome-red $([ "$EW1" = "red" ]; echo $?)
check CHK-E-outcome-endtick878 $([ "$EE1" = "878" ]; echo $?)
check CHK-E-outcome-alive-1-0 $([ "$ER1" = "1" ] && [ "$EB1" = "0" ]; echo $?)
check CHK-E-outcome-finalhash-anchor $([ "$EF1" = "$SHORTLANE_HASH" ]; echo $?)
printf '{"id":"E","deploy":%s,"ticks":1800,"run1_tick":%s,"run1_hash":"%s","run2_hash":"%s","replay_bitexact":%s,"outcome_winner":"%s","outcome_end_tick":%s,"outcome_alive_red":%s,"outcome_alive_blue":%s,"outcome_final_hash":"%s","anchor":"%s","anchor_hit":%s}\n' \
  "$PE" "$ET1" "$EH1" "$(hex_key "$E2" state_hash)" \
  "$([ "$E1" = "$E2" ] && echo true || echo false)" "$EW1" "$EE1" "$ER1" "$EB1" "$EF1" \
  "$SHORTLANE_HASH" "$([ "$EF1" = "$SHORTLANE_HASH" ] && [ "$EH1" = "$SHORTLANE_HASH" ] && echo true || echo false)" >> "$JSONL"

# ── F：混编（红 archer×8+militia×2 vs 蓝 shieldman×6+pikeman×4）lane 100 t3，
#        1800t；新锚 H_F 记录 + 两遍对拍（lane 算式见脚本头注）+ 接敌实测旁证
#        （B.2 ②/③：1800t 后双方战损 > 0 ⇒ 接敌真发生，算式 ≤754 tick 可达） ──
F1=$(run_seq "$PF" 1800); F2=$(run_seq "$PF" 1800)
FH1=$(hex_key "$F1" state_hash); FH2=$(hex_key "$F2" state_hash)
echo "F: run1=$FH1 run2=$FH2 tick=$(int_key "$F1" tick) (expected H_F=$H_F)"
check CHK-F-replay-bitexact $([ "$F1" = "$F2" ]; echo $?)
check CHK-F-new-anchor-hit $([ "$FH1" = "$H_F" ]; echo $?)
F3=$(run_seq "$PF" 1800 outcome)
FR3=$(echo "$F3" | grep -o '"alive_red":[0-9]*' | tail -1 | grep -o '[0-9]*')
FB3=$(echo "$F3" | grep -o '"alive_blue":[0-9]*' | tail -1 | grep -o '[0-9]*')
echo "F3: outcome alive_red=$FR3 alive_blue=$FB3 (初始 10/10——战损 = 接敌实据)"
check CHK-F-contact-losses $([ $((FR3 + FB3)) -lt 20 ]; echo $?)
printf '{"id":"F","deploy":%s,"ticks":1800,"run1_hash":"%s","run2_hash":"%s","tick":%s,"replay_bitexact":%s,"new_anchor":"%s","new_anchor_hit":%s,"contact_outcome_alive_red":%s,"contact_outcome_alive_blue":%s,"contact_verified":%s}\n' \
  "$PF" "$FH1" "$FH2" "$(int_key "$F1" tick)" \
  "$([ "$F1" = "$F2" ] && echo true || echo false)" "$H_F" \
  "$([ "$FH1" = "$H_F" ] && echo true || echo false)" "$FR3" "$FB3" \
  "$([ $((FR3 + FB3)) -lt 20 ] && echo true || echo false)" >> "$JSONL"

# ── G：default seed42 t1，**2000t**（越过 max_ticks 1800）；P2-2 BRP 侧钉死：
#        tick==2000、outcome draw/end_tick 2000/final_hash == run 后 state_hash ──
G1=$(run_seq "$PG" 2000 outcome); G2=$(run_seq "$PG" 2000 outcome)
GT1=$(int_key "$G1" tick); GH1=$(hex_key "$G1" state_hash)
GW1=$(str_key "$G1" winner); GE1=$(int_key "$G1" end_tick)
GF1=$(hex_key "$G1" final_hash)
echo "G: run1 tick=$GT1 hash=$GH1 outcome=winner:$GW1 end:$GE1 final:$GF1"
check CHK-G-replay-bitexact $([ "$G1" = "$G2" ]; echo $?)
check CHK-G-tick2000 $([ "$GT1" = "2000" ]; echo $?)
check CHK-G-outcome-draw $([ "$GW1" = "draw" ]; echo $?)
check CHK-G-outcome-endtick2000 $([ "$GE1" = "2000" ]; echo $?)
check CHK-G-finalhash-eq-runhash $([ "$GF1" = "$GH1" ]; echo $?)
printf '{"id":"G","deploy":%s,"ticks":2000,"run1_tick":%s,"run1_hash":"%s","run2_hash":"%s","replay_bitexact":%s,"outcome_winner":"%s","outcome_end_tick":%s,"outcome_final_hash":"%s","finalhash_eq_runhash":%s}\n' \
  "$PG" "$GT1" "$GH1" "$(hex_key "$G2" state_hash)" \
  "$([ "$G1" = "$G2" ] && echo true || echo false)" "$GW1" "$GE1" "$GF1" \
  "$([ "$GF1" = "$GH1" ] && echo true || echo false)" >> "$JSONL"

# 进程存活收尾（七配置全程无 panic / 无击穿；宿主由本脚本自起自杀）
kill -0 "$HOST_PID" 2>/dev/null; check CHK-99-process-alive $?
kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null

echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
