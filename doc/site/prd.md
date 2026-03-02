# site PRD

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
- User Stories:
  - PRD-SITE-001: As a 新访问者, I want a clear homepage narrative, so that I can understand the product quickly.
  - PRD-SITE-002: As a 技术用户, I want trustworthy download and docs links, so that I can install and verify efficiently.
  - PRD-SITE-003: As a 维护者, I want measurable quality gates, so that releases are predictable.
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
  - `doc/site/*.md`
  - `doc/readme/prd.md`
- Security & Privacy: 站点不得暴露内部凭据与敏感配置；下载链路需具备来源可验证性。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化站点信息架构与发布验收口径。
  - v1.1: 增加多语言内容一致性与截图回归基线。
  - v2.0: 建立站点发布质量趋势跟踪（性能、可访问性、失效率）。
- Technical Risks:
  - 风险-1: 内容更新频率高导致页面口径漂移。
  - 风险-2: 发布资产链接策略变化引入断链风险。
