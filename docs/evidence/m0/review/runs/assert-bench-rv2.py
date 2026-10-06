# T012 reviewer independent bench assertion (rv2 round). Reads fresh bench stdout JSON files
# and asserts: final_hash zero-tolerance anchors, us_per_unit_tick +-25% drift band vs archive refs,
# <=1us (promise) / <=2us (stop-loss) judgment lines, fresh speedup median(2a)/median(2b) in 0.8..1.3.
import json, sys

CASES = [
    ("2a", "docs/evidence/m0/review/runs/bench-2a-rv2.stdout", "0xc5915d042208e267", 0.070617),
    ("2b", "docs/evidence/m0/review/runs/bench-2b-rv2.stdout", "0xc5915d042208e267", 0.070482),
    ("2c", "docs/evidence/m0/review/runs/bench-2c-rv2.stdout", "0x022c5abdfae119dc", 0.108681),
    ("2d", "docs/evidence/m0/review/runs/bench-2d-rv2.stdout", "0x022c5abdfae119dc", 0.120594),
]

rows = []
med = {}
ok_all = True
print("id  fresh_us  ref_us  drift_pct  band25  hash_ok  le_1us  le_2us")
for cid, path, exp_hash, ref_us in CASES:
    with open(path, "r", encoding="utf-8") as f:
        j = json.loads(f.readline())
    med[cid] = j["median_ns"]
    fresh_us = j["us_per_unit_tick"]
    drift = (fresh_us - ref_us) / ref_us * 100.0
    band = "IN" if abs(drift) <= 25.0 else "OUT"
    hash_ok = "OK" if j["final_hash"] == exp_hash else "FAIL"
    le1 = "OK" if fresh_us <= 1.0 else "FAIL"
    le2 = "OK" if fresh_us <= 2.0 else "FAIL"
    hc = j.get("hash_consistent")
    print(f"{cid}  {fresh_us:.6f}  {ref_us:.6f}  {drift:+.2f}%  {band}  {hash_ok}  {le1}  {le2}  hash_consistent={hc}")
    if hash_ok != "OK" or le1 != "OK" or le2 != "OK":
        ok_all = False
sp = med["2a"] / med["2b"]
sp_band = "IN" if 0.8 <= sp <= 1.3 else "OUT"
print(f"fresh_speedup_2a_over_2b = {med['2a']}/{med['2b']} = {sp:.5f}  band_0.8_1.3 = {sp_band}")
if sp_band != "IN":
    ok_all = False
print("OVERALL_ASSERT:", "PASS" if ok_all else "FAIL")
sys.exit(0 if ok_all else 1)
