# T010 · M0-09 渲染 spike（验收⑤，独立 crate）

- 周位：W2-W3（独立 crate，与 T007-T009 可并行）
- 前置：T001（不依赖 sim 战斗系统）
- 目标：胶囊体灰盒 1 万单位同屏渲染验证——回答「渲染层灰盒能同屏多少单位」。

## 范围内

- 独立 crate `render-spike/`（bevy **完整 features**，与 sim 的 minimal 依赖隔离；入 workspace members）。
- 1 万胶囊体（bevy Mesh/Primitive胶囊体或等价灰盒几何）同屏；固定相机固定角度；单位位置由确定性伪随机场生成（seed 可复现，不依赖 sim）。
- **无动画 / 无 LOD / 无分层渲染**（灰盒口径，指标高于 M2/M3 属预期、不作常态渲染外推依据——表 6-0）。
- 帧率采集：release 构建、固定分辨率、窗口化独占口径（vsync 关闭采集 / 记录环境：CPU/GPU/驱动），采样 ≥60s，输出 avg / 1% low / 0.1% low；采集方法（帧时间环形缓冲，非挂钟平均）入档。
- bevy 渲染 API 一律查证 0.19 官方文档/examples 后使用（硬约束 2）。

## 范围外

分层渲染（公告板/点阵——那是 6.1 渲染腿失败后的 M1 关键路径动作）；战斗接入；打击感。

## 验收断言

1. 10,000 胶囊体同屏 @60fps、**1% low ≥ 45**（基准机 A）——达标/未达标如实判定。
2. 未达标路径：触发 6.1 渲染腿后果记录（分层渲染提前进 M1 关键路径 + 常态规模承诺减半）——本卡只产数据与判定，报告级修订归 T013。
3. 降档扫描：1k / 2k / 5k / 10k 四档同口径数据（给 6.1 判定留规模曲线）。
4. 采样原始帧时间数据入档（非仅汇总值）。

## 证据要求

docs/evidence/t010/：四档帧时间原始数据 + 汇总 + 环境 + 命令 + REAL_EXIT。

## 设计裁决（2026-10-05 主会话定稿 D1~D12，开工前冻结）

- **D1 crate 接入**：新成员 `render-spike/`；workspace `[workspace.dependencies]` 增别名 `bevy_full = { package = "bevy", version = "0.19", default-features = true }`（两入口同锁 0.19、Cargo.lock 单条；**不采用 member 覆写 default-features**——T001 实测 member 级声明被 workspace 级忽略；sim 的 minimal 继承入口零变化）；render-spike 代码内 `use bevy_full as bevy;` 保持惯用路径。sim 侧零改动红线（`sim/**` 与 `sim/Cargo.toml` 零 diff；根 Cargo.toml 仅 +members、+bevy_full 两处）。
- **D2 场景**：N 胶囊体 = 共享单 Mesh 资产（Capsule3d，半径 0.5 m）+ 共享 StandardMaterial（灰色）+ DirectionalLight + AmbientLight；位置由 crate 本地 SplitMix64（`--seed` 默认 42，不复用 sim 依赖）在 160 m × 160 m（±80 m）100×100 网格生成：格距 1.6 m、jitter ±0.25 m ⇒ 最小间距 1.1 m > 胶囊直径 1.0 m（不重叠算式入档）；单位 i：gx = i mod 100、gz = i div 100，x = −80 + 1.6·gx + jitter、z = −80 + 1.6·gz + jitter，y = 落地。相机固定 (0, 180, 180) looking_at 原点（覆盖粗算入档：视距 ≈254 m、fov 45° 垂直覆盖 ≈2×254×tan(22.5°) ≈ 210 m > 160 m 场地对角需求，全部单位同屏）。
- **D3 窗口口径**：窗口化 1920×1080 固定分辨率（WindowResolution），PresentMode::AutoNoVsync（vsync 关闭；0.19 API 以 docs.rs 查证为准）；环境档记录实际窗口尺寸、GPU/驱动（nvidia-smi 或 wmic win32_VideoController）、vsync 生效旁证（帧时间分布是否远离 16.7 ms 整倍数锁定）。
- **D4 帧采集**：Update 系统读 `Res<Time>` delta_ns（真实时长、无 Fixed timestep）；warmup 5 s 丢弃（吸收管线编译首帧尖峰）+ capture 65 s（≥60 s 卡面）；环形缓冲容量 240_000 帧（溢出即 panic 上报）；到时发 AppExit，退出前写 raw CSV（frame_idx,delta_ns）至 `--out`。采集方法=帧时间环形缓冲非挂钟平均（卡面原文）。
- **D5 汇总脚本（python 入库，判定行脚本生成禁手算）**：avg_fps = N ÷ (Σft/1e9)；1% low = 1e9 ÷ mean(最慢 ceil(0.01·N) 帧)；0.1% low 同式 ceil(0.001·N)（≥1 帧）；判定行：10k 档 avg_fps ≥ 60 **且** 1% low ≥ 45（表 6-0 验收⑤）→ PASS/FAIL 如实；四档表 1k/2k/5k/10k 同式。
- **D6 降档扫描**：`--units {1000,2000,5000,10000}` 四档同口径（同 seed 同场参数）。
- **D7 冒烟前置**：正式采集前 `--units 100 --duration 5` 冒烟验证窗口可创建/GPU 可用（无交互会话风险——失败 ≤2 次重试即上报，不追查）。
- **D8 门禁**：附录 A 清单全量（`cargo check --workspace -j 3` 0 警告——含 bevy full 冷编，预期 15~30 min，超 ZCode 单命令 10 min 上限 → `run_in_background` 分步取证，B.1 纪律；`cargo test -p sim -j 3` 与 `cargo test -p render-spike -j 3` 全绿；`cargo build -p render-spike --release -j 3` exit 0）。render-spike 单测：SplitMix64 黄金首 8 值（python 双盲独立实现对拍后固化）、网格映射算式自检、环形缓冲、百分位公式构造样例；**窗口/渲染路径不进单测**（冒烟档承载）。
- **D9 bevy 0.19 API 查证纪律（硬约束 2）**：所用 API（Capsule3d 构造与签名、Mesh3d/MeshMaterial3d、StandardMaterial、PresentMode、WindowResolution、AppExit、Time delta、Camera3d）逐条 docs.rs/bevy/0.19 查证，证据档留 URL + 关键签名摘录；禁凭记忆写 API。WebFetch docs.rs 失败 ≤2 次重试后上报降级。
- **D10 预注册预期**：基准机 A（i5-12490F/RTX 3050）+ 10k 静态实例化共享网格（无动画/LOD/分层）→ 预期 PASS（avg ≫ 60、1% low ≥ 45 裕度充足）；若 FAIL → 断言 2 渲染腿后果记录行如实落档（分层渲染提前进 M1 关键路径 + 常态规模承诺减半，报告 6.1 原文），报告级修订归 T013，本卡不粉饰。
- **D11 时间盒 ≤3h；四档采集机器空闲独占（窗口化独占口径）；环境噪声披露。**
- **D12 分工**：worker-2 隔离树 `trees/wanzhen/t010-a` 基线 45a0741；中断处置按项目附录 E；收获后 Lead G1 + plan-code-reviewer 完整轮（量测卡）。
