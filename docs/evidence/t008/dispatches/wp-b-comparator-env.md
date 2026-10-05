# T008 派工单 WP-B（worker-2 #1）：对拍分析器 + 环境档

## 纪律（先读，违反即停）

1. 你是执行层：只按本单干活。范围外决策（需要改 `sim/` 代码、契约含糊、夹具自测对不上）→ **停下上报主会话，不拍板**。
2. 红线：`sim/` 目录**零改动**。唯一产出 = `docs/evidence/t008/` 下的分析器、自测档、环境档。
3. 全部命令与产出文件一律**相对仓库根路径**（禁止机器绝对路径进入任何产出文件）。
4. 哈希值/判定**全部由代码解析计算**，禁止手抄任何哈希值进判定（T007「判定行代码计算，防手算漂移」纪律）。
5. 执行中断 → 已落盘保留 + 报告断点。
6. 与 WP-A（另一执行者）并行：其产出 runs/ 目录你**只读不写**；夹具自测一律放 `target/t008-fixtures/`（不污染 runs/）。

## 背景

T008 = 同种子重放确定性验证（验收④）。24 矩阵局 + 3 跨进程加样 + 2 锚局的对拍矩阵表由本单分析器代码生成。WP-A 同时在建跑批脚本与跑局；契约双方一致（下述 D1 即契约），互不等待。

## D1 `docs/evidence/t008/compare_matrix.py`（python，零第三方依赖）

输入：runs 目录（arg1，默认 `docs/evidence/t008/runs`）。

解析契约（= WP-A D4 同一份）：

- run_id 语法：矩阵局 `<scale>-th<T>-s<SEED>`（scale ∈ `red200`|`full10000`）；加样后缀 `-r2`/`-r3`；锚局 `anchor1-t006-fullscale` / `anchor2-t006-10k300`。
- `<run_id>.stdout` 行语法（出处 `sim/src/main.rs`）：sample 行 `sample=<tick> hash=0x<16hex>`（main.rs:316）；摘要 `seed=` / `ticks=` / `units=` / `final_tick=` / `hash=0x<16hex>`（main.rs:326-330，审核轮 P1-1 修订）。`exits.txt` 行 `<run_id> REAL_EXIT=<code> wall_ms=<ms>`。

判定（全代码计算）：

1. **齐全性**：24 矩阵局 + 3 加样 + 2 锚齐备（缺 → 如实列 missing，exit 3）；每局 REAL_EXIT=0。
2. **断言 1**（零 mismatch）：每 (scale, seed) 组合内 4 线程档 × 全部采样列 + 终局列逐位一致 → 组合判定 PASS/FAIL。
3. **断言 2**（中间哈希）：每 scale 采样列（red200：tick 0/450/900/1350；full10000：0/3600/7200/10800/12600）组内一致——与断言 1 同表逐列判定，另汇总一行。
4. **断言 3 可执行化**：任何 mismatch → 表中如实列出全部不一致值（禁止折叠/省略）；exit 5。
5. **加样局**（-r2/-r3）：与基准局逐列一致判定（跨进程同配置）。
6. **锚局**：anchor1 stdout 与 `docs/evidence/t006/runs/fullscale.stdout` 逐字节 diff 判定；anchor2 与 `docs/evidence/t006/runs/10k-s42-t12.stdout` 同。期望值**从这两个归档文件现场读取**，禁止手抄。
7. **种子互异 sanity**：同 (scale, threads) 下 3 种子终局哈希两两不同（不同种子必须发散——T002 口径）。
8. **覆盖披露行**（披露非判定）：每局 `units=` 终局存活数；full 规模 units<10000 → 战斗段已进入哈希的证据行。

输出：

- `docs/evidence/t008/matrix.md`——对拍矩阵表（组合 × 哈希值 × 判定）+ 汇总判定行；stdout 同内容。
- 退出码：0 全 PASS；1 I/O 错；2 输入格式错；3 缺局；5 mismatch（与 `sim/src/bin/bench.rs` 退出码风格一致）。

表格结构建议（可调，判定语义不可变）：行 = run_id，列 = 各采样 tick 哈希 + 终局哈希 + units + REAL_EXIT；组合判定行按 (scale, seed) 分组；末尾汇总判定行对应任务卡验收断言 1/2/3。

## D2 夹具自测（防分析器假绿）

- PASS 路径：把 `docs/evidence/t006/runs/1k-s42-t{1,3,6,12}.stdout` 与 `10k-s42-t{1,3,6,12}.stdout`（真 sample 行数据）复制到 `target/t008-fixtures/` 并按契约改名为 8 个矩阵 run_id（scale 用 `red200`/`full10000` 名义即可）+ 伪 exits.txt → 跑分析器 → 断言组合判定 PASS 路径通（注意：不同 units/配置的 T006 归档之间哈希不同属预期——夹具只验「解析+同组合一致判定」逻辑，可任选同组合内真一致的 4 档数据铺组）。
- FAIL 路径：任取一哈希改 1 个字符 → 断言 exit 5 且如实列出不一致值。
- 缺局路径：删 1 个文件 → 断言 exit 3。
- 自测记录 `docs/evidence/t008/fixtures-selftest.txt`：三条路径的命令 + 实际退出码 + 关键输出行（REAL_EXIT 自含）。

## D3 环境档 `docs/evidence/t008/environment.txt`

参照 `docs/evidence/t007/collect_environment.ps1`（可复用其命令段）：CPU / GPU+驱动 / RAM / OS / rustc / cargo 版本 + `rust-toolchain.toml` 内容 + `Cargo.lock` 的 bevy 行 + `target/release/sim.exe` sha256 + `target/release/bench.exe` sha256 + git HEAD commit + 声明行「T008 量测窗口机器空闲独占（2026-10-05）」。若 `target/release/sim.exe` 不存在（WP-A 可能尚未构建完）→ 自行 `cargo build -p sim --release -j 3`（注意 cargo 文件锁会自动串行）。

## 产出清单（WP-B）

`docs/evidence/t008/`：`compare_matrix.py`、`fixtures-selftest.txt`、`environment.txt`。

## 报告格式（返回主会话）

1. 分析器退出码设计与契约实现要点；2. 三条夹具自测路径的实际退出码与关键输出；3. 环境档关键行（sim.exe sha256 / rustc / git HEAD）；4. 任何上报事项。
