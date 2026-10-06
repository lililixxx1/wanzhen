#!/usr/bin/env bash
# T021 配置面冒烟（worker 实现段，D6）：BRP preset 底座 + 显式字段覆盖语义 +
# 同参重放逐位一致 + CLI auto-deploy 入口无关性黄金锚对拍 + CLI/BRP 非法路径
# 全覆盖 → 请求/响应原文与判定写 preset-smoke-run.log。
# 用法：bash preset_smoke.sh（树根执行；端口 15712/15713；机器基本空闲，非帧敏量测）
#
# 判定注（D6 第 3 项偏离留痕，Lead 上报选项 A）：派工单原文预期 last-stand
# 「run_to_tick 1800 → 冻结 tick<1800（灭绝）、一方存活 0」，经实测不可达——
# 贴身射程 + move 向前夹紧不越位（sim/src/world.rs:342/733）⇒ 接敌面恒 1v1
# 漏斗，民兵 9 dmg/20t（无甲克重甲）vs 盾兵 5 dmg/30t（被克），1800 tick 内
# 双方远未灭绝。基线 host BRP 实测（2026-10-06，未改任何代码）：run 收于上限
# tick 1800、alive 15/56、winner=blue（resolve_by_hp 上限总 hp 判定）。本脚本
# 按实测确定性现实断言（精确值 15/56 为确定性锚）；派工单原文预期保留于
# README.md 上报节供 review 裁决。
set -u
cd "$(dirname "$0")"
HOST_EXE="../../../target/release/host.exe"
LOG="preset-smoke-run.log"
PORT=15712
PORT_CLI=15713
ANCHOR1="0xe2706f0b91a2be8e"  # M0/T018 归档锚：seed42 默认构成 lane1000 tick0 布阵快照（勿改写）

PASS=0; FAIL=0
: > "$LOG"
exec > >(tee -a "$LOG") 2>&1

check() { # name condition(0=pass)
  if [ "$2" -eq 0 ]; then PASS=$((PASS+1)); echo "[$1] PASS"; else FAIL=$((FAIL+1)); echo "[$1] FAIL"; fi
}

ID=0
brp() { # method params-json [port] -> echoes response body（REQ/RESP 原文走 stderr 落档）
  ID=$((ID+1))
  local port="${3:-$PORT}"
  local body="{\"jsonrpc\":\"2.0\",\"id\":$ID,\"method\":\"$1\",\"params\":$2}"
  echo "--- REQ #$ID (port $port): $body" >&2
  local resp
  resp=$(curl -s -m 120 -H 'Content-Type: application/json' -d "$body" "http://127.0.0.1:$port/")
  echo "--- RESP #$ID: $resp" >&2
  echo "$resp"
}

echo "== T021 preset smoke $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE =="
echo "== BRP port $PORT / CLI auto-deploy instance port $PORT_CLI =="

# ── A 段：BRP preset 面（host --port 15712 纯服务形态起，D6-1） ──
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
check CHK-00-port-ready-15712 $([ "$READY" -eq 1 ]; echo $?)

# CHK-01 banner 双留痕（D5）：监听行打印实际端口 + 回环约束声明
grep -qF "127.0.0.1:15712" host-stderr.log; check CHK-01-banner-port-actual $?
grep -qF "non-loopback forbidden" host-stderr.log; check CHK-01b-banner-loopback $?

# CHK-02（D6-1）preset default + seed42 → 布阵快照哈希 == T018 显式构成路径锚
# （预设路径 == 显式构成路径，入口无关锚；黄金锚字面量逐字使用）
R=$(brp game.deploy '{"preset":"default","seed":42}')
echo "$R" | grep -qF "\"deploy_hash\":\"$ANCHOR1\""; check CHK-02-preset-default-anchor $?
echo "$R" | grep -qF '"units":60'; check CHK-02b-preset-default-units60 $?

# CHK-03（D6-2）melee-brawl seed7：units 60 → run 1800 记哈希 H1 → 同参重放逐位一致
R=$(brp game.deploy '{"preset":"melee-brawl","seed":7}')
echo "$R" | grep -qF '"units":60'; check CHK-03-melee-units60 $?
R=$(brp game.run_to_tick '{"ticks":1800}')
H1=$(echo "$R" | grep -o '"state_hash":"0x[0-9a-f]*"' | head -1)
echo "melee-brawl H1=$H1"
[ -n "$H1" ]; check CHK-03b-melee-run-hash-captured $?
R=$(brp game.deploy '{"preset":"melee-brawl","seed":7}' > /dev/null; brp game.run_to_tick '{"ticks":1800}')
echo "$R" | grep -qF "$H1"; check CHK-03c-melee-replay-bitexact $?

# CHK-04（D6-3，按实测现实断言——偏离留痕见脚本头注）last-stand seed7：
# units 80 → run 1800 收于上限 tick1800；outcome winner=blue（resolve_by_hp）、
# alive 15/56 确定性精确值、双方存活 > 0。
R=$(brp game.deploy '{"preset":"last-stand","seed":7}')
echo "$R" | grep -qF '"units":80'; check CHK-04-laststand-units80 $?
R=$(brp game.run_to_tick '{"ticks":1800}')
echo "$R" | grep -qF '"tick":1800'; check CHK-04b-laststand-run-to-cap $?
R=$(brp game.outcome '{}')
echo "$R" | grep -qF '"winner":"blue"'; check CHK-04c-laststand-winner-blue $?
echo "$R" | grep -qF '"alive_red":15'; check CHK-04d-laststand-alive-red15 $?
echo "$R" | grep -qF '"alive_blue":56'; check CHK-04e-laststand-alive-blue56 $?
echo "$R" | grep -qF '"end_tick":1800'; check CHK-04f-laststand-endtick-cap $?

