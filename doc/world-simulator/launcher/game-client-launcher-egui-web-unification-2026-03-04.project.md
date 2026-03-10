# 客户端启动器 egui Web 同层复用与静态资源服务（2026-03-04）项目管理文档

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.design.md`
- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.prd.md`

审计轮次: 5


## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-012) [test_tier_required]: 完成专题 PRD 建模、验收标准冻结与模块级索引回写。
- [x] T1 (PRD-WORLD_SIMULATOR-012) [test_tier_required]: `agent_world_client_launcher` 落地 wasm 入口，复用同一套 egui UI 渲染与 schema 字段映射。
- [x] T2 (PRD-WORLD_SIMULATOR-012) [test_tier_required]: `world_web_launcher` 改为静态资源服务（launcher wasm）+ API 路由并存。
- [x] T3 (PRD-WORLD_SIMULATOR-012) [test_tier_required]: 更新 bundle 构建脚本生成 `web-launcher/`，补齐测试与文档回写。

## 依赖
- `doc/world-simulator/launcher/game-client-launcher-egui-web-unification-2026-03-04.design.md`
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world/src/bin/world_web_launcher.rs`
- `scripts/build-game-launcher-bundle.sh`

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段: completed
- 当前任务: 无
- 备注: 目标是 native/web 启动器共享同一套 egui UI 层，web 端以静态资源方式由 `world_web_launcher` 托管。
