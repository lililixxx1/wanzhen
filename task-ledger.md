# 任务台账（万阵）

- 依据：策划报告 06 章（全职三周 + 首周投入校准降档）与 7.2 Q5（M0 收官序列）。自 T001 起逐条记录。
- 角色：M0 增值证明与三周投入校准的**唯一数据源**（首周 <30h 降档判定、身份决策评审的工时输入），任何任务开始前不可缺位。
- 纪律：任务收尾前完成登记；耗时以分针近似（挂钟时间，含等待编译）；返工以「首次判定未通过后的重做次数」计。
- 审核口径：分级——代码卡轻量自查（check 0 警告 + 验证绿 + 本表如实计）；完整 plan-code-reviewer 审核轮仅两节点（T007 量测套件交付、T012 M0 数据成档独立复算）。

## Schema

| 字段 | 说明 |
|---|---|
| 编号 | 自增，`T001` 起；与 `taskset/t*.md` 任务卡编号对齐 |
| 任务 | 任务名；标准化任务标注 taskset 条目号 |
| 类型 | `骨架` / `代码` / `量测` / `文档` / `验证` / `审核` / `其他` |
| 一次通过 | `是` / `否`。代码类口径：首次提交即过 `cargo check --workspace` 0 警告与验证清单，期间无人工改代码 |
| 返工次数 | 首次判定后的重做次数（0 = 一次通过） |
| 耗时 | 分钟（近似，挂钟） |
| 原因 | 返工/失败原因；一次通过者记关键偏差或环境问题 |
| 证据 | commit hash / 命令与结果记录 / 日志或数据档路径 |

## 记录

