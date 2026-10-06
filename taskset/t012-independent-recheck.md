# T012 · M0-11 独立复算 ★ 完整审核节点 2（收官序列第 2 步）

- 周位：W3
- 前置：T011
- 目标：plan-code-reviewer 只读独立复算 M0 全部验收数据与判定——不接受「执行者自己写、自己跑、自己交」作为验收证据（沿既有审核纪律）。

## 范围内（派工对象：plan-code-reviewer）

- 只读独立复跑：量测套件抽查 ≥3 配置（含一个 50k 采样点）、确定性对拍抽查 ≥2 组合、渲染 spike 复测 1 档、吞吐复测 1 轮。
- 判定核对：六项验收判定与原始数据复算一致；止损换算 6.1 过程复核；口径混用检查（降规模/全规模、单线程/多线程、含 AI/不含落库）。
- 裁决：通过 / 有条件通过 / 不通过 + 发现清单（P0/P1/S/N 定级沿既有惯例）。
- 必改项闭环由主会话派工落实，复跑确认后闭环。

## 验收断言

1. 审核报告入档 docs/evidence/m0/review/（含独立复跑命令与输出摘要）。
2. 裁决明确；若有必改，闭环记录（返工如实入台账）。
3. 复算与自报数字的偏差全部披露（容差内/超差）。

## 证据要求

审核报告自含（复跑命令 + 输出 + 裁决行）。

## 开工裁决（2026-10-06，Lead 冻结；派工单 = docs/evidence/m0/review/dispatch-rv.md）

