#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""verify_claims.py — T011 判定行双向核对（派工单 docs/evidence/m0/dispatches/wp-a.md §4；任务卡 D6）。

规格：
- 纯标准库；内置 CLAIMS: list[(label, source_relpath, needle)]，
  内容 = 派工单 §2 清单 A~P 全部锚子串（J/K/L 按实际定位行补全）+ 引用点扩充。
- 双向核对：needle 在 source_relpath 命中 ∧ 在 docs/evidence/m0/README.md 命中（子串级，utf-8）。
- 输出逐条 `PASS/FAIL label source` + 汇总行；退出码 0=全过、1=有 FAIL。
- FAIL 任何一条即停止上报（不得改 needle 凑过——派工单 §4 原文纪律）。

判定语义零裁量：本脚本只做子串双向核对，不产生任何新判定。
路径以脚本自身位置定位仓库根（脚本目录上溯三级），与运行时 cwd 无关。
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
README_REL = "docs/evidence/m0/README.md"

# 派工单 §2 清单 A~P（锚子串逐字；写入前已逐条 grep -n 复核源档行号）
CLAIMS = [
    # ---- A（t015/summary.md L18-21，验收①终态四条）----
    ("A1-①1k-T015", "docs/evidence/t015/summary.md",
     "- [①] units=1000: us_per_unit_tick=0.060651 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）"),
    ("A2-①5k-T015", "docs/evidence/t015/summary.md",
     "- [①] units=5000: us_per_unit_tick=0.072196 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）"),
    ("A3-①10k-T015", "docs/evidence/t015/summary.md",
     "- [①] units=10000: us_per_unit_tick=0.070617 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）"),
    ("A4-①50k-T015", "docs/evidence/t015/summary.md",
     "- [①] units=50000: us_per_unit_tick=0.108681 µs | 止损线 2µs: OK（µs <= 2） | 承诺线 1µs: 达标（µs <= 1）"),
    # ---- B（t007/summary.md L18-21，验收①时序四条）----
    ("B1-①1k-T007", "docs/evidence/t007/summary.md",
     "- [①] units=1000: us_per_unit_tick=4.012138 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）"),
    ("B2-①5k-T007", "docs/evidence/t007/summary.md",
     "- [①] units=5000: us_per_unit_tick=19.846795 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）"),
    ("B3-①10k-T007", "docs/evidence/t007/summary.md",
     "- [①] units=10000: us_per_unit_tick=41.010103 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）"),
    ("B4-①50k-T007", "docs/evidence/t007/summary.md",
     "- [①] units=50000: us_per_unit_tick=200.625962 µs | 止损线 2µs: TRIPPED（µs > 2） | 承诺线 1µs: 未达（µs > 1）"),
    # ---- C（t015/summary.md §④ 换算表 L59-62 四数据行）----
    ("C1-换算1k-T015", "docs/evidence/t015/summary.md",
     "| 1000 | 0.060651 | 0.007581 | 承诺线内（≤2×） | 0.002757 | 承诺线内（≤2×） |"),
    ("C2-换算5k-T015", "docs/evidence/t015/summary.md",
     "| 5000 | 0.072196 | 0.045122 | 承诺线内（≤2×） | 0.016408 | 承诺线内（≤2×） |"),
    ("C3-换算10k-T015", "docs/evidence/t015/summary.md",
     "| 10000 | 0.070617 | 0.088271 | 承诺线内（≤2×） | 0.032099 | 承诺线内（≤2×） |"),
    ("C4-换算50k-T015", "docs/evidence/t015/summary.md",
     "| 50000 | 0.108681 | 0.679255 | 承诺线内（≤2×） | 0.247002 | 承诺线内（≤2×） |"),
    # ---- D / E（t015/summary.md L42 / L68）----
    ("D-②终态-T015", "docs/evidence/t015/summary.md",
     "- [②] units=10000: 12 线程实测加速比=1.001913 | 止损判定（<4×）: TRIPPED | 16 线程外推加速比=1.112480 | 同判（<4×）: TRIPPED"),
    ("E-总判-T015", "docs/evidence/t015/summary.md",
     "- 总判定：TRIPPED（触发项：② 加速比<4×（锚定 10000））；按 6.1/R2 由此进入后续优化决策（本卡不优化）"),
    # ---- F / G（t007/summary.md L42 / L49）----
    ("F-②时序-T007", "docs/evidence/t007/summary.md",
     "- [②] units=10000: 12 线程实测加速比=7.221411 | 止损判定（<4×）: OK | 16 线程外推加速比=9.416188 | 同判（<4×）: OK"),
    ("G-极限十万-T007", "docs/evidence/t007/summary.md",
     "- C(100000) ÷ speedup16 = 3907.133373 ms vs 22 ms 极限预算 → [超界：极限十万目标不保留]"),
    # ---- H（t015/summary.md L49）----
    ("H-极限十万-T015", "docs/evidence/t015/summary.md",
     "- C(100000) ÷ speedup16 = 17.532997 ms vs 22 ms 极限预算 → [保留]"),
    # ---- I（t009/summary.md L180/L181）----
    ("I1-吞吐判定-T009", "docs/evidence/t009/summary.md",
     "- [吞吐判定行] games_per_hour@12t = 1099638.6 （阈值 >= 10000）→ PASS"),
    ("I2-16线程外推-T009", "docs/evidence/t009/summary.md",
     "- [16 线程外推] elapsed(T)=c1+c2/T OLS（T∈{1,3,6,12} 中位）：c1=0.6632s c2=10.3642s·T → elapsed(16)=1.3110s → 吞吐16 = 1405967.6 场/h"),
    # ---- J（t008/matrix.md §8，按实际定位行逐字）----
    ("J1-断言1-T008", "docs/evidence/t008/matrix.md",
     "- 断言 1（零 mismatch：每 (scale,seed) 组合 4 线程档 × 全部采样列 + 终局列逐位一致）: PASS — 6/6 组合逐位一致"),
    ("J2-断言2-T008", "docs/evidence/t008/matrix.md",
     "- 断言 2（中间哈希：每 scale 标准采样列组内一致）: PASS"),
    ("J3-断言3-T008", "docs/evidence/t008/matrix.md",
     "- 断言 3（mismatch 全量如实列出）: 未触发（无 mismatch）"),
    ("J4-加样局-T008", "docs/evidence/t008/matrix.md",
     "- 加样局（跨进程同配置逐列一致）: PASS — red200-th12-s42-r2 PASS；red200-th12-s42-r3 PASS；full10000-th12-s42-r2 PASS"),
    ("J5-锚局-T008", "docs/evidence/t008/matrix.md",
     "- 锚局（vs T006 归档逐字节 diff）: PASS — anchor1-t006-fullscale PASS；anchor2-t006-10k300 PASS"),
    ("J6-种子互异-T008", "docs/evidence/t008/matrix.md",
     "- 种子互异 sanity（同 (scale,threads) 下 3 种子终局哈希两两不同）: PASS — 8/8 个 (scale,threads) 下 3 种子终局哈希两两不同"),
    ("J7-退出码-T008", "docs/evidence/t008/matrix.md",
     "- 退出码: 0"),
    # ---- K（t008/README.md §2 终局哈希表 L84-89 六行）----
    ("K1-red200s42", "docs/evidence/t008/README.md", "| red200 | 42 | 0xde91d6a5a6e84d43 | 200 |"),
    ("K2-red200s43", "docs/evidence/t008/README.md", "| red200 | 43 | 0x999a5237d3b1a9b2 | 200 |"),
    ("K3-red200s44", "docs/evidence/t008/README.md", "| red200 | 44 | 0x44348b8042997ed3 | 200 |"),
    ("K4-fulls42", "docs/evidence/t008/README.md", "| full10000 | 42 | 0xd921c95a9bf1db66 | 9959 |"),
    ("K5-fulls43", "docs/evidence/t008/README.md", "| full10000 | 43 | 0x7d34b08102e34260 | 9969 |"),
    ("K6-fulls44", "docs/evidence/t008/README.md", "| full10000 | 44 | 0x9d55c4ce4f7fd880 | 9970 |"),
    # ---- L（t015/README.md，grep '29/29' / '16/16' 定位行逐字片段）----
    ("L1-29/29-T015", "docs/evidence/t015/README.md",
     "equiv-exits.txt：29/29 IDENTICAL"),
    ("L2-16/16-T015", "docs/evidence/t015/README.md",
     "final_hash vs T007 归档动态比对：**16/16 MATCH**"),
    # ---- M / N（t010/summary.md L31 / L11-14）----
    ("M-渲染判定-T010", "docs/evidence/t010/summary.md",
     "- t10000: avg_fps=333.15 (≥60 ? 是)、1% low=202.48 (≥45 ? 是) → **PASS**"),
    ("N1-t1000-T010", "docs/evidence/t010/summary.md",
     "| t1000 | 34843 | 536.03 | 260.07 | 151.61 | 1.745 | 2.567 | 3.249 | 24.113 | 34841 |  |"),
    ("N2-t2000-T010", "docs/evidence/t010/summary.md",
     "| t2000 | 33933 | 522.03 | 268.28 | 213.06 | 1.800 | 2.666 | 3.324 | 6.026 | 33933 |  |"),
    ("N3-t5000-T010", "docs/evidence/t010/summary.md",
     "| t5000 | 32936 | 506.70 | 242.93 | 191.98 | 1.824 | 2.866 | 3.650 | 6.163 | 32936 |  |"),
    ("N4-t10000-T010", "docs/evidence/t010/summary.md",
     "| t10000 | 21656 | 333.15 | 202.48 | 153.40 | 2.935 | 3.727 | 4.395 | 7.743 | 21656 | PASS |"),
    # ---- O / P（t007/summary.md L77 / L72-73）----
    ("O-⑥判定-T007", "docs/evidence/t007/summary.md",
     "- [⑥] 判定：达标（附：PrivateMemorySize64 max=4.7 MiB）"),
    ("P1-样本-T007", "docs/evidence/t007/summary.md",
     "- 样本：n=134 条（采样间隔 5s；每 5s 读 Get-Process WorkingSet64 + PrivateMemorySize64；判定用工作集）"),
    ("P2-窗口-T007", "docs/evidence/t007/summary.md",
     "- 窗口时长（末样本 elapsed_ms）：670.6 s"),
    # ---- 引用点扩充（口径句 / 归因 / 复现命令 / 披露行；均已 grep -n 复核）----
    ("X01-口径-T007L5", "docs/evidence/t007/summary.md",
     "止损 = 每单位每 tick 成本 > 2µs（单线程基线）或 12 线程实测加速比 < 4×（16 线程外推值 < 4× 同判）"),
    ("X02-降幅1846-T015R", "docs/evidence/t015/README.md",
     "581×/1846×，距止损线 2µs 裕度 ≥18×。"),
    ("X03-Amdahl-T015R", "docs/evidence/t015/README.md",
     "Amdahl 拟合 @10k：c1=630909ns、c2=61780ns"),
    ("X04-上限3.9-T015R", "docs/evidence/t015/README.md",
     "1/(0.19+0.81/12) ≈ 3.9×"),
    ("X05-D10未重跑-T015R", "docs/evidence/t015/README.md",
     "- ⑥ 内存未重跑（无内存行为变更，D10 预登记；summarize 披露行在档）。"),
    ("X06-覆盖域-T008R", "docs/evidence/t008/README.md",
     "red200 只覆盖移动段确定性；战斗段确定性由 full10000 覆盖。不得由本档外推"),
    ("X07-DPI-T010R", "docs/evidence/t010/README.md",
     "成因：显示器 125% DPI 缩放，bevy_winit 无 scale override 时按"),
    ("X08-render门禁-AGENTS", "AGENTS.md",
     "涉 render-spike 的 cargo 一律 **-j 2 从严 + 前置 commit 预检（check/test ≥10G、release ≥12G）**"),
    ("X09-长跑命令-meta", "docs/evidence/t007/runs/longrun_meta.txt",
     "长跑命令: target/release/bench.exe --units 50000 --threads 12 --ticks 478 --warmup 0 --repeats 1 --seed 42"),
    ("X10-吞吐命令-runT009", "docs/evidence/t009/run_t009.sh",
     "./target/release/arena.exe --throughput --games 512 --threads 12 --repeats 3 --out docs/evidence/t009/runs/throughput_t12"),
    ("X11-simt1命令-runT009", "docs/evidence/t009/run_t009.sh",
     "./target/release/sim.exe --comp shieldman:5000 --seed 3000000 --battle --ticks 14400 --threads 1"),
    ("X12-bench用法-AGENTS", "AGENTS.md",
     "./target/release/bench.exe --units <N> --threads <T> --ticks <K>"),
    ("X13-②十万附则-报告", "docs/万阵-游戏前期策划报告.html",
     "外推 10 万模拟单位模拟耗时 ≤ 22ms（极限档模拟预算，表 6-0 帧预算分解）方保留「极限十万」目标"),
    ("X14-⑤灰盒句-报告", "docs/万阵-游戏前期策划报告.html",
     "灰盒无动画/LOD/分层渲染，指标高于 M2/M3 属预期，不作外推依据"),
    ("X15-R2吞吐线-报告", "docs/万阵-游戏前期策划报告.html",
     "实验场吞吐低于 1 万场/小时"),
    ("X16-基准机A-报告", "docs/万阵-游戏前期策划报告.html",
     "基准机 A 已固化（2026-09-29，开发机本机）"),
    ("X17-④口径-报告", "docs/万阵-游戏前期策划报告.html",
     "同种子重放哈希一致（1/3/6/12 线程 × 3 组种子，基准机 A）"),
    ("X18-⑥口径-报告", "docs/万阵-游戏前期策划报告.html",
     "内存：50k 模拟单位 ≥ 10 分钟长跑稳态工作集 ≤ 2 GB 且无单调增长（基准机 A）"),
    ("X19-降规模定义-报告", "docs/万阵-游戏前期策划报告.html",
     "每方 100 模拟单位（共 200）、单局 ≤ 60s（≤ 1,800 ticks @30Hz）。实验场吞吐与 R2 吞吐阈值均按此口径；与全规模对局两套口径不得混用。"),
    ("X20-投入口径-AGENTS", "AGENTS.md",
     "严格字面——台账挂钟全计（主会话+worker+审核轮，含等待）、按日历窗口切分（预备周内完成的任务不计 W1，跨窗口任务按实际执行日期分摊）；降档若触发即如实接受；推进按依赖链自然节奏，不为凑时数灌水"),
    ("X21-vsync-T010", "docs/evidence/t010/summary.md",
     "- t10000: 非 vsync 锁定旁证：有（21656 帧 < 15.865 ms）"),
    ("X22-H2判定-T009", "docs/evidence/t009/summary.md",
     "存在 |z| > 1.96 的对角格 → H2 结构性偏差形态"),
    ("X23-接敌口径-T009", "docs/evidence/t009/summary.md",
     "[接敌实证判定行·口径层] 36 格全部 avg(alive_red+alive_blue) < 200（2*per_side）→ PASS；全场最大 avg 存活(和) = 194.00"),
    ("X24-接敌探针-T009", "docs/evidence/t009/summary.md",
     "[接敌实证判定行·探针层] 36 格全部 avg(alive_red+alive_blue) < 20（2*per_side）→ PASS；全场最大 avg 存活(和) = 14.00"),
    ("X25-接敌全规模-T009", "docs/evidence/t009/summary.md",
     "[接敌实证判定行·全规模层] 100 局全部 alive_red+alive_blue < 10000 → PASS；全场最大存活(和) = 9980"),
    ("X26-分层33/36-T009", "docs/evidence/t009/summary.md",
     "[分层方向一致判定行] 33/36 cell 三层方向一致（比例 91.7%）"),
    ("X27-resolved-T009", "docs/evidence/t009/summary.md",
     "口径层多数格 resolved=false（预注册设计预期）→ 实测口径层 resolved 局数 0/3600；全规模层 resolved 0/100"),
    ("X28-check复现-T008R", "docs/evidence/t008/README.md",
     "cargo check --workspace -j 3"),
    ("X29-runbatch-T008R", "docs/evidence/t008/README.md",
     "./docs/evidence/t008/run_t008.sh all"),
    ("X30-compare-T008R", "docs/evidence/t008/README.md",
     "python docs/evidence/t008/compare_matrix.py docs/evidence/t008/runs"),
    ("X31-gencomp-T008R", "docs/evidence/t008/README.md",
     "python docs/evidence/t008/gen_comp.py"),
    ("X32-summarize-T015sh", "docs/evidence/t015/run_matrix_t015.sh",
     '--summarize "$EVID/matrix.jsonl" --out "$EVID/summary.md"'),
    ("X33-benchbuild-T015sh", "docs/evidence/t015/run_matrix_t015.sh",
     "cargo build -p sim --bin bench --release -j 3"),
    ("X34-t008局命令-plan", "docs/evidence/t008/runs/plan.txt",
     './target/release/sim.exe --comp "shieldman:17,heavyknight:17,pikeman:17,swordsman:17,archer:16,militia:16" --ticks 1800 --hash-samples 0,450,900,1350 --threads 1 --seed 42'),
    ("X35-拟合b-T007", "docs/evidence/t007/summary.md",
     "拟合：a=773.554196350，b=3.997072319，R²=1.000000，最大残差=4560603.291 ns（拟合区间 1k~50k；R²/残差为附加留痕）"),
    ("X36-拟合b-T015", "docs/evidence/t015/summary.md",
     "拟合：a=62.759210207，b=0.000918316，R²=0.999960，最大残差=24223.712 ns（拟合区间 1k~50k；R²/残差为附加留痕）"),
    ("X37-空闲声明-T008env", "docs/evidence/t008/environment.txt",
     "声明: T008 量测窗口机器空闲独占（2026-10-05）"),
    ("X38-D2节名-T007", "docs/evidence/t007/summary.md",
     "## ③ 外推十万"),
    ("X39-owner裁决-T015卡", "taskset/t015-opt-round-1.md",
     "：② 字面（≥4×）vs 意图（预算全满足）+ 轮二 t016"),
]


def main() -> int:
    cache: dict = {}

    def text_of(rel: str) -> str:
        if rel not in cache:
            cache[rel] = (ROOT / rel).read_text(encoding="utf-8")
        return cache[rel]

    readme = text_of(README_REL)
    total = len(CLAIMS)
    fails = []
    for label, rel, needle in CLAIMS:
        in_src = needle in text_of(rel)
        in_readme = needle in readme
        ok = in_src and in_readme
        if not ok:
            fails.append(label)
        tag = "PASS" if ok else "FAIL"
        detail = "" if ok else f"（source={'hit' if in_src else 'MISS'} readme={'hit' if in_readme else 'MISS'}）"
        print(f"{tag} {label} {rel}{detail}")
    print(f"汇总: {total} 条 | PASS {total - len(fails)} | FAIL {len(fails)}")
    if fails:
        print("FAIL 清单: " + "；".join(fails))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
