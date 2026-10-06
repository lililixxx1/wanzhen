#!/usr/bin/env bash
# T025 三合一活判定（M5 可玩性验收——备忘录 §三「配置-观战-实验三合一」核心循环）：
# 核心循环闭环「配置→观战→战报→再来」+ ≥3 挑战经判定面 + 统计面一等玩法 + 质量红线
# （全程无 panic + 同种子+同参数序列重放逐位一致）。判定行脚本生成，REQ/RESP 原文落档。
# v2 修正（Lead 首跑 3 FAIL 复盘）：①截图条目 id 从 result 段取（信封 id 恒 1 勿取）；
# ②CAP 改退出码语义（0=过）；③手动 300t 锚改在 BRP 重部署（autorun=false）后取——
# auto-deploy 实例 autorun 先自走，tick 起点 wall-clock 依赖不可锚。
# 用法：bash trinity-demo.sh（仓库根 docs/evidence/m5/ 执行；端口 15718/15719；机器基本空闲）
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../target/release/host.exe"
LOG="trinity-demo-run.log"
P_HEAD=15718
P_SPEC=15719

PASS=0; FAIL=0
: > "$LOG"
exec > >(tee -a "$LOG") 2>&1

check() { if [ "$2" -eq 0 ]; then PASS=$((PASS+1)); echo "[$1] PASS"; else FAIL=$((FAIL+1)); echo "[$1] FAIL"; fi; }

brp() {
  local body="{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":$2}"
  echo "--- REQ (port $3): $body" >&2
  local resp
  resp=$(curl -s -m 120 -H 'Content-Type: application/json' -d "$body" "http://127.0.0.1:$3/")
  echo "--- RESP: $resp" >&2
  echo "$resp"
}

echo "== T025 trinity demo v2 $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== leg1 headless 15718 (配置+挑战+统计+重放红线) / leg2 spectate 15719 (观战+战报+截屏+再来) =="

for p in $P_HEAD $P_SPEC; do
  if curl -s -m 2 -o /dev/null -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' "http://127.0.0.1:$p/"; then
    echo "FATAL: port $p busy"; exit 2
  fi
done

"$HOST_EXE" --port $P_HEAD > host-head-stdout.log 2> host-head-stderr.log &
H1=$!
"$HOST_EXE" --port $P_SPEC --spectate --preset melee-brawl --seed 7 --max-ticks 3600 > host-spec-stdout.log 2> host-spec-stderr.log &
H2=$!
READY1=0; READY2=0
for i in $(seq 1 200); do
  [ "$READY1" -eq 0 ] && curl -s -m 2 -o /dev/null -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' "http://127.0.0.1:$P_HEAD/" && READY1=1
  [ "$READY2" -eq 0 ] && curl -s -m 2 -o /dev/null -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' "http://127.0.0.1:$P_SPEC/" && READY2=1
  [ "$READY1" -eq 1 ] && [ "$READY2" -eq 1 ] && break
  sleep 0.2
done
check CHK-00-ports-ready $([ "$READY1" -eq 1 ] && [ "$READY2" -eq 1 ]; echo $?)

# ── 配置面：preset 底座部署（三路入口同哈希 T021 已证，此处活判定一次）──
R=$(brp game.deploy '{"preset":"default","seed":42}' "$P_HEAD")
echo "$R" | grep -qF '"deploy_hash":"0xe2706f0b91a2be8e"'; check CHK-01-config-deploy-anchor $?
R=$(brp game.run_to_tick '{"ticks":900}' "$P_HEAD")
echo "$R" | grep -qF '"state_hash":"0x7fbb5bd7e46ef448"'; check CHK-02-config-run-900t-crosscheck $?

