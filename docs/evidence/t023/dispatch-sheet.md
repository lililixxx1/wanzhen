# T023 派工单（M5-06 统计面接入，席位 8）——形态已裁决，worker 执行

> 任务卡：`taskset/t023-stats-face.md`。基线 = 派发时 master HEAD（建树 commit 见工单下发记录）。
> 附录引用：`C:\Users\Administrator\Desktop\ccc\wanzhen\team-prompt\PROJECT-APPENDIX.md` 附录 A/B.1/B.2/D——只读，禁转写。

## 0. 结论先行

形态裁决（任务卡「开工裁决」三选项）= **宿主内批跑 + 扩 `game.*` +1 方法**（`game.sample_outcomes`）。理由：「玩家可达路径」在 M5 = 宿主 BRP 面（T021 配置面同面；arena bin 是 M0 开发工具、宿主外，玩家不可达）；批跑复用 sim 公共 API（deploy_versus + run_battle_with 与三入口同源），arena bin 零改动。**game.* 计数 6 → 7，计数门禁留痕（§3 D6）。**

## 1. 范围（任务卡逐字）

- 降规模口径批量采样（每方 100 共 200 单位/局——表 6-0 字面）出胜率矩阵/分布（可读输出档）。
- 口径注三件随卡：#11 降/全规模不混用；#12 样本量不足 400 场注「趋势指示、非基准」；#9 灰盒指标不作外推依据注（本卡如实标「不适用——统计面无渲染指标」）。
- 同种子批内确定性抽查（重跑一致）。
- 范围外：性能吞吐验收、AI 池扩档重测、统计界面可视化。

## 2. 方法语义定案（`game.sample_outcomes`，worker 按此实现）

- **params**：`{red: [{kind,count}...], blue: [...], lane_len_m?: int, max_ticks?: u64, seed_base: u64, games: u64, threads?: u64}`——red/blue 必填（复用 `parse_composition`，与 game.deploy 同解析同报错）；lane 缺省 `DEFAULT_LANE_LEN_M`、max_ticks 缺省 `TICK_CAP_REDUCED`（=1800 降规模字面）、threads 缺省 1（域同 deploy）。`games` 域 **1..=1000**（新域常量 `MAX_SAMPLE_GAMES = 1000`，超域 INVALID_PARAMS 消息列域）。
- **批语义（确定性红线）**：局 i ∈ 0..games-1，seed_i = `seed_base.wrapping_add(i)`（**u64 wrap 显式留痕**——序列公开可复现）；每局全新 `World::deploy_versus(red, blue, lane_q32, seed_i)` → `run_battle_with(max_ticks, pool)` → 四元组。批与批之间无状态残留（每局独立 World，沿 suite 净副作用纪律）。
- **response**：`{games, seed_base, red_wins, blue_wins, draws, win_rate_red_pp: u64（= red_wins × 10000 / games，万分比整数——**禁浮点统计**，判定与展示同源）, outcomes: [{seed, winner, end_tick, alive_red, alive_blue, final_hash}...]}`（final_hash 十六进制字符串 0x%016x，与 outcome 面同格式）。outcomes 全量返回（1000 局上限 × 6 字段，JSON 体量可接受——实测为准，超 10MB 再上报）。
- **error codes**：沿用 4xxx 正码体系；参数错 = INVALID_PARAMS（-32602）既有约定；本方法无 NOT_DEPLOYED/BATTLE_RESOLVED 路径（不触碰 HostedGame——纯只读玩法面，**不影响已布阵对局**）。

## 3. 设计裁决（D1~D6）

- **D1 形态** = 宿主内批跑 + game.* +1（见 §0）。**rpc.rs 允许改动**（+1 handler + 注册 + 常量；既有 6 方法语义零改动——冒烟不回归）。
- **D2 计数门禁**：`game.* 计数 6→7`——banner 方法行（main.rs `[host] methods:` 行）同步 + api-notes 增条目 + 本卡证据 README 留痕（收口段 Lead 回写 AGENTS.md 常用命令节）。
- **D3 确定性验收**：同参批内重跑 = outcomes 数组**逐字节一致**（冒烟脚本 jq/python 序列化比对或逐字段循环比对——判定行代码生成）；跨线程档抽查：games=16 @threads 1 vs 12 → 每局四元组逐字段一致（sim 跨线程纪律既证，此处为方法面接线复查）。
- **D4 口径注三件（逐字入档）**：输出档与 README 各带——①「降规模口径：每方 100 共 200 单位/局、单局 ≤1800 ticks（表 6-0）——不得与全规模数据混用（残余账 #11）」；②样本量注：示例档 <400 场 →「趋势指示、非基准（判据 ±10pp / ≥400 场/周，R5 功效注——残余账 #12）」；③「灰盒指标不作外推依据（残余账 #9）——本卡不适用：统计面无渲染指标，如实标注」。
- **D5 证据档**：`docs/evidence/t023/`——`sample_smoke.sh`（端口 **15707** 独占）+ 运行 log（REQ/RESP 原文）+ `matrix-example.md`（小矩阵示例：3 red 构型 × 3 blue 构型，每格 games=100、seed_base=42——共 900 局、约 ≤40s，判定行脚本生成）+ `README.md`（索引/判定行/口径注三件/锚来源）。示例档至少含一个 ≥100 局单构型批（任务卡断言 1 字面）。
- **D6 不进 run_tests 面**：统计面 = 玩法面非判定面（判定主体语义沿 T041 口径——本方法返回数据、不返回 pass/fail）；suite.rs 零改动。

## 4. 门禁（全绿才算完）

1. `cargo check --workspace -j 3` = 0 警告；`cargo test -p sim -j 3` = 0（sim/arena 零改动红线，被迫改 → 停手上报）。
2. `cargo build -p host --release -j 3` = 0（构建前附录 A 预检）。
3. `bash docs/evidence/t018/brp_smoke.sh` 46/46 不回归（既有 6 方法语义零改动的证明）。
4. `bash docs/evidence/t022/challenge_smoke.sh` 全 PASS 不回归（若 T022 已收获合并；未合并则跳过并留痕——上报 Lead 确认基线含 T022 与否）。
5. `bash docs/evidence/t023/sample_smoke.sh` 全 PASS（含确定性逐字节比对 + 跨线程抽查 + 口径注在档检查 + error 路径：games=0/games=1001/缺 red → INVALID_PARAMS 结构化错误）。
6. ZCode Bash 10 分钟限：批跑命令若预估超 8 分钟（games 上限 × 慢构型）→ 拆步或后台跑 + 轮询留痕（REAL_EXIT 取证纪律）。

## 5. 上报纪律

同附录 B.1/B.2：异常 ≤2 次未定位上报；数值三查（games×耗时预算、wrap 语义、胜率算式）；范围外决策上报不拍板。
