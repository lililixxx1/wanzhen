# T015 完整审核轮报告存档（plan-code-reviewer，2026-10-05）

> 逐字存档自审核轮返回原文（plan-code-reviewer 独立只读轮；审核范围 = 主仓工作区
> diff 1c4dd87 + docs/evidence/t015/ 全部证据档 + 派工单/跑批脚本）。
> 结论：**通过**（P1=0；P2×1 已整改——README 归因节 4.1×→3.9× 勘误，见本档
> 下方整改留痕；P3×3 随收口回填）。

---

# T015 完整审核轮报告（plan-code-reviewer，独立只读）

```
[结论] 通过
```
等价性红线全链闭合（35 既有测试零改动、黄金锚双路径复现、T008 全 29 局逐字节一致、bench 16/16 哈希跨版本一致），量测判定代码生成且 ② TRIPPED 如实披露。无 P1。P2×1（README 归因节一处算术笔误，方向性结论不受影响）。

```
[已核对]
```

**A. 代码审核**（diff = `git diff 1c4dd87 -- sim/`，改动恰为 spatial.rs 新建 463 行 + world.rs 302 行 + lib.rs 5 行两处 [模块 doc + `pub mod spatial`]，与派工单口径一致）

- **A1 通过**：`cargo check --workspace -j 3` 单独整句执行 → `Finished dev profile`，EXIT=0，0 警告。
- **A2 通过**：`cargo test -p sim -j 3` → 43 passed / 0 failed，EXIT=0（35 既有 + 8 新增 = spatial.rs 3 + world.rs 5）。**既有 35 测试逐字节零改动已 diff 实证**：`git diff 1c4dd87 -- world.rs | grep '^-` 删除行仅两个旧意图 match 块（move/combat 各 7 行），且在回退分支逐字重现（含缩进外完全相同）；tests 段零删除。
- **A3 通过**：`git diff 1c4dd87 -- sim/Cargo.toml sim/src/main.rs sim/src/pool.rs sim/src/units.rs sim/src/hash.rs sim/src/rng.rs sim/src/bin/ Cargo.toml Cargo.lock rust-toolchain.toml` 输出为空——禁改清单全零改动、零新依赖。
- **A4 通过（六点逐一核实）**：
  - ① 全序唯一性：比较器 `(x[a],a).cmp(&(x[b],b))`（spatial.rs:84,95）索引互异 ⇒ 无相等元素 ⇒ 全序 ⇒ 排序唯一；并行建序片界 `starts = (0..=t).map(|k| k*n/t)`（spatial.rs:100）与 pool.rs:39 `chunk_range` **同式**（已读 pool.rs:38-40,95-131 核实 map_chunks 按 start 升序拼装）；空片 dedup、奇数残留段携带（spatial.rs:101,115-117）、n=0 时 starts=[0] 平凡退出——归并逻辑手推 n=3/t=12 空片案例正确；单测 build_matches_serial_across_thread_tiers 覆盖 threads=3/7 × 互异/含重合 x。
  - ② 归并边界：merge_runs（spatial.rs:147-169）标准双指针、buf 暂存后 copy_from_slice 回写，全序下结果唯一。
  - ③ 扫掠写序：prev/next 两遍均为「先写 out[p] 再按 order[p] 更新携带值」（spatial.rs:319-341）= 严格侧（不含自身），携带值语义正确。
  - ④ combat 等距决胜：朴素 combat_intent_chunk（world.rs:393-399）索引升序 + 严格小于才更新 ⇒ (距离, 索引) 字典序 argmin、平局取最小索引；快路径（spatial.rs:238-254）dl<dr / dr<dl / 真等距取小单位索引；x 互异排除同侧同距离（同侧两敌距离严格不等）、异侧等距至多一对 ⇒ 二者等价；真等距 0.8m/0.8m 五民兵链单测锚定（fast[0].0 == Some(1)）。in_range 同式同常量 `dist <= r_i + r_t + MELEE_MARGIN_Q32`（spatial.rs:262-263 vs world.rs:408-412）。
  - ⑤ eligibility：`so.x_strictly_increasing(&input.x) && alive 全真`（world.rs:639,704）不成立即回退，回退代码与被删旧代码逐字相同；退化域（同 x 对 / 含墓碑 / n=0,1）单测覆盖（测 4）。Side repr(u8) Red=0/Blue=1 与 spatial.rs SIDE_* 一致（world.rs:176-181）、kind_of 镜像表逐字同（spatial.rs:49-59 vs world.rs:315-325）。
  - ⑥ 零触碰：move apply clamp 段（world.rs:653-672）、combat cd 段与 apply 重索敌段（world.rs:720-778）、retain、state_hash 均不在 diff 内 = 未改动。
