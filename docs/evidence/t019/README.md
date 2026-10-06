# T019 证据档 · M5-02 观战模式与表现层（席位 1 观战形态 + 4 + 5）

- 实现段：worker-1，2026-10-07（派工单 docs/evidence/t019/dispatch-sheet.md，D1~D9 Lead 裁决定案）。
- 整改段（修后复审轮）：worker-1 首段（P1 代码 + 段 4/5 脚本落树，配额中断）→ worker-2 续做（附录 E 转卡先例）——详见 §6。
- 基线：主仓 master c368204（隔离树导出，树内工作，无 .git）。
- 结论先行：**门禁 1（check）0 警告过；门禁 2（release）经 Lead 两级裁决放行后 REAL_EXIT=0；四项回归全绿（t018 46/46、t021 30/30、t020 m5core 19/19、t020 error_paths 22/22）；spectate 冒烟整改后 88/88 全 PASS（SCRIPT_EXIT=0；实现段基线 54/54）**——含跨模式确定性主断言（黄金锚 0xb82a248ff23515e2 逐位一致）、screenshot 两段式全流程与复审 P0/P1 整改断言（§6；续做轮复跑：check/release REAL_EXIT=0、spectate 88/88、t018 46/46）。

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
| 证据 | `docs/evidence/t019/spectate-smoke-run.log` + `host[1-7]-std*.log` | 冒烟 REQ/RESP 原文 + 各实例 stderr/stderr 全量（整改后七实例；整改首跑失败档 `runs/spectate-smoke-run-p0p1-first-fail.log`） |
| 证据 | `docs/evidence/t019/shot-1-frozen-melee-seed7.png` / `shot-2-sixkinds-deployed.png` / `shot-3-sixkinds-closeup-lane60.png` | 观战截屏人证（见 §4；shot-3 = 复审 P0 近景主证） |

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
| 6 观战冒烟 | `bash docs/evidence/t019/spectate_smoke.sh` | 0 | **54/54 PASS（实现段基线）**；整改后 88/88 见 §3、§6.4 |

内存预检（附录 A 命令）全程留痕：check 段前 11.7G（≥10G ✓）；release 段前 11.8G→11.9G（<12G，触发两次上报，见 1.1）。 CommitLimit 58.5G。

### 1.1 门禁 2 裁决链（如实入档）

1. **预检不足 12G**：release 段预检 11.8G < 12G → 按派工单纪律等 5 分钟重试一次（11.8G）→ 上报等待。
2. **Lead 一裁（分型判定放行）**：12G 门为 bevy 渲染栈冷编口径，共享 target 已有 render-spike release 工件、D2 同源依赖下大概率增量复用 → 条件放行：起跑前进程门（cargo/rustc=0 ✓）+ 起跑 60s 盯日志，出现依赖栈编译行即中止回退。
3. **命中中止条件**：65s 日志见 `Compiling bevy_ecs/bevy_app/bevy_reflect/…` = 真冷编（host 的 bevy_full+bevy_remote 特性集与既有工件均不匹配）→ taskkill 全清（进程复查 0），ABORT NOTE 入 `gate2-release-build.log`，回退等待。
4. **Lead 二裁（-j 1 冷编放行 + 监控地板）**：12G 门按 -j 2 并发冷编口径；-j 1 峰值足迹估算 ≈5-6G，11.9G 余量裕度 ≥5G，从严方向（更慢不更险）。执行条件：开跑前 Lead 确认（T022 收获窗口让路）+ 2 分钟间隔 CommitFree 监控、跌破 7G 即 taskkill 中止上报。
5. **-j 1 执行**：起跑前进程门 0 ✓、CommitFree 11.8G；全程监控读数 12.7~15.1G（未触 7G 地板）；31m 42s 完成，REAL_EXIT=0。
6. **共享 target 覆写事件**：j1 构建完成后 02:27，共享 target/release/host.exe 被主仓侧 release 构建覆盖（主仓收获门禁产物，其源无 `--spectate`——冒烟二跑 `--spectate` 报 unknown argument 实锚）。处置 = 树源增量重编（gate2c，双哈希一致）后重跑。该事件属共享 target 跨仓覆写风险，随上报移交（建议收获后主仓侧构建不会再覆盖——树交付以哈希核对为准）。

