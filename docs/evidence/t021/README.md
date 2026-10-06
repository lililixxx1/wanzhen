# T021 证据档（配置面：预设 + 参数化 + 入口无关性）

- 派工单：`docs/evidence/t021/dispatch-sheet.md`（Lead，2026-10-06；本档只引用不复制正文）。
- 依据：taskset/t021-config-face.md + 备忘录 §三（预设+参数化）+ T018 D5/D11。
- 实现（隔离树内，基线 = 主仓 4840bc5）：`host/src/presets.rs`（新增，D2 预设注册表）/
  `host/src/rpc.rs`（D1 抽 `ResolvedDeploy` + `apply_deploy` 单一执行路径 + D3 preset 参数 +
  D5 `HostRpcPlugin` port 字段）/ `host/src/main.rs`（D4 CLI 参数化 + auto-deploy +
  D5 `--port`）。sim/ 与 suite.rs 零改动。

## 文件清单

| 文件 | 内容 |
|---|---|
| `preset_smoke.sh` | 配置面冒烟脚本（curl + CLI 直调；30 项判定 CHK-00~CHK-12：BRP preset 三预设 + 覆盖语义对拍 + 同参重放逐位一致 + CLI auto-deploy 入口无关锚对拍 + CLI/BRP 非法路径 + 进程存活收尾；端口 15712/15713） |
| `preset-smoke-run.log` | 冒烟实录（2026-10-06，**30/30 PASS，SCRIPT_EXIT=0**；REQ/RESP 原文全量在档——沿 T018 P0-1 整改体例 `>&2` 落档） |
| `host-stderr.log` / `host-stdout.log` | A 段宿主运行时留痕（`--port 15712`：banner 实际端口 / 回环声明 / 6 方法清单） |
| `host-cli-stderr.log` / `host-cli-stdout.log` | B 段 CLI auto-deploy 实例留痕（`--seed 42 --preset default --port 15713`：`[host] auto-deployed default seed=42 units=60 tick=0` + 监听行 15713） |
| `cli-illegal-comp.stderr.log` / `cli-illegal-preset.stderr.log` / `cli-illegal-threads.stderr.log` | CLI 非法三发 stderr 留档（各 exit 2：未知兵种 / 未知 preset（消息列预设清单）/ threads=0 出域） |
| `README.md` | 本档 |

## 判定行汇总（30 项全 PASS，SCRIPT_EXIT=0）

- **入口无关性三路对拍**（D1/D4 核心断言）：CHK-02 BRP `{"preset":"default","seed":42}`
  deploy_hash = CHK-09 CLI `--seed 42 --preset default` 的 `game.state_hash` =
  **`0xe2706f0b91a2be8e`**（T018 显式构成路径归档锚，字面逐字使用）——预设路径 / CLI
  路径 / T018 显式路径三路同哈希，入口无关性由 `apply_deploy` 单一执行路径构造保证。
- **melee-brawl**（seed 7）：units 60；H1=`0xb82a248ff23515e2`@tick1800；同参重放逐位一致
  （CHK-03 系列）。H1 与基线 host（未改码）同参实测值一致（实现段开工前探针留证）。
- **last-stand**（seed 7）：units 80；`run_to_tick 1800` 收于上限 tick 1800（未提前冻结）；
  `game.outcome` winner=blue（sim `run_battle_with` 上限收束 `resolve_by_hp` 总 hp 判定）、
  alive_red=15 / alive_blue=56 确定性精确值（CHK-04 系列）。**与派工单 D6 第 3 项原文
  预期（灭绝 < 1800）偏离——见下方上报节第 1 条。**
- **覆盖语义**（D3）：`{"preset":"default",…显式 red/blue/lane_len_m}` 与纯显式同参调用
  deploy_hash 对拍一致 = `0x29fee617bafe37e3`（CHK-05）——「显式字段一律覆盖」逐位成立。
