# T012 reviewer independent throughput recompute (rv2). Formula source = docs/evidence/t009/summarize.py
# (throughput_section): gph = games * 3600 / wall_s_median; judge gph@12t >= 10000 (R2 threshold).
# Median semantics here = middle of 3 sorted repeats (matches arena wall_s_median field; cross-checked).
import json, statistics

P = "docs/evidence/m0/review/runs/throughput_t12-rv2/throughput_t12-rv2.json"
REF_GPH = 1099638.6  # docs/evidence/t009/summary.md:180

with open(P, encoding="utf-8") as f:
    j = json.loads(f.read())
walls = j["wall_s"]
med = statistics.median(walls)
gph = j["games"] * 3600.0 / med
print(f"wall_s repeats      = {walls}")
print(f"median(wall_s) own  = {med:.7f}   json wall_s_median = {j['wall_s_median']:.7f}")
print(f"gph own = games*3600/median = {j['games']}*3600/{med:.7f} = {gph:.1f}")
print(f"gph json field             = {j['games_per_hour']:.1f}")
print(f"ref (t009/summary.md:180)  = {REF_GPH}")
print(f"drift vs ref               = {(gph-REF_GPH)/REF_GPH*100:+.2f}%  band25 = {'IN' if abs((gph-REF_GPH)/REF_GPH*100)<=25 else 'OUT'}")
print(f"per_game_ms own            = {med*1000.0/j['games']:.3f}  json = {j['per_game_ms']:.3f}")
print(f"final_hash_xor fresh       = {j['final_hash_xor']}   archive t009 = 0xd1b28b373f7791e3 (cross-check)")
print(f"JUDGE gph>=10000           : {'PASS' if gph >= 10000.0 else 'FAIL'}")
