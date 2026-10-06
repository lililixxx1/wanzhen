# T011 selfcheck — 宣称-证据对齐自查记录（派工单 §6；树 t011-a · 基线 1d304a8）

- 生成：2026-10-06T08:52+08:00 · worker-1
- 对象：docs/evidence/m0/README.md（下称「本档索引」）+ tally_hours.py + verify_claims.py

## §1 verify_claims.py 全 PASS 声明

- 运行：`python docs/evidence/m0/verify_claims.py`（树根执行）→ 退出码 **0**，输出落 runs/verify.txt。
- 终态复跑（README §9 回填时刻戳修正后再跑）：`汇总: 81 条 | PASS 81 | FAIL 0`，REAL_EXIT=0（2026-10-06T08:50:23+08:00）。
- CLAIMS 覆盖：派工单 §2 清单 A~P 全部锚子串（A 4 + B 4 + C 4 + D 1 + E 1 + F 1 + G 1 + H 1 + I 2 + J 7 + K 6 + L 2 + M 1 + N 4 + O 1 + P 2 = 42 条）+ 引用点扩充 39 条（口径句 / 归因 / 复现命令 / 披露行 / 报告 html 片段 / AGENTS.md 门禁行）。
- 双向核对语义：每条 needle 在 source_relpath 命中 ∧ 在 docs/evidence/m0/README.md 命中（子串级，utf-8）。
- 锚行号核对：派工单 §2 所附行号（Lead 写单时实测）与树内源档 grep -n 复核结果**全部一致、零偏差**（A=t015 L18-21 / B=t007 L18-21 / C=L59-62 / D=L42 / E=L68 / F=L42 / G=L49 / H=L49 / I=L180-181 / J=L135-144 / K=L84-89 / M=L31 / N=L11-14 / O=L77 / P=L72-73）。

## §2 措辞自查

- 「约」字扫描：本档索引命中 2 行——
  - L169（§3 口径段）：「全规模对局按止损线**约** 530 场/小时」= 报告 html:538 R2 条款**逐字引用**（引用原文自带字符，非本档验收宣称；该句判定对象是公开平衡报告降频的 R2 终止判据，非 M0 六验收数字）。
  - L421（§9）：「无『约/大概』级措辞」= 本自查声明的元表述（被禁词提及）。
- 「大概」扫描：命中 1 处 = 同 L421 元表述。验收级数字上下文零命中。
- 验收级数字措辞：本档索引全部验收级数字均为逐字引用（代码生成档）或算式呈现（§6 换算 670.6/60=11.18、§8 投入小计 458/1205 与 7.63/20.08 h）；本档自身措辞无「约/大概」修饰验收数字。
- 「≈」合法性边界：仅投入耗时口径（台账分针近似，task-ledger.md 纪律行原文）与**引用原文**中自带字符（如 t015/README「≈ 3.9×」「≈8.9%」）出现；不用于本档验收宣称。

## §3 AGENTS.md「当前状态」节验收数字抽查（grep -F 对照，树内基线 1d304a8 副本）

| # | 抽查项 | AGENTS.md grep -cF | 源档 grep -cF | 结论 |
|---|---|---|---|---|
| 1 | `333.15` | AGENTS.md=1 | t010/summary.md=2（判定行+四档表） | 一致（字面） |
| 2 | `202.48` | AGENTS.md=1 | t010/summary.md=2 | 一致（字面） |
| 3 | 吞吐数字 | `1099638.6`=0；`1,099,638.6`=0；`1,099,639`=1 | t009/summary.md `1099638.6`=3 | **字面不一致、取整一致**：AGENTS.md 作「1,099,639 场/h」（整数四舍五入形态），源档代码生成判定行为 1099638.6；AGENTS.md 另称「≈110× 裕度」= 1099638.6/10000=109.96 取整，方向一致。AGENTS.md 为状态摘要档、本卡禁改——是否回改精度/补小数，**上报 Lead 裁决，不自行处理** |
| 4 | `0.061/0.072/0.071/0.109`（0.061 系） | AGENTS.md=1 | t015/summary.md 四条判定行 0.060651/0.072196/0.070617/0.108681 各命中（`0.060651`=4） | 一致（三位小数取整：0.060651→0.061、0.072196→0.072、0.070617→0.071、0.108681→0.109） |

