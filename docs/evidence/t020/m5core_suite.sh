#!/usr/bin/env bash
# T020 门禁 6 / D1 判定面：BRP 直调 m5-core 全量套件（9 断言）——suite 名 /
# total=9 / passed=9 / failed=0 + 逐断言名 pass:true + 关键黄金锚 detail 留痕；
# REQ/RESP 原文走 stderr（>&2）落 m5core-suite-run.log。
# 用法：bash m5core_suite.sh（树根执行；端口 15714；在 error_paths.sh 后跑）
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../target/release/host.exe"
LOG="m5core-suite-run.log"
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
  resp=$(curl -s -m 300 -H 'Content-Type: application/json' -d "$body" "http://127.0.0.1:$PORT/")
  echo "--- RESP #$ID: $resp" >&2
  echo "$resp"
}

echo "== T020 m5-core suite check $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE =="

# 预检：端口已被占（残留 host）即失败退出
if curl -s -m 2 -H 'Content-Type: application/json' \
     -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
     "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then
  echo "== PREFLIGHT FAIL: port $PORT already serving (残留 host?) =="
  echo "== SCRIPT_EXIT=2 =="
  exit 2
fi

"$HOST_EXE" --port $PORT > host-suite-stdout.log 2> host-suite-stderr.log &
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

# 全量套件判定（断言失败 ≠ 协议错误：pass=false 也走 200 正常返回）
R=$(brp game.run_tests '{"suite":"m5-core"}')
echo "$R" | grep -qF '"suite":"m5-core"'; check CHK-01-suite-name $?
echo "$R" | grep -qF '"total":9'; check CHK-02-total9 $?
echo "$R" | grep -qF '"passed":9'; check CHK-03-passed9 $?
echo "$R" | grep -qF '"failed":0'; check CHK-04-failed0 $?

# 逐断言名 pass:true（serde_json 键序字母序：detail < name < pass）
for n in golden_units0_seed42_1800 golden_default_comp_seed42_1800 \
         replay_pairwise_checkpoints deploy_versus_mirror_equivalence \
         cross_thread_pool_equivalence_900t battle_golden_shortlane_seed42 \
         seed_sensitivity_42_43 outcome_freeze_idempotence_shortlane \
         outcome_overrun_semantics; do
  echo "$R" | grep -qF "\"name\":\"$n\",\"pass\":true"; check "CHK-05-pass-$n" $?
done

# 关键黄金锚 detail 留痕（锚①/锚②/短 lane 锚/seed43 伴随锚）
echo "$R" | grep -qF '0xd3b6408fd46c2008'; check CHK-06-anchor1-detail $?
echo "$R" | grep -qF '0x958c5938c8682529'; check CHK-07-anchor2-detail $?
echo "$R" | grep -qF '0xfdbc4995554ee691'; check CHK-08-shortlane-detail $?
echo "$R" | grep -qF '0x54611ed6ded02540'; check CHK-09-seed43-detail $?

# 进程存活收尾
kill -0 "$HOST_PID" 2>/dev/null; check CHK-10-process-alive $?
kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null

echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
