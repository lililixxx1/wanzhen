#!/usr/bin/env bash
# T023 统计面冒烟（D5）：BRP 端口 15707（独占）——game.sample_outcomes 断言集：
#   ① M0 锚对拍（默认构成 games=1 seed_base=42 默认 lane/ticks → outcome[0]
#      = T004 黄金锚 0x958c5938c8682529 / draw@1800 / 30:30，纯只读路径）[CHK-03]
#   ② 确定性逐字节比对——同参两批 outcomes 数组原文逐字节全等 + 汇总字段全等
#      + 前缀性质（games=2 批 == games=4 批前 2 局）[CHK-04/05]
#   ③ 跨线程抽查 games=16 @threads 1 vs 12 → 每局六字段逐字段一致 + 汇总全等 [CHK-06]
#   ④ 种子序列（含 u64 wrap：seed_base=2^64-2, games=3 → …614/…615/0）[CHK-07]
#   ⑤ 边界批 games=1000（上限）全量 outcomes + JSON 体量实测（<10MB 判定）[CHK-08]
#   ⑥ 不触碰 HostedGame——deploy 后经 sample 批 → state_hash 零变化 →
#      run_to_tick 仍命中锚② [CHK-09]
#   ⑦ error 路径 8 发（games=0/1001、缺 red、缺 seed_base、未知兵种、
#      max_ticks=0、threads=1025、lane_len_m=0 → 全 -32602 + 消息文本）[CHK-10]
#   ⑧ banner 7 方法行 + rpc.discover 7 方法 + 回环端口 15707 [CHK-01/02]
#   ⑨ 口径注三件在档检查（matrix-example.md 与 README.md 各 3 条，逐字 grep）[CHK-11]
#   ⑩ 进程存活收尾 [CHK-12]。
# 依赖顺序：先跑 matrix_example.sh 生成 matrix-example.md（⑨ 检查其存在）。
# 判定行输出到 stdout 并 tee 落 sample-smoke-run.log；REQ/RESP 原文经 stderr 全量落档。
# 用法：bash sample_smoke.sh（机器基本空闲即可；非帧敏量测，无独占窗口要求）
set -u
cd "$(dirname "$0")"
export LC_ALL=C   # 口径注 CJK 字面按字节匹配（grep -F 稳定）
HOST_EXE="../../../target/release/host.exe"
LOG="sample-smoke-run.log"
TMP="_smoke_tmp"
PORT=15707
DEFAULT_COMP='[{"kind":"shieldman","count":5},{"kind":"heavyknight","count":5},{"kind":"pikeman","count":5},{"kind":"swordsman","count":5},{"kind":"archer","count":5},{"kind":"militia","count":5}]'
MIX='[{"kind":"swordsman","count":15},{"kind":"militia","count":15}]'
S10='[{"kind":"swordsman","count":10}]'
M30='[{"kind":"militia","count":30}]'
ONE_R='[{"kind":"swordsman","count":1}]'
ONE_B='[{"kind":"militia","count":1}]'
ANCHOR2="0x958c5938c8682529"  # M0 T004 归档锚：seed42 默认构成 1800 ticks（勿改写）

PASS=0; FAIL=0
: > "$LOG"
rm -rf "$TMP"; mkdir -p "$TMP"
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

# 比对辅助（python；stdout 摘要、exit 0/1）——判定行由脚本计算，人工只解读。
cat > "$TMP/check.py" <<'PY'
import json, re, sys

def load(path):
    with open(path, encoding="utf-8") as f:
        return f.read()

def result(resp):
    return json.loads(resp)["result"]

def raw_outcomes(resp):
    m = re.search(r'"outcomes":(\[.*?\])', resp)
    return m.group(1) if m else None

def fail(msg):
    print("MISMATCH: " + msg)
    sys.exit(1)

mode = sys.argv[1]
if mode == "eq":
    ra, rb = load(sys.argv[2]), load(sys.argv[3])
    a, b = result(ra), result(rb)
    if raw_outcomes(ra) != raw_outcomes(rb):
        fail("outcomes raw bytes differ")
    if a != b:
        fail("parsed result differs")
    print("EQ: outcomes arrays byte-identical + summary equal (games=%d)" % a["games"])
elif mode == "prefix":
    small, big = result(load(sys.argv[2])), result(load(sys.argv[3]))
    if big["outcomes"][:len(small["outcomes"])] != small["outcomes"]:
        fail("prefix property violated")
    print("PREFIX: %d-game outcomes == first %d of %d-game batch" % (small["games"], small["games"], big["games"]))
