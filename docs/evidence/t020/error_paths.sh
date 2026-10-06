#!/usr/bin/env bash
# T020 D3 错误路径集（整改版 18 发 30 检查）：非法参数（seed 形态 / 未知 kind /
# count 负数 / 单方超上限 / threads 与 max_ticks 出域 / lane 0 / 缺 ticks）/
# 未布阵先 state_hash（4001）/ 未知 preset / 未知 suite / screenshot 桩（4101）
# ——每发断言结构化错误码（JSON-RPC error.code，无击穿），尾部进程存活 +
# 复证「无一次非法 deploy 落地」。
# P1-2 整改（2026-10-07，review-plan-code-reviewer.md）：补独立空构成用例
# CHK-17/18——实测裁决 = 空数组**合法布阵**（单侧空=立即灭绝冻结@0；双侧空=
# draw@0；缺字段才是 -32602）——与 sim 层空局语义一致（T002 黄金锚①先例）；
# 任务卡原措辞「空构成=非法参数」按审核 B 路径修正（t020 卡范围内节裁决注）。
# 锚值来源 = Lead 2026-10-07 预构建 exe 实测（deploy_hash/final_hash 双留）。
# REQ/RESP 原文走 stderr（>&2）落 error-paths-run.log（体例沿 t018 整改版）。
# 用法：bash error_paths.sh（树根执行；端口 15714；在 replay_matrix.sh 后跑）
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../target/release/host.exe"
LOG="error-paths-run.log"
PORT=15714

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

echo "== T020 error paths $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE =="

# 预检：端口已被占（残留 host）即失败退出
if curl -s -m 2 -H 'Content-Type: application/json' \
     -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
     "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then
  echo "== PREFLIGHT FAIL: port $PORT already serving (残留 host?) =="
  echo "== SCRIPT_EXIT=2 =="
  exit 2
fi

"$HOST_EXE" --port $PORT > host-errors-stdout.log 2> host-errors-stderr.log &
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

# M：民兵 1×1 合法底座（非法字段变体基于它，避免缺参错误先行掩盖目标校验）
M='[{"kind":"militia","count":1}]'

# CHK-01 未 deploy 先 state_hash → 4001 NOT_DEPLOYED（须先于任何成功 deploy）
R=$(brp game.state_hash '{}')
echo "$R" | grep -qF '"code":4001'; check CHK-01-undeployed-state-hash-4001 $?

# CHK-02 seed 传字符串 → -32602
R=$(brp game.deploy "{\"seed\":\"42\",\"red\":$M,\"blue\":$M}")
echo "$R" | grep -qF '"code":-32602'; check CHK-02-seed-string-32602 $?

# CHK-03 未知兵种 laser → -32602 + 消息列出合法兵种表
R=$(brp game.deploy '{"seed":1,"red":[{"kind":"laser","count":5}],"blue":[]}')
echo "$R" | grep -qF '"code":-32602'; check CHK-03-unknown-kind-32602 $?
echo "$R" | grep -qF 'unknown kind'; check CHK-03b-unknown-kind-msg $?

# CHK-04 count 负数 → -32602
R=$(brp game.deploy '{"seed":1,"red":[{"kind":"militia","count":-1}],"blue":[]}')
echo "$R" | grep -qF '"code":-32602'; check CHK-04-count-negative-32602 $?

# CHK-05 单方超 100,000 上限 → -32602 + 上限消息
R=$(brp game.deploy '{"seed":1,"red":[{"kind":"militia","count":100001}],"blue":[]}')
echo "$R" | grep -qF '"code":-32602'; check CHK-05-per-side-cap-32602 $?
echo "$R" | grep -qF 'exceeding per-side cap 100000'; check CHK-05b-per-side-cap-msg $?

# CHK-06/07 threads 出域 0 / 1025 → -32602
R=$(brp game.deploy "{\"seed\":1,\"red\":$M,\"blue\":$M,\"threads\":0}")
echo "$R" | grep -qF '"code":-32602'; check CHK-06-threads0-32602 $?
R=$(brp game.deploy "{\"seed\":1,\"red\":$M,\"blue\":$M,\"threads\":1025}")
echo "$R" | grep -qF '"code":-32602'; check CHK-07-threads1025-32602 $?

# CHK-08 lane_len_m 0 → -32602
R=$(brp game.deploy "{\"seed\":1,\"red\":$M,\"blue\":$M,\"lane_len_m\":0}")
echo "$R" | grep -qF '"code":-32602'; check CHK-08-lane0-32602 $?

