#!/usr/bin/env bash
# T019 观战冒烟（worker 实现段，D9）：spectate autorun 至终局（跨模式黄金锚
# 对拍）+ 第二种子 spectate/headless 对拍 + 手动驱动入队路径（queued 立即返回
# + 跨模式哈希对拍）+ screenshot 两段式（PNG 魔数/字节/校验和）+ 全程无 panic。
# 用法：bash spectate_smoke.sh（端口 15715/15716/15717；窗口化运行——
# 非帧敏量测但机器窗口开启期间请避让其他量测任务）
#
# 判定注（D4 终局语义，Lead 批复 2026-10-07 留痕）：melee-brawl seed7 无灭绝、
# run_to_tick 1800 收于上限 tick=1800（T021 归档 preset-smoke-run.log 实锚），
# autorun 驱动在 tick≥max_ticks 且未灭绝时走 run_battle_with(max_ticks)
# resolve_by_hp 收束（不再推 tick）——final_hash = 上限 tick 态哈希，与 T021
# headless 归档锚同点。脚本预期值仅供核对，以实际运行为准，不一致即停。
#
# P0/P1 整改段（2026-10-07 修后复审轮，Lead 裁决 = 审核建议 A 近景证据补充）：
# - 段 4 近景可辨（P0）：BRP deploy 六兵种 × 双阵营各 1（12 单位）、
#   lane_len_m 60——全场镜头（present.rs D6 ScalingMode::AutoMin 覆盖算式，
#   min_width ≥ lane + 2×MARGIN）下 ~0.8m 兵种网格 ≈ 1920×(0.8/62) ≈ 25px/
#   单位，形状轮廓可辨；产出 shot-3-sixkinds-closeup-lane60.png。形状目检 =
#   Lead 侧 vision-reader 识图回填 README，本脚本只证场景构型与档案真实。
#   断言：存在 + 魔数 + 尺寸 ≥1920×1080 逻辑（16:9 物理像素——125% DPI 下
#   捕获 2400×1350，README §4 尺寸注口径）+ ≥30KB 非空白画布下界 + sha256。
# - 段 4 亦含 P1 断言①：同进程两次截图 nonce 段相同、id 递增、路径不同。
# - 段 5 截图唯一性（P1 断言②）：预置残留有效 PNG 于下次请求路径 + PowerShell
#   独占句柄（FileShare::None）物理造写盘失败 → 轮询恒 pending + detail
#   （不误报 captured）+ host stderr「Cannot save screenshot」见证
#   （bevy_render-0.19.1 screenshot.rs:147——写盘失败只记日志）；杀掉句柄后
#   复询验 mtime 分支（stale leftover suspected——整改续做修复，见段 5 内注）。
set -u
cd "$(dirname "$0")"
LOG="spectate-smoke-run.log"

# host.exe 解析：① CARGO_TARGET_DIR（共享 target，worker 纪律）；② 仓库根相对
# （主仓复跑形态，树内无 .git 同构布局）；都无 → 报错退出。
HOST_EXE=""
if [ -n "${CARGO_TARGET_DIR:-}" ] && [ -f "$CARGO_TARGET_DIR/release/host.exe" ]; then
  HOST_EXE="$CARGO_TARGET_DIR/release/host.exe"
elif [ -f "../../../target/release/host.exe" ]; then
  HOST_EXE="../../../target/release/host.exe"