- **A5 通过（独立复跑）**：`./target/release/sim.exe --seed 42 --ticks 1800` → `hash=0x958c5938c8682529`，EXIT=0；`--battle --threads 12`（stderr 丢弃）→ `final_hash=0x958c5938c8682529`，EXIT=0。附做：回退档 `--units 500 --ticks 300` t1 vs t12 stdout `cmp` IDENTICAL（hash=0x0cb0a5adf0f28dea，与 tree-a-anchors G7 一致）。

**B. 等价性证据**

- **B1 通过**：equiv-exits.txt 29 局全 `REAL_EXIT=0 verdict=IDENTICAL`（TOTAL=29 FAIL=0）。独立抽查复跑 2 局（命令逐字取自 docs/evidence/t008/runs/plan.txt）：full10000-th1-s44 与 red200-th12-s43 重跑后与 `t008/runs/<id>.stdout` `cmp` 均**逐字节一致**（含 5/4 个中间采样哈希与终局 0x9d55c4ce4f7fd880 / 0x999a5237d3b1a9b2），EXIT=0。审核者加做全量 29 局 cmp：**29/29 identical, 0 mismatch**——超出抽查要求。
- **B2 通过（独立复算）**：python 从 t015/t007 两份 matrix.jsonl 各读 16 配置：8 配置抽样（含全部 4 个 t1 档 + 1k-t3/10k-t6/10k-t12/50k-t12）8/8 MATCH，且**全 16 配置 final_hash 与 T007 归档一致**——与 hash-crosscheck.txt「16/16 MATCH」相符。
- **B3 通过**：tree-a-check.txt（G1 check 0 警告 / G2 attempt2 43 passed / G3 release，树内 E0616/unused_mut 自纠三轮**如实留痕**且 2 条 unused_mut 经核为 T006 基线既有——docs/evidence/t006/test.txt 同款在档）、tree-a-anchors.txt 与 g1-lead-recheck.txt 五组锚点（T004 锚双路径 / seed43 0x54611ed6ded02540 / 回退档跨线程 / bench 10k 0xc5915d042208e267）数值逐位一致、REAL_EXIT 自含。

**C. 量测复测**

- **C1 通过（独立重算逐位一致）**：t1 四点 median_ns 换算 `median_ns ÷ ticks ÷ units × 1e-3` = 0.060651 / 0.072196 / 0.070617 / 0.108681（1k/5k/10k/50k），与 summary.md ① 表 6 位小数**逐位相同**；全 ≤2µs 且全 ≤1µs 承诺线。
- **C2 通过（独立重算逐位一致）**：10k 档 211850600 ÷ 211446100 = **1.001913**，与 summary.md ② 判定行逐位相同；<4× → TRIPPED、16 外推 1.112480 同判 TRIPPED，如实。
- **C3 通过**：`bench.exe --summarize docs/evidence/t015/matrix.jsonl --out <临时目录>`（EXIT=0）再生成文件与归档 summary.md `diff` **逐字节一致**——判定确为代码生成非手改。
- **C4 通过**：rerun-check.txt 披露值与 runs/rerun-*.stdout.json median_ns 逐值一致（10k-t1 全 4 调用 211850600/262339500/257011200/262786900；1k-t6 77448300/71931000/69529600/72854500；极差 24.05% 复算吻合、µs 裕度 ≥22.8× 复算吻合）；REAL_EXIT.txt 24 行全 0；preflight.txt + tasklist 快照在档、无 cargo/rustc。
- **C5 通过（披露如实）**：② 实测 1.00× 低于预注册区间 1.5~2.5×——README 明示「低于预注册区间（D10 登记 1.5~2.5×），如实披露」+ 预注册对照表「池开销项低估」归因 + ②@50k 0.90× 同披露；summary.md 判定行 TRIPPED 无粉饰；owner 裁决点以「本卡只产数据与判定，不自行放行」呈现，未越权放行。
- **C6 通过**：environment.txt 在档；`certutil -hashfile target/release/bench.exe SHA256` = 5a231259...e79d48d3 与档（大写）**逐位一致**，size 588288 / mtime 21:11:37 一致；D9 profile 措辞与 Cargo.toml 实际不符处按文件实际记录（诚实披露）。

**D. 判定语义**