elif mode == "cross":
    a, b = result(load(sys.argv[2])), result(load(sys.argv[3]))
    if len(a["outcomes"]) != len(b["outcomes"]):
        fail("outcomes length differs")
    bad = 0
    for i, (x, y) in enumerate(zip(a["outcomes"], b["outcomes"])):
        for k in ("seed", "winner", "end_tick", "alive_red", "alive_blue", "final_hash"):
            if x[k] != y[k]:
                bad += 1
                print("MISMATCH: game %d field %s: %r vs %r" % (i, k, x[k], y[k]))
    for k in ("games", "seed_base", "red_wins", "blue_wins", "draws", "win_rate_red_pp"):
        if a[k] != b[k]:
            fail("summary field %s differs: %r vs %r" % (k, a[k], b[k]))
    if bad:
        sys.exit(1)
    print("CROSS: %d games x 6 fields identical (threads 1 vs 12); summary equal; r/b/d=%d/%d/%d"
          % (len(a["outcomes"]), a["red_wins"], a["blue_wins"], a["draws"]))
elif mode == "integrity":
    r = result(load(sys.argv[2]))
    games, base, outs = r["games"], r["seed_base"], r["outcomes"]
    if len(outs) != games:
        fail("outcomes length %d != games %d" % (len(outs), games))
    for i, o in enumerate(outs):
        exp = (base + i) % (1 << 64)
        if o["seed"] != exp:
            fail("seed[%d]=%d expected %d (wrap seq)" % (i, o["seed"], exp))
        if o["winner"] not in ("red", "blue", "draw"):
            fail("winner label %r" % o["winner"])
        if not re.fullmatch(r"0x[0-9a-f]{16}", o["final_hash"]):
            fail("final_hash format %r" % o["final_hash"])
        for k in ("end_tick", "alive_red", "alive_blue"):
            if not isinstance(o[k], int) or o[k] < 0:
                fail("field %s not non-negative int: %r" % (k, o[k]))
    if r["red_wins"] + r["blue_wins"] + r["draws"] != games:
        fail("counter sum != games")
    if r["win_rate_red_pp"] != r["red_wins"] * 10000 // games:
        fail("win_rate_red_pp formula broken: %d != %d*10000//%d" % (r["win_rate_red_pp"], r["red_wins"], games))
    print("INTEGRITY: games=%d outcomes=%d seeds=%d..%d counters=%d/%d/%d win_rate_red_pp=%d"
          % (games, len(outs), outs[0]["seed"], outs[-1]["seed"],
             r["red_wins"], r["blue_wins"], r["draws"], r["win_rate_red_pp"]))
elif mode == "size":
    raw = load(sys.argv[2])
    ob = raw_outcomes(raw) or ""
    total = len(raw.encode())
    if total > 10 * 1024 * 1024:
        fail("response body %d bytes exceeds 10MB" % total)
    print("SIZE: resp_bytes=%d outcomes_bytes=%d" % (total, len(ob.encode())))
else:
    fail("unknown mode")
PY

echo "== T023 sample_outcomes smoke $(date '+%Y-%m-%d %H:%M:%S') =="
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
check CHK-00-port-ready-15707 $([ "$READY" -eq 1 ]; echo $?)

# CHK-01 banner（D2 计数门禁）：7 方法清单行逐字 + 回环端口 15707 实际绑定
grep -qF '[host] methods: game.deploy, game.run_to_tick, game.state_hash, game.outcome, game.sample_outcomes, game.run_tests, game.screenshot(stub->T019)' host-stderr.log
check CHK-01-banner-methods-7 $?
grep -qF '127.0.0.1:15707' host-stderr.log; check CHK-01b-banner-loopback $?

# CHK-02 rpc.discover：7 方法全部注册
DISC=$(brp rpc.discover '{}')
for m in game.deploy game.run_to_tick game.state_hash game.outcome game.sample_outcomes game.run_tests game.screenshot; do
  echo "$DISC" | grep -qF "\"$m\""; check "CHK-02-discover-$m" $?
done

