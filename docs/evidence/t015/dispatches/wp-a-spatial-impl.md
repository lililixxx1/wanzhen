# T015 派工单 WP-A 留痕（worker-1：spatial 快路径实现）

> 留痕说明：本档为下发原文留痕（2026-10-05，主会话 → worker-1，隔离树执行）。
> 机器绝对路径按 T003/T007 P1-2 先例清理为占位符：`<REPO>` = 主仓根、
> `<TREE>` = 隔离树 t015-a 根（git archive 基线 1c4dd87 导出）、
> `<APPENDIX>` = 项目附录目录（team-prompt/，本仓 gitignored 不入库）。
> 语义零改动；执行结果见 ../tree-a-check.txt、../tree-a-anchors.txt。

---

# T015 派工单 WP-A（worker-1）：索敌/移动意图排序序快路径实现

## 纪律（先读，违反即停）

1. 你是执行层：只按本单干活。范围外决策（断言对不上、口径含糊、双盲不一致、发现需改单内禁改文件）→ **停下上报主会话，不拍板**。
2. 环境条款：先读 `<APPENDIX>/PROJECT-APPENDIX.md` 的**附录 A（环境与门禁）、附录 B.2（固化检查单）、附录 C（保真锚点）** 三节，按其执行（cargo 一律 `-j 3` 串行；机器绝对路径不进任何产出文件内容）。
3. 隔离树 = 你的唯一工作区：`<TREE>/`（git archive 导出、无 .git，基线 commit 1c4dd87）。**禁止写主仓** `<REPO>/`。树内路径下文记 `<T>` = 该绝对路径。
4. 数值纪律（附录 B.2）：本单所有哈希参考值标「仅供双盲核对」——你必须先打开所引源文件逐字核对（grep -n / 读文件），一致才可用作断言；**不一致 → 上报停止，不得静默改数**。
5. 语义双盲：本单 D4/D5 快路径等价推导是主会话写的——你实现前必须打开 `<T>/sim/src/world.rs` 逐行读旧实现（行号见下），确认旧语义与推导一致；任何不一致 → 上报停止。
6. 同一异常排查 ≤2 次即上报；跑不了的验收项如实报「未跑+原因」。
7. 时间盒 ≈4 小时。超盒即上报进度与断点，不默认续做。

## 目标

把 world.rs 两个 O(N²) 意图阶段（move 索 front / combat 索最近敌）在生产域降为「(x,索引) 全序 + O(1) 邻域查询 + O(N) 扫掠」，**模拟行为逐位不变**（黄金锚红线）。应用阶段（apply）、state_hash、retain、cd、combat 重索敌**零改动**。

## 基线与参考（行号已在主会话 grep -n 核对；你仍须树内复核 = 双盲）

- `<T>/sim/src/world.rs`：
  - :274-290 `ScanInput`（x/alive/side/kind 四数组）与 `from_units`
  - :314-325 `MoveIntent{front}` / `CombatIntent{target,in_range}`
  - :327-358 `move_intent_chunk`（旧语义：per-i 扫全 j 取 `(x[j]-x[i])*dir > 0` 最小值，平局最小 j）
  - :360-406 `combat_intent_chunk`（旧语义：per-i 扫全 j 取存活敌方 `|x[j]-x[i]|` 最小，**严格小于才更新 ⇒ 等距取最小索引 j**；in_range = `dist <= r_i + r_t + MELEE_MARGIN_Q32` 闭区间）
  - :612-648 `move_units_with`（意图段 :619-627 + 应用段 :629-647——应用段禁改）
  - :666-741 `combat_with`（意图段 :673-681 + cd 段 :684-688 + 应用段 :690-741——后两段禁改）
  - :919 T002 黄金 `15255451774252490760`（=0xd3b6408fd46c2008）；:1133 T004 黄金 `10776086108806063401`（=0x958c5938c8682529）
  - :145 `TICK_CAP_REDUCED=1800`；:149 `TICK_CAP_FULL=14400`
