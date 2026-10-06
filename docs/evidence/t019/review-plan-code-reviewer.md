# T019 轻量轮审核报告（plan-code-reviewer，2026-10-07）

> 审核对象：T019 收获提交 19c417d（以 dfc4a3c 工作区区分 T023 合并上下文）。审核器按只读纪律以最终消息返回全文（未写文件），本档 = 逐字归档（Lead 转录；文件清单绝对路径去前缀还原为仓库相对路径）。

### 合并裁决

**不通过（P0；修后复审）**。指定冒烟复跑为 54/54，跨模式哈希一致；但六兵种形状的可见性验收没有被随档截图证明：全场镜头下各兵种只能看到近似小点，不能辨认六种形状。

### 审核范围

审查基线为提交 `19c417d`（T019 收获），并以 `dfc4a3c` 工作区状态区分后续 T023 合并上下文。已加载项目规则 `AGENTS.md` 和通用审核规则 `review-protocol/rules/default.md`。另对照任务卡、派工单、证据 README 与 0.19.1 API 查证记录。

- 已审：`host/Cargo.toml`、`host/src/main.rs`、`host/src/rpc.rs`、`host/src/spectate.rs`、`host/src/present.rs`、`host/src/hud.rs`。
- 已审：T019 的 README、API 注记、冒烟脚本、门禁日志、实例日志及两张截图；已核对锁文件未随 T019 改动、`sim/` 与 `suite.rs`/`presets.rs`/`challenges.rs` 无差异。
- 带理由的跳过：t018/t020/t021/t022 并入提交的其他任务回归副产物 8 件——仅抽查改动性质与 PASS/退出码，不作为 T019 实现审查对象。
- 覆盖率：共 36 个提交文件 — 已审 28 / 跳过 8（理由如上）/ **77.8%**。

### 优点 (Strengths)

- `host/src/rpc.rs:458` 将 headless 与观战推进复用同一个 `advance_ticks`，灭绝检查与逐 tick 推进也沿用同一顺序；独立复跑的终局和手动跨模式哈希均逐位一致。
- `host/src/present.rs:256` 的表现更新只读取 `Res<HostedGame>`，写入限于表现组件；同 x 排队仅计数且不迭代 HashMap，符合模拟态确定性约束。
- `host/src/present.rs:118` 建立共享兵种网格与阵营材质；死亡单位隐藏而非反复销毁；重布阵通过 generation 重建实体池。
- `host/src/main.rs:307` 分开装配 headless 与窗口形态；独立冒烟通过，T019 自身 headless 路径仍保留原来的插件形态与桩行为。
- T019 未改 `Cargo.lock`，`host/Cargo.toml:12` 继续继承 workspace 的 Bevy 0.19 依赖。截图请求、PNG 魔数检查和归档哈希可复核。

### Critical（必须修复，阻塞合并）

- **置信度 97 | 六兵种形状的可见性验收没有成立 |** `host/src/present.rs:213-224`、`docs/evidence/t019/README.md:92-96`。默认 lane 约 1000m、兵种网格约 0.8m，镜头按全场宽度取景，模型在归档截图中只有约 2 像素量级，画面实际呈现为小点；无法从 `shot-2-sixkinds-deployed.png` 辨识 Sphere、Cuboid、Cone、Capsule、Torus 与 Cylinder。任务卡验收 5 要求六兵种形状与阵营色「可见」，README 也把该图作为形状人证，因此实现映射正确不等于验收可见。**建议**：补充能辨识轮廓的近景/局部证据，或在不改变模拟态的前提下提供可读的表现尺度；更新验收截图并用目检确认六种形状可区分，再复审。

### Important（强烈建议修复）

- **置信度 89 | 截图状态可能把旧文件误认成本次成功 |** `host/src/spectate.rs:59-64`、`host/src/spectate.rs:239-253`、`host/src/spectate.rs:306-315`。进程重启后 `next_id` 从 0 开始，路径会重用 `screenshot-0.png`；`on_captured` 只要收到捕获事件便标记 Captured，而 Bevy 0.19.1 的 `save_to_disk` 在写盘失败时只记录错误（API 查证见 `docs/evidence/t019/api-notes.md:78-90`）。若同路径残留先前有效 PNG、而本次保存失败，`poll_screenshot` 在 `host/src/spectate.rs:258-285` 只校验路径上的 PNG 魔数与字节数，可能返回旧文件作为本次 captured。**建议**：为截图生成不会跨进程复用的唯一文件名，并确保响应只在确认本次输出文件存在且校验通过后报 captured。

