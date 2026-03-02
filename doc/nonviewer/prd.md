# nonviewer PRD

## 目标
- 建立 nonviewer 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 nonviewer 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 nonviewer 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/nonviewer/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/nonviewer/prd.md`
- 项目管理入口: `doc/nonviewer/prd.project.md`
- 追踪主键: `PRD-NONVIEWER-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: non-viewer 链路承担长稳运行、协议交互、鉴权与归档，但设计信息分散，影响生产链路一致性和故障定位效率。
- Proposed Solution: 以 nonviewer PRD 统一定义无界面运行链路的目标、接口、稳定性指标与安全约束。
- Success Criteria:
  - SC-1: nonviewer 关键链路（启动、鉴权、归档）均有可追溯 PRD-ID。
  - SC-2: 长稳运行用例在 `test_tier_full` 下可复现且通过率达到目标阈值。
  - SC-3: 鉴权协议变更具备文档、测试、回归三方联动记录。
  - SC-4: 线上问题可通过日志/归档定位到明确阶段与责任模块。

## 2. User Experience & Functionality
- User Personas:
  - 运行链路维护者：需要稳定的 nonviewer 生命周期定义。
  - 协议开发者：需要明确鉴权与状态同步边界。
  - 运维/发布人员：需要故障可追溯与可回滚策略。
- User Stories:
  - PRD-NONVIEWER-001: As a 维护者, I want a deterministic nonviewer lifecycle, so that online stability improves.
  - PRD-NONVIEWER-002: As a 协议开发者, I want explicit auth and transport contracts, so that client/server evolution stays compatible.
  - PRD-NONVIEWER-003: As an 运维人员, I want traceable archive and incident evidence, so that postmortems are efficient.
- Acceptance Criteria:
  - AC-1: nonviewer PRD 定义生命周期、鉴权、归档三条主线。
  - AC-2: nonviewer project 文档维护对应任务拆解与状态。
  - AC-3: 与 `doc/nonviewer/nonviewer-onchain-auth-protocol-hardening.md` 等专题文档一致。
  - AC-4: 对外行为变更时同步补齐测试与 devlog 记录。
- Non-Goals:
  - 不在本 PRD 中重写 viewer UI 行为。
  - 不替代 p2p 共识层详细设计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 长稳脚本、协议回归测试、日志归档与审计工具。
- Evaluation Strategy: 以连续运行稳定性、鉴权失败率、故障恢复时长、可追溯证据完整度评估。

## 4. Technical Specifications
- Architecture Overview: nonviewer 作为无界面运行链路，连接 runtime/p2p 能力并提供生产可运维能力，强调协议一致性与可恢复性。
- Integration Points:
  - `doc/nonviewer/nonviewer-onchain-auth-protocol-hardening.md`
  - `doc/nonviewer/nonviewer-longrun-traceable-memory-archive-hardening-2026-02-23.md`
  - `testing-manual.md`
- Security & Privacy: 非 viewer 链路涉及鉴权密钥与节点身份，必须执行最小权限、日志脱敏、签名校验和审计留痕。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化 nonviewer 生命周期与协议边界。
  - v1.1: 补齐归档与回滚流程的一致性验证。
  - v2.0: 形成在线稳定性与鉴权质量的长期趋势指标。
- Technical Risks:
  - 风险-1: 协议版本演进导致兼容性回归。
  - 风险-2: 归档链路异常时影响问题追溯完整性。