| 编号 | 任务 | 类型 | 一次通过 | 返工次数 | 耗时 | 原因 | 证据 |
|---|---|---|---|---|---|---|---|
| T001 | 仓库骨架搭建与台账启用（预备周；taskset/t001-skeleton.md）：AGENTS.md 万阵版 + 报告 V0.9.1 收编 docs/（仓外源字节一致 85984）+ 台账新册 + Cargo workspace（sim headless）+ rust-toolchain 钉版 1.98.1 + git bot 身份 + 私有远程首推 | 骨架 | 否 | 2 | ≈150 min（含三轮执行中断重启） | 返工 2 如实计：①首轮 check 失败——rustup 1.98.1 工具链「安装被中断」损坏（会话中断连锁的次生灾害），修复安装后过；②二轮 check 退出码 0 但带 1 条 manifest 警告（member 级 default-features=false 被 workspace 级忽略→sim 误拉全量 bevy），修正 workspace 级声明后三轮 0 警告 23.2s。环境教训：执行者后台 agent 三次被用户消息中断连带杀除——长命令改由主会话分段执行；doctest/量测门禁负载敏感（并发 bevy 编译曾致另一仓门禁假红挂死），门禁须空闲独占 + cargo -j 3（owner 2026-09-30 指令入 AGENTS.md） | 首提交 707cf75（含三轮 check 证据档）+ 远程 lililixxx1/wanzhen master 核对一致 |
| T002 | M0-01 最小确定性 tick 循环（W1；taskset/t002-tick-loop.md）：sim 改 lib+bin（rng/hash/world 模块）——SplitMix64 seed 扩展 + Xoshiro256\*\* 零依赖手写、FNV-1a 64 状态哈希（固定序：tick→rng 4 状态字→单位 alive 索引序）、tick 纯计数推进 + tick 级 RNG 固定消耗点（主会话派工定稿，保证 seed 进入哈希可见路径）、薄 CLI 壳（stdout 确定性五行 / stderr 壁钟）；单测 8 项含 RNG 黄金序列与 World 黄金哈希 | 代码 | 否 | 1 | ≈15 min（worker 执行 616s 含编译等待 + 主会话验收复核） | 返工 1 如实计：首次 check 编译错误 E0277（CLI parse_num 泛型缺 `T::Err: Display` 约束），加约束后一次过 0 警告。黄金值按 PIT-M-002 纪律占位 0 实测产出（该轮 test 6 过/2 golden 失败属流程预期步骤非缺陷），经 python 独立重实现逐值对拍一致后回填、复跑 8/8 全绿；worker 复算脚本首版误复用打印后的 RNG 状态（1808 次消耗）致哈希不符——属复算工具失误已修正留痕，Rust 侧零改动。分级审核：代码卡轻量自查（完整审核轮留 T007/T012），主会话独立复核四断言全复现 | docs/evidence/t002/（9 文件，命令+输出+REAL_EXIT 自含）：跨进程对拍逐字节一致 hash=0xd3b6408fd46c2008（seed 43 → 0xa81b59deeb98e700 不同）；10,000 空单位×1,800 ticks 能力烟雾 elapsed_ms=18（阈值 <1,000ms） |
| T003 | M0-02 六兵种数据与单 lane 移动（W1；taskset/t003-units-move.md）：units.rs 六兵种表 v0（hp/攻/攻速/移速/半径/类别，Q32.32 纯整数表达式）+ 克制表 3×3（Q16.16：×1.5/×1.0/×2÷3）+ world.rs 扩展（Unit{kind,side,x}、双方对称布阵 deploy：独立 RNG seed^SALT Fisher-Yates 全洗 + 镜像排布、单 lane 移动：索引序顺序结算 + 贴身停 + 友军排队堵停）+ CLI（--comp / --dump-formation）。克制方向留痕：报告 3.1 只定义三类环形结构未落字方向，主会话按 2.1 米拉奇原型语义拍板「重甲克轻甲→轻甲克无甲→无甲克重甲」，平衡初值、实验场回归对象 | 代码 | 否 | 2 | ≈18 min（worker 执行 967s 含编译等待 + 主会话验收复核） | 返工 2 如实计（同轮发现）：①首轮 check 1 条 dead_code 警告（DEFAULT_UNITS 常量残留未删）；②首轮 test 辅助单测 Fisher-Yates 重构式写反（deploy 本体无误）。黄金占位轮 2 失败不计返工（PIT-M-002 规定动作）。主会话裁决：派工单 D2 括注速算值笔误（2×65536÷3 整除=43690、5×2^32÷100=214748364），按权威整数表达式为准不返工；--comp/--units 互斥等 4 项保守处理接受。设计观察（留 T004）：默认构成 1800 ticks 内不接敌（LANE 1000m、队首相距 999m、合速≤0.4m/tick），T004 接敌用例需手动构造近距或调 LANE_LEN/移速初值 | docs/evidence/t003/（9 文件）：布阵快照跨进程逐字节一致 0x22bce4da752220de（seed 43 → 0x70f65c2f7f585cad 不同）；新黄金（默认构成 60 单位 1800 ticks）0xf2b85bd4727c2d45；T002 黄金锚 0xd3b6408fd46c2008 原值保持；单测 17/17 |
| T004 | M0-03 索敌攻击与克制结算（W1；taskset/t004-combat.md）：world.rs 战斗阶段（最近邻索敌：存活敌方 \|dx\| 最小、平局最小索引、每 tick 重算 O(n²)；cd 冷却制：Unit 新增 hp/cd，先全存活单位 saturating_sub(1) 再索引序判定，出手才置 interval；伤害 units::damage_dealt = atk×counter_multiplier/ONE_Q16_16 截断，溢出安全留痕；死亡=墓碑 → tick 末 retain 保序压缩，跨 tick 索引不稳定 D6 留痕）+ step 阶段序固定（tick→RNG→move→combat→retain→hash，D5）+ state_hash 扩展（每单位 +hp 4B LE +cd 4B LE，hash::write_u32，D7）+ UnitSpec.range_q32 落值（近战 0/archer 30m，M0 判定不读，D8）+ MELEE_MARGIN_Q32=858993459（D3）+ CLI --dump-final-units（D9）+ 默认构成不接敌观察留痕移交 T005（D10）。单测 24（新增 A 36 组合伤害对拍/B/C/D 手算例/E 死亡对拍/F 常量自检/C2 真等距决胜链式贴身 5 单位——主会话复核补充，因 worker-2 上报例 2 等距分支未触达；改造 G 贴身停战斗化、H hp 垫高；黄金 I 实测回填；T002 锚 J 保持原值） | 代码 | 否 | 1 | ≈35 min（worker 执行 2078s 含读档推演与编译等待 + 主会话验收复核含 C2 补测；对齐 T002/T003 台账 agent 全程口径） | 返工 1 如实计：首轮 check 1 条 unused variable 警告（combat 索敌闭包参数），改写 Option<(usize,i64)> 承载后 0 警告；编辑事故（move_units 注释重复块）check 前发现修正未入门禁。黄金占位轮 1 失败不计返工（PIT-M-002 规定动作）。**主会话已裁决（确认派工单笔误，按锁定表执行为终态，克制表零改动）**：派工单 D1 例 4/5 克制倍率方向互换（民兵打重甲锁定表实为 9 非 3、重骑打民兵实为 9 非 21——43690/98304 两格写反），连带 G/H 存活推演（G：t=200 双活红 30/蓝 15、首亡红 t=274 而非派工单 t≈135/红 hp=102；H：hp150 骑士会被民兵 9/击×20 于 t=385 反杀落入 400 tick 循环，骑士 hp 同垫 6000）——实现与断言一律按锁定表执行、克制表零改动；py_recalc 双盲若在此两格出现差异，差异源即此 | docs/evidence/t004/（12 文件，命令+输出+REAL_EXIT 自含）：新黄金 0x958c5938c8682529（=10776086108806063401，测试路径与 release CLI 逐位一致；旧值 0xf2b85bd4727c2d45 因 D7 折叠扩展+战斗行为失效，golden.txt 留痕）；T002 锚 0xd3b6408fd46c2008 原值保持；seed 42/43 各自跨进程 stdout 逐字节一致且互异（0x54611ed6ded02540）；dump-final-units：60 单位 1800 ticks 全存活满血 cd=0（D10 实证）；单测 24/24（含主会话补 C2）；红线 token 扫描 0 命中；**双盲对拍闭环**（py_recalc python 独立第二实现 × Rust：seed 42/0x958c5938c8682529 + seed 43/0x54611ed6ded02540 双实现双 seed 逐位一致，手算例 1~4 快照逐字段一致，C2 哈希锚 t1=0xdcd569af50a28656/t21=0x77695db8b679c474 双盲固化——T012 独立复算方法论提前演练） |
