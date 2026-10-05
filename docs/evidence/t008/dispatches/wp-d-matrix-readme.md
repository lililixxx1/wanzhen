# T008 派工单 WP-D：全量对拍矩阵表 + 证据 README

## 纪律（先读，违反即停）

1. 你是执行层：只按本单干活。范围外决策（分析器判定 FAIL、缺局、数据异常）→ **如实记录 + 停下上报主会话，不拍板**（尤其：mismatch 不得自行重跑遮蔽——任务卡验收断言 3）。
2. 红线：`sim/` 目录**零改动**；`runs/` 已有档**只读**（matrix.md / README.md 为你的写入范围）。
3. 全部产出一律**相对仓库根路径**（公开仓卫生：完成后自查 grep `C:/Users`、`/home/` 等机器路径模式 → 0 命中）。
4. 哈希值/判定全部来自 `compare_matrix.py` 代码输出，禁止手抄。

## 背景

T008 = 同种子重放确定性验证（验收④）。WP-A（跑批基建 + 降规模 15 局）/ WP-C（全规模 14 局）已落盘，WP-B 已交付 `compare_matrix.py`（夹具自测通过）。本单出最终对拍矩阵表与证据 README。

## D1 全量对拍

```
python docs/evidence/t008/compare_matrix.py docs/evidence/t008/runs
```

- 产出 `docs/evidence/t008/matrix.md`（对拍矩阵表：组合 × 哈希值 × 判定 + 汇总判定行对应任务卡验收断言 1/2/3）。
- 退出码 0 = 全 PASS → 继续 D2；退出码 3（缺局）/5（mismatch）→ **停止上报**（把 matrix.md 原文与退出码一并带回，不得触碰 runs/）。
- 自留一行判定记录（命令 + 退出码）并入 README。

## D2 `docs/evidence/t008/README.md`（证据档索引）

必含小节：

1. **口径**：seeds {42,43,44} × threads {1,3,6,12} × 规模 {red200：每方 100 ticks 1800、full10000：每方 5000 ticks 14400}；构成映射 = bench.rs:284-301 逐字口径（引用行号）；采样点表与选点意图（full 的 10800/12600 为接敌后点——战斗段哈希覆盖证据 = 终局 units<10000；red 的 1800 ticks 内不接敌 = 移动段覆盖，战斗段由 full 规模覆盖——**如实披露覆盖域**）。
2. **矩阵结论**：直接引用 matrix.md 判定行（24 局零 mismatch / 中间哈希一致 / 加样跨进程一致 / 锚逐字节一致）。
3. **锚对照表**：anchor1/anchor2 期望值来源 = `docs/evidence/t006/runs/fullscale.stdout` 与 `10k-s42-t12.stdout`（逐字节 diff 判定）；说明锚局用 T006 的 COMP5000 串（逐字取自 t006/batch.sh）而矩阵局用 bench 映射串——两套串的用途差异如实披露。
4. **复现命令**：门禁 / 构建 / run_t008.sh 全量重跑 / compare_matrix.py。
5. **证据文件索引**：run_t008.sh / gen_comp.py / comp.txt / check.txt / compare_matrix.py / fixtures-selftest.txt / environment.txt / matrix.md / runs/（plan.txt + 29 局 ×3 档 + exits.txt + anchor*-diff.txt）。
6. **执行事件**（从 WP-A/C 报告与 exits.txt 转记，如实）：中断/接管/watchdog/重跑记录。

## D3 自查清单（报告带回）

1. `grep -r "C:/Users\|/home/\|/c/Users" docs/evidence/t008/` → 0 命中（dispatches/ 派工单目录豁免——它们是流程文档；其余全部适用）。
2. exits.txt 行数 = 29 且全部 REAL_EXIT=0（如有非 0 → 不掩盖，写进 README 披露行 + 上报）。
3. matrix.md 判定行与 exits.txt、README 结论三方相符。
4. README 中哈希值均能在 runs/ 档或 matrix.md 中溯源（抽 3 个值回查）。

## 报告格式（返回主会话）

1. compare_matrix.py 退出码与判定行摘要；2. matrix.md 关键表（粘贴判定行）；3. 自查清单 4 项结果；4. 任何上报事项。