- `<T>/sim/src/units.rs`:34 `MELEE_MARGIN_Q32 = 20 * ONE_Q32_32 / 100`（=858993459）
- `<T>/sim/src/pool.rs`:38 `chunk_range(n, chunks, k) = (k*n/chunks, (k+1)*n/chunks)`；:95 `map_chunks(n, f)`（f: Fn(start,end)->Vec<T>，各片结果按 start 升序拼装；f 不得 panic；串行调用）
- `<T>/sim/src/hash.rs`:33-36 FNV 逐字节（禁改）
- `<T>/docs/evidence/t007/summary.md` 附录 A（:81-98）：bench final_hash 参考——1k×600t s42 = `0x88b33d3124562844`；5k×300t = `0x8c284cbd5c81a033`；10k×300t = `0xc5915d042208e267`；50k×30t = `0x022c5abdfae119dc`（打开文件核对这些行后再用）。
- `<T>/sim/src/bin/bench.rs`:284-301 `composition_for`（bench 内部构成映射，本单不改 bench）。

## 写范围（只允许改/建以下内容；禁改清单之外的一切）

1. **新建 `<T>/sim/src/spatial.rs`**（本卡主体，含模块 doc + 单测）。
2. `<T>/sim/src/lib.rs`：仅允许 ①加一行模块声明（与现有 pub mod 同风格）②模块 doc 的模块清单里补 spatial 一句。
3. `<T>/sim/src/world.rs`：仅允许 ①`move_units_with` / `combat_with` 两处的**意图构造段**改为「建序 → eligibility → 快路径或回退」分发（应用段/cd 段逐字不动）②文件头模块 doc 增补 T015 小节（快路径+回退+等价性一句话级留痕）③文件尾 `#[cfg(test)] mod tests` 内**新增**测试函数（既有 35 个测试函数**逐字节零改动**，含注释）。
4. **新建证据** `<T>/docs/evidence/t015/`：`tree-a-check.txt`（门禁取证）、`tree-a-anchors.txt`（锚点取证）。文件内容用相对路径（相对仓库根的写法），禁止机器绝对路径。
- **禁改**：`sim/src/{main.rs,pool.rs,units.rs,hash.rs,rng.rs}`、`sim/src/bin/bench.rs`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`sim/Cargo.toml`、既有测试、`move_intent_chunk`/`combat_intent_chunk` 两个函数本体（逐字保留 = 回退路径 + 语义参照）。
- 零新依赖（Cargo.toml 禁改即保证）。

## 规格（D1~D7，主会话定稿；实现细节你在规格内自决）

### S1 `SortedOrder`（spatial.rs）

```rust
pub(crate) struct SortedOrder { order: Vec<usize>, pos: Vec<usize> }
// order: 单位索引按 (x[i], i) 升序；pos: pos[单位索引] = 该单位在 order 中的位置
```

- `build(x: &[i64], pool: Option<&ThreadPool>) -> SortedOrder`：
  - `pool=None`：单次 `sort_unstable_by`（比较器 `(x[a], a) < (x[b], b)` 字典序）。
  - `pool=Some(p)`：`p.map_chunks(n, |s,e| { (s..e) 收集后按同比较器局部排序 })` 得按片拼接的 Vec<usize>（各片内部有序、片界 = `chunk_range(n, p.threads(), k)`），再**串行自底向上两两归并**成全局有序（归并结果唯一——比较器是全序（索引互异 ⇒ 无相等元素），故与算法/线程数无关）。
  - pos 逆映射一遍 O(n)。n=0/1 平凡成立。
- `x_strictly_increasing(&self, x: &[i64]) -> bool`：一遍检查 `x[order[p]] < x[order[p+1]]` 对全部 p 严格成立。

### S2 快路径查询（spatial.rs，pub(crate)，入参切片、与 ScanInput 解耦）

- `move_fronts_fast(x: &[i64], side: &[u8], so: &SortedOrder, pool) -> Vec<Option<usize>>`（输出按单位索引 i 对齐）：
  - 前置：调用方已保证 eligibility（x 沿序严格递增 + alive 全真，见 S4）。
  - per 单位 i（p=pos[i]）：`side[i]==Red(0)` → `front = 若 p+1 < n 则 Some(order[p+1]) 否则 None`；`Blue(1)` → `若 p > 0 则 Some(order[p-1]) 否则 None`。
  - **等价推导（D4，你须对照 world.rs:327-358 复核）**：x 互异 ⇒ i 的严格前方最近者 = 序中紧邻（红取更大 x 侧邻、蓝取更小 x 侧邻）；无 dir 算术。
  - pool=Some：把 x/side/order/pos 克隆进 `Arc<Vec<_>>` 后 `map_chunks`（闭包 'static、不 panic）；pool=None：主线程直循环。两条路径同一 per-i 纯函数。
