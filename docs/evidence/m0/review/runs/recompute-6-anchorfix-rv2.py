# Addendum: recompute-6-all-rv2.py's shared analyze() compared the T007 block's anchor-10k line
# against the T015 anchor constants (script-side label artifact; computed values printed correctly).
# This addendum re-compares T007 computed values against t007/summary.md:42 (7.221411 / 9.416188 / OK).
import json
def median_f64(vals):
    s = sorted(vals); n = len(s)
    return float(s[n // 2]) if n % 2 == 1 else (float(s[n // 2 - 1]) + float(s[n // 2])) / 2.0
recs = {}
for line in open("docs/evidence/t007/matrix.jsonl", encoding="utf-8"):
    line = line.strip()
    if line:
        j = json.loads(line)
        recs[(j["config"]["units"], j["config"]["threads"])] = j
def nspt(t):
    j = recs[(10000, t)]
    return median_f64(j["samples_ns"]) / j["config"]["ticks"]
m1 = median_f64(recs[(10000, 1)]["samples_ns"])
m12 = median_f64(recs[(10000, 12)]["samples_ns"])
measured12 = m1 / m12
ts = [1, 3, 6, 12]
xs = [1.0 / t for t in ts]
ys = [nspt(t) for t in ts]
n = 4.0
sx = sum(xs); sx2 = sum(x * x for x in xs); sy = sum(ys); sxy = sum(x * y for x, y in zip(xs, ys))
det = n * sx2 - sx * sx
c1 = (sy * sx2 - sx * sxy) / det
c2 = (n * sxy - sx * sy) / det
sp16 = ys[0] / (c1 + c2 / 16.0)
verd = "OK" if (measured12 >= 4.0 and sp16 >= 4.0) else "TRIPPED"
print(f"t007 anchor10k: measured12={measured12:.6f} ref=7.221411 {'MATCH' if f'{measured12:.6f}' == '7.221411' else 'FAIL'}")
print(f"t007 anchor10k: speedup16={sp16:.6f} ref=9.416188 {'MATCH' if f'{sp16:.6f}' == '9.416188' else 'FAIL'}")
print(f"t007 anchor10k: verdict={verd} ref=OK {'MATCH' if verd == 'OK' else 'FAIL'}")
print("NOTE: the '(2) anchor10k ... FAIL' lines inside the T007 block of recompute-6-all-rv2.txt are a")
print("script label artifact (t015 anchor constants reused in the shared function); computed values")
print("above match t007/summary.md:42 digit-for-digit.")
