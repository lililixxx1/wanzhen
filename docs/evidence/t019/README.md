# T019 证据档 · M5-02 观战模式与表现层（席位 1 观战形态 + 4 + 5）

- 实现段：worker-1，2026-10-07（派工单 docs/evidence/t019/dispatch-sheet.md，D1~D9 Lead 裁决定案）。
- 基线：主仓 master c368204（隔离树导出，树内工作，无 .git）。
- 结论先行：**门禁 1（check）0 警告过；门禁 2（release）经 Lead 两级裁决放行后 REAL_EXIT=0；四项回归全绿（t018 46/46、t021 30/30、t020 m5core 19/19、t020 error_paths 22/22）；spectate 冒烟 54/54 全 PASS（SCRIPT_EXIT=0）**——含跨模式确定性主断言（黄金锚 0xb82a248ff23515e2 逐位一致）与 screenshot 两段式全流程。

## 0. 交付物清单（树内相对路径）

| 类别 | 路径 | 说明 |
| --- | --- | --- |
| 代码 | `host/Cargo.toml` | D2：裸 `bevy` → `bevy_full { workspace = true, features = ["bevy_remote"] }`（同一 package 不容双依赖项；渲染栈随 default features 进入） |
| 代码 | `host/src/main.rs` | D3：`--spectate` 开关（D1 单二双形态）；spectate = DefaultPlugins + 固定 1920×1080 窗口 + 无 ScheduleRunnerPlugin（winit 驱动）；autorun=true 注入（D4）；banner spectate 三行（headless 三行逐字不变） |
| 代码 | `host/src/rpc.rs` | D4：`advance_ticks` 共享推进函数（循环体自 run_to_tick handler 逐字迁移）；`HostedGame` 增 `generation`（D5）；run_to_tick spectate 入队立即返回（`queued` 字段）；deploy 复位 autorun=false；screenshot 分形态分发（headless 4101 桩逐字不变） |
| 代码 | `host/src/spectate.rs` | 新文件：SpectateState（资源存在性 = 形态判据）、30Hz 预算驱动系统、screenshot 两段式实装（受理/轮询/PNG 魔数核对）、SpectatePlugin |
| 代码 | `host/src/present.rs` | 新文件：表现层（D5 六 mesh × 双阵营色、generation 重建、死亡 Visibility::Hidden 不 despawn、逐帧只读映射）+ 相机与光（D6 正交俯视，ScalingMode::AutoMin 覆盖算式入注释） |
| 代码 | `host/src/hud.rs` | 新文件：三行 HUD（D7）：tick / alive / 终局行（WINNER=… end_tick=… hash=0x…/—），默认字体 |
| 证据 | `docs/evidence/t019/api-notes.md` | ★ 清单逐条 registry 行号查证（0.19.1） |
| 证据 | `docs/evidence/t019/spectate_smoke.sh` | 冒烟脚本（端口 15715/15716/15717，D9 断言集） |
| 证据 | `docs/evidence/t019/runs/gate*.log` | 门禁原始输出（REAL_EXIT 在档，机器路径前缀已清洗，映射见 §5） |
| 证据 | `docs/evidence/t019/spectate-smoke-run.log` + `host[1-5]-std*.log` | 冒烟 REQ/RESP 原文 + 各实例 stderr/stderr 全量 |
| 证据 | `docs/evidence/t019/shot-1-frozen-melee-seed7.png` / `shot-2-sixkinds-deployed.png` | 观战截屏人证（见 §4） |

零改动面（红线）：`sim/`、`host/src/suite.rs`、`host/src/presets.rs`、`render-spike/` 均未触碰；sim 零 bevy 依赖保持。

## 1. 门禁记录（命令全文 + 退出码）

| 门禁 | 命令（CARGO_TARGET_DIR 指向共享 target，与主仓同目录） | 退出码 | 判定 |
| --- | --- | --- | --- |
| 1 静态检查 | `cargo check --workspace -j 2` | 0（`runs/gate1-check.log` 首轮三成员全查 + `gate1-check-3.log` host 复查；grep warning/error 计数 = 0） | **0 警告 0 错误 PASS** |
| 2 release | `cargo build -p host --release -j 1` | 0（`runs/gate2-release-build.log`，Finished in 31m 42s） | PASS（见 1.1 裁决链） |
| 2b 修复重编 | `cargo build -p host --release -j 2`（增量，仅 host crate） | 0（`runs/gate2b-rebuild-fix.log`，10.72s） | PASS |
| 2c 覆写后重验 | 同 2b | 0（`runs/gate2c-rebuild-postclobber.log`，0.56s fresh + 双哈希一致 7fcb2080…） | PASS |
| 3 t018 回归 | `bash docs/evidence/t018/brp_smoke.sh` | 0 | **46/46 PASS**（headless 行为逐字不变实证） |
| 4 t021 回归 | `bash docs/evidence/t021/preset_smoke.sh` | 0 | **30/30 PASS** |
| 5a t020 错误路径 | `bash docs/evidence/t020/error_paths.sh` | 0 | **22/22 PASS**（树内版 = 整改前 14 发 22 判定，Lead 勘误口径） |
| 5b t020 全量套件 | `bash docs/evidence/t020/m5core_suite.sh` | 0 | **19/19 PASS**（树内 suite.rs 为整改前版，断言 8 detail 无 expect 字样属正常——Lead 注） |
| 6 观战冒烟 | `bash docs/evidence/t019/spectate_smoke.sh` | 0 | **54/54 PASS**（见 §3） |

