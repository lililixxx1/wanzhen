# T004 python 独立第二实现（recalc.py）——双盲对拍地基

本目录是 T004「索敌攻击与克制结算」的 **python 独立第二实现**，用于与 Rust 实现做
双盲对拍（方法论 = T012 独立复算的提前演练）。实现按派工单内联设计定稿独立推导，
与 Rust 实现对照时任何逐位差异都指向语义/移植缺陷，而非共享代码同源复制。

环境与依赖：python 3.14.4（Windows 10 / Git Bash 实测），**单文件、零第三方依赖、纯整数运算**。

---

## 1. 双盲纪律留痕

**编写期禁读清单**（未读取，以此保证独立性）：

- `sim/src/world.rs`、`sim/src/units.rs`、`sim/src/main.rs`、`sim/src/hash.rs`
  （另一执行者同步改写中，读取会污染双盲）；
- `docs/evidence/t004/` 根目录下除本子目录（`py_recalc/`）外的一切。

**编写期已读且允许读的清单**：

- `sim/src/rng.rs` —— **唯一共享来源**：Xoshiro256** / SplitMix64 算法与
  `state_words` 顺序逐行移植自该文件（含 seed 42 前 8 输出黄金断言）；
- `docs/evidence/t002/`、`docs/evidence/t003/` —— 历史证据档（黄金锚、布阵快照、
  CLI 口径、T003 布阵算式手推对照）；
- `taskset/t004-combat.md`、`AGENTS.md`。

**语义权威来源**：派工单内联设计定稿（定点数制 / 六兵种表 / 克制矩阵 / RNG /
布阵 / 每 tick 阶段序 / 移动 / 战斗 / 移除 / 哈希折叠）。rng.rs 之外，本实现
未从 Rust 侧读取任何实现细节。

---

## 2. 实现构成（recalc.py）

### 2.1 定点数制

- 位置/距离/半径：Q32.32，1 米 = 2^32 = 4294967296；x 可负（i64 语义）。
  本工具实跑场景（千 m 级 lane、千 tick 级）内 |x| ≪ 2^63，python 不做取模模拟
  与 Rust i64 运算无差异。
- 克制倍率：Q16.16，1.0 = 65536。
- 常数全部整数表达式，截断取整（见 §2.4 等价论证）：

| 常数 | 表达式 | 值 |
| --- | --- | --- |
| 0.05 m/tick 移速 | `5 * 2^32 // 100` | 214748364 |
| 0.09 m/tick 移速 | `9 * 2^32 // 100` | 386547056 |
| 0.4 m 半径 | `2 * 2^32 // 5` | 1717986918 |
| 0.8 m 半径 | `4 * 2^32 // 5` | 3435973836 |
| 布阵邻距 0.5 m | `2^32 // 2` | 2147483648 |
| 近战余量 0.2 m | `20 * 2^32 // 100` | 858993459 |
| 克制 ×1.5 | `3 * 65536 // 2` | 98304 |
| 被克 ×2/3 | `2 * 65536 // 3` | 43690 |
| lane 长 | `1000 * 2^32` | 4294967296000 |

六兵种表、克制矩阵（行/列 = Heavy/Light/Unarmored，环：Heavy→Light→Unarmored→Heavy）
与派工单定稿逐值一致，`--selftest` 含常数自检。

### 2.2 RNG

- SplitMix64 仅用于 seed 扩展出 4 个状态字；Xoshiro256** 为主发生器；
  `state_words()` 折叠顺序 = 结构字段序 `s[0], s[1], s[2], s[3]`。
- 每 tick 固定消耗 1 个 `next_u64()` 并丢弃（阶段序第 2 步，空单位局也消耗——
  这正是 legacy 锚 1 能过所验证的）。
- 布阵用独立实例 `from_seed(seed ^ 0x6465_706C_6F79_0001)`（"deploy" 盐），不消耗 tick RNG。

### 2.3 每 tick 阶段序（与定稿一一对应）

`tick += 1` → tick 级 RNG 消耗 1 个 u64（丢弃）→ 移动（索引序，in-place 最新位置）→
战斗（索引序）→ tick 末移除（保序压缩）→ 状态哈希。

- **移动**：dir 红 +1 / 蓝 -1；在存活单位中找 `(x_j - x_i) * dir > 0` 的**最小者**
  （比较的是乘积距离，非原始差值；平局取最小 j，从 0 向上扫、严格小于才更新）；
  `gap = dist - (r_i + r_j)`，前进 `min(speed_i, max(gap, 0))`；无前方者则全速。