# ── 确定性红线（同种子+同参数序列重放逐位一致）：同参序列重放两遍 == 锚② ──
R=$(brp game.deploy '{"preset":"default","seed":42}' "$P_HEAD")
R=$(brp game.run_to_tick '{"ticks":1800}' "$P_HEAD")
H_A=$(echo "$R" | grep -o '"state_hash":"0x[0-9a-f]*"' | cut -d'"' -f4)
R=$(brp game.deploy '{"preset":"default","seed":42}' "$P_HEAD")
R=$(brp game.run_to_tick '{"ticks":1800}' "$P_HEAD")
H_B=$(echo "$R" | grep -o '"state_hash":"0x[0-9a-f]*"' | cut -d'"' -f4)
[ -n "$H_A" ] && [ "$H_A" = "$H_B" ] && [ "$H_A" = "0x958c5938c8682529" ]; check CHK-03-replay-bitexact-anchor2 $?

# ── 观战面 A（autorun 自走推进活性）：两次 state_hash 读数 tick 递增 ──
R1=$(brp game.state_hash '{}' "$P_SPEC"); sleep 1.2
R2=$(brp game.state_hash '{}' "$P_SPEC")
T1=$(echo "$R1" | grep -o '"tick":[0-9]*' | grep -o '[0-9]*')
T2=$(echo "$R2" | grep -o '"tick":[0-9]*' | grep -o '[0-9]*')
[ -n "$T1" ] && [ -n "$T2" ] && [ "$T2" -gt "$T1" ]; check CHK-04-spectate-autorun-advancing $?

# ── 观战面 B（节流下确定性红线——跨模式动态对拍，沿 T019 CHK-15~16 范式）：
# spectate BRP 重部署（autorun=false）→ 入队 300t（queued 立即返回）→ 轮询至 tick≥300
# → 与 headless 同参直推 300t 的 state_hash 逐位一致（0xcdf3fef834f172ce 系 T019
# seed43/lane50 场景锚，本场景 melee-brawl seed7 故用动态对拍不引错场景锚）。
R=$(brp game.deploy '{"preset":"melee-brawl","seed":7}' "$P_SPEC")
echo "$R" | grep -qF '"deploy_hash"'; check CHK-04a-spectate-redeploy-ok $?
R=$(brp game.run_to_tick '{"ticks":300}' "$P_SPEC")
echo "$R" | grep -qF '"queued":300'; check CHK-04b-queued-300-immediate $?
SP_HASH=""; elapsed=0
while [ "$elapsed" -lt 60 ]; do
  R=$(brp game.state_hash '{}' "$P_SPEC")
  T=$(echo "$R" | grep -o '"tick":[0-9]*' | tail -1 | grep -o '[0-9]*')
  if [ -n "$T" ] && [ "$T" -ge 300 ]; then
    SP_HASH=$(echo "$R" | grep -o '"state_hash":"0x[0-9a-f]*"' | head -1 | cut -d'"' -f4); break
  fi
  sleep 1; elapsed=$((elapsed+1))
done
echo "spectate 300t: hash=$SP_HASH (${elapsed}s)"
R=$(brp game.deploy '{"preset":"melee-brawl","seed":7}' "$P_HEAD")
R=$(brp game.run_to_tick '{"ticks":300}' "$P_HEAD")
HL_HASH=$(echo "$R" | grep -o '"state_hash":"0x[0-9a-f]*"' | head -1 | cut -d'"' -f4)
echo "headless 300t: hash=$HL_HASH"
[ -n "$SP_HASH" ] && [ -n "$HL_HASH" ] && [ "$SP_HASH" = "$HL_HASH" ]; check CHK-04c-crossmode-300t-bitexact $?

# ── 截屏（观战证据，live 两段式——条目 id 取自 result 段）──
R=$(brp game.screenshot '{}' "$P_SPEC")
SHOT_ID=$(echo "$R" | sed -n 's/.*"result":{"id":\([0-9]*\),"status":"requested".*/\1/p')
echo "screenshot entry id=$SHOT_ID"
CAP_RC=1
for i in $(seq 1 60); do
  R=$(brp game.screenshot "{\"id\":$SHOT_ID}" "$P_SPEC")
  if echo "$R" | grep -qF '"status":"captured"'; then CAP_RC=0; break; fi
  sleep 0.5
