# T009 汇总判定（summarize.py 自动生成，禁止手改）

- 数据源：runs/ 下 matrix_per100 / matrix_per10 / sampling JSONL + throughput_t{1,3,6,12}.json + R5 交叉原始档
- 判定行全部由本脚本计算（防手算漂移）；阈值逐字取自派工单 §3（R2 阈值 10_000 场/h；|z|<=1.96；接敌实证阈值 2*per_side / 20 / 10000）

## 胜率矩阵（派工单 §3-1）

### 口径层 matrix_per100（per_side=100，lane=50m）

| cell（红\蓝） | 红兵种 | 蓝兵种 | n | 红胜% | 蓝胜% | Draw% | resolved% | avg_end_tick | avg 存活(和) |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | shieldman | shieldman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 194.00 |
| 1 | shieldman | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 194.00 |
| 2 | shieldman | pikeman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 191.00 |
| 3 | shieldman | swordsman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 191.00 |
| 4 | shieldman | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 194.00 |
| 5 | shieldman | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 190.00 |
| 6 | heavyknight | shieldman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 194.00 |
| 7 | heavyknight | heavyknight | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 194.00 |
| 8 | heavyknight | pikeman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 189.00 |
| 9 | heavyknight | swordsman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 190.00 |
| 10 | heavyknight | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 193.00 |
| 11 | heavyknight | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 189.00 |
| 12 | pikeman | shieldman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 191.00 |
| 13 | pikeman | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 189.00 |
| 14 | pikeman | pikeman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 186.00 |
| 15 | pikeman | swordsman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 185.00 |
| 16 | pikeman | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 186.00 |
| 17 | pikeman | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 185.00 |
| 18 | swordsman | shieldman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 191.00 |
| 19 | swordsman | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 190.00 |
| 20 | swordsman | pikeman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 185.00 |
| 21 | swordsman | swordsman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 184.00 |
| 22 | swordsman | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 184.00 |
| 23 | swordsman | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 177.00 |
| 24 | archer | shieldman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 194.00 |
| 25 | archer | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 193.00 |
| 26 | archer | pikeman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 186.00 |
| 27 | archer | swordsman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 184.00 |
| 28 | archer | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 192.00 |
| 29 | archer | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 188.00 |
| 30 | militia | shieldman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 190.00 |
| 31 | militia | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 189.00 |
| 32 | militia | pikeman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 185.00 |
| 33 | militia | swordsman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 177.00 |
| 34 | militia | archer | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 188.00 |
| 35 | militia | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 182.00 |

### 探针层 matrix_per10（per_side=10，lane=50m）

| cell（红\蓝） | 红兵种 | 蓝兵种 | n | 红胜% | 蓝胜% | Draw% | resolved% | avg_end_tick | avg 存活(和) |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | shieldman | shieldman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 14.00 |
| 1 | shieldman | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 14.00 |
| 2 | shieldman | pikeman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 11.00 |
| 3 | shieldman | swordsman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 11.00 |
| 4 | shieldman | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 14.00 |
| 5 | shieldman | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 10.00 |
| 6 | heavyknight | shieldman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 14.00 |
| 7 | heavyknight | heavyknight | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 14.00 |
| 8 | heavyknight | pikeman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 9.00 |
| 9 | heavyknight | swordsman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 10.00 |
| 10 | heavyknight | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 13.00 |
| 11 | heavyknight | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 9.00 |
| 12 | pikeman | shieldman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 11.00 |
| 13 | pikeman | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 9.00 |
| 14 | pikeman | pikeman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 6.00 |
| 15 | pikeman | swordsman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 5.00 |
| 16 | pikeman | archer | 100 | 100.0% | 0.0% | 0.0% | 100.0% | 1416.0 | 9.00 |
| 17 | pikeman | militia | 100 | 100.0% | 0.0% | 0.0% | 100.0% | 1401.0 | 8.00 |
| 18 | swordsman | shieldman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 11.00 |
| 19 | swordsman | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 10.00 |
| 20 | swordsman | pikeman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 5.00 |
| 21 | swordsman | swordsman | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 4.00 |
| 22 | swordsman | archer | 100 | 100.0% | 0.0% | 0.0% | 100.0% | 1210.0 | 9.00 |
| 23 | swordsman | militia | 100 | 100.0% | 0.0% | 0.0% | 100.0% | 951.0 | 9.00 |
| 24 | archer | shieldman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 14.00 |
| 25 | archer | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 13.00 |
| 26 | archer | pikeman | 100 | 0.0% | 100.0% | 0.0% | 100.0% | 1416.0 | 9.00 |
| 27 | archer | swordsman | 100 | 0.0% | 100.0% | 0.0% | 100.0% | 1210.0 | 9.00 |
| 28 | archer | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 12.00 |
| 29 | archer | militia | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 8.00 |
| 30 | militia | shieldman | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 10.00 |
| 31 | militia | heavyknight | 100 | 0.0% | 100.0% | 0.0% | 0.0% | 1800.0 | 9.00 |
| 32 | militia | pikeman | 100 | 0.0% | 100.0% | 0.0% | 100.0% | 1401.0 | 8.00 |
| 33 | militia | swordsman | 100 | 0.0% | 100.0% | 0.0% | 100.0% | 951.0 | 9.00 |
| 34 | militia | archer | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 8.00 |
| 35 | militia | militia | 100 | 100.0% | 0.0% | 0.0% | 0.0% | 1800.0 | 2.00 |