内存预检（附录 A 命令）全程留痕：check 段前 11.7G（≥10G ✓）；release 段前 11.8G→11.9G（<12G，触发两次上报，见 1.1）。 CommitLimit 58.5G。

### 1.1 门禁 2 裁决链（如实入档）

1. **预检不足 12G**：release 段预检 11.8G < 12G → 按派工单纪律等 5 分钟重试一次（11.8G）→ 上报等待。
2. **Lead 一裁（分型判定放行）**：12G 门为 bevy 渲染栈冷编口径，共享 target 已有 render-spike release 工件、D2 同源依赖下大概率增量复用 → 条件放行：起跑前进程门（cargo/rustc=0 ✓）+ 起跑 60s 盯日志，出现依赖栈编译行即中止回退。
3. **命中中止条件**：65s 日志见 `Compiling bevy_ecs/bevy_app/bevy_reflect/…` = 真冷编（host 的 bevy_full+bevy_remote 特性集与既有工件均不匹配）→ taskkill 全清（进程复查 0），ABORT NOTE 入 `gate2-release-build.log`，回退等待。
4. **Lead 二裁（-j 1 冷编放行 + 监控地板）**：12G 门按 -j 2 并发冷编口径；-j 1 峰值足迹估算 ≈5-6G，11.9G 余量裕度 ≥5G，从严方向（更慢不更险）。执行条件：开跑前 Lead 确认（T022 收获窗口让路）+ 2 分钟间隔 CommitFree 监控、跌破 7G 即 taskkill 中止上报。
5. **-j 1 执行**：起跑前进程门 0 ✓、CommitFree 11.8G；全程监控读数 12.7~15.1G（未触 7G 地板）；31m 42s 完成，REAL_EXIT=0。
6. **共享 target 覆写事件**：j1 构建完成后 02:27，共享 target/release/host.exe 被主仓侧 release 构建覆盖（主仓收获门禁产物，其源无 `--spectate`——冒烟二跑 `--spectate` 报 unknown argument 实锚）。处置 = 树源增量重编（gate2c，双哈希一致）后重跑。该事件属共享 target 跨仓覆写风险，随上报移交（建议收获后主仓侧构建不会再覆盖——树交付以哈希核对为准）。

## 2. 实现要点与裁决留痕

