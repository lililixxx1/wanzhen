# T008 执行事件流水（主会话维护；WP-D 转记 README、台账引用）

- 2026-10-05 09:45 T008 启动（W1 D1）：主会话规划定稿 D1~D9（taskset/t008-replay-hash.md 执行裁决记录）、四份派工单落盘 dispatches/，三层架构执行（worker-1/2 执行 + 主会话监管 + plan-code-reviewer 收尾审核）。
- Phase 1（并行）：worker-1 WP-A 与 worker-2#1 WP-B 双双完成全绿——WP-A：双盲核对零分歧、门禁三断言过、15 局（red200 矩阵 12 + 跨进程加样 2 + anchor2 锚局 1）REAL_EXIT 全 0、anchor2 与 T006 归档逐字节 IDENTICAL、red200 三种子跨 4 线程档零 mismatch；WP-B：compare_matrix.py（退出码 0/1/2/3/5）+ 夹具自测 3 必选路径 + 4 附加分支 + 对 WP-A 真数据只读预跑契约对齐 + environment.txt。
- 主会话上报裁决 D10（任务卡留痕）：①T006 batch.sh:4 机器绝对路径顺手清理（T003/T007 P1-2 先例）；②夹具 PASS 路径退出码选 A 维持现状；③退出码归类口径采纳。
- 10:12:22 WP-C（worker-2#2）开工（空闲声明成立）：全规模 14 局长跑（anchor1 → th12 → th6 → th3 → th1 → r2）。
- 11:16 执行事件（WP-C 简报，进程树经 WMI 核实存活）：harness 后台包装壳（父 PID）被系统通知 stopped/killed，**实际跑批进程树未受影响续跑**（run_t008.sh → timeout → sim.exe 存活）。影响留痕：包装壳死后脚本收尾 script_rc=/end 两行不再写；**判完成以 exits.txt 满 29 行 + BATCH_DONE 行为准**。无数据损失、无重跑。
- 11:18 WP-C 进度快照：anchor1-t006-fullscale REAL_EXIT=0 且**逐字节 diff = IDENTICAL**（T006 fullscale 跨卡黄金锚 0x29980473140ed39e 复现，双侧 sha256 94d0439f...，档 runs/anchor1-diff.txt）；full10000-th12 s42/s43/s44 三局 REAL_EXIT=0（793~816s/局）；th6-s42 在跑（tick≥7200），抽检 sample 0/3600/7200 与 th12-s42 逐位一致。
- 18:45:08 WP-C 收官：**14/14 局 REAL_EXIT=0、BATCH_DONE rc=0、零重跑、无看门狗杀**（挂钟 8h32m46s，开工 10:12:22 空闲声明成立）。exits.txt 满 29 行全 0（WP-A 15 + WP-C 14）。全规模 12 局终局三种子跨 4 线程档逐位一致：s42 units=9959 hash=0xd921c95a9bf1db66、s43 units=9969 hash=0x7d34b08102e34260、s44 units=9970 hash=0x9d55c4ce4f7fd880；r2 跨进程加样与 th12-s42 逐字节一致；种子互异 sanity 成立。锚 1 判定档 runs/anchor1-diff.txt = IDENTICAL（双侧 sha256 94d0439f...）。
- 观察披露：本卡各档耗时快于 T006 归档同配置（anchor1 775.9s vs T006 1017.0s；总体 8.6h vs 预估 10~11h）——纯耗时差（T008 非计时卡），stdout 逐字节一致性证明零漂移；两套构成串（锚局 T006 COMP5000 vs 矩阵局 bench 映射）用途差异见派工单 D6/README。运营日志 runs/wp-c-batch.log 为 WP-C 新增（脚本控制台捕获，非 D4 契约档位）。
- 19:04~19:26 plan-code-reviewer 完整审核轮（量测/复测卡口径，覆盖 110 文件 100%）：裁决**有条件通过**——独立复算逐字节一致、双锚 diff、双新进程抽查复跑、26 哈希全量溯源四路闭合，数据与判定全部验证成立；唯一 P1 = 三处解析契约行号引用 off-by-one（main.rs:327-330 应为 326-330；数据零影响，证据指针缺陷）+ P2×5。报告存档 review-plan-code-reviewer.md。
- 19:3x 主会话整改闭环：P1-1 三处行号修正（compare_matrix.py 仅注释变更、复算比对证实行为零变化；fixtures-selftest.txt 按原命令重生成、新 sha256 指纹链闭合、7 路径退出码与原一致）+ P2×5 同批落地（复现记录模式串脱敏 / events 时间占位统一 10:12:22 / 「量测/复测卡」措辞两处统一 / wp-c 括注措辞 / 台账耗时终值回填 ≈605 min）。整改后卫生扫描 0 命中、py_compile 过。**审核裁决转通过**。