### 全规模层 sampling（per_side=5000，lane=1000m，100 局）逐局另见 sampling.jsonl；按 cell 汇总：

| cell | 红 | 蓝 | n | 红胜 | 蓝胜 | Draw | resolved | avg_end_tick | avg 存活(和) |
|---:|---|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | shieldman | shieldman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9980.0 |
| 1 | shieldman | heavyknight | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9955.0 |
| 2 | shieldman | pikeman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9949.0 |
| 3 | shieldman | swordsman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9945.0 |
| 4 | shieldman | archer | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9970.0 |
| 5 | shieldman | militia | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9950.0 |
| 6 | heavyknight | shieldman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9955.0 |
| 7 | heavyknight | heavyknight | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9948.0 |
| 8 | heavyknight | pikeman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9921.0 |
| 9 | heavyknight | swordsman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9928.0 |
| 10 | heavyknight | archer | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9948.0 |
| 11 | heavyknight | militia | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9924.0 |
| 12 | pikeman | shieldman | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9949.0 |
| 13 | pikeman | heavyknight | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9921.0 |
| 14 | pikeman | pikeman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9916.0 |
| 15 | pikeman | swordsman | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9902.0 |
| 16 | pikeman | archer | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9916.0 |
| 17 | pikeman | militia | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9907.0 |
| 18 | swordsman | shieldman | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9945.0 |
| 19 | swordsman | heavyknight | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9928.0 |
| 20 | swordsman | pikeman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9904.0 |
| 21 | swordsman | swordsman | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9889.0 |
| 22 | swordsman | archer | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9896.0 |
| 23 | swordsman | militia | 3 | 3 | 0 | 0 | 0 | 14400.0 | 9854.0 |
| 24 | archer | shieldman | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9970.0 |
| 25 | archer | heavyknight | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9948.0 |
| 26 | archer | pikeman | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9916.0 |
| 27 | archer | swordsman | 3 | 0 | 3 | 0 | 0 | 14400.0 | 9896.0 |
| 28 | archer | archer | 2 | 2 | 0 | 0 | 0 | 14400.0 | 9956.0 |
| 29 | archer | militia | 2 | 2 | 0 | 0 | 0 | 14400.0 | 9927.0 |
| 30 | militia | shieldman | 2 | 0 | 2 | 0 | 0 | 14400.0 | 9950.0 |
| 31 | militia | heavyknight | 2 | 0 | 2 | 0 | 0 | 14400.0 | 9923.0 |
| 32 | militia | pikeman | 2 | 0 | 2 | 0 | 0 | 14400.0 | 9907.0 |
| 33 | militia | swordsman | 2 | 0 | 2 | 0 | 0 | 14400.0 | 9854.0 |
| 34 | militia | archer | 2 | 0 | 2 | 0 | 0 | 14400.0 | 9927.0 |
| 35 | militia | militia | 2 | 0 | 0 | 2 | 0 | 14400.0 | 9896.0 |