# CHK-09/10 max_ticks 出域 0 / 14401 → -32602
R=$(brp game.deploy "{\"seed\":1,\"red\":$M,\"blue\":$M,\"max_ticks\":0}")
echo "$R" | grep -qF '"code":-32602'; check CHK-09-maxticks0-32602 $?
R=$(brp game.deploy "{\"seed\":1,\"red\":$M,\"blue\":$M,\"max_ticks\":14401}")
echo "$R" | grep -qF '"code":-32602'; check CHK-10-maxticks14401-32602 $?

# CHK-11 run_to_tick 缺 ticks 键 → -32602（参数形状错误 ≠ 未布阵状态错误）
R=$(brp game.run_to_tick '{}')
echo "$R" | grep -qF '"code":-32602'; check CHK-11-missing-ticks-32602 $?
echo "$R" | grep -qF 'missing/invalid ticks'; check CHK-11b-missing-ticks-msg $?

# CHK-12 未知 preset → -32602 + 消息列预设清单（presets::names 单一来源）
R=$(brp game.deploy '{"preset":"nope","seed":42}')
echo "$R" | grep -qF '"code":-32602'; check CHK-12-unknown-preset-32602 $?
echo "$R" | grep -qF 'available: default, melee-brawl, last-stand'; check CHK-12b-unknown-preset-msg $?

# CHK-13 未知 suite → -32602（消息列套件清单）
R=$(brp game.run_tests '{"suite":"no-such-suite"}')
echo "$R" | grep -qF '"code":-32602'; check CHK-13-unknown-suite-32602 $?

# CHK-14 screenshot 桩 → 4101 SPECTATE_NOT_ENABLED + data.planned_task=T019
R=$(brp game.screenshot '{}')
echo "$R" | grep -qF '"code":4101'; check CHK-14-screenshot-stub-4101 $?
echo "$R" | grep -qF '"planned_task":"T019"'; check CHK-14b-screenshot-data $?

# CHK-15 全部非法 deploy 均未落地：state_hash 仍 4001（无一次静默成功；
# 其后的 CHK-17/18 为合法空构成 deploy，不影响本复证）
R=$(brp game.state_hash '{}')
echo "$R" | grep -qF '"code":4001'; check CHK-15-still-undeployed-4001 $?

# ── P1-2 整改段（2026-10-07）：空构成独立用例（接受现实口径）──

# CHK-17 单侧空构成合法：red=[] vs militia:5 → deploy 成功（alive_red 0 / units 5）
R=$(brp game.deploy '{"seed":42,"red":[],"blue":[{"kind":"militia","count":5}],"lane_len_m":60,"max_ticks":100}')
echo "$R" | grep -qF '"alive_red":0'; check CHK-17-empty-red-deploy-ok $?
echo "$R" | grep -qF '"deploy_hash":"0xf5f1b5a28609daf0"'; check CHK-17b-empty-red-deploy-hash $?
# → outcome 立即灭绝冻结：blue @ tick 0、final_hash == deploy_hash
R=$(brp game.outcome '{}')
{ echo "$R" | grep -qF '"winner":"blue"' && echo "$R" | grep -qF '"end_tick":0'; }; check CHK-17c-empty-red-immediate-blue $?
echo "$R" | grep -qF '"final_hash":"0xf5f1b5a28609daf0"'; check CHK-17d-empty-red-final-eq-deploy $?

# CHK-18 双侧空构成合法：red=[] blue=[] → units 0、空局哈希锚
R=$(brp game.deploy '{"seed":42,"red":[],"blue":[],"lane_len_m":60,"max_ticks":100}')
echo "$R" | grep -qF '"units":0'; check CHK-18-both-empty-deploy-ok $?
echo "$R" | grep -qF '"deploy_hash":"0xa6c66902e818cd64"'; check CHK-18b-both-empty-hash $?
# → outcome draw @ tick 0（双方存活 0 灭绝判定对称；字段序 end_tick 先于 winner）
R=$(brp game.outcome '{}')
{ echo "$R" | grep -qF '"winner":"draw"' && echo "$R" | grep -qF '"end_tick":0'; }; check CHK-18c-both-empty-draw $?
echo "$R" | grep -qF '"final_hash":"0xa6c66902e818cd64"'; check CHK-18d-both-empty-final-eq-deploy $?

# CHK-16 进程存活收尾（18 发全程无 panic / 无击穿）
kill -0 "$HOST_PID" 2>/dev/null; check CHK-16-process-alive $?
kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null

echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
