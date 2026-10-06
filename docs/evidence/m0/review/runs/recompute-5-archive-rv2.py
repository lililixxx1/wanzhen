# Supplementary acceptance-5 recompute: independent implementation over the ARCHIVED t010 frames.csv
# (third implementation vs summarize.py and G1's recompute) to check the archive-internal consistency
# of the 333.15 / 202.48 claim. Formula = pre-registered (avg = N/(sum/1e9); 1% low = 1e9/mean(slowest
# k1), k1 = (N+99)//100). Fresh-sample recompute is the primary body (render-recompute-rv2.txt).
import csv, glob
paths = sorted(glob.glob("docs/evidence/t010/runs/*/out/t10000/frames.csv"))
print(f"archived t10000 frames.csv candidates: {paths}")
for p in paths:
    with open(p, newline="", encoding="utf-8") as f:
        r = csv.reader(f)
        header = next(r)
        deltas = [int(row[1]) for row in r]
    n = len(deltas)
    s = sum(deltas)
    avg = n / (s / 1e9)
    k1 = max(1, (n + 99) // 100)
    slowest = sorted(deltas)[-k1:]
    low1 = 1e9 / (sum(slowest) / k1)
    print(f"{p}: N={n} sum_s={s/1e9:.3f} avg_fps={avg:.2f} (t010/summary.md:14 ref 333.15 {'MATCH' if f'{avg:.2f}' == '333.15' else 'CHECK'})"
          f" 1pct_low={low1:.2f} (ref 202.48 {'MATCH' if f'{low1:.2f}' == '202.48' else 'CHECK'})"
          f" judge(avg>=60 and low>=45)={'PASS' if (avg >= 60 and low1 >= 45) else 'FAIL'}")