# CHK-05（D6-4）覆盖语义：preset default + 显式 red/blue/lane_len_m 与纯显式
# 同参调用 deploy_hash 对拍一致（覆盖语义 == 不带 preset 的显式路径）
M5='[{"kind":"militia","count":5}]'
R=$(brp game.deploy "{\"preset\":\"default\",\"seed\":42,\"red\":$M5,\"blue\":$M5,\"lane_len_m\":10}")
HA=$(echo "$R" | grep -o '"deploy_hash":"0x[0-9a-f]*"' | head -1)
R=$(brp game.deploy "{\"seed\":42,\"red\":$M5,\"blue\":$M5,\"lane_len_m\":10}")
HB=$(echo "$R" | grep -o '"deploy_hash":"0x[0-9a-f]*"' | head -1)
echo "override HA=$HA / explicit HB=$HB"
[ -n "$HA" ] && [ "$HA" = "$HB" ]; check CHK-05-override-equals-explicit $?

# CHK-06（D6-7）BRP 非法：未知 preset → -32602；preset + 未知兵种显式覆盖 → -32602
# （第二发为派工单字面参数——无 seed 时 seed 缺参先报，同码 -32602；补第三发带
# seed 单独钉住 laser 解析错误路径）
R=$(brp game.deploy '{"preset":"nope"}')
echo "$R" | grep -qF '"code":-32602'; check CHK-06-preset-unknown-32602 $?
R=$(brp game.deploy '{"preset":"default","red":[{"kind":"laser","count":1}]}')
echo "$R" | grep -qF '"code":-32602'; check CHK-06b-preset-laser-noseed-32602 $?
R=$(brp game.deploy '{"preset":"default","seed":1,"red":[{"kind":"laser","count":1}]}')
echo "$R" | grep -qF '"code":-32602'; check CHK-06c-preset-laser-withseed-32602 $?

# CHK-07 A 段宿主存活收尾（全程无 panic / 无击穿）
kill -0 "$HOST_PID" 2>/dev/null; check CHK-07-host1-process-alive $?
kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null

# ── B 段：CLI auto-deploy 入口无关性（D6-5：--seed 42 --preset default --port 15713） ──
"$HOST_EXE" --seed 42 --preset default --port $PORT_CLI > host-cli-stdout.log 2> host-cli-stderr.log &
HOST2_PID=$!
echo "== host2 pid $HOST2_PID (port $PORT_CLI, auto-deploy) =="

READY=0
for i in $(seq 1 150); do
  if curl -s -m 2 -H 'Content-Type: application/json' \
       -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
       "http://127.0.0.1:$PORT_CLI/" > /dev/null 2>&1; then READY=1; break; fi
  sleep 0.2
done
check CHK-08-cli-port-ready-15713 $([ "$READY" -eq 1 ]; echo $?)

# CHK-09（D6-5）CLI 实例 BRP state_hash == 黄金锚（CLI 路径 == BRP 路径 == T018
# 显式构成路径，入口无关性三路对拍收口）
R=$(brp game.state_hash '{}' $PORT_CLI)
echo "$R" | grep -qF "\"state_hash\":\"$ANCHOR1\""; check CHK-09-cli-statehash-anchor $?

# CHK-10 banner：auto-deploy 行（D4 格式逐字）+ 实际端口 15713（D5）
grep -qF "[host] auto-deployed default seed=42 units=60 tick=0" host-cli-stderr.log; check CHK-10-cli-banner-autodeploy $?
grep -qF "127.0.0.1:15713" host-cli-stderr.log; check CHK-10b-cli-banner-port $?

# CHK-11（D6-6）CLI 非法三发：各自 exit 2 + stderr 留档（解析先行失败，
# 无 banner / 无监听启动）
"$HOST_EXE" --comp laser:5 > /dev/null 2> cli-illegal-comp.stderr.log
[ $? -eq 2 ]; check CHK-11-cli-comp-laser-exit2 $?
grep -qF "unknown unit kind 'laser'" cli-illegal-comp.stderr.log; check CHK-11b-cli-comp-laser-msg $?

"$HOST_EXE" --preset nope > /dev/null 2> cli-illegal-preset.stderr.log
[ $? -eq 2 ]; check CHK-11c-cli-preset-nope-exit2 $?
grep -qF "unknown preset 'nope'" cli-illegal-preset.stderr.log; check CHK-11d-cli-preset-msg $?

"$HOST_EXE" --threads 0 > /dev/null 2> cli-illegal-threads.stderr.log
[ $? -eq 2 ]; check CHK-11e-cli-threads0-exit2 $?
# -e 显式给模式：报错文案以 "--" 开头，裸传会被 grep 当选项吞掉（首轮 29/30 教训）
grep -qFe "--threads must be in 1..=1024, got 0" cli-illegal-threads.stderr.log; check CHK-11f-cli-threads-msg $?

# CHK-12 B 段宿主存活收尾（D6-8 进程存活收尾，两实例全程独立端口互不干扰）
kill -0 "$HOST2_PID" 2>/dev/null; check CHK-12-host2-process-alive $?
kill "$HOST2_PID" 2>/dev/null; sleep 1; kill -9 "$HOST2_PID" 2>/dev/null

echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