- `combat_targets_fast(x: &[i64], side: &[u8], kind: &[u8], so: &SortedOrder, pool) -> Vec<(Option<usize>, bool)>`（target 单位索引 + in_range）：
  - 先两遍 O(n) 扫掠得 4 个 per-位置数组（Option<usize> 存**单位索引**）：
    - L→R 一遍：`prev_blue[p]` / `prev_red[p]` = 位置 p 严格左侧最近的蓝/红单位（写 out[p] 用进入 p 前的携带值，写完再按 order[p] 的 side 更新携带值）；
    - R→L 一遍：`next_blue[p]` / `next_red[p]` 同理（严格右侧）。
  - per 单位 i（p=pos[i]，s=side[i]）：左候选 L = (s==Red ? prev_blue[p] : prev_red[p])，右候选 R = (s==Red ? next_blue[p] : next_red[p])；无候选侧跳过；
    - dL = x[i] − x[L]，dR = x[R] − x[i]（均为正，x 互异保证）；
    - dL < dR → target=L；dR < dL → target=R；**相等 → 取单位索引较小者**（等价旧「索引序扫描严格小于才更新 ⇒ 首个最小索引胜」：x 互异 ⇒ 同距离敌至多左右各一，见 world.rs:360-406 复核）；
    - 双侧皆无 → None；
    - in_range = `dist <= spec(kind_of(kind[i])).radius_q32 + spec(kind_of(kind[target])).radius_q32 + MELEE_MARGIN_Q32`（闭区间，与旧 :395-399 同式同常量；spec/UnitKind/MELEE_MARGIN 从 crate::units 引）。
  - 扫掠串行；per-i 查询 pool=Some 时 map_chunks 并行 / None 直循环，同一纯函数。
  - side 判别值：Red=0/Blue=1（world.rs:165-168）；越界不可达。

### S3 world.rs 意图段接入（两处同构）

```
let input = ScanInput::from_units(&self.units);            // 不动
let so = SortedOrder::build(&input.x, pool);               // 新
let intents = if so.x_strictly_increasing(&input.x) && input.alive.iter().all(|&a| a) {
    // 快路径：move_fronts_fast / combat_targets_fast → MoveIntent{front} / CombatIntent{target,in_range}
} else {
    // 回退：原 match pool { None => chunk(0,n), Some(p) => Arc+map_chunks } 逐字保留
};
```

- 应用段/cd 段/重索敌段**不动**（快路径 in_range 语义 = 快照位置现算，与旧一致；重索敌照旧走 O(n) 现扫）。
- 借用冲突（input 被回退分支 Arc 化）由你自选结构解决，语义不得变。

### S4 eligibility 语义（D2）

「x 沿 order 严格递增（无平局）**且** alive 全真」→ 快路径；否则回退旧路径。退化域（`World::new` 全体重合 x=0、含同 x 手构局）走回退 ⇒ 行为与优化前逐字一致。n≤1 恒 eligible。

### S5 新增单测（world.rs tests mod 新函数 + spatial.rs 内单测，全部确定性、无黄金占位轮）

必须包含（名称可自定但语义齐全）：

1. **spatial 基础**：小手构例断言 order 按 (x,idx) 全序、pos 互逆；`x_strictly_increasing` 对「互异序列=true / 含相等=false / n=0,1=true」。
2. **differential·move**（核心等价证据）：固定 25 个种子 s=0..24，n 取 {1,2,3,5,17,60}（n = 表[s % 6]）；构造（全部整数算式）：
   - 位置 `x[i] = (7*i + ((i*i + s) % 5)) * ONE_Q32_32`（相邻差 = 7 + [−4,4] 抖动 ≥ 3 ⇒ 严格递增 ⇒ 互异；测试内先 assert 互异）；
   - 再按确定性置换打乱索引序：s 为奇数先 reverse、再 rotate_left((s*3) % n)（n>0 时）；
   - `side[i] = ((i + s) % 3 == 0) ? Blue : Red`；`kind[i] = 六兵种循环 (i + s) % 6`；alive 全 true。
   - 对每个世界：`SortedOrder::build(&x, None)` + eligibility 必为 true → `move_fronts_fast(...)` 逐单位 vs `move_intent_chunk(&ScanInput{...}, 0, n)` 的 `front` **逐 Option 相等断言**。
