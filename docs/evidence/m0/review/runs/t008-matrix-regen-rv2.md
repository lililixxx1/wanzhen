# T008 对拍矩阵（同种子重放确定性验证，验收④）

- 生成命令: python docs/evidence/t008/compare_matrix.py docs/evidence/t008/runs docs/evidence/m0/review/runs/t008-matrix-regen-rv2.md
- runs 目录: docs/evidence/t008/runs
- 预期清单: 29 局 = 矩阵 24（red200 ×12 + full10000 ×12；threads 1/3/6/12 × seeds 42/43/44） + 加样 3（red200-th12-s42-r2 / -r3、full10000-th12-s42-r2）+ 锚 2（anchor1-t006-fullscale / anchor2-t006-10k300）
- 实收: exits.txt 29 行；stdout 档 29 个
- 退出码语义: 0 全 PASS / 1 I/O 错 / 2 输入格式错 / 3 缺局或未完成 / 5 mismatch（本次运行退出码见 §8）

## 1. 矩阵对拍表 — red200

标准采样列（派工单 D3）: 0,450,900,1350；终局 = 五行摘要 hash= 行；采样列按实际 stdout 并集动态呈现

| run_id | REAL_EXIT | t=0 | t=450 | t=900 | t=1350 | final(t=1800) | units |
|---|---|---|---|---|---|---|---|
| red200-th1-s42 | 0 | 0x4b8f1372b7893b23 | 0x82c847b71ca33d46 | 0x07eb592ec79d2650 | 0xe0cd7861ca5a211f | 0xde91d6a5a6e84d43 | 200 |
| red200-th3-s42 | 0 | 0x4b8f1372b7893b23 | 0x82c847b71ca33d46 | 0x07eb592ec79d2650 | 0xe0cd7861ca5a211f | 0xde91d6a5a6e84d43 | 200 |
| red200-th6-s42 | 0 | 0x4b8f1372b7893b23 | 0x82c847b71ca33d46 | 0x07eb592ec79d2650 | 0xe0cd7861ca5a211f | 0xde91d6a5a6e84d43 | 200 |
| red200-th12-s42 | 0 | 0x4b8f1372b7893b23 | 0x82c847b71ca33d46 | 0x07eb592ec79d2650 | 0xe0cd7861ca5a211f | 0xde91d6a5a6e84d43 | 200 |
| 判定 red200-s42（4/4 档）→ PASS | — | PASS | PASS | PASS | PASS | PASS | — |

| run_id | REAL_EXIT | t=0 | t=450 | t=900 | t=1350 | final(t=1800) | units |
|---|---|---|---|---|---|---|---|
| red200-th1-s43 | 0 | 0x3cc0117972cf2d00 | 0x04eef0b31ae9f41f | 0x66d1f445a2733e07 | 0x64fc48209855b3bd | 0x999a5237d3b1a9b2 | 200 |
| red200-th3-s43 | 0 | 0x3cc0117972cf2d00 | 0x04eef0b31ae9f41f | 0x66d1f445a2733e07 | 0x64fc48209855b3bd | 0x999a5237d3b1a9b2 | 200 |
| red200-th6-s43 | 0 | 0x3cc0117972cf2d00 | 0x04eef0b31ae9f41f | 0x66d1f445a2733e07 | 0x64fc48209855b3bd | 0x999a5237d3b1a9b2 | 200 |
| red200-th12-s43 | 0 | 0x3cc0117972cf2d00 | 0x04eef0b31ae9f41f | 0x66d1f445a2733e07 | 0x64fc48209855b3bd | 0x999a5237d3b1a9b2 | 200 |
| 判定 red200-s43（4/4 档）→ PASS | — | PASS | PASS | PASS | PASS | PASS | — |

