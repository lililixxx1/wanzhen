# 万阵 (Project Myriad)

米拉奇式万人大战自动对战策略游戏——战前排兵布阵，战中万军自动交锋，战后一枚种子码完整重放与分享。

> **试金石，不是产品野心**：本项目是 [bevy-ai-workflow](https://github.com/lililixxx1/bevy-ai-workflow)（AI 长期驻场 Bevy 游戏开发工作流）的选品试金石，用于逼出真实约束、喂真实教训。M0 预验证（六验收实测判定）+ M5 可玩性（配置-观战-实验三合一）两阶段交付后，2026-10-07 owner 拍板收官归档（决策记录见 [`docs/m5-identity-decision.md`](./docs/m5-identity-decision.md)）。归档态冻结 Bevy 0.19.1 不再升级，无新开发计划；重启 = 新立项。

## 仓库结构

- `sim/` — headless 确定性模拟核心（六兵种克制 / 两阶段并行更新 / 黄金锚回归）
  - `sim` bin：对局入口（`--comp` 布阵 / `--ticks` / `--seed` / `--threads` / `--hash-samples` / `--battle` 终局）
  - `bench` bin：量测套件（µs 成本 / 加速比 / 长跑内存，判定行脚本生成）
  - `arena` bin：平衡实验场（两层胜率矩阵 / 吞吐 / 全规模抽样）
- `host/` — BRP 宿主（Bevy MinimalPlugins 60Hz，JSON-RPC over HTTP 回环 `127.0.0.1:15702`，`game.*` 7 方法：deploy / run_to_tick / state_hash / outcome / sample_outcomes / run_tests / screenshot）+ `--spectate` 观战形态（1920×1080 表现层 + HUD，三挑战预设 few-elite / counter-militia / iron-wall）
- `render-spike/` — 渲染灰盒 spike（窗口化帧采集，万级单位 @60fps 实测）
- `docs/` — 策划报告（V1.0 实测修订，M0 六验收判定表）+ 逐任务证据档（`docs/evidence/`）
- `taskset/` + `task-ledger.md` — 逐任务拆卡与台账（T001~T028 全记录：类型 / 一次通过 / 返工 / 耗时 / 原因）

## 快速上手

```bash
cargo check --workspace -j 3                       # 门禁：0 警告
cargo run -p sim --release -- --battle --ticks 1800                 # 默认布阵一局（终局判定）
cargo run -p sim --release -- --comp swordsman:100,militia:900 --ticks 3600 --seed 42
cargo run -p sim --release -- --units 10000 --threads 12 --ticks 1800 --hash-samples 0,900,1800
cargo build -p host --release -j 2                 # BRP 宿主（依赖较重，-j 2 从严）
./target/release/host.exe --preset melee-brawl --spectate           # 观战形态（窗口）
```

host 冒烟（BRP 判定面）：`bash docs/evidence/t018/brp_smoke.sh`（46 判定）/ `bash docs/evidence/t022/challenge_smoke.sh`（37 判定）。

## 核心文档

- [`docs/万阵-游戏前期策划报告.html`](./docs/万阵-游戏前期策划报告.html)——最高上下文（V1.0：范围 / 里程碑 / 验收阈值 / 确定性纪律 / 表 6-2 M0 六验收实测判定）。
- [`docs/evidence/m0/README.md`](./docs/evidence/m0/README.md) / [`docs/evidence/m5/README.md`](./docs/evidence/m5/README.md)——M0 / M5 验收判定档（口径唯一来源）。
- [`docs/m5-identity-decision.md`](./docs/m5-identity-decision.md)——收官归档决策记录（含分项处置与复议机制）。
- [`AGENTS.md`](./AGENTS.md)——仓库硬约束（版本锁 0.19 / 确定性纪律 / 量测口径）与常用命令。

## 纪律亮点

- **确定性纪律**：模拟态禁 HashMap 迭代序（Rust RandomState 每次运行不同）、浮点归约按固定索引序、跨单位影响两阶段更新（结果与线程数无关）、超越函数禁入模拟态；黄金锚（同种子状态哈希逐位一致）全量回归。
- **三层协作**：主会话规划监管 + 双 worker 隔离树执行 + 独立审核轮的三层架构；每卡过审、分轻重（代码卡轻量轮 / 节点量测卡完整轮 / 收官独立复算轮）。
- **证据自含**：逐任务证据档「命令 + 原始输出 + REAL_EXIT」同档保存，可独立复跑；判定行由脚本计算生成防手算漂移；量测预注册、复测超标即加样全档披露。
- **保持可公开态**：零密钥 / 受管面零机器路径（四变体 grep -F 终扫在档）/ 全历史统一 bot 身份。

## License

MIT OR Apache-2.0 双许可，任选其一：[LICENSE-MIT](./LICENSE-MIT) · [LICENSE-APACHE](./LICENSE-APACHE)。向本仓库提交的贡献默认按同一双许可授权，无需附加条款。
