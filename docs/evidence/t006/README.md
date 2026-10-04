# T006 证据档（M0-05 并行化 v0 与两阶段更新）

- 任务卡：taskset/t006-parallel.md（含主会话执行裁决记录）；派工单设计定稿
  D1~D12；执行：worker-1（2026-10-04，后段因使用配额中断）+ 主会话接管收尾；
- 结论：**门禁全绿**——cargo check 0 警告；cargo test 35/35（既有 30 全保持 +
  T006 新增 5：近距对拍 / 重索敌 / 穿插防御 / 分片纯函数 / 线程池生命周期）；
  **16 局对拍矩阵零 mismatch**（同规格 threads{3,6,12} 与 threads=1 参照 stdout
  逐字节一致 ×12）；**黄金锚零漂移**：T002 锚 0xd3b6408fd46c2008 与 T004 锚
  0x958c5938c8682529 原值通过（35 测内含），且 `--battle --threads 12` 与
  `--battle`（threads=1）终局哈希双档逐位 = T004 锚（并行黄金交叉，
  recheck-main-session.txt）。
- 执行事件如实留痕：worker-1 完成代码 / 测试 / 16 局矩阵跑批（runs/）与
  parity-notes.md 后，于证据汇总阶段触达 5 小时使用配额上限中断——check.txt /
  test.txt / matrix-summary.txt / fullscale-14400.txt / README.md / 台账行由
  主会话接管完成（代码与 runs/ 未因中断变动，主会话独立复现门禁三断言 +
  并行黄金交叉 + CLI 边界 4 项后接管）。

## 1) 文件索引

| 文件 | 内容 |
|---|---|
| check.txt | `cargo check --workspace -j 3` 全文（0 警告）+ REAL_EXIT=0 |
| test.txt | `cargo test -p sim -j 3` 全文 + REAL_EXIT=0（35/35） |
| matrix-summary.txt | 16 局对拍汇总（跨线程 diff×12 全 identical + 16 局 REAL_EXIT + threads=1 参照局输出全文）——主会话复核生成 |
| runs/ | worker-1 batch.sh 跑批原始输出（1k/10k × s42/s43 × t{1,3,6,12} 各 stdout/stderr + exits.txt） |
| fullscale-14400.txt | 验收 4 长跑：10k × 14400 ticks @ threads=12 seed=42（1800s 看门狗）输出 + REAL_EXIT + 壁钟 |
| parity-notes.md | 两阶段 ≡ 原串行等价性论证（构造性证明 + 等价域注记 + 单测↔CLI 覆盖分工 + 首轮门禁返工留痕） |
| recheck-main-session.txt | 主会话独立复现记录（并行黄金交叉 / CLI 边界 4 项 exit 2 / 默认路径回归 / 门禁三断言） |
| batch.sh | worker-1 跑批脚本（矩阵 16 局 + fullscale，可复现） |

## 2) 验收断言对照（任务卡）

1. **check 0 警告** ✓（check.txt）。
2. **线程数无关性（16 局对拍零 mismatch）** ✓——1k（每方 500，14400 ticks，
   samples 0/1800/4000/8000/11000/14400）与 10k（每方 5000，300 ticks，
   samples 0/100/200/300）× threads{1,3,6,12} × seeds{42,43}；同规格四档
   stdout（sample 行 + 五行摘要）逐字节一致（matrix-summary.txt diff×12）。
   1k 局 14400 ticks 内接敌开战（终局 units=963/969 < 1000）——战斗数值路径
   覆盖 ✓（闭合速度下界 0.1 m/tick ⇒ 首接触 ≤ 9990 ticks）。
3. **并行不改变语义** ✓——三重证据：①矩阵内 threads=1 参照（同代码路径）；
   ②35 测零改动全绿含双黄金锚；③并行黄金交叉（recheck-main-session.txt）。