## §4 环境总档五档一致性观察（t007/t008/t009/t010/t015 environment.txt）

五档同值（逐字段比对）：

- CPU：12th Gen Intel(R) Core(TM) i5-12490F，6 核 / 12 逻辑处理器 / Max 3000 MHz —— 五档一致。
- RAM：TotalPhysicalMemory 34187943936 bytes；32 GiB 3600 MHz Kingston —— 五档一致。
- OS：Microsoft Windows 10 IoT 企业版 LTSC 10.0.19044（Build 19044）64 位 —— 五档一致。
- rustc：1.98.1 (48a229cea 2026-09-01) —— 五档一致。
- cargo：1.98.1 (797e8a9bc 2026-08-05) —— 五档一致。
- rust-toolchain.toml：channel = "1.98.1" —— 五档一致。
- bevy lock：0.19.1 —— 五档一致。

差异（如实列，不合并不择优）：

1. GPU 采集行：t010 档多一行 nvidia-smi 旁行（RTX 3050, 537.58）；五档 WMI 行同值（RTX 3050 31.0.15.3758 + GameViewer Virtual Display Adapter 15.6.5.199）。记法差异（WMI DriverVersion 与 nvidia-smi 版本号两种记法同卡），非驱动漂移。
2. rust-toolchain 呈现：t008 附文件全文三行（含注释）；其余四档为摘要行。详略差异。
3. bevy lock 注释措辞：t010 为「sim 经 'bevy'(default-features=false) 最小面、render-spike 经别名 bevy_full 全量同锁」；其余四档为「代码路径零 bevy API 参与」类注。版本同值 0.19.1。
4. 构建 profile 节：t007/t009/t015 档含自证节（[profile.release] 仅声明 debug=false，opt-level/lto 默认）；t008/t010 档无此节。
5. git HEAD 字段：仅 t008 档含（b2dad10a438d295bdcfbdfd5cfd7bce0d5a8b917）；t009/t010 基线 commit 记于各自 README（45a0741 / 5ab440b）；t007/t015 档内无 HEAD 字段（t015 树基线 1c4dd87 记于 README）。采集器（复用 T007 collect_environment.ps1）字段集不完全一致所致，m0/README.md §7 分卡指纹表已按档内实有分行列出。
6. 二进制指纹：五档各自记录不同 exe 与不同时点状态（bench.exe 三态：D8E251…/09971d…/80ebcc…；sim.exe 两态；arena.exe；render-spike.exe）——各卡产物与时点不同所致，非环境漂移；t007 三态链在 t007/README.md §5 留痕。

## §5 执行事件留痕（如实）

1. **双盲拦截实例（制度生效）**：tally_hours.py 首版按「整行首个 ≈N min」解析，误抓 T010 行任务列字样「≈92 min 零中断」（耗时列真值 140）→ 输出 W1=1157 min ≠ 手工速算 1205 min，按纪律停止核对；定位为解析口径偏离派工单 §3「**耗时列**中首个匹配」，改为第 6 列（未转义竖线切分，兼容 T004 行 `\|dx\|` 转义）后两值一致收敛（458/1205）。台账零改动。留痕：runs/tally.txt 第三段 + 脚本 docstring。
2. **首条门禁误落主仓（cwd 重置事件）**：第一次 `cargo build -p sim --release -j 3` 执行时 shell 工作目录被宿主重置回主仓（编译输出路径显示主仓 sim），未在树内产生门禁产物；随即核实主仓 HEAD=1d304a8（与树基线一致）且 status 跟踪面改动均为 Lead 既有内容（taskset/t011-data-pack.md 修改 + docs/evidence/m0/ 派工单目录），本次误跑零跟踪文件影响；随后在树根重跑同一门禁命令成功（REAL_EXIT=0，1m13s，编译路径确认为树内 sim）。**同一 cargo 门禁在树内仅此一条执行，未追加其他 cargo 命令。**
3. 派工单锚子串与源档**零不符**（42 条锚 + 39 条扩充全部双向命中）；派工单所附行号零偏差。
4. 抽查三档全部一次通过（无重试）：档 a diff 空、档 b/c 哈希 MATCH、REAL_EXIT 全 0。
