#!/usr/bin/env bash
# T023 示例矩阵生成（D5）：降规模口径（每方 100 共 200 单位/局——表 6-0 字面）
# 3 red 构型 × 3 blue 构型（单兵种 ×100/方）× games=100/格（seed_base=42）
# = 900 局，经 game.sample_outcomes BRP 路径（玩家可达面；端口 15707 独占）。
# 产出：matrix-example.md（判定行由本脚本 python 段计算生成，人工只解读不计算；
# 含口径注三件 D4 逐字、接敌可达性算式、胜率矩阵、逐格判定行、M0 T009 归档
# 同格 sanity 对照）——全量 REQ/RESP 原文落 matrix-example-run.log；终端只出摘要。
# 用法：bash matrix_example.sh
set -u
cd "$(dirname "$0")"
export LC_ALL=C
HOST_EXE="../../../target/release/host.exe"
LOG="matrix-example-run.log"
TMP="_matrix_tmp"
PORT=15707
# threads=1（方法缺省）：探针实测局内池在 200 单位规模下开销显著（单格 100 局：
# threads=1 → 2.83s；threads=12 → 17.14s；两档结果逐位一致）——示例档取缺省档
# （跨线程确定性由 sample_smoke CHK-06 覆盖；性能非本卡验收面）。
THREADS=1
GAMES=100
SEED_BASE=42
LANE_M=50
OUT_MD="matrix-example.md"
T009_JSONL="../t009/runs/matrix_per100/matrix_per100.jsonl"
KINDS=(shieldman militia swordsman)

rm -rf "$TMP"; mkdir -p "$TMP"
: > "$LOG"

exec 3>&1               # 终端摘要通道（fd3 = 原 stdout）
exec >> "$LOG" 2>&1     # 全量输出只落档（RESP 体量大，不回显终端）

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

echo "== T023 matrix example $(date '+%Y-%m-%d %H:%M:%S') =="
echo "== host exe: $HOST_EXE =="

"$HOST_EXE" --port $PORT > host-stdout.log 2> host-stderr.log &
HOST_PID=$!
echo "== host pid $HOST_PID (port $PORT) =="

READY=0
for i in $(seq 1 150); do
  if curl -s -m 2 -H 'Content-Type: application/json' \
       -d '{"jsonrpc":"2.0","id":0,"method":"rpc.discover","params":{}}' \
       "http://127.0.0.1:$PORT/" > /dev/null 2>&1; then READY=1; break; fi
  sleep 0.2
done
if [ "$READY" -ne 1 ]; then
  echo "FATAL: host not ready on port $PORT" >&3
  kill "$HOST_PID" 2>/dev/null
  exit 1
fi

# 9 格批跑（每格：单兵种 ×100/方，games=100，seed_base=42，lane 50m，ticks 缺省 1800）
START_NS=$(date +%s%N)
for i in "${!KINDS[@]}"; do
  for j in "${!KINDS[@]}"; do
    RK=${KINDS[$i]}; BK=${KINDS[$j]}
    R=$(brp game.sample_outcomes \
      "{\"red\":[{\"kind\":\"$RK\",\"count\":100}],\"blue\":[{\"kind\":\"$BK\",\"count\":100}],\"seed_base\":$SEED_BASE,\"games\":$GAMES,\"lane_len_m\":$LANE_M,\"threads\":$THREADS}")
    echo "$R" > "$TMP/cell_${i}_${j}.json"
  done
done
END_NS=$(date +%s%N)
WALL_MS=$(( (END_NS - START_NS) / 1000000 ))
echo "== 900 games wall_ms=$WALL_MS =="

# 判定行与文档生成（python 计算；完整性校验不过 → 非零退出）
python - "$OUT_MD" "$T009_JSONL" "$TMP" "$WALL_MS" "$THREADS" "$SEED_BASE" "$GAMES" "${KINDS[@]}" <<'PY'
import json, os, re, sys

out_md, t009_path, tmpdir = sys.argv[1], sys.argv[2], sys.argv[3]
wall_ms, threads = sys.argv[4], sys.argv[5]
seed_base, exp_games = int(sys.argv[6]), int(sys.argv[7])
kinds = sys.argv[8:]

def load_result(path):
    with open(path, encoding="utf-8") as f:
        r = json.load(f)
    if "error" in r:
        raise SystemExit("FATAL: error response in %s: %r" % (path, r["error"]))
    return r["result"]

