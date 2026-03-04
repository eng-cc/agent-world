# 客户端启动器 native/web 同控制面与客户端服务端分离（2026-03-04）项目管理文档

- 对应设计文档: doc/world-simulator/launcher/game-client-launcher-native-web-control-plane-unification-2026-03-04.prd.md

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-015) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块文档树回写。
- [x] T1 (PRD-WORLD_SIMULATOR-015) [test_tier_required]: 升级 `world_web_launcher` 为游戏/区块链独立编排控制面，新增链独立启停 API 与状态快照。
- [ ] T2 (PRD-WORLD_SIMULATOR-015) [test_tier_required]: `agent_world_client_launcher` native 改为客户端-服务端分离并复用同一 API 控制链路，恢复 web 端链启停按钮与状态对齐。
- [ ] T3 (PRD-WORLD_SIMULATOR-015) [test_tier_required]: 执行 `cargo test/check` + Playwright headed 闭环（含链/游戏独立启停），归档证据并收口文档。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world_client_launcher/src/app_process.rs`
- `crates/agent_world_client_launcher/src/app_process_web.rs`
- `output/playwright/`

## 状态
- 当前阶段: in_progress
- 当前任务: T2
- 备注: 目标是“native/web 功能行为对齐且链路统一”，完成后回写主 PRD 与模块项目文档。
