# T024 派工单（M5-07 观战档性能验收与 C-1 关账，席位 9，★完整轮量测）——Lead 设计已定，worker 执行

> 任务卡：`taskset/t024-perf-c1.md`。基线 = 派发时 master HEAD。
> 附录引用：`team-prompt/PROJECT-APPENDIX.md`（主仓相对路径）附录 A（量测窗口纪律/门禁）、B.1/B.2、D。
> **排程前提：本卡在全部并行卡收口后由 Lead 独占窗口派发**——量测机器空闲独占（表 6-0 纪律），预检声明 + 负载实录强制。

## 0. 结论先行

两条腿：**A 观战档验收**（spectate 10k 帧采集 → avg ≥60 / 1% low ≥45 判定行脚本生成）+ **B C-1 关账**（同窗口 render-spike t10000 干净复测，红绿如实）。插桩面已定（D1），量测口径逐字沿 T010 先例。

## 1. 设计裁决（D1~D7）

- **D1 帧采集插桩（唯一代码面）**：spectate 形态新系统 `frame_capture`——**沿 render-spike `capture_system` 体例逐字移植口径**（`render-spike/src/main.rs:8-11` + `capture.rs`：`Res<Time>::delta()` 每帧入环形缓冲，warmup **5s 丢弃** + capture **65s 满** → 写 `frames.csv`（逐帧 delta_ns 一列）+ `meta.json` → `AppExit::Success` 自然退出）。CLI：`--frame-capture <dir>`（**仅 `--spectate` 下合法**——无 `--spectate` 给此参 → exit 2 对齐 CLI 体例；缺省关）。**红线：headless 形态零行为变化**（插桩系统只挂 spectate 形态插件集；七组既有冒烟 46/19/30/30/37/52/54 全绿不回归）。sim/ 零改动。
- **D2 量测场景**：`host --spectate --comp <10k 构成> --seed 42 --max-ticks 3600 --frame-capture <dir>`（autorun 自走至终局或 capture 满自退）。**10k 构成 = sim 默认构成等比缩放至每方 5000**（等比算式与默认构成源行号注档；总数 10,000 = 备忘录「万人常态 10k」字面）。`--max-ticks 3600`（= 30Hz 下 120s > warmup 5s + capture 65s——保证采集窗内战局存活非冻结态；1800 会在窗尾 5s 落入冻结，3600 避开）。
- **D3 判定脚本**：`docs/evidence/t024/summarize_t024.py`——**口径逐字沿 `docs/evidence/t010/summarize.py`**（avg_fps = N/(Σdelta_ns/1e9)；1% low = 1e9/mean(最慢 ⌈N/100⌉ 帧)；0.1% low 同理 ÷1000；阈值常量 avg ≥60 / 1% low ≥45 注源 = 验收⑤/T010 先例）。判定行脚本生成禁手算；输出 summary.md（表格 + 判定行）。
- **D4 C-1 关账腿**：同干净窗口：① `cargo build -p render-spike --release -j 2`（涉渲染从严 -j 2 + 前置 commit 预检 ≥12G，不足上报）② `./target/release/render-spike.exe --units 10000 --seed 42 --warmup-sec 5 --capture-sec 65 --res 1920x1080 --out <dir>` ③ `python docs/evidence/t010/summarize.py`（复用原脚本零改动）→ 判定行**如实红绿**（C-1 关账语义 = 干净窗口实测入档——T012 P1-1 带载不可复现账的正式闭合，非必须翻绿）④ **m0 侧指针回写**：`docs/evidence/m0/README.md` 追加 C-1 关账行（指向本档，注明 T013 条件性披露的最终归宿——worker 写本档、m0 侧行由 Lead 收口段回写或 worker 按本单附录留痕格式预写，二选一由 worker 上报定）。
- **D5 干净窗口纪律（强制）**：三段留痕——① 跑前进程快照（tasklist Get-Process 全量落档）+ 「无并发负载」声明；② 量测中每 20s 一次负载抽样（CPU/CommitFree）入档；③ 结束复扫。窗口内**禁**任何 cargo / 其他冒烟 / host 实例（构建腿 D4① 在采集腿之前完成并留空档）。预注册声明 = 附录 G 体例。
- **D6 口径注与预注册**：① 灰盒边界注（残余账 #9：无动画/LOD/特效，指标不作 M2/M3 外推依据——沿 T010 口径）② 预注册预期（方向性）：**PASS 预期**（T010 归档 render-spike t10000 avg 333.15 / 1% low 202.48——spectate 版叠加 HUD/表现映射/宿主循环，裕度收窄但仍应远超阈值）；**不设数值区间**，只注册方向 + 阈值；实测偏离如实披露禁事后改口。③ 观战帧率 ≠ 模拟吞吐（表 6-0 口径分立注）。
- **D7 环境档**：`docs/evidence/t024/env.md`——host.exe / render-spike.exe sha256 + 尺寸 + 构建命令与耗时 + 工具链版本（rust-toolchain.toml 钉版引用）。

## 2. 交付物

1. 代码：host/src/spectate.rs（或新 capture.rs——体例随现有结构，worker 定但不改 D1 语义）+ main.rs CLI 一参 + 相关注册。
2. 证据档 `docs/evidence/t024/`：`measure_spectate.sh` + `measure_c1_render_spike.sh`（或合一脚本两段）+ 两腿 frames.csv/meta.json 归档 + `summarize_t024.py` + 两份 summary 判定行 + 负载实录三段 + env.md + README.md（索引/判定行/口径注/预注册/锚来源）。
3. 门禁：check 0 警告（插桩后）+ release host（-j 3，host 非新依赖增量秒级）+ render-spike release（-j 2 预检）+ **七组既有冒烟全回归**（插桩零侵入证明）+ spectate_smoke 带 --frame-capture 冒烟 1 发（capture 自退 + frames.csv 非空 + AppExit 码）。

## 3. 上报纪律

同附录 B.1/B.2：异常 ≤2 次上报；数值三查（窗口算式 warmup+capture vs max-ticks、10k 等比算式、判定常量注源）；量测窗口负载事件即中止重排（附录 A 纪律——负载敏感假红教训）；范围外（阈值变更/优化轮/多分辨率）上报不拍板。