fi
[ -n "$HOST_EXE" ] || { echo "FATAL: host.exe not found (set CARGO_TARGET_DIR or build first)"; exit 1; }
# 段 5 需要切换 CWD 注入写盘失败——host.exe 先绝对化（段 5 起 CWD 无关）。
# 整改续做修复（2026-10-07）：补 Windows 盘符绝对路径分支——CARGO_TARGET_DIR 以
# C:/... 传入时不匹配 /*，旧逻辑误判为相对路径而前缀 $PWD，路径作废致宿主无法
# 启动（中断轮实锚：host1-stderr「No such file or directory」、整轮恒 FAIL）。
case "$HOST_EXE" in
  /*) ;;  # MSYS 绝对（/c/...）
  [A-Za-z]:/* | [A-Za-z]:\\*)
    HOST_EXE=$(cygpath -u "$HOST_EXE" 2>/dev/null || echo "$HOST_EXE") ;;  # Windows 盘符绝对（CARGO_TARGET_DIR=C:/...）→ MSYS 绝对
  *) HOST_EXE="$PWD/$HOST_EXE" ;;  # 仓库根相对（../../../target/release/host.exe）
esac

PORT_SP1=15715   # spectate autorun：melee-brawl seed7（黄金锚对拍）+ 冻结后 screenshot
PORT_SP2=15716   # spectate autorun：seed43 swordsman:10 lane50（对拍记录 H2）
PORT_HL=15717    # headless 对拍腿（同参 outcome 直推）
ANCHOR_MB="0xb82a248ff23515e2"  # melee-brawl seed7 1800t（T021 归档锚，勿改写；以实行为准）

PASS=0; FAIL=0
: > "$LOG"
exec > >(tee -a "$LOG") 2>&1

check() { # name condition(0=pass)
  if [ "$2" -eq 0 ]; then PASS=$((PASS+1)); echo "[$1] PASS"; else FAIL=$((FAIL+1)); echo "[$1] FAIL"; fi
}

ID=0
brp() { # method params-json [port] -> echoes response body（REQ/RESP 原文走 stderr 落档）
  ID=$((ID+1))
  local method="$1"; local params="$2"; local port="${3:-15715}"
  local body="{\"jsonrpc\":\"2.0\",\"id\":$ID,\"method\":\"$method\",\"params\":$params}"
  echo "--- REQ #$ID (port $port): $body" >&2
  local resp
  resp=$(curl -s -m 120 -H 'Content-Type: application/json' -d "$body" "http://127.0.0.1:$port/")
  echo "--- RESP #$ID (port $port): $resp" >&2
  echo "$resp"
}

wait_ready() { # port [tries] -> 0/1（rpc.discover 通即续）
  local port="$1"; local tries="${2:-150}"
  for i in $(seq 1 "$tries"); do
    if curl -s -m 2 -H 'Content-Type: application/json' \
         -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
         "http://127.0.0.1:$port/" > /dev/null 2>&1; then return 0; fi
    sleep 0.2
  done
  return 1
}

wait_frozen() { # port timeout_s max_ticks -> echoes 冻结时 state_hash 响应（超时回空）
  # 仅轮询 state_hash 至 tick ≥ max_ticks（autorun 自走的 max_ticks 收束点，
  # D4：tick ≥ max_ticks 且未灭绝 → 驱动系统 run_battle_with(max_ticks) 冻结）。
  # **禁用 game.outcome 作冻结探针**——outcome 对未冻结对局会驱动至终局
  # （sim run_battle_with 语义，t018 落档），会污染手动驱动段（首轮脚本缺陷
  # 实锚：CHK-16 在 tick=6 被 outcome 探针直驱至终局）。
  local port="$1"; local timeout="$2"; local maxt="$3"; local elapsed=0
  while [ "$elapsed" -lt "$timeout" ]; do
    R=$(brp game.state_hash '{}' "$port")
    T=$(echo "$R" | grep -o '"tick":[0-9]*' | tail -1 | grep -o '[0-9]*')
    if [ -n "$T" ] && [ "$T" -ge "$maxt" ]; then echo "$R"; return 0; fi
    sleep 1; elapsed=$((elapsed+1))
  done
  echo ""; return 1
}

assert_no_panic() { # stderr-log name
  grep -q "panic" "$1"; check "$2" $([ $? -ne 0 ]; echo $?)
}

png_magic_ok() { # file -> 0/1（\x89PNG\r\n\x1a\n = 89504e470d0a1a0a）
  [ "$(head -c 8 "$1" | od -An -tx1 | tr -d ' \n')" = "89504e470d0a1a0a" ]
}

png_dims() { # file -> echoes "WxH"（IHDR：字节 17-20 = 宽 BE u32、21-24 = 高 BE u32）
  local h
  h=$(head -c 24 "$1" | tail -c 8 | od -An -tx1 | tr -d ' \n')
  echo "$((16#${h:0:8}))x$((16#${h:8:8}))"
}

port_free() { # port -> 0=free
  ! netstat -ano | grep -q ":$1 .*LISTENING"
}

echo "== T019 spectate smoke $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE =="

# 端口占用预检（三口全须空闲——防误连残留实例）
for P in $PORT_SP1 $PORT_SP2 $PORT_HL; do
  port_free "$P"; check "CHK-00-port-$P-free" $?
done

# ── 段 1（D9-1）：spectate autorun melee-brawl seed7 → 冻结 → 黄金锚对拍 ──
"$HOST_EXE" --spectate --preset melee-brawl --seed 7 --port $PORT_SP1 > host1-stdout.log 2> host1-stderr.log &
HOST1_PID=$!
echo "== host1 pid $HOST1_PID (spectate autorun, port $PORT_SP1) =="

wait_ready "$PORT_SP1"; check "CHK-01-port-ready" $?

# banner 双留痕（D3：mode=spectate 行 + 回环行）；autorun auto-deploy 行（D4）
grep -qF "[host] mode=spectate" host1-stderr.log; check "CHK-02-banner-mode-spectate" $?
grep -qF "127.0.0.1:$PORT_SP1" host1-stderr.log; check "CHK-02b-banner-loopback-port" $?
grep -qF "non-loopback forbidden" host1-stderr.log; check "CHK-02c-banner-loopback-constraint" $?
grep -qF "[host] auto-deployed melee-brawl seed=7 units=60 tick=0" host1-stderr.log; check "CHK-02d-banner-autodeploy" $?

# 布阵快照立即可查（autorun 已自走，tick 可能 >0——只断言 deploy 行 + 哈希可取）
R=$(brp game.state_hash '{}' "$PORT_SP1")
echo "$R" | grep -qF '"state_hash":"0x'; check "CHK-03-statehash-shape" $?

# 轮询至 autorun 自走收束（30Hz 节流：1800 tick ≈ 60 s 挂钟 + 裕量），再取终局四元组
FROZEN1=$(wait_frozen "$PORT_SP1" 180 1800)
[ -n "$FROZEN1" ]; check "CHK-04-autoran-frozen" $?
OUT1=$(brp game.outcome '{}' "$PORT_SP1")
END1=$(echo "$OUT1" | grep -o '"end_tick":[0-9]*' | head -1 | grep -o '[0-9]*')
H1=$(echo "$OUT1" | grep -o '"final_hash":"0x[0-9a-f]*"' | head -1)
echo "spectate melee-brawl seed7: end_tick=$END1 final_hash=$H1 (frozen_at: $FROZEN1)"
echo "$OUT1" | grep -qF "\"final_hash\":\"$ANCHOR_MB\""; check "CHK-05-anchor-mb-bitexact" $?

# 冻结后 state_hash 与 outcome.final_hash 一致（resolve 不推 tick）
R=$(brp game.state_hash '{}' "$PORT_SP1")
echo "$R" | grep -qF "\"state_hash\":\"$ANCHOR_MB\""; check "CHK-05b-statehash-frozen-consistent" $?

# 已冻结 run_to_tick → 4002（D4：spectate 形态同域）
R=$(brp game.run_to_tick '{"ticks":10}' "$PORT_SP1")
echo "$R" | grep -qF '"code":4002'; check "CHK-06-frozen-run-4002" $?

# 冻结后 screenshot（终局 HUD + 战场人证；两段式全流程断言）
R=$(brp game.screenshot '{}' "$PORT_SP1")
echo "$R" | grep -qF '"status":"requested"'; check "CHK-07-shot-requested" $?
# id 提取取**末位**匹配（响应含 JSON-RPC 回显 id 在前、result.id 在后——
# 首轮脚本缺陷：head -1 取到回显 id 致轮询 {"id":1} 恒 -32602）
SHOT1_ID=$(echo "$R" | grep -o '"id":[0-9]*' | tail -1 | grep -o '[0-9]*')
SHOT1_PATH=""
if [ -n "$SHOT1_ID" ]; then
  for i in $(seq 1 60); do
    R=$(brp game.screenshot "{\"id\":$SHOT1_ID}" "$PORT_SP1")
    if echo "$R" | grep -qF '"status":"captured"'; then break; fi
    sleep 0.5
  done
  echo "$R" | grep -qF '"status":"captured"'; check "CHK-07b-shot-captured" $?
  SHOT1_PATH=$(echo "$R" | grep -o '"path":"[^"]*"' | head -1 | sed 's/"path":"//;s/"//')
  SHOT1_BYTES=$(echo "$R" | grep -o '"bytes":[0-9]*' | head -1 | grep -o '[0-9]*')
  echo "shot1: id=$SHOT1_ID path=$SHOT1_PATH bytes=$SHOT1_BYTES"
  [ -n "$SHOT1_PATH" ] && [ -f "$SHOT1_PATH" ]; check "CHK-07c-shot-file-exists" $?
  [ -n "$SHOT1_BYTES" ] && [ "$SHOT1_BYTES" -gt 0 ]; check "CHK-07d-shot-bytes-nonzero" $?
  png_magic_ok "$SHOT1_PATH"; check "CHK-07e-shot-png-magic" $?
  mv "$SHOT1_PATH" "shot-1-frozen-melee-seed7.png"
  sha256sum "shot-1-frozen-melee-seed7.png"
  check "CHK-07f-shot-moved-evidence" $?
else
  check "CHK-07b-shot-captured" 1; check "CHK-07c-shot-file-exists" 1
  check "CHK-07d-shot-bytes-nonzero" 1; check "CHK-07e-shot-png-magic" 1; check "CHK-07f-shot-moved-evidence" 1
fi

kill -0 "$HOST1_PID" 2>/dev/null; check "CHK-08-host1-alive" $?
kill "$HOST1_PID" 2>/dev/null; sleep 1; kill -9 "$HOST1_PID" 2>/dev/null
assert_no_panic host1-stderr.log CHK-08b-host1-no-panic

# ── 段 2（D9-2）：spectate 第二种子 autorun → 记录 H2 → headless 同参对拍 ──
"$HOST_EXE" --spectate --seed 43 --comp swordsman:10 --lane-len-m 50 --port $PORT_SP2 > host2-stdout.log 2> host2-stderr.log &
HOST2_PID=$!
echo "== host2 pid $HOST2_PID (spectate autorun, port $PORT_SP2) =="
wait_ready "$PORT_SP2"; check "CHK-09-port2-ready" $?

OUT2=$(wait_frozen "$PORT_SP2" 180 1800)
[ -n "$OUT2" ]; check "CHK-10-autoran2-frozen" $?
OUT2=$(brp game.outcome '{}' "$PORT_SP2")
H2=$(echo "$OUT2" | grep -o '"final_hash":"0x[0-9a-f]*"' | head -1)
W2=$(echo "$OUT2" | grep -o '"winner":"[a-z]*"' | head -1)
E2=$(echo "$OUT2" | grep -o '"end_tick":[0-9]*' | head -1)
echo "spectate seed43 swordsman:10 lane50: $W2 $E2 $H2"
[ -n "$H2" ]; check "CHK-10b-hash2-captured" $?

kill -0 "$HOST2_PID" 2>/dev/null; check "CHK-10c-host2-alive" $?
kill "$HOST2_PID" 2>/dev/null; sleep 1; kill -9 "$HOST2_PID" 2>/dev/null
assert_no_panic host2-stderr.log CHK-10d-host2-no-panic

# headless 对拍腿：同参 CLI auto-deploy + BRP outcome 直推（headless 形态）
"$HOST_EXE" --seed 43 --comp swordsman:10 --lane-len-m 50 --port $PORT_HL > host3-stdout.log 2> host3-stderr.log &
HOST3_PID=$!
echo "== host3 pid $HOST3_PID (headless, port $PORT_HL) =="
wait_ready "$PORT_HL"; check "CHK-11-port3-ready" $?
grep -qF "[host] mode=headless" host3-stderr.log; check "CHK-11b-banner-headless" $?

R=$(brp game.outcome '{}' "$PORT_HL")
H3=$(echo "$R" | grep -o '"final_hash":"0x[0-9a-f]*"' | head -1)
echo "headless seed43 swordsman:10 lane50: $H3"
[ "$H2" = "$H3" ] && [ -n "$H2" ]; check "CHK-12-spectate-headless-bitexact" $?

kill -0 "$HOST3_PID" 2>/dev/null; check "CHK-12b-host3-alive" $?
kill "$HOST3_PID" 2>/dev/null; sleep 1; kill -9 "$HOST3_PID" 2>/dev/null
assert_no_panic host3-stderr.log CHK-12c-host3-no-panic

# ── 段 3（D9-3）：spectate 纯服务 + BRP deploy + run_to_tick 入队立即返回 ──
"$HOST_EXE" --spectate --port $PORT_SP1 > host4-stdout.log 2> host4-stderr.log &
HOST4_PID=$!
echo "== host4 pid $HOST4_PID (spectate pure service, port $PORT_SP1) =="
wait_ready "$PORT_SP1"; check "CHK-13-port-ready-again" $?

D2='{"kind":"swordsman","count":10}'
R=$(brp game.deploy "{\"seed\":43,\"red\":[$D2],\"blue\":[$D2],\"lane_len_m\":50}" "$PORT_SP1")
echo "$R" | grep -qF '"tick":0'; check "CHK-14-brp-deploy-tick0" $?
echo "$R" | grep -qF '"units":20'; check "CHK-14b-brp-deploy-units20" $?

# 入队立即返回：queued=300 + 当前 tick=0（D4 响应形态；键序 = serde_json
# 字典序，tick 为末键）
R=$(brp game.run_to_tick '{"ticks":300}' "$PORT_SP1")
echo "$R" | grep -qF '"queued":300'; check "CHK-15-queued-300-immediate" $?
echo "$R" | grep -qF '"tick":0}'; check "CHK-15b-queued-tick0" $?

# 轮询至 tick==300（30Hz ≈ 10 s + 裕量）——**仅 state_hash**，不用 outcome 探针
# （会驱动未冻结对局，见 wait_frozen 注）；本参数对局灭绝 > 300（seg2 实锚
# end_tick=1800），tick==300 必达。
SP_TICK=""; SP_HASH=""; elapsed=0
while [ "$elapsed" -lt 60 ]; do
  R=$(brp game.state_hash '{}' "$PORT_SP1")
  T=$(echo "$R" | grep -o '"tick":[0-9]*' | tail -1 | grep -o '[0-9]*')
  if [ -n "$T" ] && [ "$T" -ge 300 ]; then
    SP_TICK="$T"; SP_HASH=$(echo "$R" | grep -o '"state_hash":"0x[0-9a-f]*"' | head -1); break
  fi
  sleep 1; elapsed=$((elapsed+1))
done
echo "spectate manual-drive: tick=$SP_TICK hash=$SP_HASH"
[ -n "$SP_HASH" ]; check "CHK-16-manual-advanced" $?

# headless 同参 300t 对拍（纯服务 + BRP deploy + run_to_tick 直推）
"$HOST_EXE" --port $PORT_HL > host5-stdout.log 2> host5-stderr.log &
HOST5_PID=$!
echo "== host5 pid $HOST5_PID (headless pure service, port $PORT_HL) =="
wait_ready "$PORT_HL"; check "CHK-17-port3-ready-again" $?
R=$(brp game.deploy "{\"seed\":43,\"red\":[$D2],\"blue\":[$D2],\"lane_len_m\":50}" "$PORT_HL")
echo "$R" | grep -qF '"tick":0'; check "CHK-17b-headless-deploy" $?
R=$(brp game.run_to_tick '{"ticks":300}' "$PORT_HL")
HL_TICK=$(echo "$R" | grep -o '"tick":[0-9]*' | head -1 | grep -o '[0-9]*')
HL_HASH=$(echo "$R" | grep -o '"state_hash":"0x[0-9a-f]*"' | head -1)
echo "headless 300t: tick=$HL_TICK hash=$HL_HASH"
[ "$SP_TICK" = "$HL_TICK" ] && [ "$SP_HASH" = "$HL_HASH" ] && [ -n "$SP_HASH" ]
check "CHK-18-manual-crossmode-bitexact" $?

# 重部署世代替换（D5）：BRP deploy default 构成（六兵种）→ 表现层重建
R=$(brp game.deploy '{"preset":"default","seed":42}' "$PORT_SP1")
echo "$R" | grep -qF '"units":60'; check "CHK-19-redeploy-default-units60" $?

# screenshot 两段式（六兵种形状 × 阵营色人证，任务卡断言 5）
sleep 2   # 留一帧窗口给表现层重建
R=$(brp game.screenshot '{}' "$PORT_SP1")
echo "$R" | grep -qF '"status":"requested"'; check "CHK-20-shot2-requested" $?
SHOT2_ID=$(echo "$R" | grep -o '"id":[0-9]*' | tail -1 | grep -o '[0-9]*')
SHOT2_PATH=""
if [ -n "$SHOT2_ID" ]; then
  for i in $(seq 1 60); do
    R=$(brp game.screenshot "{\"id\":$SHOT2_ID}" "$PORT_SP1")
    if echo "$R" | grep -qF '"status":"captured"'; then break; fi
    sleep 0.5
  done
  echo "$R" | grep -qF '"status":"captured"'; check "CHK-20b-shot2-captured" $?
  SHOT2_PATH=$(echo "$R" | grep -o '"path":"[^"]*"' | head -1 | sed 's/"path":"//;s/"//')
  SHOT2_BYTES=$(echo "$R" | grep -o '"bytes":[0-9]*' | head -1 | grep -o '[0-9]*')
  echo "shot2: id=$SHOT2_ID path=$SHOT2_PATH bytes=$SHOT2_BYTES"
  [ -n "$SHOT2_PATH" ] && [ -f "$SHOT2_PATH" ]; check "CHK-20c-shot2-file-exists" $?
  [ -n "$SHOT2_BYTES" ] && [ "$SHOT2_BYTES" -gt 0 ]; check "CHK-20d-shot2-bytes-nonzero" $?
  png_magic_ok "$SHOT2_PATH"; check "CHK-20e-shot2-png-magic" $?
  mv "$SHOT2_PATH" "shot-2-sixkinds-deployed.png"
  sha256sum "shot-2-sixkinds-deployed.png"
  check "CHK-20f-shot2-moved-evidence" $?
else
  check "CHK-20b-shot2-captured" 1; check "CHK-20c-shot2-file-exists" 1
  check "CHK-20d-shot2-bytes-nonzero" 1; check "CHK-20e-shot2-png-magic" 1; check "CHK-20f-shot2-moved-evidence" 1
fi

kill -0 "$HOST4_PID" 2>/dev/null; check "CHK-21-host4-alive" $?
kill "$HOST4_PID" 2>/dev/null; sleep 1; kill -9 "$HOST4_PID" 2>/dev/null
kill -0 "$HOST5_PID" 2>/dev/null; check "CHK-21b-host5-alive" $?
kill "$HOST5_PID" 2>/dev/null; sleep 1; kill -9 "$HOST5_PID" 2>/dev/null
assert_no_panic host4-stderr.log CHK-21c-host4-no-panic
assert_no_panic host5-stderr.log CHK-21d-host5-no-panic

# 进程收尾核验（三口全空——无残留实例）
sleep 1
for P in $PORT_SP1 $PORT_SP2 $PORT_HL; do
  port_free "$P"; check "CHK-22-port-$P-reaped" $?
done

# ── 段 4（P0 整改 + P1 断言①）：近景可辨——六兵种 × 双阵营 lane 60m（shot-3）──
"$HOST_EXE" --spectate --port $PORT_SP2 > host6-stdout.log 2> host6-stderr.log &
HOST6_PID=$!
echo "== host6 pid $HOST6_PID (spectate closeup, port $PORT_SP2) =="
wait_ready "$PORT_SP2"; check "CHK-23-port2-ready-closeup" $?

# 六兵种各 1 × 双阵营 = 12 单位、lane 60m（算式见脚本头注：~25px/单位）
SIX='[{"kind":"shieldman","count":1},{"kind":"heavyknight","count":1},{"kind":"pikeman","count":1},{"kind":"swordsman","count":1},{"kind":"archer","count":1},{"kind":"militia","count":1}]'
R=$(brp game.deploy "{\"seed\":42,\"red\":$SIX,\"blue\":$SIX,\"lane_len_m\":60}" "$PORT_SP2")
echo "$R" | grep -qF '"tick":0'; check "CHK-24-closeup-deploy-tick0" $?
echo "$R" | grep -qF '"units":12'; check "CHK-24b-closeup-deploy-units12" $?

# shot-3 两段式（近景形状人证；BRP deploy 恒 autorun=false → tick 冻在 0，
# 六兵种布阵态 + 阵营色同框）
sleep 2   # 留帧窗口给表现层重建（CHK-19 同款）
R=$(brp game.screenshot '{}' "$PORT_SP2")
echo "$R" | grep -qF '"status":"requested"'; check "CHK-25-closeup-shot-requested" $?
SHOT3_ID=$(echo "$R" | grep -o '"id":[0-9]*' | tail -1 | grep -o '[0-9]*')
SHOT3_FULLPATH=""
SHOT3_OK=0
if [ -n "$SHOT3_ID" ]; then
  for i in $(seq 1 60); do
    R=$(brp game.screenshot "{\"id\":$SHOT3_ID}" "$PORT_SP2")
    if echo "$R" | grep -qF '"status":"captured"'; then break; fi
    sleep 0.5
  done
  echo "$R" | grep -qF '"status":"captured"'; check "CHK-25b-closeup-shot-captured" $?
  SHOT3_FULLPATH=$(echo "$R" | grep -o '"path":"[^"]*"' | head -1 | sed 's/"path":"//;s/"//')
  SHOT3_BYTES=$(echo "$R" | grep -o '"bytes":[0-9]*' | head -1 | grep -o '[0-9]*')
  echo "shot3: id=$SHOT3_ID path=$SHOT3_FULLPATH bytes=$SHOT3_BYTES"
  if [ -n "$SHOT3_FULLPATH" ] && [ -f "$SHOT3_FULLPATH" ]; then
    check "CHK-25c-closeup-shot-file-exists" 0
    [ -n "$SHOT3_BYTES" ] && [ "$SHOT3_BYTES" -gt 0 ]; check "CHK-25d-closeup-shot-bytes-nonzero" $?
    png_magic_ok "$SHOT3_FULLPATH"; check "CHK-25e-closeup-shot-png-magic" $?
    DIMS=$(png_dims "$SHOT3_FULLPATH")
    W=$(echo "$DIMS" | cut -dx -f1); H=$(echo "$DIMS" | cut -dx -f2)
    echo "shot3 dims: ${W}x${H}（逻辑 1920x1080；125% DPI 物理捕获 2400x1350，README §4 注）"
    [ "$W" -ge 1920 ] && [ "$H" -ge 1080 ] && [ $((W*9)) -eq $((H*16)) ]
    check "CHK-25f-closeup-shot-dims-1080p-16v9" $?
    [ -n "$SHOT3_BYTES" ] && [ "$SHOT3_BYTES" -ge 30720 ]; check "CHK-25g-closeup-shot-min30k" $?
    mv "$SHOT3_FULLPATH" "shot-3-sixkinds-closeup-lane60.png"
    sha256sum "shot-3-sixkinds-closeup-lane60.png"
    check "CHK-25h-closeup-shot-moved-evidence" $?
    SHOT3_OK=1
  else
    check "CHK-25c-closeup-shot-file-exists" 1; check "CHK-25d-closeup-shot-bytes-nonzero" 1
    check "CHK-25e-closeup-shot-png-magic" 1; check "CHK-25f-closeup-shot-dims-1080p-16v9" 1
    check "CHK-25g-closeup-shot-min30k" 1; check "CHK-25h-closeup-shot-moved-evidence" 1
  fi
else
  check "CHK-25b-closeup-shot-captured" 1; check "CHK-25c-closeup-shot-file-exists" 1
  check "CHK-25d-closeup-shot-bytes-nonzero" 1; check "CHK-25e-closeup-shot-png-magic" 1
  check "CHK-25f-closeup-shot-dims-1080p-16v9" 1; check "CHK-25g-closeup-shot-min30k" 1
  check "CHK-25h-closeup-shot-moved-evidence" 1
fi

# P1 断言①：同进程第二次截图——nonce 段相同、id 递增、路径不同（跨进程
# 唯一性由 nonce 保证，见 spectate.rs ScreenshotLog 注）
R=$(brp game.screenshot '{}' "$PORT_SP2")
echo "$R" | grep -qF '"status":"requested"'; check "CHK-26-shot2nd-requested" $?
SHOTB_ID=$(echo "$R" | grep -o '"id":[0-9]*' | tail -1 | grep -o '[0-9]*')
SHOTB_PATH=""
if [ -n "$SHOTB_ID" ]; then
  for i in $(seq 1 60); do
    R=$(brp game.screenshot "{\"id\":$SHOTB_ID}" "$PORT_SP2")
    if echo "$R" | grep -qF '"status":"captured"'; then break; fi
    sleep 0.5
  done
  echo "$R" | grep -qF '"status":"captured"'; check "CHK-26b-shot2nd-captured" $?
  SHOTB_PATH=$(echo "$R" | grep -o '"path":"[^"]*"' | head -1 | sed 's/"path":"//;s/"//')
  echo "shot3b: id=$SHOTB_ID path=$SHOTB_PATH（唯一性断言用，用完即弃不进证据集）"
  A_NONCE=$(basename "$SHOT3_FULLPATH" | sed -n 's/^screenshot-\([0-9]*-[0-9]*\)-[0-9]*\.png$/\1/p')
  A_IDNUM=$(basename "$SHOT3_FULLPATH" | sed -n 's/^screenshot-[0-9]*-[0-9]*-\([0-9]*\)\.png$/\1/p')
  B_NONCE=$(basename "$SHOTB_PATH" | sed -n 's/^screenshot-\([0-9]*-[0-9]*\)-[0-9]*\.png$/\1/p')
  B_IDNUM=$(basename "$SHOTB_PATH" | sed -n 's/^screenshot-[0-9]*-[0-9]*-\([0-9]*\)\.png$/\1/p')
  echo "shot naming: A(nonce=$A_NONCE id=$A_IDNUM) B(nonce=$B_NONCE id=$B_IDNUM)"
  # 三段名 screenshot-{pid}-{nonce}-{id}.png（修后复审 Important 86 整改体例；
  # A_NONCE/B_NONCE 变量现承载 pid-nonce 两段）
  A_PID=$(echo "$A_NONCE" | cut -d- -f1); B_PID=$(echo "$B_NONCE" | cut -d- -f1)
  [ -n "$A_NONCE" ] && [ "$A_NONCE" = "$B_NONCE" ]; check "CHK-26c-shot-pidnonce-same-process" $?
  [ "$A_PID" != "0" ] && [ "$A_PID" = "$B_PID" ]; check "CHK-26c2-shot-pid-live-nonzero" $?
  [ -n "$A_IDNUM" ] && [ -n "$B_IDNUM" ] && [ "$B_IDNUM" -eq $((A_IDNUM+1)) ]
  check "CHK-26d-shot-id-increment" $?
  [ -n "$SHOTB_PATH" ] && [ "$SHOTB_PATH" != "$SHOT3_FULLPATH" ]; check "CHK-26e-shot-paths-distinct" $?
  [ -f "$SHOTB_PATH" ] && sha256sum "$SHOTB_PATH"
  rm -f "$SHOTB_PATH"; check "CHK-26f-shot2nd-cleaned" $?
else
  check "CHK-26b-shot2nd-captured" 1; check "CHK-26c-shot-nonce-same-process" 1
  check "CHK-26d-shot-id-increment" 1; check "CHK-26e-shot-paths-distinct" 1
  check "CHK-26f-shot2nd-cleaned" 1
fi

kill -0 "$HOST6_PID" 2>/dev/null; check "CHK-26g-host6-alive" $?
kill "$HOST6_PID" 2>/dev/null; sleep 1; kill -9 "$HOST6_PID" 2>/dev/null
assert_no_panic host6-stderr.log CHK-26h-host6-no-panic

# ── 段 5（P1 断言②）：预置残留 + 物理写盘失败 → 恒 pending（不误报 captured）──
# 场景 = 复审 P1 原文最坏情形：本次保存失败（独占句柄锁住目标路径）且同路径
# 预置有效 PNG 残留——nonce 唯一路径 + mtime ≥ 受理时刻界桩共同闭合。
INJDIR="$PWD/p1-inject-tmp"
INJDIR_WIN=$(cygpath -m "$INJDIR" 2>/dev/null || echo "$INJDIR")
rm -rf "$INJDIR"; mkdir -p "$INJDIR"
EVDIR="$PWD"
cd "$INJDIR"
"$HOST_EXE" --spectate --port $PORT_SP1 > "$EVDIR/host7-stdout.log" 2> "$EVDIR/host7-stderr.log" &
HOST7_PID=$!
cd "$EVDIR"
echo "== host7 pid $HOST7_PID (spectate inject, port $PORT_SP1, cwd $INJDIR) =="
wait_ready "$PORT_SP1"; check "CHK-27-port1-ready-inject" $?

# 基线：小构型布阵 + 正常截一张（写盘通路 OK），从响应路径取得 nonce 体例
R=$(brp game.deploy '{"seed":42,"red":[{"kind":"swordsman","count":1}],"blue":[{"kind":"swordsman","count":1}],"lane_len_m":60}' "$PORT_SP1")
echo "$R" | grep -qF '"units":2'; check "CHK-27b-inject-deploy" $?
R=$(brp game.screenshot '{}' "$PORT_SP1")
INJ_ID=$(echo "$R" | grep -o '"id":[0-9]*' | tail -1 | grep -o '[0-9]*')
INJ_PATH0=""
if [ -n "$INJ_ID" ]; then
  for i in $(seq 1 60); do
    R=$(brp game.screenshot "{\"id\":$INJ_ID}" "$PORT_SP1")
    if echo "$R" | grep -qF '"status":"captured"'; then break; fi
    sleep 0.5
  done
  echo "$R" | grep -qF '"status":"captured"'; check "CHK-28-inject-shot0-captured" $?
  INJ_PATH0=$(echo "$R" | grep -o '"path":"[^"]*"' | head -1 | sed 's/"path":"//;s/"//')
  echo "inject baseline: path=$INJ_PATH0"
else
  check "CHK-28-inject-shot0-captured" 1
fi

INJ_PREPARED=0
if [ -n "$INJ_PATH0" ] && [ -f "$INJDIR/$INJ_PATH0" ]; then
  # 预置残留：有效 PNG 复制到下一次请求将用的路径（screenshot-{pid}-{nonce}-1.png，
  # 同进程 pid+nonce 同段——id 序号可预知）。残留 mtime 早于下一次受理时刻。
  INJ_NONCE=$(basename "$INJ_PATH0" | sed -n 's/^screenshot-\([0-9]*-[0-9]*\)-[0-9]*\.png$/\1/p')
  INJ_PATH1="screenshot-${INJ_NONCE}-1.png"
  cp "$INJDIR/$INJ_PATH0" "$INJDIR/$INJ_PATH1"
  png_magic_ok "$INJDIR/$INJ_PATH1"; check "CHK-28b-inject-plant-valid-png" $?
  sha256sum "$INJDIR/$INJ_PATH1"
  # 物理注入：PowerShell 独占句柄（FileShare::None）——std::fs 写打开被拒
  # sharing violation → save_to_disk 写盘失败（screenshot.rs:145-147 只记日志）。
  powershell -NoProfile -Command "\$fs=[System.IO.File]::Open('$INJDIR_WIN/$INJ_PATH1',[System.IO.FileMode]::Open,[System.IO.FileAccess]::ReadWrite,[System.IO.FileShare]::None); Start-Sleep -Seconds 40; \$fs.Close()" &
  HOLDER_PID=$!
  sleep 1.5
  INJ_PREPARED=1
fi
if [ "$INJ_PREPARED" -eq 1 ]; then
  # 下一次请求（id=1）：受理时刻晚于残留 mtime；保存必失败
  R=$(brp game.screenshot '{}' "$PORT_SP1")
  INJ_ID1=$(echo "$R" | grep -o '"id":[0-9]*' | tail -1 | grep -o '[0-9]*')
  INJ_CAPT=0
  if [ -n "$INJ_ID1" ]; then
    for i in $(seq 1 12); do
      R=$(brp game.screenshot "{\"id\":$INJ_ID1}" "$PORT_SP1")
      echo "  inject poll #$i: $R"
      echo "$R" | grep -qF '"status":"captured"' && INJ_CAPT=1
      [ "$INJ_CAPT" -eq 1 ] && break
      sleep 0.5
    done
    [ "$INJ_CAPT" -eq 0 ]; check "CHK-29-inject-shot1-never-captured" $?
    echo "$R" | grep -qF '"status":"pending"'; check "CHK-29b-inject-shot1-pending-final" $?
    echo "$R" | grep -qF '"detail"'; check "CHK-29c-inject-shot1-detail-present" $?
  else
    check "CHK-29-inject-shot1-never-captured" 1; check "CHK-29b-inject-shot1-pending-final" 1
    check "CHK-29c-inject-shot1-detail-present" 1
  fi
  # 物理见证：bevy save_to_disk 写盘失败日志确实发生（注入成立）
  grep -qF "Cannot save screenshot" "$EVDIR/host7-stderr.log"
  check "CHK-29e-inject-save-failure-witness" $?
  # 收尾：杀句柄持有者 → 文件锁释放
  kill "$HOLDER_PID" 2>/dev/null; sleep 0.5; kill -9 "$HOLDER_PID" 2>/dev/null
  ! kill -0 "$HOLDER_PID" 2>/dev/null; check "CHK-29f-inject-holder-reaped" $?
  # P1 时间戳分支实证（整改续做修复 2026-10-07）：锁持有期内文件打开被
  # sharing violation 拒绝（os error 32 实锚），mtime 界桩分支不可达——杀掉
  # 句柄持有者后复询：本次输出文件可读、mtime 早于受理时刻 → 恒 pending +
  # detail = stale leftover suspected（不误报 captured；mini-test 预验通过）。
  # 原 29d 断言置于锁持有期内属脚本缺陷（mtime 分支不可达），本修复移至此。
  if [ -n "$INJ_ID1" ]; then
    sleep 1
    R=$(brp game.screenshot "{\"id\":$INJ_ID1}" "$PORT_SP1")
    echo "  inject post-release poll: $R"
    echo "$R" | grep -qF 'stale leftover suspected'; check "CHK-29d-inject-shot1-stale-flagged" $?
  else
    check "CHK-29d-inject-shot1-stale-flagged" 1
  fi
else
  check "CHK-28b-inject-plant-valid-png" 1; check "CHK-29-inject-shot1-never-captured" 1
  check "CHK-29b-inject-shot1-pending-final" 1; check "CHK-29c-inject-shot1-detail-present" 1
  check "CHK-29d-inject-shot1-stale-flagged" 1; check "CHK-29e-inject-save-failure-witness" 1
  check "CHK-29f-inject-holder-reaped" 1
fi

kill -0 "$HOST7_PID" 2>/dev/null; check "CHK-30-host7-alive" $?
kill "$HOST7_PID" 2>/dev/null; sleep 1; kill -9 "$HOST7_PID" 2>/dev/null
assert_no_panic "$EVDIR/host7-stderr.log" CHK-30b-host7-no-panic
rm -rf "$INJDIR"; [ ! -d "$INJDIR" ]; check "CHK-30c-inject-tmpdir-cleaned" $?

# 进程收尾核验（段 4/5 两口全空——无残留实例）
sleep 1
for P in $PORT_SP2 $PORT_SP1; do
  port_free "$P"; check "CHK-30d-port-$P-reaped-closeup-inject" $?
done

echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
