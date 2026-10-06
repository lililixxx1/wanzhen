# T012 reviewer independent W1 effort recompute: parse task-ledger.md effort column (escaped-pipe aware)
# and split windows per the investment rule (prep-week tasks T001-T006; T007 split at the prep/W1
# boundary 10-04/10-05 with stage decomposition 99 prep / tail 65+25+1=91 W1; T008/T009/T015/T010/T011 -> W1).
import re
rows = {}
for line in open("task-ledger.md", encoding="utf-8"):
    if not re.match(r"^\| T\d{3} \|", line):
        continue
    cells = re.split(r"(?<!\)\|", line)
    tid = cells[1].strip()
    effort = cells[6]
    m = re.search(r"≈(\d+) min", effort)
    stages = [int(x) for x in re.findall(r"≈(\d+) min", effort)]
    rows[tid] = (int(m.group(1)), stages)
prep_ids = ["T001", "T002", "T003", "T004", "T005", "T006"]
w1_ids = ["T008", "T009", "T015", "T010", "T011"]
prep = 0
for t in prep_ids:
    v, _ = rows[t]
    prep += v
    print(f"{t} total={v} -> prep")
t007_total, t007_stages = rows["T007"]
t007_prep = t007_stages[1] if len(t007_stages) >= 2 else 99
t007_w1 = t007_total - t007_prep
prep += t007_prep
print(f"T007 total={t007_total} stages={t007_stages} -> prep={t007_prep} W1={t007_w1} (stage sum {sum(t007_stages[1:])} + tail {t007_total - sum(t007_stages)} = {t007_w1})")
w1 = t007_w1
for t in w1_ids:
    v, _ = rows[t]
    w1 += v
    print(f"{t} total={v} -> W1")
print(f"prep_total = {prep} min = {prep/60:.2f} h   (m0/README §8 ref: 557 min = 9.28 h)")
print(f"W1_total   = {w1} min = {w1/60:.2f} h   (m0/README §8 ref: 1246 min = 20.77 h)")
print(f"prep {'MATCH' if prep == 557 else 'FAIL'} | W1 {'MATCH' if w1 == 1246 else 'FAIL'}")