- **非法路径**：BRP 未知 preset → -32602（消息列 `presets::names()`）；preset + 未知兵种
  显式覆盖 → -32602（CHK-06 系列：派工单字面无 seed 发 + 带 seed 补发钉住 laser 解析
  路径）；CLI `--comp laser:5` / `--preset nope` / `--threads 0` 各 exit 2 + stderr 留档
  （CHK-11 系列）。
- **banner**（D4/D5）：auto-deploy 行格式逐字（`[host] auto-deployed default seed=42
  units=60 tick=0`）+ 监听行打印实际端口（15712/15713）+ 回环约束声明（CHK-01/10 系列）。
- **进程收尾**：两实例全程存活、收尾杀净、无残留（CHK-07/12 + 脚本外 tasklist 复核）。

## 门禁对账（worker 实现段四门禁，2026-10-06）

1. `cargo check --workspace -j 2`（树根执行，`CARGO_TARGET_DIR` 前缀共享主仓 target）→
   0 警告 0 错误（exit 0）。
2. `cargo build -p host --release -j 2` → 成功（exit 0；产物落共享 target——树根
   `target/release/host.exe` 为门禁 3/4 的运行副本，与该产物同源拷贝，非入库内容）。
3. `bash docs/evidence/t018/brp_smoke.sh`（树根）→ **46/46 PASS，SCRIPT_EXIT=0**（T018
   回归原绿 = deploy 重构零漂移判定门禁；脚本自写的运行日志落 docs/evidence/t018/ 属
   运行副产物，t018 档内文件零人工改动）。
4. `bash docs/evidence/t021/preset_smoke.sh` → 30/30 PASS，SCRIPT_EXIT=0。

## 上报节（待 Lead / review 裁决）

1. **D6 第 3 项预期偏离（last-stand 灭绝 < 1800 不可达）**：派工单原文预期「run_to_tick
   1800 → 冻结 tick<1800（灭绝）、一方存活 0」。基线 host（未改任何代码）BRP 实测
   （2026-10-06，seed=7/lane 60/threads=1）：run 收于 1800 上限、alive_red=15 /
   alive_blue=56、winner=blue（`resolve_by_hp` 上限总 hp 判定）。结构性归因（附录 B.2 ②
   场景可达性）：M0 贴身射程（|dx| ≤ r_i+r_j+0.2 m）+ move 意图向前夹紧不越位
   （sim/src/world.rs:342 move_intent / :733 move_units_with）⇒ 同侧队列（盾兵间距
   0.5+0.5+0.5=1.5 m、民兵 0.4+0.4+0.5=1.3 m）后位始终在前位射程（0.5+0.4+0.2=1.1 m）
   之外，接敌面恒 1v1 漏斗：民兵 9 dmg/20t（无甲克重甲 ×1.5）杀盾兵 ≈280 tick/个、
   盾兵 5 dmg/30t（被克 ×0.67 截断）杀民兵 ≈300 tick/个，1800 tick 内双方远未灭绝。
   接敌本身可达（≤415 tick < 1800，算式注见 presets.rs 模块注释）。另注：该参数表下
   蓝方（人数 + 克制双优）上限判定胜，「以少胜多」为设计原型名、非当前数值表的模拟
   结论（presets.rs 已如实标注；数值表 = T022 平衡回归对象）。worker 开工实测后即上报
   三选项（A 现实断言 / B 调预设数值 / C 弱断言），**暂按 A 实现**（CHK-04 系列按实测
   确定性现实断言）；如 Lead 改选 B 需重定数值并重跑门禁 4。
2. **首轮冒烟 29/30（脚本缺陷非产品缺陷）**：CHK-11f 判定的报错文案以 `--` 开头，裸传
   被 grep 当选项吞掉（`unknown option`）；`grep -e` 显式给模式后重跑 30/30。重跑实录即
   本档 `preset-smoke-run.log`（脚本 `: > "$LOG"` 整档覆盖，首轮日志不在档，教训留本节）。