| run_id | REAL_EXIT | t=0 | t=450 | t=900 | t=1350 | final(t=1800) | units |
|---|---|---|---|---|---|---|---|
| red200-th1-s44 | 0 | 0x8b029d36ef2176e9 | 0xe71842f613519201 | 0x949a13fa1bf0af47 | 0x555dc32a1b7e1c2a | 0x44348b8042997ed3 | 200 |
| red200-th3-s44 | 0 | 0x8b029d36ef2176e9 | 0xe71842f613519201 | 0x949a13fa1bf0af47 | 0x555dc32a1b7e1c2a | 0x44348b8042997ed3 | 200 |
| red200-th6-s44 | 0 | 0x8b029d36ef2176e9 | 0xe71842f613519201 | 0x949a13fa1bf0af47 | 0x555dc32a1b7e1c2a | 0x44348b8042997ed3 | 200 |
| red200-th12-s44 | 0 | 0x8b029d36ef2176e9 | 0xe71842f613519201 | 0x949a13fa1bf0af47 | 0x555dc32a1b7e1c2a | 0x44348b8042997ed3 | 200 |
| 判定 red200-s44（4/4 档）→ PASS | — | PASS | PASS | PASS | PASS | PASS | — |

## 2. 矩阵对拍表 — full10000

标准采样列（派工单 D3）: 0,3600,7200,10800,12600；终局 = 五行摘要 hash= 行；采样列按实际 stdout 并集动态呈现

| run_id | REAL_EXIT | t=0 | t=3600 | t=7200 | t=10800 | t=12600 | final(t=14400) | units |
|---|---|---|---|---|---|---|---|---|
| full10000-th1-s42 | 0 | 0xc9d8fa745afbe696 | 0xaf9ade93bde87854 | 0x737e825f92bc9703 | 0x9d69d944e16bd71c | 0x62f2195b23b72c52 | 0xd921c95a9bf1db66 | 9959 |
| full10000-th3-s42 | 0 | 0xc9d8fa745afbe696 | 0xaf9ade93bde87854 | 0x737e825f92bc9703 | 0x9d69d944e16bd71c | 0x62f2195b23b72c52 | 0xd921c95a9bf1db66 | 9959 |
| full10000-th6-s42 | 0 | 0xc9d8fa745afbe696 | 0xaf9ade93bde87854 | 0x737e825f92bc9703 | 0x9d69d944e16bd71c | 0x62f2195b23b72c52 | 0xd921c95a9bf1db66 | 9959 |
| full10000-th12-s42 | 0 | 0xc9d8fa745afbe696 | 0xaf9ade93bde87854 | 0x737e825f92bc9703 | 0x9d69d944e16bd71c | 0x62f2195b23b72c52 | 0xd921c95a9bf1db66 | 9959 |
| 判定 full10000-s42（4/4 档）→ PASS | — | PASS | PASS | PASS | PASS | PASS | PASS | — |

| run_id | REAL_EXIT | t=0 | t=3600 | t=7200 | t=10800 | t=12600 | final(t=14400) | units |
|---|---|---|---|---|---|---|---|---|
| full10000-th1-s43 | 0 | 0x1d0ec71dcb064bb3 | 0x8453bed29beaceaa | 0xa8ef4f13e5fe2d42 | 0xc5c12a01eb9711a9 | 0x4dcf2a4725d42266 | 0x7d34b08102e34260 | 9969 |
| full10000-th3-s43 | 0 | 0x1d0ec71dcb064bb3 | 0x8453bed29beaceaa | 0xa8ef4f13e5fe2d42 | 0xc5c12a01eb9711a9 | 0x4dcf2a4725d42266 | 0x7d34b08102e34260 | 9969 |
| full10000-th6-s43 | 0 | 0x1d0ec71dcb064bb3 | 0x8453bed29beaceaa | 0xa8ef4f13e5fe2d42 | 0xc5c12a01eb9711a9 | 0x4dcf2a4725d42266 | 0x7d34b08102e34260 | 9969 |
| full10000-th12-s43 | 0 | 0x1d0ec71dcb064bb3 | 0x8453bed29beaceaa | 0xa8ef4f13e5fe2d42 | 0xc5c12a01eb9711a9 | 0x4dcf2a4725d42266 | 0x7d34b08102e34260 | 9969 |
| 判定 full10000-s43（4/4 档）→ PASS | — | PASS | PASS | PASS | PASS | PASS | PASS | — |

