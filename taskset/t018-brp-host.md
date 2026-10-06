# T018 M5-01 BRP 宿主与 game.* 初始方法面（席位 1 headless 形态 + 席位 2）

- 周位：W1 余~W2 初（10-07 起）
- 前置：T017（备忘录定本 v1.0——口径唯一来源）
- 类型：代码（轻量审核轮）
- 预估：6h（含 M5 全量拆卡 T018~T025 的 Lead 规划段，收口时如实分段：拆卡段 10-06 归 W1）
- 审核分级：plan-code-reviewer 轻量轮（diff + 门禁三断言核对 + 抽查复跑）

## 范围内

- workspace 新 crate（render-spike 旁；命名开工裁决，候选 `host`）——bevy 依赖**最小 feature 集 + `bevy_remote`**（`{ workspace = true }` 继承版本锁；不引渲染 feature，避开 bevy full 重编译足迹）。
- `RemotePlugin` + `RemoteHttpPlugin` 接入，**仅回环 127.0.0.1**（备忘录 §五硬约束：BRP 无鉴权、禁止绑定非回环——绑定地址代码 + 运行时输出双留痕）。
- sim 以 bevy Resource 形态挂载：宿主**只调 sim 公共 API**；sim lib 保持零 bevy 纯 lib（若需 lib 增量 API，须设计裁决留痕且不动模拟行为——黄金锚红线）。
- **game.* 初始集 6 方法**（备忘录 §五逐字）：
  - `game.deploy`（config：构成/参数/种子）→ 布阵快照哈希；
  - `game.run_to_tick`（n）→ 推进 n tick（headless 直推；观战模式按帧率节流表现随 T019）;
  - `game.state_hash` → 当前状态哈希（断言锚）；
  - `game.outcome` → 终局四元组（胜负/终局 tick/双方存活/final_hash）；
  - `game.run_tests` → 套件判定面（pass/fail + 断言清单——**判定主体**；本卡先立最小冒烟集，全量随 T020）；
  - `game.screenshot` → 观战截屏（**本卡 headless 桩**：结构化错误「观战模式未启用」，实装随 T019——方法面 6/6 在位，语义分阶段）。
- headless 模式 BRP 直调验证：脚本（curl 或等价）驱动 6 方法全调用，请求/响应原文留档 docs/evidence/t018/。
- `game.run_tests` 冒烟集：同 seed+参数 state_hash 对拍 M0 归档黄金锚（锚值清单开工定）+ 同种子重放一致。
- 环境档（沿 M0 体例）；0.19 API 查证留痕（RemotePlugin/RemoteHttpPlugin 注册形态、BRP 方法注册与响应类型、错误路径实际签名——docs.rs / 官方 examples，沿 T010 api-notes 先例；**禁凭记忆写 Bevy API**）。

## 范围外

- 观战模式（窗口/渲染插件/表现层/相机/HUD）与 `game.screenshot` 实装——T019。
- 断言面全量与断言清单任务化——T020（席位 3）。
- 配置面预设与参数化选择——T021（席位 6）。
- MCP 薄桥（T041 保持冻结、按需解冻——备忘录 §五）。
- game.* 方法扩展（初始集之后 +1 亦须同步计数门禁留痕——席位 2 说明）。

## 验收断言

1. `cargo check --workspace` 0 警告；sim crate 零 bevy 保持（依赖图证据）；宿主对 sim 只 import 公共 API。
2. game.* 6 方法经 BRP 直调全部可达且响应符合备忘录 §五签名语义（证据档留调用/响应原文；screenshot 桩 = 结构化错误不击穿进程——质量红线）。
3. 同种子+同参数序列经 BRP 重放 state_hash 逐位一致，且与 M0 归档黄金锚至少一枚对拍命中（跨线程档 ≥2 档抽查）。
4. `game.run_tests` 冒烟集 pass（pass/fail + 断言清单输出形态在位——判定面骨架）。
5. 回环硬约束留痕（监听 127.0.0.1 代码与运行时双证）。
6. 涉 bevy 的 cargo 构建按 AGENTS.md 并行上限与内存预检纪律（若触发重型冷编，-j 从严 + 前置 commit 预检）。

## 执行记录

- （开工后补：设计裁决 D1~Dn 开工时主会话定稿——crate 命名 / BRP 响应错误码形态 / 黄金锚清单 / 直调脚本形态等）