### 1.2 整改续做轮门禁（2026-10-07，修后复审；worker-2）

- 门禁 1：`cargo check --workspace -j 2`（CARGO_TARGET_DIR = 共享 target；跑前 cargo/rustc 进程计数 = 0）→ 0 警告 / 0 错误，0.56s fresh（`runs/gate1-check-r2.log`）。
- 门禁 2：`cargo build -p host --release -j 2`（同 target）→ 0.65s fresh、无 bevy 栈编译行、REAL_EXIT=0（`runs/gate2-release-r2.log`）。
- 预检：CommitFree ≈ 47G（CommitLimit 58.53G − Committed 11.56G；≥13G 裕度充分）。
- 判定汇总见 §6.4（含 spectate 88/88 与 t018 46/46 复跑）。

## 2. 实现要点与裁决留痕

- **D4 终局语义（Lead 批复 2026-10-07）**：melee-brawl seed7 无灭绝、run_to_tick 1800 收于上限 tick=1800（T021 归档 preset-smoke-run.log 实锚 `0xb82a248ff23515e2`）→ autorun 驱动在 `tick ≥ max_ticks` 且未灭绝时调 `run_battle_with(max_ticks)` 走 resolve_by_hp 上限判定（**不再推 tick**，final_hash = 上限 tick 态哈希）——属 D4「终局 = outcome 存在」语义内；灭绝路径仍由 advance_ticks 内逐 tick 检查收束（两语义共存，spectate.rs 模块注释 + 驱动系统内注释）。
- **确定性（验收断言 4 的构造证明）**：headless 与 spectate 的唯一模拟推进路径 = `rpc::advance_ticks`（循环体自 T018 handler 逐字迁移）；`run_with(n)` 即 n 次 `step_with`（sim/src/world.rs:895-899）→ 分帧切块（spectate 30Hz 预算）与单次直推（headless）终态逐位一致。冒烟 CHK-12/CHK-18 双断言实测（§3）。
- **表现层单向只读（验收断言 1）**：present.rs / hud.rs 全部系统对 `HostedGame` 只取 `Res<>`（无 `ResMut`/`Mut`）；模拟态写面仅 spectate 驱动系统（`ResMut<HostedGame>` → advance_ticks）与 BRP handler（既有路径）。
- **headless 逐字不变（回归面）**：无 `--spectate` 时无 SpectatePlugin（资源不存在即形态判据）——run_to_tick 走 advance_ticks 直推（与原循环体同函数同序）、screenshot 4101 桩原文原样、deploy 无 autorun 复位操作（资源 no-op）；t018 46/46 + t020 22/22 + t021 30/30 实证。
- **P1 修复（树内迭代，冒烟一跑暴露）**：`PresentationRoot` 漏 `init_resource` → `rebuild_on_deploy` 首帧 `ResMut` 校验失败 panic、宿主退出（第一跑 host1-stderr 有 panic 实录，因日志被续跑覆盖，根因与修复以代码注释 spectate.rs「P1 修复」+ 本档留痕）。修复后四跑全绿、全程无 panic。
- **冒烟脚本自身缺陷两处（脚本侧，非宿主代码）**：① `game.outcome` 被用作冻结探针——outcome 对未冻结对局会驱动至终局（t018 语义），污染手动驱动段（实锚：CHK-16 在 tick=6 被直驱至终局）→ 改为仅轮询 state_hash 至 tick≥max_ticks（wait_frozen），终局四元组在冻结后单次调取；② screenshot id 提取 `head -1` 取到 JSON-RPC 回显 id 致轮询 {"id":1} 恒 -32602 → 改取末位匹配（result.id）。
- **D3 补充留痕**：窗口 `focused: true` 显式钉住（= bevy_window-0.19.1/src/window.rs:498 缺省值，防上游漂移）；「多分辨率不做」范围预裁剪 = 固定 1920×1080。