- **D4 终局语义（Lead 批复 2026-10-07）**：melee-brawl seed7 无灭绝、run_to_tick 1800 收于上限 tick=1800（T021 归档 preset-smoke-run.log 实锚 `0xb82a248ff23515e2`）→ autorun 驱动在 `tick ≥ max_ticks` 且未灭绝时调 `run_battle_with(max_ticks)` 走 resolve_by_hp 上限判定（**不再推 tick**，final_hash = 上限 tick 态哈希）——属 D4「终局 = outcome 存在」语义内；灭绝路径仍由 advance_ticks 内逐 tick 检查收束（两语义共存，spectate.rs 模块注释 + 驱动系统内注释）。
- **确定性（验收断言 4 的构造证明）**：headless 与 spectate 的唯一模拟推进路径 = `rpc::advance_ticks`（循环体自 T018 handler 逐字迁移）；`run_with(n)` 即 n 次 `step_with`（sim/src/world.rs:895-899）→ 分帧切块（spectate 30Hz 预算）与单次直推（headless）终态逐位一致。冒烟 CHK-12/CHK-18 双断言实测（§3）。
- **表现层单向只读（验收断言 1）**：present.rs / hud.rs 全部系统对 `HostedGame` 只取 `Res<>`（无 `ResMut`/`Mut`）；模拟态写面仅 spectate 驱动系统（`ResMut<HostedGame>` → advance_ticks）与 BRP handler（既有路径）。
- **headless 逐字不变（回归面）**：无 `--spectate` 时无 SpectatePlugin（资源不存在即形态判据）——run_to_tick 走 advance_ticks 直推（与原循环体同函数同序）、screenshot 4101 桩原文原样、deploy 无 autorun 复位操作（资源 no-op）；t018 46/46 + t020 22/22 + t021 30/30 实证。
- **P1 修复（树内迭代，冒烟一跑暴露）**：`PresentationRoot` 漏 `init_resource` → `rebuild_on_deploy` 首帧 `ResMut` 校验失败 panic、宿主退出（第一跑 host1-stderr 有 panic 实录，因日志被续跑覆盖，根因与修复以代码注释 spectate.rs「P1 修复」+ 本档留痕）。修复后四跑全绿、全程无 panic。
- **冒烟脚本自身缺陷两处（脚本侧，非宿主代码）**：① `game.outcome` 被用作冻结探针——outcome 对未冻结对局会驱动至终局（t018 语义），污染手动驱动段（实锚：CHK-16 在 tick=6 被直驱至终局）→ 改为仅轮询 state_hash 至 tick≥max_ticks（wait_frozen），终局四元组在冻结后单次调取；② screenshot id 提取 `head -1` 取到 JSON-RPC 回显 id 致轮询 {"id":1} 恒 -32602 → 改取末位匹配（result.id）。
- **D3 补充留痕**：窗口 `focused: true` 显式钉住（= bevy_window-0.19.1/src/window.rs:498 缺省值，防上游漂移）；「多分辨率不做」范围预裁剪 = 固定 1920×1080。

## 3. 观战冒烟判定（54/54，全判定行见 spectate-smoke-run.log）

端口纪律：仅 15715（spectate 主体 ×2 段）/ 15716（spectate 第二种子）/ 15717（headless 对拍）；收尾三口 reap 断言 PASS，进程零残留。

### D9 断言集 → 任务卡验收断言对账

| 任务卡断言 | 冒烟实证 | 判定 |
| --- | --- | --- |
| 1. check 0 警告；sim 零 bevy；表现层对 sim 单向只读 | 门禁 1（§1）；sim/、suite.rs、presets.rs 零改动；present/hud 系统签名只读（编译期） | PASS |
| 2. 观战终局无 panic（≥2 种子）；HUD 终局战报与 game.outcome 一致 | 种子 7（CHK-04~08）+ 种子 43（CHK-09~10）两 autorun 全程无 panic（CHK-08b/10d/21c/21d）；HUD 终局行与 `game.outcome` 同一数据源（outcome() 纯读，hud.rs 注释）——shot-1 截图含 WINNER 行人证（§4） | PASS |
| 3. game.screenshot 观战截屏落盘可开 | CHK-07*/CHK-20*：两段式（requested→captured）+ 文件存在 + 字节非零 + PNG 魔数 + sha256 入档（§4） | PASS |
| 4. run_to_tick 观战节流生效且跨模式逐位一致 | 节流：CHK-15/15b（入队立即返回 queued=300、tick=0）+ CHK-16（30Hz 实进——首查 tick=14，300 tick ≈ 10 s 达点）；逐位一致：CHK-18（spectate 300t `0xcdf3fef834f172ce` == headless 300t 同值）+ CHK-12（终局 `0x26c77d5372dc7ad0` 双形态同值） | PASS |
| 5. 六兵种形状 × 阵营色程序化区分可见 | shot-2（default 构成六兵种布阵后截屏，§4 人证；CHK-19 units:60 重建路径一并实证） | PASS |
| 跨模式黄金锚主断言（D9-1） | CHK-05/05b：melee-brawl seed7 autorun 自走至 tick=1800 冻结，state_hash/outcome.final_hash 均为 `0xb82a248ff23515e2`（T021 归档锚逐位一致；frozen_at 响应原文在档——冻结由驱动系统 max_ticks 收束达成，非探针驱动） | PASS |
| 全程无 panic（D9 各段） | CHK-08b/10d/12c/21c/21d 五实例 stderr 无 panic | PASS |

### 关键值摘录（以 spectate-smoke-run.log 原文为准）

```text
spectate melee-brawl seed7: end_tick=1800 final_hash="final_hash":"0xb82a248ff23515e2"（autorun 自走冻结，frozen_at tick=1800）
spectate seed43 swordsman:10 lane50: winner=red end_tick=1800 final_hash=0x26c77d5372dc7ad0（autorun）
headless  seed43 swordsman:10 lane50: final_hash=0x26c77d5372dc7ad0（outcome 直推——逐位一致）
spectate manual-drive: tick=300 hash=0xcdf3fef834f172ce（queued=300 入队，30Hz 实进）
headless 300t:          tick=300 hash=0xcdf3fef834f172ce（run_to_tick 直推——逐位一致）
```