| run_id | REAL_EXIT | t=0 | t=3600 | t=7200 | t=10800 | t=12600 | final(t=14400) | units |
|---|---|---|---|---|---|---|---|---|
| full10000-th1-s44 | 0 | 0x4287d88a300cf7b3 | 0xf3062ac9faaf4e0a | 0xcc97d72acb83069a | 0x2ca1de388a4d90a6 | 0xe42b46bc06bffb35 | 0x9d55c4ce4f7fd880 | 9970 |
| full10000-th3-s44 | 0 | 0x4287d88a300cf7b3 | 0xf3062ac9faaf4e0a | 0xcc97d72acb83069a | 0x2ca1de388a4d90a6 | 0xe42b46bc06bffb35 | 0x9d55c4ce4f7fd880 | 9970 |
| full10000-th6-s44 | 0 | 0x4287d88a300cf7b3 | 0xf3062ac9faaf4e0a | 0xcc97d72acb83069a | 0x2ca1de388a4d90a6 | 0xe42b46bc06bffb35 | 0x9d55c4ce4f7fd880 | 9970 |
| full10000-th12-s44 | 0 | 0x4287d88a300cf7b3 | 0xf3062ac9faaf4e0a | 0xcc97d72acb83069a | 0x2ca1de388a4d90a6 | 0xe42b46bc06bffb35 | 0x9d55c4ce4f7fd880 | 9970 |
| 判定 full10000-s44（4/4 档）→ PASS | — | PASS | PASS | PASS | PASS | PASS | PASS | — |

## 3. 加样局（跨进程同配置逐列一致）

| 加样局 | 基准局 | 判定 | 明细 |
|---|---|---|---|
| red200-th12-s42-r2 | red200-th12-s42 | PASS | 5/5 列一致（采样 4 + 终局 1） |
| red200-th12-s42-r3 | red200-th12-s42 | PASS | 5/5 列一致（采样 4 + 终局 1） |
| full10000-th12-s42-r2 | full10000-th12-s42 | PASS | 6/6 列一致（采样 5 + 终局 1） |

## 4. 锚局（跨卡黄金锚，逐字节 diff；期望现场读取自 T006 归档）

| 锚局 | 参照档（现场读取） | 参照 bytes | 实测 bytes | 逐字节 diff 判定 | REAL_EXIT | 实测 hash（解析自 stdout） | 参照 hash（解析自参照档） |
|---|---|---|---|---|---|---|---|
| anchor1-t006-fullscale | docs/evidence/t006/runs/fullscale.stdout | 72 | 72 | PASS — 逐字节一致（72 bytes） | 0 | 0x29980473140ed39e | 0x29980473140ed39e |
| anchor2-t006-10k300 | docs/evidence/t006/runs/10k-s42-t12.stdout | 207 | 207 | PASS — 逐字节一致（207 bytes） | 0 | 0x9e17408bc4b7b909 | 0x9e17408bc4b7b909 |

## 5. 不一致全量明细（禁止折叠/省略）

无 mismatch（无哈希不一致、无锚局逐字节差异、无种子互异失效）。

## 6. 缺局 / 失败局清单

无缺局、无失败局（29 局齐备且 REAL_EXIT=0）。

## 7. 覆盖披露（披露非判定）

### 7.1 每局 units（终局存活数）与参数

