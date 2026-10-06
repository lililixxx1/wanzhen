# T012 reviewer independent render recompute (rv2). Formula source = docs/evidence/t010/summarize.py
# header pre-registered lines: avg_fps = N/(sum(delta_ns)/1e9); 1% low = 1e9/mean(slowest k1 frames);
# k1 = max(1, (N+99)//100). Judgment = t10000 avg_fps >= 60 AND 1% low >= 45. Drift band +-25%.
import csv, json

SAMPLES = [
    ("s1(render-t10000-rv2)", "docs/evidence/m0/review/runs/render-t10000-rv2/t10000"),
    ("s2(-s2-rv2)", "docs/evidence/m0/review/runs/render-t10000-s2-rv2/t10000"),
    ("s3(-s3-rv2)", "docs/evidence/m0/review/runs/render-t10000-s3-rv2/t10000"),
]
REF_AVG, REF_LOW = 333.15, 202.48
VSYNC_EVIDENCE_LT_NS = 15_865_000

print("sample  N  sum_s  avg_fps  k1  1pct_low  max_delta_ms  min_delta_ms  non_vsync_lt15865ms  J_avg60  J_low45  drift_avg  drift_low  win_res_actual")
for name, base in SAMPLES:
    with open(base + "/frames.csv", newline="", encoding="utf-8") as f:
        r = csv.reader(f)
        header = next(r)
        assert [c.strip() for c in header] == ["idx", "delta_ns"], header
        deltas = [int(row[1]) for row in r]
    with open(base + "/meta.json", encoding="utf-8") as f:
        meta = json.load(f)
    n = len(deltas)
    s = sum(deltas)
    avg = n / (s / 1e9)
    k1 = max(1, (n + 99) // 100)
    slowest = sorted(deltas)[-k1:]
    low1 = 1e9 / (sum(slowest) / k1)
    d_avg = (avg - REF_AVG) / REF_AVG * 100.0
    d_low = (low1 - REF_LOW) / REF_LOW * 100.0
    j_avg = "PASS" if avg >= 60.0 else "FAIL"
    j_low = "PASS" if low1 >= 45.0 else "FAIL"
    nonv = sum(1 for d in deltas if d < VSYNC_EVIDENCE_LT_NS)
    print(f"{name}  {n}  {s/1e9:.3f}  {avg:.4f}  {k1}  {low1:.4f}  {max(deltas)/1e6:.2f}  {min(deltas)/1e6:.3f}  {nonv}  {j_avg}  {j_low}  {d_avg:+.2f}%  {d_low:+.2f}%  {meta['window_resolution_actual']} (meta captured_frames={meta['captured_frames']})")
print("REF: t010/summary.md:14/:31  avg_fps=333.15  1pct_low=202.48  verdict PASS (drift band +-25%)")
print("PRE-REGISTERED EXPECTATION (dispatch): t10000 judgment line produced AND PASS -> deviation disclosed as-is if not.")