- **战斗**：先对全部存活单位 `cd = max(cd-1, 0)`；再按索引序行动（须存活）：
  最近存活敌方（`|dx|` 最小，平局最小 j）；在
  `|dx| <= r_i + r_j + 858993459`（闭区间）且 `cd == 0` 时出手：
  `dmg = attack_i * COUNTER[armor_i][armor_j] // 65536`；`hp_j -= dmg`，
  `hp_j <= 0` → 墓碑（当 tick 内不可被选、不可行动）；`cd_i = interval_i`。
- **移除**：`[u for u in units if u.alive]`（保序压缩，存活者相对序不变）。

### 2.4 floor / trunc 等价论证（派工单要求留痕）

python `//` 向下取整，Rust `/` 向零截断；两者在**被除数非负且除数正**时恒等。
本实现全部除法点逐一列举：

1. Q 常数推导：`num * 2^32 // den`（num ≥ 0, den > 0）→ 非负；
2. 洗牌取模：`next_u64() % (i+1)`（两者非负）→ 与 Rust `%` 同；
3. 伤害：`attack * mult // 65536`，attack ∈ [6,14]、mult ∈ {43690, 65536, 98304} → 全正；
4. 余量：`20 * 2^32 // 100` → 非负。

模拟态中再无其他除法/取模；负数只出现在比较（`(x_j - x_i) * dir > 0`）与
加法/减法，不参与除法。**结论：本域内 python `//` 与 Rust 截断除法逐位等价。**

### 2.5 状态哈希（FNV-1a 64，两模式）

FNV-1a：`h = 0xCBF29CE484222325`，每字节 `h = (h ^ b) * 0x100000001B3 mod 2^64`。
折叠序：`tick u64 LE` → 4 个 RNG 状态字各 u64 LE（`state_words` 序）→ 每单位（索引序）：

- **legacy 模式**（T003 折叠，供历史锚校验）：
  `[alive 1B + kind 1B + side 1B + x u64 LE]`；
- **t004 模式**（新折叠，含战斗状态）：
  同上，但单位段为 `[alive 1B + kind 1B + side 1B + hp i32 4B LE + cd u32 4B LE + x u64 LE]`。

负值编码：hp/x 负数先 `& MASK64/32` 取补码再 LE。备注（实现注释同步）：t004 折叠下
负 hp 实际不可达（`hp <= 0` 即墓碑、tick 末必被移除，进入哈希的单位 hp ≥ 1），
但仍按补码实现以防语义演进。

---

## 3. 用法

```
python recalc.py --selftest
python recalc.py --mode legacy|t004 --seed N --ticks N [--comp kind:count,...] [--units N]
python recalc.py --case 1|2|3|4 --seed N --ticks N [--mode ...]
```

- `--mode`：哈希折叠模式，默认 `t004`（输出首行 `mode=` 显式回显）。
- `--comp`：英文 id（shieldman/heavyknight/pikeman/swordsman/archer/militia），
  例 `--comp militia:10,archer:2`；与 `--units` 互斥；未知 id / count≤0 / 格式错 → exit 2。
- `--units N`：T002/T003 裸单位占位语义（Shieldman/Red/x=0）× N；`--units 0` = 空局。
- 缺省（无 comp/units/case）= 默认构成：六兵种各 5，双方对称共 60 单位。
- `--case N`：直接构造手算四例之一（见 §3.2），与 `--comp/--units` 互斥；
  输出逐单位快照行 `u{idx} {side} {kind} hp={} cd={} x={}`（x 为 Q32.32 原始整数）。
- 输出：摘要在前（`mode/seed/ticks/units/final_tick/hash`），普通模式**末行 = hash**；
  case 模式快照行排在 hash 行之后（与 T003 `--dump-formation` 输出同构）。
  `units=` 为终局清除死者后的数量。

### 3.1 复现命令（留档文件的生成命令）

```
python recalc.py --selftest
python recalc.py --mode legacy --units 0 --seed 42 --ticks 1800     # 锚 A1
python recalc.py --mode legacy --seed 42 --ticks 1800               # 锚 A2
python recalc.py --mode legacy --seed 43 --ticks 0                  # 锚 A3
python recalc.py --case 1 --seed 42 --ticks 151 / 181 / 185
python recalc.py --case 2 --seed 42 --ticks 1 / 21
python recalc.py --case 3 --seed 42 --ticks 1 / 10
python recalc.py --case 4 --seed 42 --ticks 1 / 21                       # 真等距构造
python recalc.py --mode t004 --seed 42 --ticks 1800                 # 供 Rust 双盲对拍
```