## 3. 观战冒烟判定（整改后 88/88，全判定行见 spectate-smoke-run.log；实现段基线 54/54）

端口纪律：仅 15715（spectate 主体 ×2 段）/ 15716（spectate 第二种子）/ 15717（headless 对拍）；收尾三口 reap 断言 PASS，进程零残留。

### D9 断言集 → 任务卡验收断言对账

| 任务卡断言 | 冒烟实证 | 判定 |
| --- | --- | --- |
| 1. check 0 警告；sim 零 bevy；表现层对 sim 单向只读 | 门禁 1（§1）；sim/、suite.rs、presets.rs 零改动；present/hud 系统签名只读（编译期） | PASS |
| 2. 观战终局无 panic（≥2 种子）；HUD 终局战报与 game.outcome 一致 | 种子 7（CHK-04~08）+ 种子 43（CHK-09~10）两 autorun 全程无 panic（CHK-08b/10d/21c/21d）；HUD 终局行与 `game.outcome` 同一数据源（outcome() 纯读，hud.rs 注释）——shot-1 截图含 WINNER 行人证（§4） | PASS |
| 3. game.screenshot 观战截屏落盘可开 | CHK-07*/CHK-20*：两段式（requested→captured）+ 文件存在 + 字节非零 + PNG 魔数 + sha256 入档（§4） | PASS |
| 4. run_to_tick 观战节流生效且跨模式逐位一致 | 节流：CHK-15/15b（入队立即返回 queued=300、tick=0）+ CHK-16（30Hz 实进——首查 tick=14，300 tick ≈ 10 s 达点）；逐位一致：CHK-18（spectate 300t `0xcdf3fef834f172ce` == headless 300t 同值）+ CHK-12（终局 `0x26c77d5372dc7ad0` 双形态同值） | PASS |
| 5. 六兵种形状 × 阵营色程序化区分可见 | **shot-3（复审 P0 整改主证，段 4 CHK-23~25h）**：六兵种 × 双阵营各 1（12 单位）lane 60m → ~25px/单位形状轮廓可辨 + 尺寸/魔数/字节/sha256 断言；shot-2 保留为全场视角形态对照——**如实注：全场镜头下单位为像素级小点，形状近景人证见 shot-3**（§4） | PASS |
| 复审 P1 ①（截图文件名跨进程唯一） | 段 4 CHK-26~26f：同进程两次截图 nonce 段相同、id 递增、路径互异（`screenshot-{nonce}-{id}.png`；跨进程唯一性由启动毫秒 nonce 保证） | PASS |
| 复审 P1 ②（captured 仅本次文件校验通过后报） | 段 5 CHK-27~30d（15 判定）：预置残留有效 PNG + 独占句柄物理写盘失败 → 轮询恒 pending + detail（锁持有期 = file open failed os error 32；释放后复询 = stale leftover suspected）+ stderr「Cannot save screenshot」见证 | PASS |
| 跨模式黄金锚主断言（D9-1） | CHK-05/05b：melee-brawl seed7 autorun 自走至 tick=1800 冻结，state_hash/outcome.final_hash 均为 `0xb82a248ff23515e2`（T021 归档锚逐位一致；frozen_at 响应原文在档——冻结由驱动系统 max_ticks 收束达成，非探针驱动） | PASS |
| 全程无 panic（D9 各段） | CHK-08b/10d/12c/21c/21d 五实例 stderr 无 panic | PASS |

### 关键值摘录（以 spectate-smoke-run.log 原文为准）

```text
spectate melee-brawl seed7: end_tick=1800 final_hash="final_hash":"0xb82a248ff23515e2"（autorun 自走冻结，frozen_at tick=1800）
spectate seed43 swordsman:10 lane50: winner=red end_tick=1800 final_hash=0x26c77d5372dc7ad0（autorun）
headless  seed43 swordsman:10 lane50: final_hash=0x26c77d5372dc7ad0（outcome 直推——逐位一致）
spectate manual-drive: tick=300 hash=0xcdf3fef834f172ce（queued=300 入队，30Hz 实进）
headless 300t:          tick=300 hash=0xcdf3fef834f172ce（run_to_tick 直推——逐位一致）
shot-3 近景（段 4）: 12 units lane60 dims=2400x1350 bytes=70171 sha256=4a447ad1…（§4）
inject 复询（段 5）: status=pending detail="file mtime … predates request acceptance … (stale leftover suspected)"（锁释放后；锁持有期 detail = file open failed os error 32）
```

