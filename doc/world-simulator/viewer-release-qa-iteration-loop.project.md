# Viewer 发行验收测试迭代闭环（项目管理）

## 任务拆解
- [x] VRQ-0 文档建档：设计文档 + 项目管理文档
- [x] VRQ-1 基线执行：跑现有套件并输出首次完成度/视觉结论
- [x] VRQ-2 套件优化：实现一键化 Web 语义闭环验收脚本
- [ ] VRQ-3 复验收口：重跑并输出报告，更新手册与状态

## 依赖
- `testing-manual.md`
- `scripts/viewer-visual-baseline.sh`
- `scripts/run-viewer-web.sh`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `/Users/scc/.codex/skills/playwright/scripts/playwright_cli.sh`

## 状态
- 当前阶段：VRQ-3 进行中
- 下一步：将一键验收脚本接入手册并完成复验收口
- 最近更新：2026-02-21（VRQ-2 完成，首轮缺口已转化为自动门禁）
