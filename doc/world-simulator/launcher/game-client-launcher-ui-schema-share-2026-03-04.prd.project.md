# 客户端启动器 UI Schema 共享（2026-03-04）项目管理文档

审计轮次: 5
- 对应设计文档: doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.prd.md

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-011) [test_tier_required]: 建档并冻结“native/web 启动器 UI schema 共享”需求与验收标准。
- [x] T1 (PRD-WORLD_SIMULATOR-011) [test_tier_required]: 新增共享 schema crate，沉淀字段定义与可见性策略。
- [x] T2 (PRD-WORLD_SIMULATOR-011) [test_tier_required]: native 启动器改为 schema 驱动渲染核心配置区。
- [x] T3 (PRD-WORLD_SIMULATOR-011) [test_tier_required]: web 控制台新增 `/api/ui/schema` 并改为动态表单渲染。
- [x] T4 (PRD-WORLD_SIMULATOR-011) [test_tier_required]: 拆分 `world_web_launcher` 大文件（<=1200 行），补齐测试与文档回写。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.prd.md`
- `crates/agent_world_client_launcher/src/main.rs`
- `crates/agent_world/src/bin/world_web_launcher.rs`

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段: completed
- 当前任务: 无
- 备注: 目标是实现“同一份 UI schema，多端渲染适配”。
