#!/usr/bin/env bash
# T018 BRP 直调冒烟（Lead 段，D9）：起 release host → rpc.discover + game.* 6 方法
# 全调用 + 错误路径三发 → 黄金锚对拍（锚②经 BRP 路径）+ 跨线程对拍 + BRP 重放
# 逐位一致 → 请求/响应原文与判定写 brp-smoke-run.log。
# 用法：bash brp_smoke.sh（机器基本空闲；非帧敏量测，无独占窗口要求）
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../target/release/host.exe"
LOG="brp-smoke-run.log"
PORT=15702
DEFAULT_COMP='[{"kind":"shieldman","count":5},{"kind":"heavyknight","count":5},{"kind":"pikeman","count":5},{"kind":"swordsman","count":5},{"kind":"archer","count":5},{"kind":"militia","count":5}]'
ANCHOR2="0x958c5938c8682529"  # M0 T004 归档锚：seed42 默认构成 1800 ticks（勿改写）

PASS=0; FAIL=0
: > "$LOG"
exec > >(tee -a "$LOG") 2>&1

check() { # name condition(0=pass)
  if [ "$2" -eq 0 ]; then PASS=$((PASS+1)); echo "[$1] PASS"; else FAIL=$((FAIL+1)); echo "[$1] FAIL"; fi
}

ID=0
brp() { # method params-json -> echoes response body
  # P0-1 整改：REQ/RESP 原文走 stderr（>&2）——命令替换只捕获 stdout，stderr
  # 经 exec 2>&1 直接落 log（首轮缺陷：echo 落 stdout 被捕获进变量丢弃，原文未入档）。
  ID=$((ID+1))
  local body="{\"jsonrpc\":\"2.0\",\"id\":$ID,\"method\":\"$1\",\"params\":$2}"
  echo "--- REQ #$ID: $body" >&2
  local resp
  resp=$(curl -s -m 120 -H 'Content-Type: application/json' -d "$body" "http://127.0.0.1:$PORT/")
  echo "--- RESP #$ID: $resp" >&2
  echo "$resp"
}

echo "== T018 BRP smoke $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE =="

# 起宿主（stderr 落 host-stderr.log——banner 运行时留痕）
"../../../target/release/host.exe" > host-stdout.log 2> host-stderr.log &
HOST_PID=$!
echo "== host pid $HOST_PID =="

# 端口就绪等待（rpc.discover 通即续）
READY=0
for i in $(seq 1 150); do
  if curl -s -m 2 -H 'Content-Type: application/json' \
       -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
       "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then READY=1; break; fi
  sleep 0.2
done
check CHK-00-port-ready $([ "$READY" -eq 1 ]; echo $?)

# CHK-01 banner 双留痕（代码侧见 rpc.rs 显式 LOCALHOST；此处运行时侧）
grep -qF "127.0.0.1:15702" host-stderr.log; check CHK-01-banner-loopback $?
grep -qF "non-loopback forbidden" host-stderr.log; check CHK-01b-banner-constraint $?

# CHK-02 discover：6 方法全部注册
DISC=$(brp rpc.discover '{}')
for m in game.deploy game.run_to_tick game.state_hash game.outcome game.run_tests game.screenshot; do
  echo "$DISC" | grep -qF "\"$m\""; check "CHK-02-discover-$m" $?
done

# CHK-03 错误路径：未 deploy 先 state_hash → 4001 NOT_DEPLOYED
R=$(brp game.state_hash '{}')
echo "$R" | grep -qF '"code":4001'; check CHK-03-not-deployed $?

# CHK-04 deploy 默认构成（threads=1）：形状断言
R=$(brp game.deploy "{\"seed\":42,\"red\":$DEFAULT_COMP,\"blue\":$DEFAULT_COMP,\"threads\":1}")
DEPLOY_HASH=$(echo "$R" | grep -o '"deploy_hash":"0x[0-9a-f]*"' | head -1)
echo "deploy_hash=$DEPLOY_HASH"
[ -n "$DEPLOY_HASH" ] && [ "$DEPLOY_HASH" != '"deploy_hash":"0x0000000000000000"' ]; check CHK-04-deploy-hash $?
echo "$R" | grep -qF '"tick":0'; check CHK-04b-deploy-tick0 $?
echo "$R" | grep -qF '"units":60'; check CHK-04c-deploy-units60 $?
echo "$R" | grep -qF '"alive_red":30'; check CHK-04d-deploy-alive-red $?
echo "$R" | grep -qF '"alive_blue":30'; check CHK-04d-deploy-alive-blue $?

# CHK-05 黄金锚②（BRP 路径）：run_to_tick 1800 → 0x958c5938c8682529
R=$(brp game.run_to_tick '{"ticks":1800}')
echo "$R" | grep -qF '"tick":1800'; check CHK-05-run-tick-1800 $?
HASH1=$(echo "$R" | grep -oF "\"state_hash\":\"$ANCHOR2\"" | head -1)
[ -n "$HASH1" ]; check CHK-05-anchor2-hit $?

# CHK-06 state_hash 一致
R=$(brp game.state_hash '{}')
echo "$R" | grep -qF "\"state_hash\":\"$ANCHOR2\""; check CHK-06-state-hash-consistent $?

# CHK-07 outcome（tick 已 1800 = max_ticks 默认；对称未接敌 → Draw@1800，final_hash 同锚）
R=$(brp game.outcome '{}')
echo "$R" | grep -qF '"winner":"draw"'; check CHK-07-outcome-draw $?
echo "$R" | grep -qF '"end_tick":1800'; check CHK-07b-outcome-endtick $?
echo "$R" | grep -qF "\"final_hash\":\"$ANCHOR2\""; check CHK-07c-outcome-finalhash $?

