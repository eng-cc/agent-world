# 客户端启动器中英文切换与必填配置校验（2026-03-02）项目管理

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.design.md`
- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.prd.md`

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-001)：建档（设计文档 + 项目管理文档）。
- [x] T1 (PRD-WORLD_SIMULATOR-002)：启动器 UI 增加中英文切换（标题、字段、按钮、状态与关键提示）。
- [x] T2 (PRD-WORLD_SIMULATOR-002/003)：启动器增加必填配置阻断校验与提示清单，并补单元测试。
- [x] T3 (PRD-WORLD_SIMULATOR-003)：回归测试、文档收口与任务日志更新。

## 依赖
- doc/world-simulator/launcher/game-client-launcher-i18n-required-config-2026-03-02.prd.md
- `crates/oasis7_client_launcher` 现有 GUI 与参数构建链路。
- `oasis7_game_launcher` 现有参数语义（用于对齐校验规则）。
- `testing-manual.md` 的测试分层与记录规范。

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段：已完成（T0~T3）。
- 当前任务：无（项目结项）。
