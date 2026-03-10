# 客户端启动器 UI Schema 共享（2026-03-04）

- 对应设计文档: `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.design.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-ui-schema-share-2026-03-04.project.md`

审计轮次: 5

## 1. Executive Summary
- Problem Statement: 当前启动器 native 与 web 控制台在 UI 字段定义上分别维护，字段新增/改名/分组时容易发生漂移。
- Proposed Solution: 抽取共享 `launcher UI schema`，由 native GUI 与 web 控制台共同消费，确保字段、分组、文案与可见性策略统一。
- Success Criteria:
  - SC-1: native 与 web 表单字段来源于同一份 schema 定义。
  - SC-2: web 页面表单由 schema 动态渲染，不再硬编码输入项列表。
  - SC-3: native UI 的核心配置行由 schema 驱动渲染，减少重复定义。
  - SC-4: 新增/修改字段时仅需修改共享 schema 即可同步两端 UI。

## 2. User Experience & Functionality
- User Personas:
  - 启动器开发者：希望 UI 字段定义单点维护。
  - 运维/测试人员：希望 native 与 web 的配置项一致，避免认知错位。
- User Scenarios & Frequency:
  - 每次新增 launcher 配置项时至少触发 1 次 schema 变更。
  - 每次发布前执行 web/native UI 一致性快速检查。
- User Stories:
  - As a 启动器开发者, I want a shared UI schema, so that native/web forms stay aligned.
  - As a 测试人员, I want web/native form parity, so that test cases do not branch by UI drift.
- Critical User Flows:
  1. Flow-LAUNCHER-UI-001（字段统一）:
     `修改共享 schema -> native 启动器渲染新字段 -> web 控制台渲染同字段`
  2. Flow-LAUNCHER-UI-002（Web 动态渲染）:
     `web 页面加载 -> 请求 /api/ui/schema -> 按 section 生成表单 -> 提交配置`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 共享 UI schema | `id/section/kind/label_zh/label_en/web_visible/native_visible` | UI 运行时按 schema 渲染 | `schema_loaded -> form_rendered` | section 内按 schema 顺序渲染 | schema 仅读 |
| Web schema 接口 | `/api/ui/schema` 返回可见字段列表 | 页面加载时拉取并构建表单 | `loading -> ready/failed` | 仅返回 `web_visible=true` 字段 | 只读接口 |
| Native schema 渲染 | native 主面板按 schema 渲染核心输入 | 启动器 UI 实时绑定配置状态 | `idle/running` 不变 | section 行布局保持一致 | 本地 GUI 会话 |
- Acceptance Criteria:
  - AC-1: 新增共享 crate（或模块）承载 launcher UI schema。
  - AC-2: native 启动器主配置区域改为 schema 驱动渲染。
  - AC-3: web 控制台通过 `/api/ui/schema` 动态渲染表单字段。
  - AC-4: `world_web_launcher` 主文件拆分后单文件不超过 1200 行。
  - AC-5: 测试覆盖 schema 输出与核心映射行为。
- Non-Goals:
  - 不在本轮统一 native/web 的全部交互样式和视觉主题。
  - 不在本轮改造进程编排后端协议。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- Architecture Overview:
  - 新增共享 crate：`agent_world_launcher_ui`，提供 schema 数据结构与静态字段定义。
  - `agent_world_client_launcher`：消费 schema 渲染配置输入区。
  - `world_web_launcher`：新增 `/api/ui/schema`，前端按 schema 动态构建表单。
- Integration Points:
  - `crates/agent_world_client_launcher/src/main.rs`
  - `crates/agent_world/src/bin/world_web_launcher.rs`
  - `scripts/build-game-launcher-bundle.sh`
- Edge Cases & Error Handling:
  - schema 接口失败时，web 页面展示错误并保留最近状态。
  - schema 字段缺失映射时，native/web 渲染忽略未知字段并记录诊断日志。
  - section 为空时不渲染空容器。
- Non-Functional Requirements:
  - NFR-1: `/api/ui/schema` 响应 `p95 <= 100ms`（本地网络）。
  - NFR-2: schema 结构变更需保持向后兼容（新增字段不破坏既有渲染）。
- Security & Privacy:
  - schema 仅暴露 UI 元数据，不包含密钥或敏感运行时数据。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1: PRD 建模与任务拆解。
  - M2: 共享 schema crate 落地并接入 native/web。
  - M3: 回归测试与文档收口。
- Technical Risks:
  - 风险-1: 字段映射遗漏导致某些输入项不可编辑。
  - 风险-2: 动态渲染引入前端初始化失败路径。

## 6. Validation & Decision Record
- Test Plan & Traceability:
  - PRD-WORLD_SIMULATOR-011 -> TASK-WORLD_SIMULATOR-025/026 -> `test_tier_required`。
- Decision Log:
  - DEC-LAUNCHER-UI-001: 采用共享 schema + 双端适配渲染，而非继续双端硬编码字段。理由：变更成本更低且可追溯性更强。