4. **10k × 12 线程 × 14400 ticks 跑通不 panic 无死锁** ✓——fullscale-14400.txt
   （1800s 看门狗内正常退出 REAL_EXIT=0，实测壁钟 1017s、余量 1.77×；
   口径留痕见 §3）。

## 3) 验收 4 口径与看门狗两次裁决（留痕）

- 派工时裁决（任务卡）：朴素 O(N²) 下 60s 物理不可达（内存带宽推算 ≈300s+），
  「60s 超时保护」调整为防死锁兜底语义，初定看门狗 600s。
- 执行期再修正：worker batch.sh 首跑 fullscale 于 600s 看门狗超时（exit 124，
  runs/exits.txt 留痕）；矩阵 10k 局实测 12 线程 52ms/tick（300 ticks = 15.6s）
  外推 14400 ticks ≈ 750s > 600s——看门狗不足最坏正常时长，**再修正为 1800s**，
  主会话重跑取证（fullscale-14400.txt）。
- 实测结果（主会话重跑）：**REAL_EXIT=0、壁钟 1017s**（elapsed_ms=1017022，
  终局 units=9959 / hash=0x29980473140ed39e）——高于纯移动段 52ms/tick 外推的
  750s：t≈10000 接敌后战斗阶段「目标被先手击杀 → 重索敌」分支抬升每 tick 成本
  （矩阵 300 ticks 局全程未接敌，为纯移动口径；均摊 ≈70.6ms/tick）。1800s
  看门狗覆盖实测值余量 1.77×，无 panic / 无死锁 / 无超时截断。
- 「60s 内跑完」转为 T007 性能输入（空间划分 / 数据布局优化方向；T006 范围
  明文排除性能调优）。

## 4) 性能观察（非断言、T007 先导参考）

- 10k × 300 ticks 单线程 119.3s → 12 线程 15.6s（**加速比 ≈7.6×**，10 硬件
  线程：i5-12490F 6P+4E 无 HT，含每 tick 双快照 + 通道往返开销）；
- 1k × 14400 ticks 单线程 56.8s；矩阵总跑批 ≈10 分钟；
- fullscale 10k × 14400 ticks @ 12 线程 1017s（均摊 ≈70.6ms/tick，含接敌后
  战斗段 vs 纯移动段 52ms/tick——战斗重索敌分支的量级参考，T007 输入）；
- 正式量测（基准机 A 口径、单线程基线 µs/单位/tick、1/3/6/12 档 + 外推 16）
  归 T007 基准套件。

## 5) 上报项（worker → 主会话，均已裁决）

1. **派工单 D3 移动应用式两处修正**（连续第四卡派工单笔误，主会话复核采纳）：
   ① gap 差值漏乘 `dir`——蓝方 dir=−1 裸差值恒负恒停，首轮门禁黄金锚炸掉
   （29/32，返工 1 次如实计）后修正；② `desire`（gap_old 预折算）设计缺陷
   ——同侧友军先结算前进时 gap_now > gap_old，双层 min 把前进错误截断在旧
   间隙，与原版 min(speed, gap_now) 分歧（主会话等价性分情形证明只覆盖相向
   收缩半边、漏同侧扩张半边）。终版：意图只缓冲 front（O(N²) 索引热点即并行
   目标），应用阶段以原版公式逐字现算——等价性论证反而更直接（parity-notes
   §2）。教训回流主会话记忆（第四卡）。
2. **派工单 §3.3 构造笔误修正**：原红队间距 0.9 m < 剑士半径和 1.0 m，初始
   即自穿插与该测断言直接矛盾——按断言口径修为恰 1.0 m、蓝队首 +0.2 m 保持
   首对 gap 0.05 被测动力学不变；连带末态断言改结构性（贴身链波传播松弛后
   恢复贴身，非「恒停」）。
3. **MoveIntent 结构简化**（相对派工单 D3）：desire 不入缓冲（上报项 1② 的
   直接后果），意图结构仅 `front: Option<usize>`。