### 3.2 手算四例的构造口径（整数约定，供互证）

- 例 1：`[红 shieldman x=0.5m, 蓝 pikeman x=1.5m]`（0.5m/1.5m 均为精确值）。
- 例 2：`[红 militia x=0, 蓝 militia A x=+0.8m, 蓝 militia B x=-0.8m]`；
  **B 初始 x = -A 的存储值 = -3435973836**（非 floor(-0.8×2^32) = -3435973837，差 1，
  此为负值构造约定，已在 case2.txt 注明）。
- 例 3：`[蓝 militia x=1.8m (hp 直填 6), 红 militia x=1.0m, 红 militia x=0]`；
  1.8m 存储值 = 7730941132。
- 例 4（真等距，主会话复核裁决追加）：`[红 x=0, 蓝 A +0.8m, 蓝 B -0.8m, 蓝 C -1.6m, 红 D -2.4m]`，
  0.8m 存储值 step = 3435973836，C = -2×step、D = -3×step（精确整数倍；
  若按 floor 负除法会差 1 个 Q 单位、破坏「链节间距=半径和、gap=0 全静止」）。
- 四例 cd 初值均 0、hp 缺省取表值、alive=True；case 模式哈希默认走 t004 折叠。

---

## 4. 验证结果

### 4.1 legacy 三锚点（必须全过）—— 全 PASS

| 锚 | 命令参数 | 期望 | 实测 | 判定 |
| --- | --- | --- | --- | --- |
| A1 空单位（T002 口径） | `--mode legacy --units 0 --seed 42 --ticks 1800` | 0xd3b6408fd46c2008 | 0xd3b6408fd46c2008 | PASS |
| A2 默认构成（T003 黄金） | `--mode legacy --seed 42 --ticks 1800` | 0xf2b85bd4727c2d45 | 0xf2b85bd4727c2d45 | PASS |
| A3 布阵快照（T003 dump-formation） | `--mode legacy --seed 43 --ticks 0` | 0x70f65c2f7f585cad | 0x70f65c2f7f585cad | PASS |

三锚同时覆盖：RNG 状态字顺序与字节序、tick 级固定消耗、布阵洗牌/递推/镜像、
移动（含贴身停与索引序）、FNV 折叠（含负 x 补码）。原始记录见 `legacy-check.txt`。

附加字段级互证（见 `smoke-t004.txt` S6）：本实现 `deploy(42, 默认构成)` 的
60 个单位 `(side, kind, x)` 与 `docs/evidence/t003/deploy-snap1.txt` 逐行一致（60/60）。

### 4.2 手算四例 —— 与派工单期望逐字段一致

原始记录见 `case1.txt` / `case2.txt` / `case3.txt` / `case4.txt`（含命令、完整输出、
退出码、逐行 PASS）。

| 例 | 检查点 | 派工单期望（符号式） | 实测（Q32.32 整数） | 判定 |
| --- | --- | --- | --- | --- |
| 1 | t=151 | 红 hp 84 / 蓝 hp 8、cd 均 30 | u0 hp=84 cd=30 x=2147483648；u1 hp=8 cd=30 x=6442450944 | PASS |
| 1 | t=181 | 蓝被击杀移除（count=1、红 hp 84、x=0.5m） | units=1；u0 hp=84 cd=30 x=2147483648 | PASS |
| 1 | t=185 | 红 x=0.5m+4×0.05m、cd=26 | x=3006477104（=2147483648+4×214748364）、cd=26 | PASS |
| 2 | t=1 | 红 38、A 44、B 50（红被 A/B 各命中） | u0 38；u1 44；u2 50；B.x=-3822520892 | PASS |
| 2 | t=21 | 红 32、A 38、B 50、B.cd=0 | u0 32 cd=20；u1 38 cd=20；u2 50 cd=0 x=-11553462012 | PASS |
| 3 | t=1 | count=2；units[0] 44/20/1.0m；units[1] 50/0/0.09m | units=2；u0 x=4294967296；u1 x=386547056 | PASS |
| 3 | t=10 | x=1.0m+9×0.09m 与 10×0.09m | x=7773890800 与 3865470560 | PASS |
| 4 | t=1 | 红0 38、A 44、B 50、C 44、D 44；cd 全 20；x 全不变；count=5 | u0 38；u1 44；u2 50；u3 44；u4 44；x=0/3435973836/-3435973836/-6871947672/-10307921508 | PASS |
| 4 | t=21 | 红0 26、A 38、B 50、C 38、D 38；cd 全 20 | u0 26；u1 38；u2 50；u3 38；u4 38；cd 全 20 | PASS |

