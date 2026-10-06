# T012 reviewer independent W1 effort recompute (retry after regex-transport mangling: the shell
# heredoc transport ate a doubled backslash in the lookbehind of the first attempt, which kept its
# REAL_EXIT=1 trace in recompute-w1-rv2.txt). Backslash-free implementation: escaped-pipe handling
# via chr(92), digit classes via [0-9]. Window split per investment rule: T001-T006 prep;
# T007 split (99 prep on 10-04 / remainder W1 on 10-05, D3-1); T008/T009/T015/T010/T011 W1.
import re
BS = chr(92)
rows = {}
for line in open("task-ledger.md", encoding="utf-8"):
    if not re.match(r"^[|] T[0-9][0-9][0-9] [|]", line):
        continue
    cells = line.replace(BS + "|", chr(1)).split("|")
    tid = cells[1].strip()
    effort = cells[6]
    stages = [int(x) for x in re.findall(r"≈([0-9]+) min", effort)]
    rows[tid] = (stages[0], stages)
prep_ids = ["T001", "T002", "T003", "T004", "T005", "T006"]
w1_ids = ["T008", "T009", "T015", "T010", "T011"]
prep = 0
for t in prep_ids:
    v, _ = rows[t]
    prep += v
    print(f"{t} total={v} -> prep")
t007_total, t007_stages = rows["T007"]
t007_prep = t007_stages[1]
t007_w1 = t007_total - t007_prep
prep += t007_prep
print(f"T007 total={t007_total} stages={t007_stages} -> prep={t007_prep} W1={t007_w1} (stages[1:] sum {sum(t007_stages[1:])} + tail {t007_total - sum(t007_stages)} = {t007_w1})")
w1 = t007_w1
for t in w1_ids:
    v, _ = rows[t]
    w1 += v
    print(f"{t} total={v} -> W1")
print(f"prep_components = 150+15+18+35+110+130+99 = {prep} min = {prep/60:.2f} h   (m0/README §8 ref 557 = 9.28 h)")
print(f"W1_components   = 91+605+145+125+140+140 = {w1} min = {w1/60:.2f} h   (m0/README §8 ref 1246 = 20.77 h)")
print(f"prep {'MATCH' if prep == 557 else 'FAIL'} | W1 {'MATCH' if w1 == 1246 else 'FAIL'}")