### Minor（可选，nitpick）

无。

### 门禁与复跑

独立执行 `bash docs/evidence/t019/spectate_smoke.sh`，结果 **54/54，`SCRIPT_EXIT=0`**。`0x26c77d5372dc7ad0` 终局哈希、`0xcdf3fef834f172ce` 手动 300 tick 哈希均与 headless 对拍一致；三个测试端口均释放。两个 PNG 可读取，归档 SHA-256 与 README 一致。提交附带门禁日志记录最终 check 通过、release 构建成功；复跑期间未执行 cargo 命令。

---

## Lead 处置记录（转录后回填，2026-10-07）

- 报告归档：本档（审核器最终消息逐字转录）。
- **整改派发**：P0 修复路线 = 近景证据补充（审核建议二选一之 A）：短 lane 场景（六兵种 × 双阵营、lane 60m——全场镜头下单位 ≈25px 可辨形状）新增验收截图 + 视觉核验（vision-reader 子代理识图留证）——不改模拟态、不动相机裁决；P1 修复 = 截图文件名跨进程唯一（启动 nonce 前缀）+ captured 仅在本次输出文件校验通过后报。整改人 = worker-1（原卡执行者，树 t019-a 续做——pathspec 与 T023 收获零重叠）。整改后修后复审。

---

## 修后复审（2026-10-07，plan-code-reviewer——首轮整改后复审，报告原文逐字转录）

### 合并裁决

**修后可合并。** P0 可见性缺口已由近景截图和视觉核验补齐；P1 的「捕获事件不等于落盘成功」已修复，但毫秒级 nonce 仍存在跨进程碰撞边界，不能严格保证唯一。

### 复审要点（摘）

- P0：present.rs 六兵种→网格槽映射与创建顺序一致核验；近景场景不改表现相机/模拟代码（smoke :326-330 部署 12 单位 lane 60m + 断言 tick=0）；shot-3 sha256 实测与记录一致（4a447ad1…）；视觉报告如实注明 swordsman 尺寸+排除法辨认与 HUD 第三行缺字形框，未掩饰证据边界——验收判定成立。
- P1：observer 只标记 event_arrived（spectate.rs:402-415）；captured 仅文件校验通过后置（:338-355）；冒烟段 5 实际注入有效旧 PNG + Windows 独占句柄，记录锁持有时 pending、释放后 stale leftover suspected。
- 回归证据核验（归档）：check 无警告/错误 EXIT=0、release REAL_EXIT=0、spectate 88/88、t018 46/46。

### Critical

无。

### Important（置信度 86 | 毫秒 nonce 不能保证跨进程唯一）

nonce 只取进程启动毫秒而 next_id 每进程从 0 起——同毫秒双进程同目录时构造相同 `screenshot-{nonce}-0.png`，并发时他进程文件可能满足 mtime 检查被误认。建议强随机唯一标识或原子分配 + 双进程测试。「不推翻常规流程下的 P1 修复」。

**最终裁决：修后可合并。P0 已关闭；P1 落盘核验已关闭，毫秒 nonce 极低概率碰撞仍建议修复。**

---

## Lead 终处置（2026-10-07，Important 86 闭环）

- **修复 = 文件名加 pid 段**：`screenshot-{pid}-{nonce}-{id}.png`（spectate.rs 四处注记 + ScreenshotLog 增 pid 字段 + 构造点三段）——**活进程 pid 全机唯一**，恰好封死复审指名的「同毫秒双进程并存」场景（两者 pid 必异）；残余仅剩「pid 退出复用 + 同毫秒 + 同目录」三重叠加，且 mtime ≥ 受理时刻核验仍兜底。表现层非模拟态（确定性红线不涉）。
- 冒烟同步：三段 sed 模式（A/B/INJ 三处）+ **新增 CHK-26c2-shot-pid-live-nonzero**（pid 段非零且同进程一致——跨进程唯一性构造证明）。
- 复跑门禁全绿：check 0 警告 + release 9.38s + **spectate_smoke 89/89**（含新断言）+ t018 46/46。
- **T019 终态：修后复审「修后可合并」+ Important 86 同日闭环 = 通过。** HUD 第三行 CJK 缺字形框（视觉核验发现）= 备忘录「内置默认字体」口径内已知显示限制，如实留痕不阻断（候选改善挂 M1，非验收判据）。