# ── ① M0 锚对拍（纯只读；未 deploy 所有权局状态）──
# 默认 lane（1000m）默认 ticks（1800）下对称未接敌 → outcome[0] 与 T004 黄金锚逐位一致
R=$(brp game.sample_outcomes "{\"red\":$DEFAULT_COMP,\"blue\":$DEFAULT_COMP,\"seed_base\":42,\"games\":1}")
echo "$R" > "$TMP/anchor.json"
echo "$R" | grep -qF '"games":1,'; check CHK-03-games-echo $?
echo "$R" | grep -qF '"seed_base":42,'; check CHK-03b-seedbase-echo $?
echo "$R" | grep -qF '"winner":"draw"'; check CHK-03c-winner-draw $?
echo "$R" | grep -qF '"end_tick":1800'; check CHK-03d-endtick-1800 $?
echo "$R" | grep -qF '"alive_red":30'; check CHK-03e-alive-red-30 $?
echo "$R" | grep -qF '"alive_blue":30'; check CHK-03f-alive-blue-30 $?
echo "$R" | grep -qF "\"final_hash\":\"$ANCHOR2\""; check CHK-03g-anchor2-hit $?
echo "$R" | grep -qF '"red_wins":0'; check CHK-03h-red-wins-0 $?
echo "$R" | grep -qF '"draws":1,'; check CHK-03i-draws-1 $?
echo "$R" | grep -qF '"win_rate_red_pp":0'; check CHK-03j-winrate-0 $?
python "$TMP/check.py" integrity "$TMP/anchor.json"; check CHK-03k-anchor-integrity $?

# ── ② 确定性逐字节比对 + 前缀性质（异质对局：swordsman:10 vs militia:30 lane60）──
PD="{\"red\":$S10,\"blue\":$M30,\"seed_base\":42,\"lane_len_m\":60"
R=$(brp game.sample_outcomes "$PD,\"games\":8}")
echo "$R" > "$TMP/det_a.json"
R=$(brp game.sample_outcomes "$PD,\"games\":8}")
echo "$R" > "$TMP/det_b.json"
python "$TMP/check.py" eq "$TMP/det_a.json" "$TMP/det_b.json"; check CHK-04-replay-byte-eq $?
python "$TMP/check.py" integrity "$TMP/det_a.json"; check CHK-04b-det-integrity $?
R=$(brp game.sample_outcomes "$PD,\"games\":2}")
echo "$R" > "$TMP/pre_2.json"
R=$(brp game.sample_outcomes "$PD,\"games\":4}")
echo "$R" > "$TMP/pre_4.json"
python "$TMP/check.py" prefix "$TMP/pre_2.json" "$TMP/pre_4.json"; check CHK-05-prefix-property $?

# ── ③ 跨线程抽查：games=16 @threads 1 vs 12，每局六字段逐字段一致 ──
PC="{\"red\":$MIX,\"blue\":$MIX,\"seed_base\":42,\"lane_len_m\":100"
R=$(brp game.sample_outcomes "$PC,\"games\":16,\"threads\":1}")
echo "$R" > "$TMP/cross_t1.json"
R=$(brp game.sample_outcomes "$PC,\"games\":16,\"threads\":12}")
echo "$R" > "$TMP/cross_t12.json"
python "$TMP/check.py" cross "$TMP/cross_t1.json" "$TMP/cross_t12.json"; check CHK-06-cross-thread-fields $?
python "$TMP/check.py" integrity "$TMP/cross_t12.json"; check CHK-06b-cross-integrity $?

# ── ④ 种子序列 u64 wrap（seed_base=2^64-2，games=3 → …614/…615/0）──
R=$(brp game.sample_outcomes "{\"red\":$ONE_R,\"blue\":$ONE_B,\"seed_base\":18446744073709551614,\"games\":3,\"lane_len_m\":10}")
echo "$R" > "$TMP/wrap.json"
python "$TMP/check.py" integrity "$TMP/wrap.json"; check CHK-07-wrap-seed-seq $?
echo "$R" | grep -qF '"seed":18446744073709551614'; check CHK-07b-wrap-first $?
echo "$R" | grep -qF '"seed":0'; check CHK-07c-wrap-to-zero $?

# ── ⑤ 边界批 games=1000（域上限）全量 outcomes + JSON 体量实测（<10MB）──
R=$(brp game.sample_outcomes "{\"red\":$ONE_R,\"blue\":$ONE_B,\"seed_base\":42,\"games\":1000,\"lane_len_m\":10,\"threads\":12}")
echo "$R" > "$TMP/max1000.json"
python "$TMP/check.py" integrity "$TMP/max1000.json"; check CHK-08-games1000-integrity $?
python "$TMP/check.py" size "$TMP/max1000.json"; check CHK-08b-games1000-size $?