节流旁证： autorun 首查 `state_hash` tick=12（整改后复跑实测；实现段基线 14）→ 30Hz 预算实进；manual 段入队到 tick=300 ≈ 10 s（60 帧/2 tick 节奏）。

## 4. 截屏产物（任务卡断言 3/5 人证）

| 文件 | 内容 | 尺寸(px) | 字节 | sha256 |
| --- | --- | --- | --- | --- |
| `shot-1-frozen-melee-seed7.png` | melee-brawl seed7 冻结后主窗：战场（红/蓝双阵营色单位 + HUD 三行含 `WINNER=… end_tick=1800 hash=0xb82a248ff23515e2` 终局行） | 2400×1350 | 104,239 | 04443929ecd7e0ecbef0007c1022016f82ab544628551b2360c83dfaac82eaa5 |
| `shot-2-sixkinds-deployed.png` | default 构成（六兵种）布阵态主窗：六兵种形状 × 双阵营色 + HUD tick/alive 行（**全场视角形态对照**——复审如实注：单位为像素级小点；形状近景人证见 shot-3） | 2400×1350 | 65,627 | 4743fb272be28e6ef35a3d874adbf0f28adf61be64ebe7ba43171a74dbdb55c5 |
| `shot-3-sixkinds-closeup-lane60.png` | **复审 P0 整改主证（近景）**：六兵种 × 双阵营各 1（12 单位）lane 60m 布阵态主窗——形状轮廓可辨（~25px/单位）× 阵营色 + HUD tick/alive 行 | 2400×1350 | 70,171 | 4a447ad187e5418345e92d1befd827f71c1cfdc5eba166d51c2c4f7d503fc4d5 |

- 尺寸注：窗口逻辑分辨率 1920×1080（D3 固定），2400×1350 = 捕获帧缓冲含 125% 系统 DPI 缩放（cosmetic，非多分辨率实现——范围预裁剪口径不变）。
- 六兵种形状映射（shieldman=Sphere / heavyknight=Cuboid / pikeman=Cone / swordsman=Capsule3d / archer=Torus / militia=Cylinder）与阵营色（红 0.85,0.2,0.2 / 蓝 0.2,0.35,0.9）见 present.rs（D5 字面数值）。**复审结论（2026-10-07 初轮）**：shot-2 全场镜头下单位为像素级小点不可辨（= 复审 P0）→ shot-3 近景补齐为形状主证（段 4 lane 60m，见 §6.1）；形状/颜色的画面人证归修后复审目检（vision-reader 归 Lead 侧，本档只证文件真实性与完整性）。
- 性能/帧率零宣称（T024 领域）。

## 5. 可复跑（自含）与路径清洗声明

```bash
# 前置：cargo build -p host --release -j 2（产物落 CARGO_TARGET_DIR/release/host.exe，
#        或仓库根 target/release/host.exe——冒烟脚本按此顺序解析）
bash docs/evidence/t018/brp_smoke.sh        # 期望 46/46, SCRIPT_EXIT=0
bash docs/evidence/t021/preset_smoke.sh     # 期望 30/30, SCRIPT_EXIT=0
bash docs/evidence/t020/error_paths.sh      # 期望 22/22, SCRIPT_EXIT=0（树内整改前版）
bash docs/evidence/t020/m5core_suite.sh     # 期望 19/19, SCRIPT_EXIT=0
bash docs/evidence/t019/spectate_smoke.sh   # 期望 88/88, SCRIPT_EXIT=0（窗口化运行；端口 15715/15716/15717）
```