对拍补充说明（两点口径观察，不影响判定）：

1. **符号式 → 存储常量口径**：0.05m/0.8m/0.09m 等常数以 Q32.32 **存储值**参与运算
   （如 0.05m=214748364 而非 214748364.8；0.8m=3435973836 而非 3435973836.8）。
   上表实测量按此口径给出精确整数；若以浮点缩放比对会差个位数量级 1–3，
   属常量截断差，不是语义差。Rust 侧使用同一批表达式，且 seed 43 终局对拍已逐位一致
   （见 §4.3），该口径得到双实现验证。
2. **等距平局分支的覆盖**：例 2 战斗晚于移动，t=1 时 B 已移至 -0.89m，实际按最近距离
   命中 A，未触达平局分支（该观察首轮已上报）；按主会话复核裁决追加的**例 4** 已真实
   触达——t=1 红0 对 A/B 距离严格相等（均 3435973836）、平局取最小索引 j=1，证据为
   A 被击（44）而 B 未被击（50）；同时链式互堵使 t=1 移动阶段 x 全不变。

### 4.3 t004 折叠模式（无历史锚，供 Rust 双盲对拍）

| 场景 | 命令 | 实测 hash |
| --- | --- | --- |
| 默认构成 seed 42 / 1800 ticks | `--mode t004 --seed 42 --ticks 1800` | 0x958c5938c8682529 |
| 默认构成 seed 43 / 1800 ticks | `--mode t004 --seed 43 --ticks 1800` | 0x54611ed6ded02540（与 Rust 侧转发值逐位一致，见 case4.txt 附节） |
| 自定义构成 seed 7 / 500 ticks | `--mode t004 --comp militia:10,archer:2 --seed 7 --ticks 500` | 0xfdb4fd1279a23dcc |
| 例 4 真等距 t=1 | `--case 4 --seed 42 --ticks 1` | 0xf0ec0f4e2ca0f363 |
| 例 4 真等距 t=21 | `--case 4 --seed 42 --ticks 21` | 0xd8185d8d28034edf |

双实现 x 双 seed 交叉验证：seed 42 终局 0x958c5938c8682529（python 侧实测，待 Rust 侧同参数复核）、
seed 43 终局 0x54611ed6ded02540（Rust 侧转发值，python 侧实测逐位一致）。
其余 case 快照哈希见各 case 文件。`units=0` 时两模式等价（单位段为空），实测
t004 == legacy == 0xd3b6408fd46c2008（`smoke-t004.txt` S3）。

---

## 5. 文件清单

| 文件 | 内容 |
| --- | --- |
| `recalc.py` | 单文件零依赖实现（含 `--selftest`） |
| `legacy-check.txt` | selftest + 三锚运行留档（命令/完整输出/REAL_EXIT/比对） |
| `case1.txt` / `case2.txt` / `case3.txt` | 手算三例快照留档（同上格式） |
| `case4.txt` | 手算例 4（真等距构造）快照留档 + seed 43 t004 终局对拍附节（同上格式） |
| `smoke-t004.txt` | t004 模式烟雾、确定性 cmp、CLI 错误路径、布阵字段级互证 |
| `README.md` | 本文件 |

## 6. 与派工单的偏差 / 上报候选（供主会话裁决）

1. 上文 §4.2 观察 1（常数存储口径）仍为记录性说明（该口径已获 seed 43 双实现对拍支持）；
   观察 2（等距平局未被例 2 覆盖）已经主会话复核裁决，以追加的例 4 补齐并全 PASS
   （见 `case4.txt`）。此外按追加要求完成 seed 43 t004 终局对拍：python 实测
   0x54611ed6ded02540 与 Rust 侧转发值逐位一致。
2. `--case` 模式默认哈希走 t004 折叠（派工单未指定 case 模式折叠模式）；如需
   legacy 折叠的 case 哈希，`--mode legacy` 可直接复算。
3. `--case` 与 `--comp/--units` 互斥、`--units` 占位语义镜像 T003 文档口径——
  为保守处理，均以 exit 2 / 文档注明收口。