## 镜像 sanity（派工单 §3-2）

### 镜像 sanity（口径层，对角 6 格：镜像局蓝胜率应 50/50）

| cell | 兵种 | n | 红胜 | 蓝胜 | Draw | p_blue | 95% CI | z | 判定 |
|---|---|---:|---:|---:|---:|---:|---|---:|---|
| 0 | shieldman | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=0（shieldman）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 7 | heavyknight | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=7（heavyknight）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 14 | pikeman | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=14（pikeman）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 21 | swordsman | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=21（swordsman）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 28 | archer | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=28（archer）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 35 | militia | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=35（militia）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%

- [镜像 sanity 判定行·口径层] 存在 |z| > 1.96 的对角格 → H2 结构性偏差形态（三态分布已逐格披露）

### 镜像 sanity（探针层，对角 6 格：镜像局蓝胜率应 50/50）

| cell | 兵种 | n | 红胜 | 蓝胜 | Draw | p_blue | 95% CI | z | 判定 |
|---|---|---:|---:|---:|---:|---:|---|---:|---|
| 0 | shieldman | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=0（shieldman）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 7 | heavyknight | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=7（heavyknight）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 14 | pikeman | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=14（pikeman）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 21 | swordsman | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=21（swordsman）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 28 | archer | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=28（archer）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%
| 35 | militia | 100 | 100 | 0 | 0 | 0.0000 | ±0.0000 | -10.000 | H2 结构性偏差形态 |
  - [H2 披露] cell=35（militia）三态分布：红 100.0% / 蓝 0.0% / Draw 0.0%

- [镜像 sanity 判定行·探针层] 存在 |z| > 1.96 的对角格 → H2 结构性偏差形态（三态分布已逐格披露）

## 吞吐判定（派工单 §3-3；R2 阈值 games_per_hour@12t >= 10_000）

| threads | games | wall_s(中位) | games_per_hour | per_game_ms |
|---:|---:|---:|---:|---:|
| 1 | 512 | 11.0664 | 166557.9 | 21.614 |
| 3 | 512 | 3.9975 | 461088.1 | 7.808 |
| 6 | 512 | 2.3227 | 793548.1 | 4.537 |
| 12 | 512 | 1.6762 | 1099638.6 | 3.274 |

- [吞吐判定行] games_per_hour@12t = 1099638.6 （阈值 >= 10000）→ PASS
- [16 线程外推] elapsed(T)=c1+c2/T OLS（T∈{1,3,6,12} 中位）：c1=0.6632s c2=10.3642s·T → elapsed(16)=1.3110s → 吞吐16 = 1405967.6 场/h
  - 拟合残差（s）：T=1:-0.0390, T=3:+0.1204, T=6:+0.0679, T=12:-0.1493

## 接敌实证判定（派工单 §3-4）

- [接敌实证判定行·口径层] 36 格全部 avg(alive_red+alive_blue) < 200（2*per_side）→ PASS；全场最大 avg 存活(和) = 194.00
- [接敌实证判定行·探针层] 36 格全部 avg(alive_red+alive_blue) < 20（2*per_side）→ PASS；全场最大 avg 存活(和) = 14.00
- [接敌实证判定行·全规模层] 100 局全部 alive_red+alive_blue < 10000 → PASS；全场最大存活(和) = 9980

## 分层比对（派工单 §3-5）

### 逐 cell 胜方方向（多数胜方；无多数 = mixed；全 Draw = draw）