- 冒烟脚本 host.exe 解析顺序：`$CARGO_TARGET_DIR/release/host.exe`（支持 Windows 盘符路径——自动经 cygpath 归一为 MSYS 绝对；整改续做修复见 §6.3）→ 仓库根相对 `../../../target/release/host.exe`（主仓复跑两态均适用）；端口占用预检（残留实例即 FAIL）。
- **路径清洗**（附录 G，只改前缀、命中行内容原样）：runs/*.log 与 spectate-smoke-run.log 中三类机器前缀已占位符化（含整改续做轮新日志）——`<t019-tree>` = 本任务隔离树根；`<main-repo>` = 主仓根（共享 target 所在）；`<cargo-registry>/rsproxy.cn-e3de039b2554c837` = 本机 cargo registry 源目录（registry 哈希保留）。占位符 ↔ 实际前缀的对照由 worker 报告移交 Lead（清洗映射原文不在档——避免清洗后仍残留机器路径）。清洗后 grep：runs/、spectate-smoke-run.log、host[1-7]-std*.log 零命中；`dispatch-sheet.md` 例外（基线拷贝；主仓 e0d92d7 已清洗闭环，见 §7-1）。
- 零密钥、零个人信息；机器名不出现在判定行。

## 6. P0/P1 整改记录（修后复审轮，2026-10-07；worker-1 首段 → 配额中断 → worker-2 续做）

> 复审依据：审核报告与 Lead 处置记录（主仓 docs/evidence/t019/review-plan-code-reviewer.md 转录档）。整改路线 = 审核建议 A（近景证据补充，不改模拟态、不动相机裁决）+ P1 唯一文件名与核验后置。

### 6.1 复审 P0（形状可见性）——近景证据补充

- 修复：新增冒烟段 4——BRP deploy 六兵种 × 双阵营各 1（12 单位）、lane_len_m=60 → 全场镜头（present.rs D6 `ScalingMode::AutoMin` 覆盖算式，min_width ≥ lane + 2×MARGIN）下 ~0.8m 兵种网格 ≈ 1920×(0.8/62) ≈ 25px/单位，形状轮廓可辨；产出 `shot-3-sixkinds-closeup-lane60.png`（sha256 与尺寸见 §4）。
- 断言集（CHK-23~25h）：deploy tick=0 / units=12 → screenshot 两段式（captured 前置核验含 mtime 界桩，见 §6.2）→ 存在 + 字节非零 + PNG 魔数 + 2400×1350（逻辑 1920×1080，16:9）+ ≥30KB + sha256 归档。
- 形状目检 = Lead 侧 vision-reader 识图（在途）；本档只证场景构型与档案真实。shot-2 保留为全场视角形态对照——**如实注：全场镜头下单位为像素级小点，形状近景人证见 shot-3**。

### 6.2 复审 P1（截图旧文件误认）——唯一文件名 + 核验后置

- 代码（`host/src/spectate.rs`，worker-1 04:13 落树）：文件名 `screenshot-{nonce}-{id}.png`（nonce = 进程启动毫秒，跨进程不复用）；`on_captured` 仅记 `event_arrived`；captured 恒由轮询现场对本次输出文件核验通过后置（存在 + PNG 魔数 + 非零字节 + **mtime ≥ 受理时刻** stale 界桩——0.19.1 `save_to_disk` 写盘失败只记日志、事件到达 ≠ 落盘）。
- 冒烟断言：段 4 P1①（CHK-26~26f：同进程 nonce 相同 / id 递增 / 路径互异）+ 段 5 P1②（CHK-27~30d：预置残留有效 PNG + 独占句柄物理写盘失败 → 恒 pending + detail；锁持有期 detail = `file open failed … (os error 32)`，杀句柄复询 = `stale leftover suspected`；stderr「Cannot save screenshot」见证不误报 captured）。

### 6.3 转卡与续做修复事实（如实注）

- 复审派发 worker-1（原卡执行者）：P1 代码 04:13 落树、段 4/5 脚本 04:17 落树；04:32 冒烟整轮**中断**——**根因 = 脚本 HOST_EXE 解析缺陷**：`CARGO_TARGET_DIR` 以 `C:/...` 传入时不匹配 `/*` 分支，被误判相对路径前缀 `$PWD` → 路径作废、宿主未启动、整轮恒 FAIL（host1-stderr「No such file or directory」实锚；host 进程零残留，Lead 已核）。worker-1 配额中断 → worker-2 续做（附录 E 转卡先例）。
- worker-2 续做修复两处（均脚本侧，宿主代码零改动）：① HOST_EXE 补 Windows 盘符绝对路径分支（cygpath 归一为 MSYS 绝对）；② CHK-29d 两相改造——原断言置于锁持有期内（文件打开被 sharing violation 拒绝、mtime 分支不可达；首跑实锚 FAIL 1 处，原始输出归档 `runs/spectate-smoke-run-p0p1-first-fail.log` = 87/88 / SCRIPT_EXIT=1）→ 杀句柄后复询再断 stale 分支（独立 mini-test 预验通过；全量重跑 88/88）。
- 树 target 冷编中止留痕：t018 回归所需树 target 无依赖工件（其 host.exe 系共享构建拷贝产物，gate2c 先例）→ 重编落入全量冷编路径，按冷编纪律 taskkill 全清（`runs/gate2-tree-target-r2.log` ABORT NOTE，进程复查 0）；替代 = 共享 target 现源构建拷入树 target（sha256 `9dedac211cba1e250df7ce2c74ef5e99cc190e5175ce7e6ce5b3d3d9e3d2329f` 两址一致），t018 回归即对该产物执行。

### 6.4 整改续做轮验证结果（本档数据源）

| 项 | 命令 | 结果 |
| --- | --- | --- |
| 门禁 1 | `cargo check --workspace -j 2`（共享 target） | 0 警告 / 0 错误，0.56s fresh（`runs/gate1-check-r2.log`） |
| 门禁 2 | `cargo build -p host --release -j 2`（共享 target） | REAL_EXIT=0，0.65s fresh、无 bevy 栈编译行（`runs/gate2-release-r2.log`） |
| 观战冒烟 | `bash docs/evidence/t019/spectate_smoke.sh` | **88/88 PASS，SCRIPT_EXIT=0**（段 4 近景全绿 + shot-3 产出；段 5 两相 P1 断言全绿） |
| t018 回归 | `bash docs/evidence/t018/brp_smoke.sh`（树 target = 现源拷贝） | **46/46 PASS，SCRIPT_EXIT=0**（原文见 `t018/brp-smoke-run.log`） |

## 7. 上报与移交

1. **dispatch-sheet.md 含机器路径**（Lead 发单原文引主仓/工作流仓绝对路径）——worker 无权改动发单原文，随本报告上报，收获前请 Lead 决定清洗或豁免。**续做轮核实补充**：主仓侧 e0d92d7 已全量清洗 19 档（含 T019 派工单；主仓 19c417d 起该档零机器路径）；本树该档及 t002~t010/m0 等遗产档为基线 c368204 原样拷贝（T019 证据面除本档外零残留），收获时以主仓已清洗版为准即可。
2. **共享 target 跨仓覆写风险**（§1.1 条目 6）：主仓收获门禁构建会覆盖共享 target/release/host.exe——建议收获以树内哈希核对为准；后续并行波次若共享 target，宜错峰或分目录。整改续做轮树 target 冷编中止事件与 t018 回归二进制来源另见 §6.3。
3. **任务台账回写**（task-ledger.md 实测耗时/返工次数）按三层分工归 Lead 收口；本卡返工：实现段 2 次（PresentationRoot 漏 init；冒烟脚本 outcome 探针 + id 提取缺陷）+ 修后复审轮（复审 P0 近景证据补充、复审 P1 截图唯一性；含续做修复 2 处脚本缺陷——§6.3）。
4. **任务卡执行记录**（taskset/t019-spectate-view.md）：现有执行记录 = 实现段口径（54/54、返工 2 次），整改续做轮结果（88/88、续做修复 2 处、转卡事实）未入卡——是否由 Lead 收口回写或另行指派，请裁决。
