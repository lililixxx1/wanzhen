#!/usr/bin/env bash
# T022 挑战冒烟（D6）：BRP 端口 15706（独占）——三挑战逐条 BRP 驱动对拍
# （game.deploy 挑战参数 → game.outcome → 六字段 vs 数据档锚：deploy_hash +
# winner/end_tick/alive_red/alive_blue/final_hash）+ run_tests 套件面
# （challenges 3/3、m5-all 12/12、m5-core 9 不回归、缺省套件 4 不回归）+
# rpc.discover 面 run_tests 不变；REQ/RESP 原文走 stderr 落 challenge-smoke-run.log。
# 锚值逐字取自 docs/evidence/t022/design/measure-candidates-run.log（勿改写）；
# 挑战参数与 host/src/challenges.rs 注册表逐字一致（数据档单一来源）。
# 用法：bash challenge_smoke.sh（机器基本空闲；非帧敏量测，无独占窗口要求）
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../target/release/host.exe"
LOG="challenge-smoke-run.log"
PORT=15706

PASS=0; FAIL=0
: > "$LOG"
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

echo "== T022 challenge smoke $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE =="

"$HOST_EXE" --port $PORT > host-stdout.log 2> host-stderr.log &
HOST_PID=$!
echo "== host pid $HOST_PID (port $PORT) =="

# 端口就绪等待（rpc.discover 通即续）
READY=0
for i in $(seq 1 150); do
  if curl -s -m 2 -H 'Content-Type: application/json' \
       -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
       "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then READY=1; break; fi
  sleep 0.2
done
check CHK-00-port-ready-15706 $([ "$READY" -eq 1 ]; echo $?)

# CHK-01 rpc.discover 面 run_tests 不变（T022 不改方法面/不加方法）
DISC=$(brp rpc.discover '{}')
echo "$DISC" | grep -qF '"game.run_tests"'; check CHK-01-discover-run-tests $?

# ── 三挑战 BRP 驱动对拍（六字段 vs 数据档锚） ──
# CHK-02 few-elite：swordsman:10 / militia:30 / lane60 / max_ticks3600 / seed42
# 锚：deploy 0x583ddfef1fc446cf；outcome red/2474/7/0 0xa48ce79b1e83e105
R=$(brp game.deploy '{"seed":42,"red":[{"kind":"swordsman","count":10}],"blue":[{"kind":"militia","count":30}],"lane_len_m":60,"max_ticks":3600}')
echo "$R" | grep -qF '"deploy_hash":"0x583ddfef1fc446cf"'; check CHK-02-few-elite-deploy-hash $?
R=$(brp game.outcome '{}')
echo "$R" | grep -qF '"winner":"red"'; check CHK-02b-few-elite-winner $?
echo "$R" | grep -qF '"end_tick":2474'; check CHK-02c-few-elite-end-tick $?
echo "$R" | grep -qF '"alive_red":7'; check CHK-02d-few-elite-alive-red $?
echo "$R" | grep -qF '"alive_blue":0'; check CHK-02e-few-elite-alive-blue $?
echo "$R" | grep -qF '"final_hash":"0xa48ce79b1e83e105"'; check CHK-02f-few-elite-final-hash $?

# CHK-03 counter-militia：heavyknight:10 / militia:30 / lane60 / max_ticks1800 / seed42
# 锚：deploy 0x6a96df6f3c94e741；outcome blue/1800/6/24 0x3247aae383d5d5bd
R=$(brp game.deploy '{"seed":42,"red":[{"kind":"heavyknight","count":10}],"blue":[{"kind":"militia","count":30}],"lane_len_m":60,"max_ticks":1800}')
echo "$R" | grep -qF '"deploy_hash":"0x6a96df6f3c94e741"'; check CHK-03-counter-militia-deploy-hash $?
R=$(brp game.outcome '{}')
echo "$R" | grep -qF '"winner":"blue"'; check CHK-03b-counter-militia-winner $?
echo "$R" | grep -qF '"end_tick":1800'; check CHK-03c-counter-militia-end-tick $?
echo "$R" | grep -qF '"alive_red":6'; check CHK-03d-counter-militia-alive-red $?
echo "$R" | grep -qF '"alive_blue":24'; check CHK-03e-counter-militia-alive-blue $?
echo "$R" | grep -qF '"final_hash":"0x3247aae383d5d5bd"'; check CHK-03f-counter-militia-final-hash $?

