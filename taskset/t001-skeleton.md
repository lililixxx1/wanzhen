# T001 · 仓库骨架搭建与台账启用（预备周）

- 周位：预备周（2026-09-30 ~ 10-04，不计入首周 30h 观测）
- 前置：无
- 目标：最小纪律骨架落地——AGENTS.md（硬约束）/ README / 台账新册 / 策划报告 V0.9.1 收编 docs/ / Cargo workspace（sim headless crate）/ rust-toolchain 钉版 1.98.1 / git bot 身份 / 私有远程首推。
- 范围外：任何 bevy API 使用（骨架仅验证依赖解析与工具链）；M0 任务卡之外的代码。

## 验收断言

1. `cargo check --workspace` 0 警告 0 error。
2. `git log` 首提交身份 = `wanzhen <bot@wanzhen.invalid>`；远程 lililixxx1/wanzhen（private）master == 本地 HEAD。
3. `docs/万阵-游戏前期策划报告.html` 与仓外源文件字节数一致（V0.9.1 原样收编）。
4. 台账新册 schema 在位、本卡行已登记。

## 证据要求

命令与退出码（ls/check/git log/ls-remote/wc -c）入汇报；失败重试过程如实记录。