| cell | 红 | 蓝 | 口径层 | 探针层 | 全规模层 | 三层一致 |
|---:|---|---|---|---|---|---|
| 0 | shieldman | shieldman | red | red | red（小样本 n=3） | 一致 |
| 1 | shieldman | heavyknight | blue | blue | blue（小样本 n=3） | 一致 |
| 2 | shieldman | pikeman | red | red | red（小样本 n=3） | 一致 |
| 3 | shieldman | swordsman | red | red | red（小样本 n=3） | 一致 |
| 4 | shieldman | archer | red | red | red（小样本 n=3） | 一致 |
| 5 | shieldman | militia | red | red | red（小样本 n=3） | 一致 |
| 6 | heavyknight | shieldman | red | red | red（小样本 n=3） | 一致 |
| 7 | heavyknight | heavyknight | red | red | red（小样本 n=3） | 一致 |
| 8 | heavyknight | pikeman | red | red | red（小样本 n=3） | 一致 |
| 9 | heavyknight | swordsman | red | red | red（小样本 n=3） | 一致 |
| 10 | heavyknight | archer | red | red | red（小样本 n=3） | 一致 |
| 11 | heavyknight | militia | red | red | red（小样本 n=3） | 一致 |
| 12 | pikeman | shieldman | blue | blue | blue（小样本 n=3） | 一致 |
| 13 | pikeman | heavyknight | blue | blue | blue（小样本 n=3） | 一致 |
| 14 | pikeman | pikeman | red | red | red（小样本 n=3） | 一致 |
| 15 | pikeman | swordsman | blue | blue | blue（小样本 n=3） | 一致 |
| 16 | pikeman | archer | red | red | red（小样本 n=3） | 一致 |
| 17 | pikeman | militia | red | red | red（小样本 n=3） | 一致 |
| 18 | swordsman | shieldman | blue | blue | blue（小样本 n=3） | 一致 |
| 19 | swordsman | heavyknight | blue | blue | blue（小样本 n=3） | 一致 |
| 20 | swordsman | pikeman | red | red | red（小样本 n=3） | 一致 |
| 21 | swordsman | swordsman | red | red | red（小样本 n=3） | 一致 |
| 22 | swordsman | archer | red | red | red（小样本 n=3） | 一致 |
| 23 | swordsman | militia | red | red | red（小样本 n=3） | 一致 |
| 24 | archer | shieldman | blue | blue | blue（小样本 n=3） | 一致 |
| 25 | archer | heavyknight | blue | blue | blue（小样本 n=3） | 一致 |
| 26 | archer | pikeman | blue | blue | blue（小样本 n=3） | 一致 |
| 27 | archer | swordsman | blue | blue | blue（小样本 n=3） | 一致 |
| 28 | archer | archer | red | red | red（小样本 n=2） | 一致 |
| 29 | archer | militia | red | blue | red（小样本 n=2） | 不一致 |
| 30 | militia | shieldman | blue | blue | blue（小样本 n=2） | 一致 |
| 31 | militia | heavyknight | blue | blue | blue（小样本 n=2） | 一致 |
| 32 | militia | pikeman | blue | blue | blue（小样本 n=2） | 一致 |
| 33 | militia | swordsman | blue | blue | blue（小样本 n=2） | 一致 |
| 34 | militia | archer | blue | red | blue（小样本 n=2） | 不一致 |
| 35 | militia | militia | red | red | draw（小样本 n=2） | 不一致 |

- [分层方向一致判定行] 33/36 cell 三层方向一致（比例 91.7%）

### 击溃率（v0：有胜方局中 胜方存活 >= 0.8*per_side 局占比）与 avg_end_tick

| 层 | 击溃率 | 有胜方局数 | avg_end_tick |
|---|---:|---:|---:|
| 口径层 | 100.0% | 3600 | 1800.0 |
| 探针层 | 50.0% | 3600 | 1676.6 |
| 全规模层 | 100.0% | 98 | 14400.0 |

## CLI 黄金交叉（派工单 §3-6 / R5）

- arena=0x564cf46fdf191710
- sim_t1=0x564cf46fdf191710
- sim_t12=0x564cf46fdf191710
- [CLI 交叉判定行] 三方 final_hash 逐位一致 → PASS

## 预注册预期对照（派工单登记；仅供披露，判定线见上各节）

- 吞吐@12t 预期 >= 100_000 场/h 量级（判定线 10_000）→ 实测 1099638.6 场/h
- 采样单局 ≈11.6s（T015 同规模同档参考）→ 实测中位 13.12s（偏离 ±100% 区间 [0.0, 23.2] → 区间内）
- 口径层多数格 resolved=false（预注册设计预期）→ 实测口径层 resolved 局数 0/3600；全规模层 resolved 0/100


（end；由 summarize.py 从 runs/ 原始档复算生成）