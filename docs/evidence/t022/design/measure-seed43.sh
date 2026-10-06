#!/usr/bin/env bash
# T022 设计段 seed 43 敏感性数据（三定选构型；设计注用，不作断言锚——断言锚只钉 seed 42）
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../../target/release/host.exe"
LOG="measure-seed43-run.log"
PORT=15705
: > "$LOG"
exec > >(tee -a "$LOG") 2>&1
ID=0
brp() {
  ID=$((ID+1))
  local body="{\"jsonrpc\":\"2.0\",\"id\":$ID,\"method\":\"$1\",\"params\":$2}"
  echo "--- REQ #$ID: $body" >&2
  local resp
  resp=$(curl -s -m 300 -H 'Content-Type: application/json' -d "$body" "http://127.0.0.1:$PORT/")
  echo "--- RESP #$ID: $resp" >&2
  echo "$resp"
}
echo "== T022 design seed-43 sensitivity $(date '+%Y-%m-%d %H:%M:%S') =="
"$HOST_EXE" --port $PORT > host-s43-stdout.log 2> host-s43-stderr.log &
HOST_PID=$!
READY=0
for i in $(seq 1 150); do
  if curl -s -m 2 -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then READY=1; break; fi
  sleep 0.2
done
[ "$READY" -ne 1 ] && { echo "FATAL: port not ready"; kill $HOST_PID 2>/dev/null; exit 1; }
measure() {
  local label="$1" red="$2" blue="$3" mt="$4"
  echo "===== $label (seed=43 lane=60 max_ticks=$mt) ====="
  brp game.deploy "{\"seed\":43,\"red\":$red,\"blue\":$blue,\"lane_len_m\":60,\"max_ticks\":$mt}" >/dev/null
  brp game.outcome '{}'
}
measure s43-ch1a-sw10-mi30 '[{"kind":"swordsman","count":10}]' '[{"kind":"militia","count":30}]' 3600
measure s43-ch2a-hk10-mi30 '[{"kind":"heavyknight","count":10}]' '[{"kind":"militia","count":30}]' 1800
measure s43-ch3a-sh20-mi60 '[{"kind":"shieldman","count":20}]' '[{"kind":"militia","count":60}]' 1800
kill $HOST_PID 2>/dev/null
wait $HOST_PID 2>/dev/null
echo "== done =="
