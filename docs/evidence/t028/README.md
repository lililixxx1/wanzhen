# T028 证据档（万阵归档与开源准备 + 跨仓回流收尾）

- 任务卡：taskset/t028-archive-opensource.md（★完整轮）
- 决策依据：docs/m5-identity-decision.md §4-3（开源沉淀按推荐采纳）/ §4-5（回流收尾）/ §5-1（随线动作 1）

## 索引

| 档 | 内容 | 判定 |
|---|---|---|
| open-source-scan.txt | 可公开态终扫 v2（四变体 + 敏感词 + 全历史 bot 身份 + gitignore 覆盖，沿 T039 体例；判定口径 = 受管面、含扫描基准） | PASS（首发 3 处形态名清洗 + 终审 P0-1 整改（任务卡模式字面自命中）后复扫零命中） |

## 回流收尾（工作流仓侧，跨仓引用不复制正文）

五项入库落点与留痕锚见工作流仓 assets-methodology/patterns.md（PAT-M-010~013）与 bevy-dev/patterns/（PAT-B-021）——判定与 claim-lint 双绿留档以工作流仓台账 T028 行为权威（19ba994；PAT-M-010 同日随终审 P0 整改修订——防自命中条款）。

## License / README / 发布状态

- LICENSE-MIT / LICENSE-APACHE：沿工作流仓已审权威文本（T039 终审核逐字比对一致先例）改版权行 `wanzhen contributors`；双许可说明并入根 README（沿 T039 建议 N5 不单设说明文件）。
- 发布动作：**已执行（2026-10-07，owner 终审拍板「现在发布 / 仓名 wanzhen」）**——全历史推送 e27e4fd..858b929 + 可见性 PRIVATE→PUBLIC；公开仓 = https://github.com/lililixxx1/wanzhen（原 PRIVATE 备份仓转公开，非新建仓——如实记）。