节流旁证： autorun 首查 `state_hash` tick=14（就绪后 ~0.5 s）→ 30Hz 预算实进；manual 段入队到 tick=300 ≈ 10 s（60 帧/2 tick 节奏）。

## 4. 截屏产物（任务卡断言 3/5 人证）

| 文件 | 内容 | 尺寸(px) | 字节 | sha256 |
| --- | --- | --- | --- | --- |
| `shot-1-frozen-melee-seed7.png` | melee-brawl seed7 冻结后主窗：战场（红/蓝双阵营色单位 + HUD 三行含 `WINNER=… end_tick=1800 hash=0xb82a248ff23515e2` 终局行） | 2400×1350 | 104,239 | 04443929ecd7e0ecbef0007c1022016f82ab544628551b2360c83dfaac82eaa5 |
| `shot-2-sixkinds-deployed.png` | default 构成（六兵种）布阵态主窗：六兵种形状 × 双阵营色 + HUD tick/alive 行 | 2400×1350 | 65,627 | 4743fb272be28e6ef35a3d874adbf0f28adf61be64ebe7ba43171a74dbdb55c5 |

- 尺寸注：窗口逻辑分辨率 1920×1080（D3 固定），2400×1350 = 捕获帧缓冲含 125% 系统 DPI 缩放（cosmetic，非多分辨率实现——范围预裁剪口径不变）。
- 六兵种形状映射（shieldman=Sphere / heavyknight=Cuboid / pikeman=Cone / swordsman=Capsule3d / archer=Torus / militia=Cylinder）与阵营色（红 0.85,0.2,0.2 / 蓝 0.2,0.35,0.9）见 present.rs（D5 字面数值）；形状/颜色的画面人证归 review 轮目检（本档只证文件真实性与完整性）。
- 性能/帧率零宣称（T024 领域）。

## 5. 可复跑（自含）与路径清洗声明

```bash
# 前置：cargo build -p host --release -j 2（产物落 CARGO_TARGET_DIR/release/host.exe，
#        或仓库根 target/release/host.exe——冒烟脚本按此顺序解析）
bash docs/evidence/t018/brp_smoke.sh        # 期望 46/46, SCRIPT_EXIT=0
bash docs/evidence/t021/preset_smoke.sh     # 期望 30/30, SCRIPT_EXIT=0
bash docs/evidence/t020/error_paths.sh      # 期望 22/22, SCRIPT_EXIT=0（树内整改前版）
bash docs/evidence/t020/m5core_suite.sh     # 期望 19/19, SCRIPT_EXIT=0
bash docs/evidence/t019/spectate_smoke.sh   # 期望 54/54, SCRIPT_EXIT=0（窗口化运行；端口 15715/15716/15717）
```

- 冒烟脚本 host.exe 解析顺序：`$CARGO_TARGET_DIR/release/host.exe` → 仓库根相对 `../../../target/release/host.exe`（主仓复跑两态均适用）；端口占用预检（残留实例即 FAIL）。
- **路径清洗**（附录 G，只改前缀、命中行内容原样）：runs/*.log 中三类机器前缀已占位符化——`<t019-tree>` = 本任务隔离树根；`<main-repo>` = 主仓根（共享 target 所在）；`<cargo-registry>/rsproxy.cn-e3de039b2554c837` = 本机 cargo registry 源目录（registry 哈希保留）。占位符 ↔ 实际前缀的对照由 worker 报告移交 Lead（清洗映射原文不在档——避免清洗后仍残留机器路径）。清洗后 grep（机器路径特征串）零命中。
- 零密钥、零个人信息；机器名不出现在判定行。

## 6. 上报与移交

1. **dispatch-sheet.md 含机器路径**（Lead 发单原文引主仓/工作流仓绝对路径）——worker 无权改动发单原文，随本报告上报，收获前请 Lead 决定清洗或豁免。
2. **共享 target 跨仓覆写风险**（§1.1 条目 6）：主仓收获门禁构建会覆盖共享 target/release/host.exe——建议收获以树内哈希核对为准；后续并行波次若共享 target，宜错峰或分目录。
3. **任务台账回写**（task-ledger.md 实测耗时/返工次数）按三层分工归 Lead 收口；本卡返工 2 次（P1 PresentationRoot 漏 init；冒烟脚本 outcome 探针 + id 提取缺陷——均树内迭代闭环，门禁全绿后交付）。
4. 任务卡执行记录已按 Lead 要求留痕（taskset/t019-spectate-view.md）。
