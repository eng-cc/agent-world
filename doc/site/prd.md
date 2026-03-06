# site PRD

审计轮次: 3

## 目标
- 建立 site 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 site 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 site 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/site/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/site/prd.md`
- 项目管理入口: `doc/site/prd.project.md`
- 文件级索引: doc/site/prd.index.md
- 追踪主键: `PRD-SITE-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 站点页面、发布下载入口、文档镜像与 SEO 优化在多轮迭代中快速演进，缺少统一设计文档来约束信息架构与发布质量。
- Proposed Solution: 将 site PRD 作为站点设计主入口，统一页面结构、内容同步、发布链路与质量门禁。
- Success Criteria:
  - SC-1: 首页关键信息架构与模块设计口径保持一致。
  - SC-2: 发布下载链接在版本发布后可用率达到 100%。
  - SC-3: 关键页面的可访问性与性能指标持续满足目标阈值。
  - SC-4: 站点改动任务全部映射到 PRD-SITE-ID。

## 2. User Experience & Functionality
- User Personas:
  - 新访问者：需要快速理解项目价值与安装入口。
  - 技术用户：需要稳定访问文档与发布资产。
  - 站点维护者：需要统一发布与验收标准。
- User Scenarios & Frequency:
  - 首页信息浏览：每位新访问者首次访问执行。
  - 文档与下载访问：技术用户按需高频访问。
  - 发布前巡检：每次版本发布前执行一次完整检查。
  - 发布后回归：每次发布后执行稳定性与断链复核。
- User Stories:
  - PRD-SITE-001: As a 新访问者, I want a clear homepage narrative, so that I can understand the product quickly.
  - PRD-SITE-002: As a 技术用户, I want trustworthy download and docs links, so that I can install and verify efficiently.
  - PRD-SITE-003: As a 维护者, I want measurable quality gates, so that releases are predictable.
- Critical User Flows:
  1. Flow-SITE-001: `访问首页 -> 理解价值与入口 -> 跳转安装/文档`
  2. Flow-SITE-002: `发布前执行链接检查 -> 处理断链 -> 复测通过`
  3. Flow-SITE-003: `发布后监控质量指标 -> 发现退化 -> 回滚或修复`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 首页信息架构 | 版块标题、入口链接、版本信息 | 点击跳转安装/文档/下载 | `draft -> published -> revised` | 关键信息优先上屏 | 站点维护者可修改 |
| 发布下载链路 | 版本号、资产地址、校验信息 | 发布后自动校验可用性 | `prepared -> published -> verified` | 最新版本优先展示 | 发布负责人审批上线 |
| 质量门禁巡检 | 链接状态、性能指标、可访问性结果 | 巡检失败阻断发布 | `checking -> passed/blocked` | 严重问题优先修复 | 维护者可解阻断（需说明） |
- Acceptance Criteria:
  - AC-1: site PRD 定义页面层级、内容同步和发布链路。
  - AC-2: site project 文档任务映射 PRD-SITE-ID。
  - AC-3: 与 `site/doc` 与 GitHub Pages 相关设计文档口径一致。
  - AC-4: 发布前完成链接有效性与基础质量检查。
- Non-Goals:
  - 不在 site PRD 中定义 runtime/p2p 低层实现。
  - 不覆盖内部测试流程细节（由 testing 模块负责）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 静态站构建链路、链接检查、截图/视觉基线流程。
- Evaluation Strategy: 以页面可用性、链接有效率、发布后回滚次数与问题修复时长评估。

## 4. Technical Specifications
- Architecture Overview: site 模块负责对外展示与文档镜像层，与 readme/testing/world-simulator 等模块联动维护入口一致性。
- Integration Points:
  - `site/`
  - `site/doc/`
  - `doc/site/github-pages/`
  - `doc/site/manual/`
  - `doc/readme/prd.md`
- Edge Cases & Error Handling:
  - 断链：发现下载或文档断链时阻断发布并进入修复流程。
  - 空页面：关键页面内容缺失时展示维护提示并记录异常。
  - 权限不足：发布权限缺失时拒绝上线并提示责任人。
  - 超时：构建/巡检超时时输出中间结果并允许重试。
  - 并发发布：同版本并发发布时只允许一个发布会话生效。
  - 数据异常：版本元数据错误时不展示到公开页面。
- Non-Functional Requirements:
  - NFR-SITE-1: 发布后关键链接可用率 100%。
  - NFR-SITE-2: 核心页面性能与可访问性指标达到门禁阈值。
  - NFR-SITE-3: 多语言内容口径一致并可追溯。
  - NFR-SITE-4: 发布回滚流程可在限定时间内执行。
  - NFR-SITE-5: 站点输出不得暴露内部敏感配置。
- Security & Privacy: 站点不得暴露内部凭据与敏感配置；下载链路需具备来源可验证性。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化站点信息架构与发布验收口径。
  - v1.1: 增加多语言内容一致性与截图回归基线。
  - v2.0: 建立站点发布质量趋势跟踪（性能、可访问性、失效率）。
- Technical Risks:
  - 风险-1: 内容更新频率高导致页面口径漂移。
  - 风险-2: 发布资产链接策略变化引入断链风险。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-SITE-001 | TASK-SITE-001/002/005 | `test_tier_required` | 首页结构与导航检查 | 用户首次访问体验 |
| PRD-SITE-002 | TASK-SITE-002/003/005 | `test_tier_required` | 下载与文档链接巡检 | 安装与文档可用性 |
| PRD-SITE-003 | TASK-SITE-003/004/005 | `test_tier_required` + `test_tier_full` | 发布门禁与回归节奏复核 | 发布稳定性与回滚能力 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-SITE-001 | 发布前强制执行质量巡检 | 发布后补检查 | 可提前发现阻断问题。 |
| DEC-SITE-002 | 下载链路绑定版本与校验信息 | 仅展示下载地址 | 可提升来源可信度。 |
| DEC-SITE-003 | 站点口径与 readme 联动维护 | 独立维护站点文案 | 可降低对外口径漂移。 |