- 出口判据字面核对：达标 = ① 全 ≤2µs **且** ② ≥4×。实测 ① PASS / ② TRIPPED（1.001913×）→ README 与 summary.md 口径一致，均如实判「未达标（②）」并披露缺口数字（对照 D10 预登记与归因），无任何把 ② 说成达标的话术；② 字面 vs 意图 + Amdahl 论证仅作 owner 裁决点建议呈现——正确。
- 台账：task-ledger.md 无 T015 行、taskset/README.md:25 状态列仍「立项就绪」——**待回填**（按审核指令口径不判阻塞）。任务卡增量 = 设计裁决 D1~D12 开工补记（卡面立项时已预告该动作）；时间盒（≤2 工作日）单日内完成，在盒内。

**独立复算留痕汇总**（完整轮 ≥1 项，实际 6 项）：复跑 ① `--seed 42 --ticks 1800` EXIT=0 hash=0x958c5938c8682529；② `--battle --threads 12` EXIT=0 同锚；③ full10000-th1-s44 复跑 cmp t008 归档 IDENTICAL；④ red200-th12-s43 同上；⑤ 回退档 t1/t12 cmp IDENTICAL；⑥ bench --summarize 再生成 diff IDENTICAL；复算脚本：python 独立重算 ① 四点 / ② 1.001913 / 16 配置哈希对照 / rerun 极差与裕度，全部与汇报值逐位一致。

```
[阻塞项]
```

无（P1 = 0）。

P2（应改，不阻塞）：

1. **docs/evidence/t015/README.md:45-46**——「S(12)@10k 上限 ≈ 1/(0.19+0.81/12) ≈ 4.1×」算术笔误：该式 = **3.88×**（python 复算 3.883495），应写 ≈3.9×。依据：README 本身给出完整算式可复算；任务卡 D10 原文写的是「≈3~4×（恰在 4× 边界）」（正确）。影响：归因节是 owner 裁决（② 口径修订与否）的输入材料，数字应准；但式子在场可复算、非伪造，且 3.9× 比 4.1× **更强**支持「轮二无法以诚实手段闭合 ≥4×」的结论，判定行（代码生成的 1.001913 TRIPPED）不受影响。

P3（记录，不阻塞）：taskset/README.md:25 状态列滞后（与台账同批回填即可）；summary.md 标题为 T007 模板字样（bench.rs 禁改所致，数据源行已明示 t015，结构使然）；`.gitignore` +team-prompt/（派工单附录目录 gitignore 化的仓库卫生改动，非 sim/ 代码，合理）。

```
[最小修复指令]
```

P2-1：docs/evidence/t015/README.md 归因节第 4 点，将「≈ 1/(0.19+0.81/12) ≈ 4.1×」改为「≈ 1/(0.19+0.81/12) ≈ 3.9×」（或改占比数字使命式相符——推荐改结果值，D10 原文「3~4×」本就正确）。改后连带复核「加回必串行 apply 后 ≈2.6~3.5×」（该区间与 3.9× 上限自洽，无需动）。此改动为证据档文字勘误，不触发量测重跑、不触 sim/ 代码（check 门禁不受影响）。

```
[复验命令]
```

P2-1 修复后（或连同收口回填一并）：

1. `grep -n "4.1×" docs/evidence/t015/README.md` → 应零命中（改后 3.9×）；`grep -n "3.9×" docs/evidence/t015/README.md` 应命中归因节第 4 点。
2. 门禁复验（文字改动不触代码，跑一次确认零影响）：`cargo check --workspace -j 3`（期望 EXIT=0 / 0 警告）。
3. 台账回填后：`grep -n "T015" task-ledger.md`（期望出现 T015 行，类型=优化/量测复测，一次通过与耗时如实）；`grep -n "T015" taskset/README.md`（状态列应为完成态）。

—
附审核范围说明：本卡为指定范围全量审核（diff 1c4dd87 全部 sim/ 改动 3 文件已审 + docs/evidence/t015/ 全部证据档 24 文件已核 + 派工单/跑批脚本抽查），覆盖率 100%；跳过项：无。

---

## 整改留痕（主会话，2026-10-05 收口）

- P2-1 已整改：README 归因节第 4 点 4.1× → 3.9×，连带「恰在 4× 线上」→「理论上限已低于 4× 线」并注明勘误来源（审核轮 python 复算 3.883495）；「≈2.6~3.5×」与 3.9× 上限自洽未动（审核处方确认无需动）。
- P3×3：taskset/README.md 状态列 + 台账 T015 行随本收口回填；summary.md 标题字样在 README 索引表加注（bench.rs 禁改故保留）；.gitignore +team-prompt/ 属仓库卫生改动如实入提交说明。
- 复验：`grep -n "4.1×" docs/evidence/t015/README.md` 零命中 / `grep -n "3.9×"` 命中归因节；`cargo check --workspace -j 3` EXIT=0 / 0 警告（见收口提交前复验记录）。
