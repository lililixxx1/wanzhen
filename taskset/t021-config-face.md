# T021 M5-04 配置面（席位 6）

- 周位：W2~W3（依赖仅 T018，可与 T019/T020 并行）
- 前置：T018
- 类型：代码（轻量审核轮）
- 预估：3h

## 范围内

- `game.deploy` config 面玩家可达参数化（备忘录 §三：**预设 + 参数化**）：构成（六兵种数量）、对局参数（以 sim 公共 API 既有面为准——lane/tick 上限等）、种子。
- CLI 参数化（宿主启动参数，沿 M0 sim CLI 风格——`--comp/--seed` 系）。
- 配置校验：非法值结构化错误（不击穿进程）。
- 核心循环「配置」腿成形：玩家选预设 → 覆盖参数 → 起局（观战腿 = T019、战报腿 = T018 `game.outcome`）。

## 范围外

- **自由布阵编辑**（范围预裁剪逐字）；设置菜单；持久存档（对局经 deploy 参数重载——预裁剪）。
- 挑战预设（T022——本卡只供通用配置机制）。
- sim 侧新参数面（若须 lib 增量——上报开裁决留痕）。

## 验收断言

1. 预设集（含 M0 默认构成）+ 参数化覆盖经 BRP 直调可部署起局（布阵快照哈希返回）。
2. 同参数经 CLI 与 BRP 两路径 state_hash 逐位一致（入口无关性）。
3. 非法配置错误路径结构化、进程存活（≥3 类非法输入用例在档）。
4. check 0 警告；sim 零改动（或附黄金锚零漂移证据）。

## 执行记录

### 设计裁决 D1~D7（2026-10-06 Lead 定稿，全文 = `docs/evidence/t021/dispatch-sheet.md`）

- D1 部署核心抽取 `apply_deploy` + `ResolvedDeploy`（BRP/CLI 单一代码路径——入口无关性由构造保证）；D2 预设注册表 presets.rs（default/melee-brawl/last-stand，不含 seed/threads）；D3 BRP `"preset"` 底座 + 显式字段覆盖；D4 CLI 七参 + 任一配置参数即 auto-deploy（insert_resource 先于 init_resource——bevy_ecs 0.19.1 已存在不覆盖，源码核实）；D5 `--port`（T018 D11 修正案：地址恒回环、端口默认 15702 可覆盖——并行波次隔离需要，留痕 rpc.rs）；D6 证据面 preset_smoke；D7 T018 语义逐字保持（冒烟 46/46 为判定门禁）。

### 执行与审核链（2026-10-06）

- 三层执行：Lead 预备重构（4840bc5：run_tests 套件注册委托 suite.rs——T020/T021 树 pathspec 零交集）→ worker-1 树 t021-a（基线 4840bc5）四门禁全绿 → Lead 收获（f9f30d9）主仓门禁复跑全绿 → **plan-code-reviewer 轻量轮 = 通过（P0=0/P1=0/P2×2/S×3，报告 `docs/evidence/t021/review-plan-code-reviewer.md`）** → P2×2 收口顺手清（presets.rs 行号锚 :114→:118；README 探针宣称改写）。
- **上报裁决留痕**：派工单 last-stand「灭绝<1800」预期系 Lead 场景可达性笔误（B.2 ② 类：只算接敌可达未算击杀速率——1v1 漏斗互磨推演由 worker 完成）；选项 A 批准（数值保持、断言实测现实 blue@1800 15/56）；「以少胜多」为设计原型名非模拟结论、数值表归 T022 平衡回归（presets.rs 注释如实框定）。
- 关键对拍值：三路入口同哈希 `0xe2706f0b91a2be8e`（预设/CLI/显式构成）；覆盖语义对拍 `0x29fee617bafe37e3`；melee-brawl seed7 重放 `0xb82a248ff23515e2`。
- S×3 处置：S-1 备注在档；S-2 派工单红线措辞已纳后续卡口径；S-3 随下卡顺手。
- 耗时（挂钟全计，终填）：≈180 min 全计 W1 10-06（Lead 预备+派工单 ≈35 + worker-1 ≈43（agent 2601s）+ Lead 收获门禁 ≈25 + 审核轮 ≈15 + 收口回写 ≈30 + 树管理等 ≈30）。
