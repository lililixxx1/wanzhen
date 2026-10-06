# T012 reviewer independent recompute (rv2) of M0 judgments (1)(2)(2-annex)(3)(6-arithmetic)(6.1).
# Semantics read from source before writing: median = bench.rs:923 (odd->middle, even->mean of two);
# Amdahl y=c1+c2*(1/T) 2x2 normal equations; speedup16 = t1_ns_per_tick/(c1+c2/16) (bench.rs:1249);
# quad y=a*N+b*N^2 det=s2*s4-s3*s3 (bench.rs:887-920); annex speedup16 source = 50k tier (bench.rs:100);
# 6.1 req = units*us/(budget_ms*1000), labels <=2/=<=4/else (bench.rs:1449-1458).
import json, statistics

def median_f64(vals):
    s = sorted(vals); n = len(s)
    return float(s[n // 2]) if n % 2 == 1 else (float(s[n // 2 - 1]) + float(s[n // 2])) / 2.0

def fit_amdahl(pts):
    xs = [1.0 / t for t, _ in pts]
    ys = [y for _, y in pts]
    n = float(len(xs))
    sx = sum(xs); sx2 = sum(x * x for x in xs); sy = sum(ys); sxy = sum(x * y for x, y in zip(xs, ys))
    det = n * sx2 - sx * sx
    c1 = (sy * sx2 - sx * sxy) / det
    c2 = (n * sxy - sx * sy) / det
    ybar = sy / n
    ss_tot = sum((y - ybar) ** 2 for y in ys)
    ress = [abs(y - (c1 + c2 * x)) for x, y in zip(xs, ys)]
    ss_res = sum(r * r for r in ress)
    r2 = (1.0 - ss_res / ss_tot) if ss_tot > 0 else None
    return c1, c2, r2, max(ress)

def fit_quad(pts):
    ns = [x for x, _ in pts]; ys = [y for _, y in pts]
    n = float(len(ns))
    s2 = sum(x * x for x in ns); s3 = sum(x ** 3 for x in ns); s4 = sum(x ** 4 for x in ns)
    sy1 = sum(x * y for x, y in zip(ns, ys)); sy2 = sum(x * x * y for x, y in zip(ns, ys))
    det = s2 * s4 - s3 * s3
    a = (sy1 * s4 - s3 * sy2) / det
    b = (s2 * sy2 - s3 * sy1) / det
    ybar = sum(ys) / n
    ss_tot = sum((y - ybar) ** 2 for y in ys)
    ress = [abs(y - (a * x + b * x * x)) for x, y in zip(ns, ys)]
    ss_res = sum(r * r for r in ress)
    r2 = (1.0 - ss_res / ss_tot) if ss_tot > 0 else None
    return a, b, r2, max(ress)

def load_matrix(path):
    recs = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            j = json.loads(line)
            recs[(j["config"]["units"], j["config"]["threads"])] = j
    return recs

def nspt_of(j):
    return median_f64(j["samples_ns"]) / j["config"]["ticks"]

TIERS = [1000, 5000, 10000, 50000]
THREADS = [1, 3, 6, 12]

def analyze(path, tag, ref_appendix, ref_tab2, ref_amdahl, ref_annex):
    print(f"===== {tag} ({path}) =====")
    recs = load_matrix(path)
    print("--- (1) 16-config median / us_per_unit_tick recompute vs archive appendix A ---")
    ok_all = True
    for (u, t), ref in sorted(ref_appendix.items()):
        j = recs[(u, t)]
        med = median_f64(j["samples_ns"])
        nspt = med / j["config"]["ticks"]
        us = nspt / u / 1000.0
        ok_med = (int(med) == ref["median_ns"])
        ok_us = (f"{us:.6f}" == ref["us"])
        ok_hash = (j["final_hash"] == ref["hash"])
        if not (ok_med and ok_us and ok_hash):
            ok_all = False
        print(f"u={u} t={t}: median {int(med)} vs {ref['median_ns']} {'MATCH' if ok_med else 'FAIL'}"
              f" | us {us:.6f} vs {ref['us']} {'MATCH' if ok_us else 'FAIL'}"
              f" | hash {j['final_hash']} vs {ref['hash']} {'MATCH' if ok_hash else 'FAIL'}"
              f" | median_field_consistent={int(med) == j['median_ns']}")
    print(f"(1) OVERALL: {'PASS' if ok_all else 'FAIL'}")

    print("--- (2) speedups / Amdahl OLS / speedup16 vs archive tables ---")
    for u in TIERS:
        med1 = median_f64(recs[(u, 1)]["samples_ns"])
        nspt1 = nspt_of(recs[(u, 1)])
        cells = [f"{nspt_of(recs[(u, t)]):.1f}" for t in THREADS]
        ratios = [f"{med1 / median_f64(recs[(u, t)]['samples_ns']):.6f}" for t in THREADS[1:]]
        pts = [(float(t), nspt_of(recs[(u, t)])) for t in THREADS]
        c1, c2, r2, mres = fit_amdahl(pts)
        sp16 = nspt1 / (c1 + c2 / 16.0)
        r2s = f"{r2:.6f}" if r2 is not None else "null"
        mine = f"| {u} | " + " | ".join(cells) + " | " + " | ".join(ratios) + f" | {sp16:.6f} |"
        print(f"row mine: {mine}")
        print(f"row ref : {ref_tab2[u]}")
        print(f"row_match={'MATCH' if mine == ref_tab2[u] else 'FAIL'}")
        mrow = f"| {u} | {c1:.3f} | {c2:.3f} | {r2s} | {mres:.3f} | {sp16:.6f} |"
        print(f"amd mine: {mrow}")
        print(f"amd ref : {ref_amdahl[u]}")
        print(f"amd_match={'MATCH' if mrow == ref_amdahl[u] else 'FAIL'}")
    m1 = median_f64(recs[(10000, 1)]["samples_ns"])
    m12 = median_f64(recs[(10000, 12)]["samples_ns"])
    measured12 = m1 / m12
    pts10 = [(float(t), nspt_of(recs[(10000, t)])) for t in THREADS]
    c1, c2, _, _ = fit_amdahl(pts10)
    sp16 = nspt_of(recs[(10000, 1)]) / (c1 + c2 / 16.0)
    verd = "TRIPPED" if not (measured12 >= 4.0 and sp16 >= 4.0) else "OK"
    print(f"(2) anchor10k: measured12={measured12:.6f} vs ref 1.001913 {'MATCH' if f'{measured12:.6f}' == '1.001913' else 'FAIL'}"
          f" | speedup16={sp16:.6f} vs ref 1.112480 {'MATCH' if f'{sp16:.6f}' == '1.112480' else 'FAIL'}"
          f" | verdict={verd} vs ref TRIPPED {'MATCH' if verd == 'TRIPPED' else 'FAIL'}")

    print("--- (2-annex) quadratic C(N)=a*N+b*N^2 -> C(100000)/speedup16(50k) ---")
    qpts = [(float(u), nspt_of(recs[(u, 1)])) for u in TIERS]
    a, b, r2, mres = fit_quad(qpts)
    c100k = a * 100000.0 + b * 100000.0 ** 2
    pts50 = [(float(t), nspt_of(recs[(50000, t)])) for t in THREADS]
    c1_50, c2_50, _, _ = fit_amdahl(pts50)
    sp16_50 = nspt_of(recs[(50000, 1)]) / (c1_50 + c2_50 / 16.0)
    pred_ms = c100k / sp16_50 / 1e6
    r2s = f"{r2:.6f}" if r2 is not None else "null"
    print(f"fit mine: a={a:.9f} b={b:.9f} R2={r2s} max_res={mres:.3f}")
    print(f"fit ref : {ref_annex['fit']}")
    print(f"C(100000) mine={c100k:.1f} ref={ref_annex['c100k']} {'MATCH' if f'{c100k:.1f}' == ref_annex['c100k'] else 'FAIL'}")
    print(f"speedup16(50k) mine={sp16_50:.6f} ref={ref_annex['sp16']} {'MATCH' if f'{sp16_50:.6f}' == ref_annex['sp16'] else 'FAIL'}")
    verd = "[保留]" if pred_ms <= 22.0 else "[超界：极限十万目标不保留]"
    print(f"predicted mine={pred_ms:.6f} ms ref={ref_annex['pred_ms']} ms {'MATCH' if f'{pred_ms:.6f}' == ref_annex['pred_ms'] else 'FAIL'} verdict={verd} ref={ref_annex['verdict']}")
    req = c100k / (22.0 * 1e6)
    print(f"needed@22ms mine={req:.6f} ref={ref_annex['req']} {'MATCH' if f'{req:.6f}' == ref_annex['req'] else 'FAIL'}")
    if "sp16_alt" in ref_annex:
        alt = c100k / ref_annex["sp16_alt"] / 1e6
        print(f"[DISPATCH-ANCHOR-VARIANT] with dispatch-stated speedup16={ref_annex['sp16_alt']} -> predicted={alt:.6f} ms"
              f" (archive {ref_annex['pred_ms']} ms used 50k-tier speedup16={ref_annex['sp16']}; bench.rs:100)")
    return ok_all

REF_T015_APPX = {}
for u, t, med, us, h in [
    (1000, 1, 36390400, "0.060651", "0x88b33d3124562844"), (1000, 3, 63733700, "0.106223", "0x88b33d3124562844"),
    (1000, 6, 77448300, "0.129080", "0x88b33d3124562844"), (1000, 12, 91970400, "0.153284", "0x88b33d3124562844"),
    (5000, 1, 108293300, "0.072196", "0x8c284cbd5c81a033"), (5000, 3, 103267000, "0.068845", "0x8c284cbd5c81a033"),
    (5000, 6, 112471200, "0.074981", "0x8c284cbd5c81a033"), (5000, 12, 114038000, "0.076025", "0x8c284cbd5c81a033"),
    (10000, 1, 211850600, "0.070617", "0xc5915d042208e267"), (10000, 3, 185545500, "0.061849", "0xc5915d042208e267"),
    (10000, 6, 177593800, "0.059198", "0xc5915d042208e267"), (10000, 12, 211446100, "0.070482", "0xc5915d042208e267"),
    (50000, 1, 163021200, "0.108681", "0x022c5abdfae119dc"), (50000, 3, 193672400, "0.129115", "0x022c5abdfae119dc"),
    (50000, 6, 175418500, "0.116946", "0x022c5abdfae119dc"), (50000, 12, 180891300, "0.120594", "0x022c5abdfae119dc"),
]:
    REF_T015_APPX[(u, t)] = {"median_ns": med, "us": us, "hash": h}

REF_T015_TAB2 = {
    1000: "| 1000 | 60650.7 | 106222.8 | 129080.5 | 153284.0 | 0.570976 | 0.469867 | 0.395675 | 0.424733 |",
    5000: "| 5000 | 360977.7 | 344223.3 | 374904.0 | 380126.7 | 1.048673 | 0.962854 | 0.949625 | 0.974884 |",
    10000: "| 10000 | 706168.7 | 618485.0 | 591979.3 | 704820.3 | 1.141772 | 1.192894 | 1.001913 | 1.112480 |",
    50000: "| 50000 | 5434040.0 | 6455746.7 | 5847283.3 | 6029710.0 | 0.841737 | 0.929327 | 0.901211 | 0.881714 |",
}
REF_T015_AMD = {
    1000: "| 1000 | 148513.470 | -91462.662 | 0.930689 | 12392.418 | 0.424733 |",
    5000: "| 5000 | 371256.137 | -15658.662 | 0.164286 | 21813.250 | 0.974884 |",
    10000: "| 10000 | 630908.639 | 61780.281 | 0.190422 | 68763.338 | 1.112480 |",
    50000: "| 50000 | 6204547.079 | -664047.358 | 0.424974 | 472548.707 | 0.881714 |",
}
REF_T015_ANNEX = {
    "fit": "a=62.759210207 b=0.000918316 R2=0.999960 max_res=24223.712",
    "c100k": "15459082.3", "sp16": "0.881714", "pred_ms": "17.532997", "verdict": "[保留]", "req": "0.702686",
}
ok15 = analyze("docs/evidence/t015/matrix.jsonl", "T015 matrix (post-optimization)",
               REF_T015_APPX, REF_T015_TAB2, REF_T015_AMD, REF_T015_ANNEX)

REF_T007_APPX = {}
for u, t, med, us, h in [
    (1000, 1, 2407283100, "4.012138", "0x88b33d3124562844"), (1000, 3, 855234400, "1.425391", "0x88b33d3124562844"),
    (1000, 6, 596894500, "0.994824", "0x88b33d3124562844"), (1000, 12, 514074500, "0.856791", "0x88b33d3124562844"),
    (5000, 1, 29770192700, "19.846795", "0x8c284cbd5c81a033"), (5000, 3, 10762327300, "7.174885", "0x8c284cbd5c81a033"),
    (5000, 6, 6475209400, "4.316806", "0x8c284cbd5c81a033"), (5000, 12, 4268875000, "2.845917", "0x8c284cbd5c81a033"),
    (10000, 1, 123030308400, "41.010103", "0xc5915d042208e267"), (10000, 3, 43533564600, "14.511188", "0xc5915d042208e267"),
    (10000, 6, 24500439100, "8.166813", "0xc5915d042208e267"), (10000, 12, 17036880000, "5.678960", "0xc5915d042208e267"),
    (50000, 1, 300938942600, "200.625962", "0x022c5abdfae119dc"), (50000, 3, 106095722700, "70.730482", "0x022c5abdfae119dc"),
    (50000, 6, 57852565800, "38.568377", "0x022c5abdfae119dc"), (50000, 12, 37985549800, "25.323700", "0x022c5abdfae119dc"),
]:
    REF_T007_APPX[(u, t)] = {"median_ns": med, "us": us, "hash": h}
REF_T007_TAB2 = {
    1000: "| 1000 | 4012138.5 | 1425390.7 | 994824.2 | 856790.8 | 2.814764 | 4.033013 | 4.682751 | 6.256547 |",
    5000: "| 5000 | 99233975.7 | 35874424.3 | 21584031.3 | 14229583.3 | 2.766148 | 4.597564 | 6.973779 | 8.479246 |",
    10000: "| 10000 | 410101028.0 | 145111882.0 | 81668130.3 | 56789600.0 | 2.826102 | 5.021555 | 7.221411 | 9.416188 |",
    50000: "| 50000 | 10031298086.7 | 3536524090.0 | 1928418860.0 | 1266184993.3 | 2.836485 | 5.201825 | 7.922459 | 10.249990 |",
}
REF_T007_AMD = {
    1000: "| 1000 | 419830.016 | 3543046.803 | 0.991852 | 175454.950 | 6.256547 |",
    5000: "| 5000 | 5885533.260 | 93082030.502 | 0.999660 | 1038452.427 | 8.479246 |",
    10000: "| 10000 | 19203099.954 | 389594678.221 | 0.999370 | 5120276.861 | 9.416188 |",
    50000: "| 50000 | 376424979.253 | 9635827018.729 | 0.999720 | 86774429.186 | 10.249990 |",
}
REF_T007_ANNEX = {
    "fit": "a=773.554196350 b=3.997072319 R2=1.000000 max_res=4560603.291",
    "c100k": "40048078610.0", "sp16": "10.249990", "pred_ms": "3907.133373",
    "verdict": "[超界：极限十万目标不保留]", "req": "1820.367210", "sp16_alt": 9.416188,
}
ok07 = analyze("docs/evidence/t007/matrix.jsonl", "T007 matrix (pre-optimization baseline)",
               REF_T007_APPX, REF_T007_TAB2, REF_T007_AMD, REF_T007_ANNEX)

print("===== (3) throughput recompute from t009 raw JSONs =====")
walls = {}
for t in [1, 3, 6, 12]:
    with open(f"docs/evidence/t009/runs/throughput_t{t}/throughput_t{t}.json", encoding="utf-8") as f:
        j = json.loads(f.read())
    walls[t] = j
    med = statistics.median(j["wall_s"])
    gph = j["games"] * 3600.0 / med
    pg = med * 1000.0 / j["games"]
    print(f"t={t}: wall_s={j['wall_s']} median_own={med:.7f} (field {j['wall_s_median']:.7f})"
          f" gph_own={gph:.1f} (field {j['games_per_hour']:.1f}) per_game_ms_own={pg:.3f} (field {j['per_game_ms']:.3f})")
pts = [(float(t), statistics.median(walls[t]["wall_s"])) for t in [1, 3, 6, 12]]
n = 4.0
sx = sum(1.0 / t for t, _ in pts); sx2 = sum((1.0 / t) ** 2 for t, _ in pts)
sy = sum(y for _, y in pts); sxy = sum((1.0 / t) * y for t, y in pts)
det = n * sx2 - sx * sx
c1 = (sy * sx2 - sx * sxy) / det; c2 = (n * sxy - sx * sy) / det
e16 = c1 + c2 / 16.0
g16 = walls[12]["games"] * 3600.0 / e16
print(f"OLS mine: c1={c1:.4f}s c2={c2:.4f}s -> elapsed(16)={e16:.4f}s -> gph16={g16:.1f}")
print("OLS ref : c1=0.6632s c2=10.3642s -> elapsed(16)=1.3110s -> gph16=1405967.6 (t009/summary.md:181)")
g12 = walls[12]["games"] * 3600.0 / statistics.median(walls[12]["wall_s"])
print(f"gph@12t mine={g12:.1f} ref=1099638.6 (t009/summary.md:180)"
      f" {'MATCH' if f'{g12:.1f}' == '1099638.6' else 'FAIL'} judge>=10000: {'PASS' if g12 >= 10000 else 'FAIL'}")

print("===== 6.1 conversion table recompute (t015 side; full-precision us) =====")
recs15 = load_matrix("docs/evidence/t015/matrix.jsonl")
ref61 = {1000: ("0.007581", "0.002757"), 5000: ("0.045122", "0.016408"),
         10000: ("0.088271", "0.032099"), 50000: ("0.679255", "0.247002")}
def label(req):
    return "承诺线内（≤2×）" if req <= 2.0 else ("未超止损线（≤4×）" if req <= 4.0 else "超止损线（>4×）")
for u in TIERS:
    j = recs15[(u, 1)]
    us = nspt_of(j) / u / 1000.0
    req8 = u * us / 8000.0
    req22 = u * us / 22000.0
    r8, r22 = ref61[u]
    print(f"mine: | {u} | {us:.6f} | {req8:.6f} | {label(req8)} | {req22:.6f} | {label(req22)} |")
    print(f"ref : | {u} | {us:.6f} | {r8} | 承诺线内（≤2×） | {r22} | 承诺线内（≤2×） |")
    print(f"cell8={'MATCH' if f'{req8:.6f}' == r8 else 'FAIL'} cell22={'MATCH' if f'{req22:.6f}' == r22 else 'FAIL'}"
          f" label_ok={label(req8) == '承诺线内（≤2×）' and label(req22) == '承诺线内（≤2×）'}")

print("===== (6) acceptance-6 arithmetic (in-file) =====")
print(f"670.6/60 = {670.6/60:.4f} -> round2 = {round(670.6/60, 2)} >= 10 min :"
      f" {'OK' if 670.6 / 60 >= 10 else 'FAIL'} (t007/summary.md L72-73 window 670.6s n=134; L75 high-water line; L77 verdict)")
print(f"SCRIPT_RESULT: t015={'PASS' if ok15 else 'FAIL'} t007={'PASS' if ok07 else 'FAIL'}")