- **D1 复跑清单冻结（超卡面最低 ≥3/≥2/1/1）**：bench 新鲜复跑 4 配置（10k-t1-300t / 10k-t12-300t / 50k-t1-30t / 50k-t12-30t，seed42 默认 warmup1/repeats5）；确定性新鲜复跑 2 矩阵局（red200-th1-s42 + full10000-th12-s44，对归档 stdout 逐字节 diff）+ 1 局 CLI 黄金交叉（t009 r5-sim-t1）；渲染复测 1 档（t10000，65s 采集）；吞吐复测 1 轮（512×12t×repeats3）。理由：T015 后全档秒级~12s，单条成本低，锚覆盖 ①②③④⑤。⑥ 不重跑（卡面未列；无内存行为变更沿 T015 D10 预登记）。
- **D2 二进制出处**：reviewer 在主仓 HEAD b1b72e7 重建 sim 包（`cargo build -p sim --release -j 3`，沿 T011 D8 纯复算卡替代 --workspace check 先例）并记录 sim/bench/arena 三 bin sha256；render-spike.exe 由 Lead 预先冷编（-j 2 从严、CommitFree 14.9G≥12G 预检过），reviewer 核验存在性 + sha256 + `git diff 1d304a8..HEAD -- render-spike/` 为空（源零变更证明；基线 = T010 收口提交 1d304a8，5ab440b 是 T010 之前的基线不可用——Lead 派工单自查修正留痕）。二进制 sha 与历史档不一致时如实披露 provenance 注记（源经多卡演进属预期）；确定性锚一律以 final_hash 为准，不以 bin sha 为准。
- **D3 容差政策（预注册）**：bit-exact 类零容差（终局哈希/采样列/CLI 黄金/stdout diff/compare_matrix 与 bench --summarize 再生成 diff）；timing 类复跑判定 = 验收阈值独立判定（µs≤1/≤2、games/h≥10,000、fps≥60/≥45）+ 相对漂移全披露（容差内/超差逐条）；漂移容差带 ±25%（依据 = T015 复测最大正当漂移 23.8% 先例，docs/evidence/t015/README），超带即加样复测 2 次全档披露（沿 T015 处方）；② 新鲜加速比带 0.8~1.3×（结构性 ≈1.0 判定复现），② 判定以 <4× 为准。
- **D4 判定核对与复算集（独立实现，不复用汇总脚本数字）**：① 从 t015/matrix.jsonl samples_ns 重算 median/us_per_unit_tick（median 语义先读 sim/src/bin/bench.rs:923 后独立实现）；② 加速比/16t 外推/C(100000)÷speedup16 按 bench.rs:1193-1348/1361+ 独立重算；③ 从 t009/runs/throughput_t12/throughput_t12.json 重算 games_per_hour@12t 与 16t OLS 外推；④ compare_matrix.py 对归档 runs 再生成（out 重定向，exit 0 + 与归档 matrix.md diff 预期空）+ 2 局新鲜 byte-diff；⑤ frames.csv 独立 python 重算 avg/1%low + summarize.py 单档再生成（exit 3 = 缺档**预期**，t10000 判定行仍须产出）；⑥ 档内核对（n=134 / 670.6s / 高水位口径行 670.6/60=11.18）；6.1 换算表四行×两预算复算；W1 投入 tally 复算（557/1246）+ tally_hours.py 与 verify_claims.py 重跑（stdout 重定向到 review 目录，禁覆写归档）。另：`bench --summarize docs/evidence/t015/matrix.jsonl --out <review 目录>` 再生成与归档 t015/summary.md diff 预期逐字节空（档无时间戳）。
- **D5 口径混用检查**：表 6-0 原文（报告 :485/:490/:498）vs m0/README §0~§8 + t007/t009/t010/t015 summary 判定行 + AGENTS.md 状态节——逐验收核对「降规模/全规模、单线程基线/多线程、含镜像 AI 决策/不含落库」三对口径无混用，产出逐项清单表。
- **D6 报告落位**：docs/evidence/m0/review/report.md + runs/（一命令一档：命令全文 + 原始输出 + REAL_EXIT）；零机器绝对路径（附录 G 可公开态）。
- **D7 定级与裁决词汇**：P0（伪造/缺失证据/判定失实）/ P1 必改 / P2 次级必改 / S 建议 / N 注记；输出五字段（结论/已核对/阻塞项/最小修复指令/复验命令）。
- **D8 写范围与禁改**：仅新增 docs/evidence/m0/review/** 文件；禁改一切既有文件、禁 git 写操作、禁并行 cargo/量测（串行 + 每 timing 批前空闲预检留痕）。
- **D9 双盲纪律**：派工单锚值仅供核对，reviewer 必须从源档独立取值对拍，不一致即停上报（附录 B.2⑤）。
- **D10 时间盒 90 min**（超盒上报不默认续做）；本卡挂钟全计 W1（10-06 窗口内）。
- **D11 必改闭环**：P0/P1/P2 由主会话派工或接管落实并复跑确认，reviewer 复审关账；S/N 移交 T013（报告 V1.0 修订吸收）。
- **D12 环境留痕**：report.md 含环境要素行（rustc/cargo 版本、git HEAD、五 bin sha256、量测窗口空闲声明指针）。

## 条件账（G2 审核轮开出，编号 C-xx；上限 3 条）

- **C-1（P1-1，验收⑤ 新鲜复测不可复现——带载环境）**：T012 新鲜渲染复测 3 样本（+ 前轮遗留旁证 3）avg 59.41~64.10 / 1% low 22.16~23.87（漂移 -79%~-89% 全超 ±25% 带，判定行 FAIL），但承测窗口非空闲独占（precheck 实录 SLDWORKS/AutoCAD 系/钉钉/uTools，CPU 7~38%）且 meta `window_resolution_actual="0x0"`（归档/G1 均 2400×1350）——按附录 A「共享负载下红绿均不可信」，本轮红**不可推翻亦不可确认**归档 PASS（归档三实现复算自洽 + G1 fresh 329.33/177.95 PASS 双佐证）。
  - **验收方式（关账闸门 = T013 开工）**：① 届时取得真空闲独占窗口（precheck 无用户负载 + window_resolution_actual=2400×1350 可复核）→ 补跑一次 t10000（命令 = report.md §0 复验命令 1）重判 ⑤；② 若无干净窗口 → T013 修订稿写入条件性披露「⑤ 归档 PASS 依赖干净量测窗口（2026-10-06 两轮 6 样本带载均不可复现）」且渲染腿后果路径（报告 :525）引用一律带「仅 ⑤ 最终判 FAIL 时触发」限定语（S-3）。
  - 状态：**未关账**（2026-10-06 机器带载，补测不可行；C-1 随 T012 交付移交 T013）。

## 执行与收口记录（2026-10-06）

- **执行形态**：派工对象 = plan-code-reviewer（本卡即审核卡，按卡面）；两轮执行——第一轮因用户「暂停一下」中止（遗留 runs 档 10:04~10:29 保留为旁证、报告 §2.2 已降级引用，N-1；G1 裁定保留不删）；第二轮完整执行 ≈44 min（10:42 起，-rv2 零碰撞命名）。
- **裁决**：**有条件通过**（P0=0；①②②附则③④⑥ + 6.1 + W1 + 口径混用 D5 全部独立复算逐位通过；完整报告 = docs/evidence/m0/review/report.md）。
- **必改闭环**：P2×2（派工单 speedup16 锚值取错档位 / 附录 A 行号 off-by-one）= Lead 派工单缺陷，勘误已追加 dispatch-rv.md 修正留痕节，本提交闭环；P1×1 = C-1 条件账（关账闸门 T013）。S×3（AGENTS.md ③ 补「降规模」标签——本卡随收口顺手落地；5a 型再生成预注册措辞、渲染腿后果引用限定语）→ 移交 T013。N×5 处置见报告 §7 与 g1-lead-recheck.md §2。
- **G1（Lead）**：通过——独立抽查复跑 1a 逐字节一致 + 十项证据抽验逐位相符（docs/evidence/m0/review/g1-lead-recheck.md）。
- **验收断言核对**：① 审核报告入档 docs/evidence/m0/review/（report.md + runs/ 一命令一档含 REAL_EXIT）= PASS；② 裁决明确（有条件通过 + 发现清单 P0=0/P1×1/P2×2/S×3/N×5）+ 必改闭环记录（P2 本提交、P1 条件账 C-1）= PASS；③ 偏差全部披露（report §6 预注册偏差表：bench +2.11%~+19.22% 带内、吞吐 -21.39% 带内、渲染 -79%~-89% 超带全披露、5a argv 行机械差异降级排查）= PASS。
- **时机注记**：机器在 reviewer 执行窗口被用户占用（CAD 类负载）——①②③④ 的 timing 复跑均在带载下取得但仍全数带内/达标（阈值判定裕度 9×~110×），bit-exact 项不受影响；⑤ 因 GPU/呈现路径敏感成为唯一实质受害者（C-1）。