# CHK-08 错误路径：冻结后再 run_to_tick → 4002
R=$(brp game.run_to_tick '{"ticks":10}')
echo "$R" | grep -qF '"code":4002'; check CHK-08-battle-resolved $?

# CHK-09 BRP 重放：同 seed 同参数序列再来一遍 → 逐位一致
R=$(brp game.deploy "{\"seed\":42,\"red\":$DEFAULT_COMP,\"blue\":$DEFAULT_COMP,\"threads\":1}" > /dev/null; brp game.run_to_tick '{"ticks":1800}')
echo "$R" | grep -qF "\"state_hash\":\"$ANCHOR2\""; check CHK-09-replay-bitexact $?

# CHK-10 跨线程档：threads=12 同锚逐位一致（T006 跨线程纪律 BRP 路径复证）
R=$(brp game.deploy "{\"seed\":42,\"red\":$DEFAULT_COMP,\"blue\":$DEFAULT_COMP,\"threads\":12}" > /dev/null; brp game.run_to_tick '{"ticks":1800}')
echo "$R" | grep -qF "\"state_hash\":\"$ANCHOR2\""; check CHK-10-threads12-anchor $?

# CHK-11 run_tests 冒烟集（4 断言全过——含锚①经进程内 World::new 路径）
R=$(brp game.run_tests '{}')
echo "$R" | grep -qF '"total":4'; check CHK-11-tests-total4 $?
echo "$R" | grep -qF '"passed":4'; check CHK-11b-tests-passed4 $?
echo "$R" | grep -qF '"failed":0'; check CHK-11c-tests-failed0 $?
R=$(brp game.run_tests '{"suite":"t018-smoke"}')
echo "$R" | grep -qF '"suite":"t018-smoke"'; check CHK-11d-tests-suite-name $?
R=$(brp game.run_tests '{"suite":"no-such-suite"}')
echo "$R" | grep -qF '"code":-32602'; check CHK-11e-tests-unknown-suite $?

# CHK-12 错误路径：screenshot 桩 → 4101 + T019 data
R=$(brp game.screenshot '{}')
echo "$R" | grep -qF '"code":4101'; check CHK-12-screenshot-stub $?
echo "$R" | grep -qF '"planned_task":"T019"'; check CHK-12b-screenshot-data $?

# CHK-13 错误路径：未知兵种 → -32602；suite 非字符串 → -32602（P2-1 整改验证）
R=$(brp game.deploy "{\"seed\":1,\"red\":[{\"kind\":\"laser\",\"count\":5}],\"blue\":[]}")
echo "$R" | grep -qF '"code":-32602'; check CHK-13-unknown-kind $?
R=$(brp game.run_tests '{"suite":123}')
echo "$R" | grep -qF '"code":-32602'; check CHK-13b-suite-non-string $?

# CHK-15 灭绝冻结路径 A（空侧 deploy，tick 0 即收束）：red 空 → run_to_tick 停在 0、
# outcome blue@tick0、再推 4002（P2-3 整改补面——宿主最复杂分支的首轮零覆盖）
R=$(brp game.deploy '{"seed":42,"red":[],"blue":[{"kind":"militia","count":5}],"threads":1}')
echo "$R" | grep -qF '"alive_red":0'; check CHK-15-empty-red-deploy $?
R=$(brp game.run_to_tick '{"ticks":100}')
echo "$R" | grep -qF '"tick":0'; check CHK-15b-run-frozen-at-0 $?
R=$(brp game.outcome '{}')
echo "$R" | grep -qF '"winner":"blue"'; check CHK-15c-outcome-blue $?
echo "$R" | grep -qF '"end_tick":0'; check CHK-15d-endtick-0 $?
echo "$R" | grep -qF '"alive_red":0'; check CHK-15e-outcome-alive-red-0 $?
R=$(brp game.run_to_tick '{"ticks":10}')
echo "$R" | grep -qF '"code":4002'; check CHK-15f-run-after-freeze-4002 $?

# CHK-16 灭绝冻结路径 B（短 lane 真交战至灭绝）：5v5 民兵 lane 10m → 灭绝 tick < 1800、
# winner 非 draw、一方存活 0、终局幂等
R=$(brp game.deploy '{"seed":42,"red":[{"kind":"militia","count":5}],"blue":[{"kind":"militia","count":5}],"lane_len_m":10,"threads":1}')
echo "$R" | grep -qF '"units":10'; check CHK-16-shortlane-deploy $?
R=$(brp game.run_to_tick '{"ticks":1800}')
END_TICK=$(echo "$R" | grep -o '"tick":[0-9]*' | head -1 | grep -o '[0-9]*')
echo "shortlane end tick = $END_TICK"
[ -n "$END_TICK" ] && [ "$END_TICK" -lt 1800 ]; check CHK-16b-extinction-before-cap $?
R2=$(brp game.outcome '{}')
echo "$R2" | grep -qF '"winner":"red"' || echo "$R2" | grep -qF '"winner":"blue"'; check CHK-16c-winner-nondraw $?
echo "$R2" | grep -qF "\"end_tick\":$END_TICK"; check CHK-16d-endtick-matches $?
echo "$R2" | grep -qF '"alive_red":0' || echo "$R2" | grep -qF '"alive_blue":0'; check CHK-16e-one-side-extinct $?
R=$(brp game.run_to_tick '{"ticks":10}')
echo "$R" | grep -qF '"code":4002'; check CHK-16f-run-after-freeze-4002 $?

# CHK-14 进程存活收尾（全程无 panic / 无击穿）
kill -0 "$HOST_PID" 2>/dev/null; check CHK-14-process-alive $?
kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null

echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