done
check CHK-06-spectate-screenshot-captured $CAP_RC
SHOT_PATH=$(echo "$R" | grep -o '"path":"[^"]*"' | head -1 | sed 's/"path":"//;s/"//')
if [ -n "$SHOT_PATH" ] && [ -f "$SHOT_PATH" ]; then
  head -c 8 "$SHOT_PATH" | od -An -tx1 | grep -q "89 50 4e 47"; check CHK-07-screenshot-png-magic $?
  SZ=$(stat -c%s "$SHOT_PATH" 2>/dev/null || wc -c < "$SHOT_PATH")
  [ "$SZ" -gt 30000 ]; check CHK-08-screenshot-nontrivial-size $?
  sha256sum "$SHOT_PATH" | tee screenshot.sha256
  cp "$SHOT_PATH" "shot-trinity-spectate.png" && rm -f "$SHOT_PATH"
else
  check CHK-07-screenshot-png-magic 1; check CHK-08-screenshot-nontrivial-size 1
fi

# ── 战报（幂等四元组）──
R=$(brp game.outcome '{}' "$P_SPEC")
echo "$R" | grep -qF '"winner"'; check CHK-05-spectate-outcome-report $?

# ── 再来（核心循环第 4 步）：观战实例重新部署新对局并再战报（last-stand 15/56 实锚）──
R=$(brp game.deploy '{"preset":"last-stand","seed":42}' "$P_SPEC")
echo "$R" | grep -qF '"deploy_hash":"0x9a926533cbf1d445"'; check CHK-09-again-redeploy-laststand $?
R=$(brp game.run_to_tick '{"ticks":1800}' "$P_SPEC")
R=$(brp game.outcome '{}' "$P_SPEC")
{ echo "$R" | grep -qF '"winner":"blue"' && echo "$R" | grep -qF '"alive_red":15' && echo "$R" | grep -qF '"alive_blue":56'; }; check CHK-10-again-outcome-15-56 $?

# ── 挑战面（判定主体 = game.run_tests；m5-all 12 = M5 完整判定面）──
R=$(brp game.run_tests '{"suite":"m5-all"}' "$P_HEAD")
{ echo "$R" | grep -qF '"total":12' && echo "$R" | grep -qF '"passed":12' && echo "$R" | grep -qF '"failed":0'; }; check CHK-11-challenges-m5all-12-12 $?
R=$(brp game.run_tests '{"suite":"challenges"}' "$P_HEAD")
{ echo "$R" | grep -qF '"total":3' && echo "$R" | grep -qF '"passed":3'; }; check CHK-12-challenges-3-3 $?

# ── 统计面（一等玩法：批量采样 + 胜率；同参批内重放 outcomes 逐位一致）──
R=$(brp game.sample_outcomes '{"red":[{"kind":"swordsman","count":100}],"blue":[{"kind":"militia","count":100}],"seed_base":42,"games":100}' "$P_HEAD")
{ echo "$R" | grep -qF '"games":100' && echo "$R" | grep -qF '"win_rate_red_pp"'; }; check CHK-13-stats-sample-100 $?
R2=$(brp game.sample_outcomes '{"red":[{"kind":"swordsman","count":100}],"blue":[{"kind":"militia","count":100}],"seed_base":42,"games":100}' "$P_HEAD")
ARR_A=$(echo "$R"  | grep -o '"outcomes":\[.*\]')
ARR_B=$(echo "$R2" | grep -o '"outcomes":\[.*\]')
[ -n "$ARR_A" ] && [ "$ARR_A" = "$ARR_B" ]; check CHK-14-stats-replay-bitexact $?
RW=$(echo "$R" | grep -o '"red_wins":[0-9]*' | grep -o '[0-9]*')
echo "stats: swordsman100 vs militia100 x100 games -> red_wins=$RW (win_rate_red_pp=$((RW*10000/100)))"

# ── 质量红线：两实例全程存活无 panic ──
kill -0 $H1 2>/dev/null; check CHK-15-headless-alive $?
kill -0 $H2 2>/dev/null; check CHK-16-spectate-alive $?
grep -qi "panic" host-head-stderr.log host-spec-stderr.log; [ $? -eq 1 ]; check CHK-17-no-panic-stderr $?

kill $H1 $H2 2>/dev/null; sleep 1; kill -9 $H1 $H2 2>/dev/null
echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