def pct(pp):
    return "%d.%02d%%" % (pp // 100, pp % 100)

cells = {}
tot = {"r": 0, "b": 0, "d": 0, "n": 0}
for i, rk in enumerate(kinds):
    for j, bk in enumerate(kinds):
        res = load_result(os.path.join(tmpdir, "cell_%d_%d.json" % (i, j)))
        n, outs = res["games"], res["outcomes"]
        if n != exp_games or len(outs) != exp_games:
            raise SystemExit("FATAL: cell %s vs %s games=%d outcomes=%d" % (rk, bk, n, len(outs)))
        if res["red_wins"] + res["blue_wins"] + res["draws"] != n:
            raise SystemExit("FATAL: cell %s vs %s counter sum != games" % (rk, bk))
        if res["win_rate_red_pp"] != res["red_wins"] * 10000 // n:
            raise SystemExit("FATAL: cell %s vs %s win_rate formula broken" % (rk, bk))
        for k, o in enumerate(outs):
            if o["seed"] != (seed_base + k) % (1 << 64):
                raise SystemExit("FATAL: cell %s vs %s seed[%d]=%d" % (rk, bk, k, o["seed"]))
            if not re.fullmatch(r"0x[0-9a-f]{16}", o["final_hash"]):
                raise SystemExit("FATAL: cell %s vs %s final_hash %r" % (rk, bk, o["final_hash"]))
        cells[(rk, bk)] = res
        tot["r"] += res["red_wins"]; tot["b"] += res["blue_wins"]
        tot["d"] += res["draws"]; tot["n"] += n

# T009 归档 sanity 对照（只读；同口径 per100 层：单兵种 ×100/方、lane 50m、cap 1800、100 局/格）
want = set(cells.keys())
t9 = {}
if os.path.exists(t009_path):
    with open(t009_path, encoding="utf-8") as f:
        for line in f:
            row = json.loads(line)
            key = (row["red"], row["blue"])
            if row["per_side"] == 100 and key in want:
                t9.setdefault(key, []).append(row["winner"])
    for key in want:
        if len(t9.get(key, [])) != 100:
            raise SystemExit("FATAL: t009 cell %r rows=%d" % (key, len(t9.get(key, []))))

lines = []
lines.append("# T023 批量采样示例矩阵（game.sample_outcomes，900 局）")
lines.append("")
lines.append("- 生成：`matrix_example.sh`（本档判定行由脚本计算生成，人工只解读不计算；勿手改）。")
lines.append("- 基线：隔离树 t023-a（基线 commit bb3f4ba）+ `game.sample_outcomes`（本卡新增，M5-06 席位 8）。")
lines.append("- 宿主：`target/release/host.exe`（release 构建，BRP 端口 15707 回环独占）；批跑耗时 wall_ms=%s（900 局，threads=%s）。" % (wall_ms, threads))
lines.append("- 参数：red/blue 各为单兵种 ×100（每方 100 共 200 单位/局——表 6-0 降规模对局字面）；")
lines.append("  `lane_len_m=50`（沿 M0 T009 矩阵缩比道口径）；`max_ticks` 缺省（1800=`TICK_CAP_REDUCED`）；")
lines.append("  `games=%d`/格；`seed_base=%d`（每格种子 %d..%d）；`threads=%s`。"
             % (exp_games, seed_base, seed_base, seed_base + exp_games - 1, threads))
lines.append("")
lines.append("## 口径注（D4 逐字，残余账随卡）")
lines.append("")
lines.append("① 降规模口径：每方 100 共 200 单位/局、单局 ≤1800 ticks（表 6-0）——不得与全规模数据混用（残余账 #11）")
lines.append("")
lines.append("② 样本量注：本档每格 100 局（<400 场）——趋势指示、非基准（判据 ±10pp / ≥400 场/周，R5 功效注——残余账 #12）")
lines.append("")
lines.append("③ 灰盒指标不作外推依据（残余账 #9）——本卡不适用：统计面无渲染指标，如实标注")
lines.append("")
lines.append("## 接敌可达性算式（附录 B.2 ②；仅核对，以算式为准）")
lines.append("")
lines.append("最慢闭合对（盾兵×盾兵）：(50 − 0.5 − 0.5 − (0.5+0.5+0.2)) ÷ (0.05+0.05)")
lines.append("= (49.0 − 1.2) ÷ 0.10 = 478 ≤ 1800 tick ✓ —— 全部 9 格任意 seed 接敌可达")
lines.append("（半径/移速源 sim/src/units.rs；队头布阵式 sim/src/world.rs:601-622；")
lines.append("单兵种阵列洗牌不改变队头兵种——任何 seed 队头半径恒定）。")
lines.append("")
lines.append("## 胜率矩阵（红方视角；判定行由脚本从响应计算）")
lines.append("")
lines.append("格子格式：`win_rate_red_pp`（红胜/蓝胜/平）——pp ÷ 100 = 红方胜率百分数（整数换算）。")
lines.append("")
lines.append("| red \\ blue | " + " | ".join(kinds) + " | 行合计 red_wins |")
lines.append("|---|" + "---|" * (len(kinds) + 1))
for rk in kinds:
    row_cells = []
    row_r = 0
    for bk in kinds:
        res = cells[(rk, bk)]
        row_r += res["red_wins"]
        row_cells.append("%d（%d/%d/%d）" % (res["win_rate_red_pp"], res["red_wins"], res["blue_wins"], res["draws"]))
    lines.append("| " + rk + " | " + " | ".join(row_cells) + " | " + str(row_r) + " |")
lines.append("")
lines.append("每格 n=100（胜率百分数 = pp/100，如 4700 → 47.00%）。")
lines.append("")
lines.append("## 逐格判定行（脚本生成）")
lines.append("")
for rk in kinds:
    for bk in kinds:
        res = cells[(rk, bk)]
        lines.append("- cell red=%s blue=%s: games=%d red_wins=%d blue_wins=%d draws=%d win_rate_red_pp=%d (%s)"
                     % (rk, bk, res["games"], res["red_wins"], res["blue_wins"], res["draws"], res["win_rate_red_pp"], pct(res["win_rate_red_pp"])))
lines.append("")
lines.append("## 全表汇总（脚本生成）")
lines.append("")
lines.append("- 合计 games=%d red_wins=%d blue_wins=%d draws=%d（混合 9 配置——非单一命题样本，仅作量级索引）。"
             % (tot["n"], tot["r"], tot["b"], tot["d"]))
lines.append("")
if t9:
    lines.append("## M0 T009 归档 sanity 对照（只读引用；种子域不同——非断言）")
    lines.append("")
    lines.append("T009 `matrix_per100` 层同口径（单兵种 ×100/方、lane 50m、cap 1800、100 局/格）；")
    lines.append("种子域 1_000_000+cell*100+k ≠ 本档 %d..%d——两批独立 100 局抽样，Δ 属抽样误差范畴"
                 % (seed_base, seed_base + exp_games - 1))
    lines.append("（±10pp 判据口径），仅作量级 sanity；大幅背离即上报复核。")
    lines.append("")
    lines.append("| cell（red vs blue） | T023 pp（n=100） | T009 pp（n=100） | Δpp |")
    lines.append("|---|---|---|---|")
    max_abs = 0
    for rk in kinds:
        for bk in kinds:
            t9r = t9[(rk, bk)].count("red")
            t9pp = t9r * 10000 // 100
            d = cells[(rk, bk)]["win_rate_red_pp"] - t9pp
            max_abs = max(max_abs, abs(d))
            lines.append("| %s vs %s | %d | %d | %+d |" % (rk, bk, cells[(rk, bk)]["win_rate_red_pp"], t9pp, d))
    lines.append("")
    lines.append("- max |Δpp| = %d（对照批 = T009 归档 3600 局 JSONL 只读引用：`../t009/runs/matrix_per100/matrix_per100.jsonl`）。" % max_abs)
    lines.append("")
lines.append("## 溯源")
lines.append("")
lines.append("- 默认值口径：`host/src/rpc.rs`（`DEFAULT_LANE_LEN_M` / 缺省 `TICK_CAP_REDUCED` / `MAX_SAMPLE_GAMES=1000`）。")
lines.append("- 终局四元组与哈希口径：`sim/src/world.rs`（`BattleOutcome` / `run_battle_with` / `Winner::label`）。")
lines.append("- M0 锚（本档不涉及，冒烟档 `sample_smoke.sh` CHK-03 用）：T004 黄金 `0x958c5938c8682529`（sim/src/world.rs 单测常量）。")
lines.append("")

with open(out_md, "w", encoding="utf-8") as f:
    f.write("\n".join(lines) + "\n")
print("WROTE %s" % out_md)
print("TOTAL games=%d r/b/d=%d/%d/%d" % (tot["n"], tot["r"], tot["b"], tot["d"]))
if t9:
    print("T009 sanity done (cells=%d)" % len(t9))
PY
GEN_RC=$?
if [ "$GEN_RC" -ne 0 ]; then
  echo "FATAL: matrix generation failed rc=$GEN_RC" >&3
  kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null
  exit 1
fi

kill -0 "$HOST_PID" 2>/dev/null
ALIVE=$?
kill "$HOST_PID" 2>/dev/null; sleep 1; kill -9 "$HOST_PID" 2>/dev/null
rm -rf "$TMP"

echo "== SUMMARY: cells=9 games=900 wall_ms=$WALL_MS host_alive_before_kill=$([ "$ALIVE" -eq 0 ] && echo yes || echo no) ==" >&3
RC=0
[ "$ALIVE" -eq 0 ] || RC=1
echo "== SCRIPT_EXIT=$RC ==" >&3
exit $RC
