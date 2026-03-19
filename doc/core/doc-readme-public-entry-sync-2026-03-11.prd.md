# oasis7: 文档总入口公开阅读路径同步（2026-03-11）

- 对应设计文档: `doc/core/doc-readme-public-entry-sync-2026-03-11.design.md`
- 对应项目管理文档: `doc/core/doc-readme-public-entry-sync-2026-03-11.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: `doc/README.md` 仍停留在旧时间与旧快速阅读顺序，尚未反映当前新增的站点公开入口、技术预览口径与 release communication 链，导致工程总入口与最新公开阅读路径脱节。
- Proposed Solution: 在 core 下补一份“文档总入口公开阅读路径同步”专题，并回写 `doc/README.md` 的更新时间与快速阅读路径，使工程总入口同时覆盖项目总览、公开状态入口与模块执行入口。
- Success Criteria:
  - SC-1: `doc/README.md` 更新到 `2026-03-11`。
  - SC-2: 快速阅读路径显式纳入根 `README.md` 与 `site/index.html` 的公开状态入口。
  - SC-3: core 主项目可以追踪这次入口同步任务。

## 2. User Experience & Functionality
- User Personas:
  - 新协作者：需要先理解公开状态，再进入工程细节。
  - `producer_system_designer`：需要工程总入口不落后于最新公开口径。
  - 模块 owner：需要统一的全局导航顺序。
- User Scenarios & Frequency:
  - 新人入项：按总入口读取。
  - 跨模块对齐：按总入口确认最新阅读顺序。
  - 状态变化后：同步总入口导航。
- User Stories:
  - PRD-CORE-007: As a 新协作者, I want `doc/README.md` to include the current public-preview reading path, so that I start from the right entry points.
  - PRD-CORE-008: As a `producer_system_designer`, I want the global docs hub synced with repo/site posture, so that navigation stays consistent.
- Critical User Flows:
  1. `读 doc/README -> 先读根 README / site 了解公开状态 -> 再读 core / module PRD`
  2. `状态变化 -> 更新总入口路径 -> 保持各层入口一致`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| quick path | 阅读顺序、入口路径、目的 | 更新推荐路径 | `stale -> synced` | 先公开状态，再全局设计，再模块执行 | `producer_system_designer` 维护 |
| update stamp | 更新时间、摘要说明 | 标记总入口新鲜度 | `old -> current` | 最新同步优先 | 文档 owner 可改 |
- Acceptance Criteria:
  - AC-1: 产出专题 PRD / Design / Project。
  - AC-2: `doc/README.md` 的更新时间和快速阅读路径已同步。
  - AC-3: `doc/core/project.md` 能追踪该任务。
- Non-Goals:
  - 不重写整份 `doc/README.md`。
  - 不改模块矩阵本身。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 该专题位于 core，聚焦总入口导航与公开入口的同步顺序。
- Integration Points:
  - `doc/README.md`
  - `README.md`
  - `site/index.html`
  - `doc/core/prd.md`
- Edge Cases & Error Handling:
  - 如果公开状态再次变化：总入口需再次同步。
  - 如果模块矩阵不变：只更新阅读顺序与时间戳，不额外扩写。
- Non-Functional Requirements:
  - NFR-DOC-HUB-1: 快速阅读路径应在 1 分钟内可理解。
  - NFR-DOC-HUB-2: 更新时间必须反映实际同步日期。
- Security & Privacy: 仅更新导航，不新增敏感信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (`DOC-HUB-1`): 同步 `doc/README.md` 的公开阅读路径。
  - v1.1 (`DOC-HUB-2`): 如公开入口再变化，继续增量回写。
- Technical Risks:
  - 风险-1: 若总入口不同步，会让新协作者沿旧路径理解项目状态。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-CORE-007 | `TASK-CORE-023` | `test_tier_required` | 检查 `doc/README.md` 含根 README / site 阅读入口 | 全局导航准确性 |
| PRD-CORE-008 | `TASK-CORE-023` | `test_tier_required` | 检查更新时间与新阅读顺序存在 | 公开口径同步性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| `DEC-DOC-HUB-001` | 仅同步快速阅读路径与更新时间 | 重写整份总入口 | 最小改动即可追平当前导航状态。 |
