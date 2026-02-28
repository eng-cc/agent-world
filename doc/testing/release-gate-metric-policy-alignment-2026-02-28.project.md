# 发布门禁指标策略对齐（2026-02-28）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [ ] T1 脚本改造：S9/S10 增加 `--strict-metrics`，默认发布策略放宽为“insufficient_data 仅告警”。
- [ ] T2 回归验证：语法检查 + 复跑 S9/S10 发布门禁命令并核对 `summary.json`。
- [ ] T3 文档收口：更新 `testing-manual.md`、补 devlog、项目结项。

## 依赖
- `scripts/p2p-longrun-soak.sh`
- `scripts/s10-five-node-game-soak.sh`
- `testing-manual.md`
- `.tmp/release_gate_p2p/20260228-211606/summary.json`
- `.tmp/release_gate_s10/20260228-211606/summary.json`

## 状态
- 当前阶段：进行中（T0 已完成，执行 T1）。
- 当前任务：T1 脚本改造。
