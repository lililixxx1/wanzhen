#!/usr/bin/env bash
# T022 Lead 设计段候选构型实测（2026-10-07）：挑战预设 ≥3 选型数据。
# 派工单锚值来源 = 本档实测（速算推导仅供核对，以脚本为准——B.2 ⑤ 双盲：
# worker 须独立复测，不一致即上报停止）。
# 语义前提（T021 判定注）：贴身接敌面恒 1v1 漏斗（sim/src/world.rs:342/733），
# 对局 = 前线顺序决斗 + 队列推进；max_ticks 为挑战自带参数（BRP 域 1..=14400，
# rpc.rs deploy 校验段）。
# 用法：bash measure-candidates.sh（仓库根执行也可，脚本自 cd；端口 15705；
# 非帧敏量测，机器基本空闲即可）
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../../target/release/host.exe"
LOG="measure-candidates-run.log"
PORT=15705

: > "$LOG"
exec > >(tee -a "$LOG") 2>&1

ID=0
brp() { # method params-json -> echoes response body（REQ/RESP 原文走 stderr 落档）
  ID=$((ID+1))
  local body="{\"jsonrpc\":\"2.0\",\"id\":$ID,\"method\":\"$1\",\"params\":$2}"
  echo "--- REQ #$ID: $body" >&2
  local resp
  resp=$(curl -s -m 300 -H 'Content-Type: application/json' -d "$body" "http://127.0.0.1:$PORT/")
  echo "--- RESP #$ID: $resp" >&2
  echo "$resp"
}

echo "== T022 design candidate measurement $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE, BRP port $PORT =="

"$HOST_EXE" --port $PORT > host-stdout.log 2> host-stderr.log &
HOST_PID=$!
echo "== host pid $HOST_PID =="

READY=0
for i in $(seq 1 150); do
  if curl -s -m 2 -H 'Content-Type: application/json' \
       -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
       "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then READY=1; break; fi
  sleep 0.2
done
if [ "$READY" -ne 1 ]; then echo "FATAL: port $PORT not ready"; kill $HOST_PID 2>/dev/null; exit 1; fi
echo "== port ready =="

# measure <label> <seed> <red-json> <blue-json> <lane> <max_ticks>
measure() {
  local label="$1" seed="$2" red="$3" blue="$4" lane="$5" mt="$6"
  echo ""
  echo "===== $label (seed=$seed lane=$lane max_ticks=$mt) ====="
  local d o
  d=$(brp game.deploy "{\"seed\":$seed,\"red\":$red,\"blue\":$blue,\"lane_len_m\":$lane,\"max_ticks\":$mt}")
  echo "DEPLOY: $d"
  o=$(brp game.outcome '{}')
  echo "OUTCOME: $o"
}

# ── 以少胜多候选（红少胜蓝多：1v1 漏斗下杀效率最高 = swordsman 18dmg/击 vs 民兵 50hp ⇒ 3 击 75t/杀，B.2 ② 算式）──
measure ch1a-sw10-vs-mi30-mt3600 42 '[{"kind":"swordsman","count":10}]' '[{"kind":"militia","count":30}]' 60 3600
measure ch1b-sw8-vs-mi24-mt3600  42 '[{"kind":"swordsman","count":8}]'  '[{"kind":"militia","count":24}]' 60 3600
measure ch1c-pi10-vs-mi30-mt3600 42 '[{"kind":"pikeman","count":10}]'   '[{"kind":"militia","count":30}]' 60 3600

# ── 克制极端例候选（无甲克重甲 ×1.5 / 重甲攻无甲被克 ×0.67：民兵对重骑 9dmg/20t vs 骑对民兵 9dmg/45t）──
measure ch2a-hk10-vs-mi30-mt1800 42 '[{"kind":"heavyknight","count":10}]' '[{"kind":"militia","count":30}]' 60 1800
measure ch2b-hk10-vs-mi20-mt1800 42 '[{"kind":"heavyknight","count":10}]' '[{"kind":"militia","count":20}]' 60 1800
measure ch2c-hk6-vs-mi18-mt1800  42 '[{"kind":"heavyknight","count":6}]'  '[{"kind":"militia","count":18}]' 60 1800

# ── 上限判定候选（T021 last-stand 同构：僵局收于上限，resolve_by_hp 判定）──
measure ch3a-sh20-vs-mi60-mt1800 42 '[{"kind":"shieldman","count":20}]' '[{"kind":"militia","count":60}]' 60 1800

kill $HOST_PID 2>/dev/null
wait $HOST_PID 2>/dev/null
echo ""
echo "== done, host stopped =="