| run_id | 类别 | seed | ticks | final_tick | units | REAL_EXIT |
|---|---|---|---|---|---|---|
| red200-th1-s42 | matrix | 42 | 1800 | 1800 | 200 | 0 |
| red200-th3-s42 | matrix | 42 | 1800 | 1800 | 200 | 0 |
| red200-th6-s42 | matrix | 42 | 1800 | 1800 | 200 | 0 |
| red200-th12-s42 | matrix | 42 | 1800 | 1800 | 200 | 0 |
| red200-th1-s43 | matrix | 43 | 1800 | 1800 | 200 | 0 |
| red200-th3-s43 | matrix | 43 | 1800 | 1800 | 200 | 0 |
| red200-th6-s43 | matrix | 43 | 1800 | 1800 | 200 | 0 |
| red200-th12-s43 | matrix | 43 | 1800 | 1800 | 200 | 0 |
| red200-th1-s44 | matrix | 44 | 1800 | 1800 | 200 | 0 |
| red200-th3-s44 | matrix | 44 | 1800 | 1800 | 200 | 0 |
| red200-th6-s44 | matrix | 44 | 1800 | 1800 | 200 | 0 |
| red200-th12-s44 | matrix | 44 | 1800 | 1800 | 200 | 0 |
| full10000-th1-s42 | matrix | 42 | 14400 | 14400 | 9959 | 0 |
| full10000-th3-s42 | matrix | 42 | 14400 | 14400 | 9959 | 0 |
| full10000-th6-s42 | matrix | 42 | 14400 | 14400 | 9959 | 0 |
| full10000-th12-s42 | matrix | 42 | 14400 | 14400 | 9959 | 0 |
| full10000-th1-s43 | matrix | 43 | 14400 | 14400 | 9969 | 0 |
| full10000-th3-s43 | matrix | 43 | 14400 | 14400 | 9969 | 0 |
| full10000-th6-s43 | matrix | 43 | 14400 | 14400 | 9969 | 0 |
| full10000-th12-s43 | matrix | 43 | 14400 | 14400 | 9969 | 0 |
| full10000-th1-s44 | matrix | 44 | 14400 | 14400 | 9970 | 0 |
| full10000-th3-s44 | matrix | 44 | 14400 | 14400 | 9970 | 0 |
| full10000-th6-s44 | matrix | 44 | 14400 | 14400 | 9970 | 0 |
| full10000-th12-s44 | matrix | 44 | 14400 | 14400 | 9970 | 0 |
| red200-th12-s42-r2 | spike | 42 | 1800 | 1800 | 200 | 0 |
| red200-th12-s42-r3 | spike | 42 | 1800 | 1800 | 200 | 0 |
| full10000-th12-s42-r2 | spike | 42 | 14400 | 14400 | 9959 | 0 |
| anchor1-t006-fullscale | anchor | 42 | 14400 | 14400 | 9959 | 0 |
| anchor2-t006-10k300 | anchor | 42 | 300 | 300 | 10000 | 0 |

### 7.2 full 规模战斗段证据（units<10000 ⇒ 战斗段已进入哈希）

观察到 14/14 个 full 规模局终局 units<10000（战斗段已进入哈希）：full10000-th1-s42 units=9959；full10000-th3-s42 units=9959；full10000-th6-s42 units=9959；full10000-th12-s42 units=9959；full10000-th1-s43 units=9969；full10000-th3-s43 units=9969；full10000-th6-s43 units=9969；full10000-th12-s43 units=9969；full10000-th1-s44 units=9970；full10000-th3-s44 units=9970；full10000-th6-s44 units=9970；full10000-th12-s44 units=9970；full10000-th12-s42-r2 units=9959；anchor1-t006-fullscale units=9959

### 7.3 标准采样列覆盖（按 scale，ok 矩阵/加样局中该列出现数）

- red200: t=0: 14/14；t=450: 14/14；t=900: 14/14；t=1350: 14/14
- full10000: t=0: 13/13；t=3600: 13/13；t=7200: 13/13；t=10800: 13/13；t=12600: 13/13

## 8. 汇总判定（任务卡验收断言 1/2/3）

- 断言 1（零 mismatch：每 (scale,seed) 组合 4 线程档 × 全部采样列 + 终局列逐位一致）: PASS — 6/6 组合逐位一致
  - 组合明细: red200-s42 PASS；red200-s43 PASS；red200-s44 PASS；full10000-s42 PASS；full10000-s43 PASS；full10000-s44 PASS
- 断言 2（中间哈希：每 scale 标准采样列组内一致）: PASS
  - red200: PASS（12/12 判定点 = 3 组合 × 4 中间列组内一致）
  - full10000: PASS（15/15 判定点 = 3 组合 × 5 中间列组内一致）
- 断言 3（mismatch 全量如实列出）: 未触发（无 mismatch）
- 加样局（跨进程同配置逐列一致）: PASS — red200-th12-s42-r2 PASS；red200-th12-s42-r3 PASS；full10000-th12-s42-r2 PASS
- 锚局（vs T006 归档逐字节 diff）: PASS — anchor1-t006-fullscale PASS；anchor2-t006-10k300 PASS
- 种子互异 sanity（同 (scale,threads) 下 3 种子终局哈希两两不同）: PASS — 8/8 个 (scale,threads) 下 3 种子终局哈希两两不同
- 退出码: 0

