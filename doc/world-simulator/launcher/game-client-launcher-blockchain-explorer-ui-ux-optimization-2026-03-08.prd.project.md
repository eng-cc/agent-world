# 客户端启动器区块链浏览器视觉与交互优化（2026-03-08）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-blockchain-explorer-ui-ux-optimization-2026-03-08.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-028) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-028) [test_tier_required]: 优化启动器区块链浏览器面板视觉层级与交互流程（概览分组、状态徽标、筛选恢复、列表-详情协同），并完成 native/web 回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/explorer_window.rs`
- `crates/agent_world_client_launcher/src/explorer_window_p1.rs`
- `crates/agent_world_client_launcher/src/app_process.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-08
- 当前阶段: in_progress
- 当前任务: T1
- 备注: T0 已完成，等待 T1 实现与回归收口。
