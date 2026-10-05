# T008 派工单 WP-C（worker-2 #2）：全规模矩阵 14 局长跑 + 锚 1 复现

## 纪律（先读，违反即停）

1. 你是执行层：只按本单干活。范围外决策（mismatch、非 0 退出、watchdog 杀、怀疑环境问题）→ **如实记录 + 停下上报主会话，不拍板**。
2. 红线：`sim/` 目录**零改动**。你只运行既有脚本并落盘证据。
3. 全部命令与产出文件一律**相对仓库根路径**。
4. **断言 3（任务卡原文）：任一 mismatch → 全量如实记录并上报，不得静默重跑遮蔽。** 任何失败局保留原档，禁止覆盖重跑；重跑只能由主会话裁决后以 `-r<N>` 新 run_id 追加。
5. 执行中断（配额/消息，连续四卡先例）→ 已落盘不回滚，报告断点（exits.txt 最后一行 + 在跑局 run_id），主会话接管续跑。
6. **量测窗口机器空闲独占**：跑批期间不得并行任何重负载任务（含其他 worker 的构建/跑局——本单执行期间机器归你独占）。

## 背景

T008 = 同种子重放确定性验证（验收④）。WP-A 已完成跑批基建（`docs/evidence/t008/run_t008.sh` 断点续跑 + `runs/plan.txt` 全 29 局清单 + comp.txt 双盲核对）。本单跑全规模 14 局（12 矩阵 + 1 加样 + 1 锚 1），预计挂钟 **≈10~11 小时**，是 T008 的关键路径。

## D0 前置核对（执行前）

打开确认存在且可读：`docs/evidence/t008/run_t008.sh`、`runs/plan.txt`、`comp.txt`。若缺失/不完整 → 上报停止（WP-A 未就绪）。`target/release/sim.exe` 不存在则 `cargo build -p sim --release -j 3`。

## D1 执行（断点续跑，禁止中途杀进程）

```
./docs/evidence/t008/run_t008.sh anchor1 full10000
```

（确切用法以脚本 usage 为准；选择范围 = `anchor1-t006-fullscale` + `full10000-*` 全部 13 局（含 r2 加样）。）

- 计划 14 局：锚 1 ×1、矩阵 full10000 ×th{12,6,3,1}×s{42,43,44} = 12、加样 `full10000-th12-s42-r2` ×1。
- 执行序以 plan.txt 为准（快档先行：anchor1 → th12 → th6 → th3 → th1 → r2；尽早暴露 mismatch）。
- 每局参数要点（以 plan.txt 逐字为准，此处仅说明）：`--comp <comp.txt 全规模串>` `--ticks 14400` `--hash-samples 0,3600,7200,10800,12600` `--threads <T>` `--seed <S>`。
- 预估单局挂钟（T006 归档 th12=1017s + T007 加速比 7.22× 外推，仅供你监控预期，禁止当验收判据）：th12 ≈ 17min、th6 ≈ 27min、th3 ≈ 46min、th1 ≈ 2h。
- 运行方式：后台运行 + 定期查看 `runs/exits.txt` 尾部进度；**耐心等待，禁止 kill 在跑局**。watchdog（脚本内置，防死锁兜底）杀 = exit 124 如实入档并上报。

## D2 锚 1 复现（跨卡黄金锚，逐字节 diff）

`anchor1-t006-fullscale` 命令逐字复现 T006（comp 串逐字取自 `docs/evidence/t006/batch.sh`——**打开该文件核对**；无 `--hash-samples`）：

```
./target/release/sim.exe --comp shieldman:840,heavyknight:830,pikeman:830,swordsman:830,archer:830,militia:840 --ticks 14400 --threads 12 --seed 42
```

判定 = stdout 与 `docs/evidence/t006/runs/fullscale.stdout` **逐字节 diff**（预期 identical，含 `hash=0x29980473140ed39e` 行——期望值以归档文件原文为准，禁止手抄）→ 结果写 `runs/anchor1-diff.txt`。差异 → 如实记录 + 上报。

## D3 收尾报告（返回主会话）

1. 14 局 REAL_EXIT 清单（exits.txt 原文粘贴）；2. 锚 1 diff 判定；3. 全规模 12 局终局 `units=` 与 `hash=` 行一览（stdout 原文行粘贴，不手抄）；4. 断点/中断/异常事件如实；5. 任何上报事项。