# CHK-04 iron-wall：shieldman:20 / militia:60 / lane60 / max_ticks1800 / seed42
# 锚：deploy 0x9a926533cbf1d445；outcome blue/1800/15/56 0x81166458a5437951
R=$(brp game.deploy '{"seed":42,"red":[{"kind":"shieldman","count":20}],"blue":[{"kind":"militia","count":60}],"lane_len_m":60,"max_ticks":1800}')
echo "$R" | grep -qF '"deploy_hash":"0x9a926533cbf1d445"'; check CHK-04-iron-wall-deploy-hash $?
R=$(brp game.outcome '{}')
echo "$R" | grep -qF '"winner":"blue"'; check CHK-04b-iron-wall-winner $?
echo "$R" | grep -qF '"end_tick":1800'; check CHK-04c-iron-wall-end-tick $?
echo "$R" | grep -qF '"alive_red":15'; check CHK-04d-iron-wall-alive-red $?
echo "$R" | grep -qF '"alive_blue":56'; check CHK-04e-iron-wall-alive-blue $?
echo "$R" | grep -qF '"final_hash":"0x81166458a5437951"'; check CHK-04f-iron-wall-final-hash $?

# CHK-05 challenges 套件（判定面接入）：total 3 / passed 3 / failed 0 + 逐断言名
R=$(brp game.run_tests '{"suite":"challenges"}')
echo "$R" | grep -qF '"suite":"challenges"'; check CHK-05-suite-challenges-name $?
echo "$R" | grep -qF '"total":3'; check CHK-05b-challenges-total3 $?
echo "$R" | grep -qF '"passed":3'; check CHK-05c-challenges-passed3 $?
echo "$R" | grep -qF '"failed":0'; check CHK-05d-challenges-failed0 $?
echo "$R" | grep -qF '"name":"challenge::few-elite","pass":true'; check CHK-05e-assertion-few-elite $?
echo "$R" | grep -qF '"name":"challenge::counter-militia","pass":true'; check CHK-05f-assertion-counter-militia $?
echo "$R" | grep -qF '"name":"challenge::iron-wall","pass":true'; check CHK-05g-assertion-iron-wall $?

# CHK-06 m5-all 聚合（M5 完整判定面）：total 12 / passed 12 / failed 0
R=$(brp game.run_tests '{"suite":"m5-all"}')
echo "$R" | grep -qF '"suite":"m5-all"'; check CHK-06-m5-all-suite-name $?
echo "$R" | grep -qF '"total":12'; check CHK-06b-m5-all-total12 $?
echo "$R" | grep -qF '"passed":12'; check CHK-06c-m5-all-passed12 $?
echo "$R" | grep -qF '"failed":0'; check CHK-06d-m5-all-failed0 $?

# CHK-07 m5-core 不回归：total 9 / passed 9
R=$(brp game.run_tests '{"suite":"m5-core"}')
echo "$R" | grep -qF '"total":9'; check CHK-07-m5-core-total9 $?
echo "$R" | grep -qF '"passed":9'; check CHK-07b-m5-core-passed9 $?

# CHK-08 缺省套件不回归：t018-smoke / total 4 / passed 4
R=$(brp game.run_tests '{}')
echo "$R" | grep -qF '"suite":"t018-smoke"'; check CHK-08-default-suite-name $?
echo "$R" | grep -qF '"total":4'; check CHK-08b-default-total4 $?
echo "$R" | grep -qF '"passed":4'; check CHK-08c-default-passed4 $?

# CHK-09 进程存活收尾（全程无 panic / 无击穿）
kill -0 "$HOST_PID" 2>/dev/null; check CHK-09-process-alive $?
kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null

echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
