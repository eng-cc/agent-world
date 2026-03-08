# 客户端启动器全量可用性收口修复（2026-03-08）项目管理文档

审计轮次: 1
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-full-usability-remediation-2026-03-08.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-029) [test_tier_required]: 完成专题 PRD 建模、任务拆解与模块文档树回写。
- [ ] T1 (PRD-WORLD_SIMULATOR-029) [test_tier_required]: 落地配置防回写与请求并发域拆分（native/web 同步），补齐定向回归测试。
- [ ] T2 (PRD-WORLD_SIMULATOR-029) [test_tier_required]: 落地反馈窗口草稿保护、顶栏响应式与转账过滤重置，并完成 native/web 回归。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/app_process.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `crates/agent_world_client_launcher/src/feedback_window.rs`
- `crates/agent_world_client_launcher/src/feedback_window_web.rs`
- `crates/agent_world_client_launcher/src/transfer_window.rs`
- `crates/agent_world_client_launcher/src/explorer_window.rs`
- `crates/agent_world_client_launcher/src/main_tests.rs`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-08
- 当前阶段: in_progress
- 当前任务: T1
- 备注: 本专题用于收口启动器残余可用性风险（配置回写、全局 in-flight 串行、反馈草稿中断、顶栏拥挤、过滤重置缺失）。
