# 客户端启动器全量可用性收口修复（2026-03-08）项目管理文档

审计轮次: 4
- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-full-usability-remediation-2026-03-08.prd.md`

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-029) [test_tier_required]: 完成专题 PRD 建模、任务拆解与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-029) [test_tier_required]: 落地配置防回写与请求并发域拆分（native/web 同步），补齐定向回归测试。
- [x] T2 (PRD-WORLD_SIMULATOR-029) [test_tier_required]: 落地反馈窗口草稿保护、顶栏响应式与转账过滤重置，并完成 native/web 回归。
- [x] T3 (PRD-WORLD_SIMULATOR-029) [test_tier_required]: 修复主 PRD 启动器条款冲突（AC 重号/语义重复、集成点重复）并通过文档治理检查。
- [ ] T4 (PRD-WORLD_SIMULATOR-029) [test_tier_required]: 拆分 `agent_world_client_launcher` 超长文件（`main.rs`、`explorer_window.rs`）至单文件 <=1200 行并完成 native/wasm 回归。

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
- 当前任务: T4（启动器超长文件拆分与回归）
- 备注: T0/T1/T2/T3 已完成，继续执行 T4 收口工程规范风险。
