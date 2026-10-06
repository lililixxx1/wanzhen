# T024 观战档汇总（帧采集判定；脚本生成，判定行禁手改）

- 输入目录: <trees-root>/t024-a/docs/evidence/t024/runs/spectate-10k
- 判定口径: 表 6-0 验收⑤（T010 先例）——avg_fps ≥ 60 且 1% low ≥ 45；不达标如实 TRIPPED
- vsync 旁证阈值: delta_ns < 15865000（0.95 × 16.7 ms）

## 观战 10k 指标（窗口 = warmup 5 s 丢弃 + capture 65 s）

| 档位 | 帧数 N | avg_fps | 1% low | 0.1% low | p50 (ms) | p95 (ms) | p99 (ms) | max (ms) | 非vsync旁证帧数 | 判定 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| spectate-10k | 19774 | 304.21 | 198.75 | 148.25 | 3.198 | 3.888 | 4.357 | 7.798 | 19774 | PASS |

- 非 vsync 锁定旁证：有（19774 帧 < 15.865 ms）

## meta.json 摘录（落档现场读数）

- mode: spectate
- resolution: 1920x1080
- window_resolution_actual: 2400x1350
- present_mode: AutoNoVsync
- warmup_s: 5
- capture_s: 65
- captured_frames: 19774
- units_total: 10000
- tick_at_write: 2100
- seed: 42
- max_ticks: 3600

## 判定行（表 6-0 验收⑤ / T010 先例）

- spectate-10k: avg_fps=304.21 (≥60 ? 是)、1% low=198.75 (≥45 ? 是) → **PASS**