3. **differential·combat**：同 25 世界 → `combat_targets_fast(...)` 的 (target, in_range) vs `combat_intent_chunk` 逐单位相等；**必须含一个真等距决胜局**：五民兵链（复用 world.rs 既有单测 C2 构造——位置 0, +0.8, −0.8, −1.6, −2.4（Q32.32 = k * 4*ONE/5），side 红/蓝/蓝/蓝/红）assert 两侧实现都给 i=0 → target=Some(1)（等距平局最小索引）。
4. **回退触发**：手构含同 x 对（两单位同 x 异 side）→ `x_strictly_increasing` false；全 alive 但含重合 → eligibility false（文档化回退）。
5. **世界级线程对拍（快路径 + 战斗段）**：近距接敌局——红 12 人队列（Shieldman 循环）队首 x=10*ONE 向后排（相邻间距 1.5*ONE = r+r+ONE/2，同 deploy 式），蓝 12 人（Militia）镜像于 x=13*ONE；接敌可达性算式：初距 3m，最慢合闭合 ≥ 0.05+0.05=0.10 m/tick，射程阈值 ≥ 0.5+0.4+0.2=1.1m ⇒ 接敌 ≤ (3−1.1)/0.10 = 19 ticks ≪ 600。跑 600 ticks：assert 终局存活 < 24（战斗发生）；三个 World（pool None / threads=3 / threads=12）在检查点 {0,100,350,600} 的 `state_hash()` 两两相等（模式照抄既有 T006 对拍测试）。
6. **既有 35 测试零改动全绿**（不是新测试，是验收——git diff 确认 tests 段仅新增）。

### S6 性能烟测（记录非门禁，机器可能非空闲）

`./target/release/bench.exe --units 10000 --threads 1 --ticks 300 --seed 42`：**门禁断言** stdout JSON `final_hash` == `0xc5915d042208e267`（双盲核对 summary.md 后用）；median_ns **只记录不设阈值**（树内非空闲窗口，正式量测在主会话收获后）。

## 验收标准（门禁命令 + 退出码 + 断言；全部在 `<T>` 内执行、单独整句跑、证据入 `<T>/docs/evidence/t015/`）

| # | 命令（`cd <T>` 后） | 退出码 | 关键断言 |
|---|---|---|---|
| G1 | `cargo check --workspace -j 3` | 0 | 输出 0 警告（`Finished` 且无 `warning` 字样）|
| G2 | `cargo test -p sim -j 3` | 0 | 既有 35 + 新增全部通过；测试总数如实报告 |
| G3 | `cargo build -p sim --release -j 3 && cargo build -p sim --bin bench --release -j 3` | 0 | — |
| G4 | `./target/release/sim.exe --seed 42 --ticks 1800` | 0 | stdout 含 `hash=0x958c5938c8682529`（T004 锚）|
| G5 | `./target/release/sim.exe --seed 43 --ticks 1800` | 0 | `hash=0x54611ed6ded02540`（T004 seed43 伴随值，源 docs/evidence/t004/）|
| G6 | `./target/release/sim.exe --battle --threads 12` | 0 | `final_hash=0x958c5938c8682529`（--battle 黄金交叉，T006 先例）|
| G7 | `./target/release/sim.exe --units 500 --ticks 300 --threads 1` 与 `--threads 12` 各跑一次 | 均 0 | 两份 stdout `hash=` 行一致（回退路径跨线程不变——注意该局全体同 x=0 全红方，走回退）|
| G8 | `./target/release/bench.exe --units 10000 --threads 1 --ticks 300 --seed 42` | 0 | JSON `final_hash` == `0xc5915d042208e267`（S6）|

- 证据档：G1/G2 → `docs/evidence/t015/tree-a-check.txt`（命令+原始输出+REAL_EXIT）；G4~G8 → `docs/evidence/t015/tree-a-anchors.txt`（同格式；bench 附 median_ns 记录行）。
- 单命令 >10 分钟会触发 ZCode 硬超时：拆步单独整句重跑，不追查超时本身。

## 上报条件

- 双盲不一致（任何参考哈希/行号/语义推导与树内实际不符）；
- 写范围/禁改冲突；等价性断言失败（differential 或锚点炸）→ **不许改期望值**，如实上报现场；
- 时间盒耗尽；同一异常 2 次未定位。

## 汇报格式（返回主会话，四要素缺一不可）

1. 完成项：改动文件清单（含每文件改动要点）、新增测试名与总数；
2. 命令与退出码：G1~G8 逐条（贴原始关键行）；
3. 结果：双盲核对结论逐项（行号/哈希/语义推导三类）、differential 25+1 局结论、锚点 5 项结论、bench median_ns 观察值；
4. 未决点/偏离：任何规格偏离与理由（无则写「无」）。