# ── ⑥ 不触碰 HostedGame：deploy → sample 批 → state_hash 零变化 → 锚②仍可达 ──
R=$(brp game.deploy "{\"seed\":42,\"red\":$DEFAULT_COMP,\"blue\":$DEFAULT_COMP,\"threads\":1}")
echo "$R" > "$TMP/deploy.json"
DEPLOY_HASH=$(echo "$R" | grep -o '"deploy_hash":"0x[0-9a-f]*"' | head -1 | grep -o '0x[0-9a-f]*')
echo "deploy_hash=$DEPLOY_HASH"
[ -n "$DEPLOY_HASH" ] && [ "$DEPLOY_HASH" != "0x0000000000000000" ]; check CHK-09-deploy-ok $?
R=$(brp game.state_hash '{}')
echo "$R" | grep -qF "\"state_hash\":\"$DEPLOY_HASH\""; check CHK-09b-state-before $?
R=$(brp game.sample_outcomes "$PC,\"games\":4}")
echo "$R" > "$TMP/post_sample.json"
python "$TMP/check.py" integrity "$TMP/post_sample.json"; check CHK-09c-sample-under-deploy $?
R=$(brp game.state_hash '{}')
echo "$R" > "$TMP/state_after.json"
echo "$R" | grep -qF "\"state_hash\":\"$DEPLOY_HASH\""; check CHK-09d-state-after-unchanged $?
echo "$R" | grep -qF '"tick":0'; check CHK-09e-tick-after-0 $?
R=$(brp game.run_to_tick '{"ticks":1800}')
echo "$R" | grep -qF "\"state_hash\":\"$ANCHOR2\""; check CHK-09f-anchor2-after-sample $?

# ── ⑦ error 路径 8 发（全部 -32602 参数错 + 消息文本）──
E="{\"red\":$MIX,\"blue\":$MIX,\"seed_base\":42"
R=$(brp game.sample_outcomes "$E,\"games\":0}")
echo "$R" | grep -qF '"code":-32602' && echo "$R" | grep -qF 'invalid games (1..=1000 required)'; check CHK-10-games-0 $?
R=$(brp game.sample_outcomes "$E,\"games\":1001}")
echo "$R" | grep -qF '"code":-32602' && echo "$R" | grep -qF 'invalid games (1..=1000 required)'; check CHK-10b-games-1001 $?
R=$(brp game.sample_outcomes "{\"blue\":$MIX,\"seed_base\":42,\"games\":4}")
echo "$R" | grep -qF '"code":-32602' && echo "$R" | grep -qF 'missing red (array of {kind, count} required)'; check CHK-10c-missing-red $?
R=$(brp game.sample_outcomes "{\"red\":$MIX,\"blue\":$MIX,\"games\":4}")
echo "$R" | grep -qF '"code":-32602' && echo "$R" | grep -qF 'missing/invalid seed_base (u64 required)'; check CHK-10d-missing-seedbase $?
R=$(brp game.sample_outcomes "{\"red\":[{\"kind\":\"laser\",\"count\":5}],\"blue\":$MIX,\"seed_base\":42,\"games\":4}")
echo "$R" | grep -qF '"code":-32602' && echo "$R" | grep -qF 'unknown kind \"laser\"'; check CHK-10e-unknown-kind $?
R=$(brp game.sample_outcomes "$E,\"games\":4,\"max_ticks\":0}")
echo "$R" | grep -qF '"code":-32602' && echo "$R" | grep -qF 'invalid max_ticks (1..=14400 required)'; check CHK-10f-max-ticks-0 $?
R=$(brp game.sample_outcomes "$E,\"games\":4,\"threads\":1025}")
echo "$R" | grep -qF '"code":-32602' && echo "$R" | grep -qF 'invalid threads (1..=1024 required)'; check CHK-10g-threads-1025 $?
R=$(brp game.sample_outcomes "$E,\"games\":4,\"lane_len_m\":0}")
echo "$R" | grep -qF '"code":-32602' && echo "$R" | grep -qF 'invalid lane_len_m (integer >= 1 required)'; check CHK-10h-lane-0 $?

# ── ⑨ 口径注三件在档（matrix-example.md 与 README.md 各 3 条逐字；D4）──
for f in matrix-example.md README.md; do
  grep -qF '不得与全规模数据混用（残余账 #11）' "$f"; check "CHK-11-notes-11-$f" $?
  grep -qF '趋势指示、非基准' "$f"; check "CHK-11-notes-12-$f" $?
  grep -qF '灰盒指标不作外推依据（残余账 #9）' "$f"; check "CHK-11-notes-9-$f" $?
done

# ── ⑩ 进程存活收尾（全程无 panic / 无击穿）──
kill -0 "$HOST_PID" 2>/dev/null; check CHK-12-process-alive $?
kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null
rm -rf "$TMP"

echo "== SUMMARY: PASS=$PASS FAIL=$FAIL =="
RC=0; [ "$FAIL" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC =="
exit $RC
