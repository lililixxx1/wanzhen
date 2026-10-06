# T018 环境档（M5-01 BRP 宿主；沿 M0 体例）

- 生成时间：2026-10-06 21:20 (+08:00)。基准机 A = 开发机本机（报告表 6-0）——**与 T010 环境档同日同机**
  （2026-10-06 00:42 采集，`docs/evidence/t010/environment.txt`），机器面（i5-12490F / 32GB / RTX 3050 /
  Win10 19044 / rustc 1.98.1）不重复抄录，以该档为准。
- 本卡增量环境面：
  - 构建纪律（D10）：bevy feature 统一（render-spike `bevy_full` ∪ host `bevy_remote`）触发 bevy facade
    重编；内存预检 = 构建前 free 24,837,296 KB（≈23.7 GiB）≥ 12G 门禁 ✓；cargo 一律 `-j 2`。
  - 工具链：rustc 1.98.1 (48a229cea 2026-09-01)，`rust-toolchain.toml` 钉版；bevy 锁 0.19.1（Cargo.lock）。
  - 产物（双版本时点如实记）：首轮 33,324,544 B / sha256 前 16 位 `19f88c4b71635fc1`（worker 轮 14m10s）；**整改重建后（归档行使版，21:48 冒烟所用）33,325,056 B / `4540c39a8a10e2e7`**（P1-1/P2-1/S-3 整改后 5.31s 增量重建；审核复验轮独立实测一致）。
  - BRP 端点：127.0.0.1:15702（写死回环，D11）；直调客户端 = Git Bash curl（brp_smoke.sh）。
- 依赖图证据（验收断言 1）：
  - `cargo tree -p host --depth 1` = `bevy v0.19.1` + `serde_json v1.0.151` + `sim v0.1.0`（host 最小面，
    无渲染 feature 引入）。
  - sim 代码零 bevy API 使用：`grep -rE "use bevy|bevy::" sim/src/` 零命中（lib.rs 文档注释「不调用任何
    bevy API」与实现一致）；sim/Cargo.toml 骨架期（T002 前）遗留的未用 bevy 声明本卡不动（D2 注记——
    备忘录「零 bevy 纯 lib 不动」，清理留后续卡裁决）。
  - 宿主对 sim 只 import 公共 API：host/src 中 sim 侧引用全限定 `sim::world::*` / `sim::units::*` /
    `sim::pool::ThreadPool`（公共模块），无内部模块触碰。
- 冒烟执行窗口：首轮 2026-10-06 21:16（33/33，REQ/RESP 未落档——脚本缺陷随审核轮 P0-1 整改）；
  **归档版 21:48（46/46 PASS，SCRIPT_EXIT=0，原文全量在档）**。机器常规空闲（非帧敏量测，无独占窗口要求）。
