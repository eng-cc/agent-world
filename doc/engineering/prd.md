# engineering PRD

## 目标
- 建立 engineering 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 engineering 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 engineering 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/engineering/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/engineering/prd.md`
- 项目管理入口: `doc/engineering/prd.project.md`
- 追踪主键: `PRD-ENGINEERING-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 工程规范分散在多个专题文档，导致文件体量控制、提交门禁、脚本治理与代码质量标准不够统一。
- Proposed Solution: 将 engineering 模块定义为工程治理主文档，统一维护规范、质量门禁、改造节奏与验收口径。
- Success Criteria:
  - SC-1: Rust 单文件超 1200 行新增违规数为 0。
  - SC-2: Markdown 单文件超 500 行新增违规数为 0。
  - SC-3: `scripts/doc-governance-check.sh` 在 required gate 连续通过。
  - SC-4: 工程类任务 100% 映射到 PRD-ENGINEERING-ID。
  - SC-5: `doc/` 根目录与模块根目录平铺文档新增违规数为 0（allowlist 冻结机制）。
  - SC-6: 重点模块（world-simulator/p2p/world-runtime/testing/site/readme/scripts/game/headless-runtime）根目录平铺专题文档迁移完成并保持引用闭环。
  - SC-7: 活跃设计文档（非 archive/devlog）中 strict schema 覆盖率按周提升，并保留逐篇人工迁移记录。
  - SC-8: 完成四人并行迁移分工，待迁移清单有冻结快照且每日可追踪燃尽进度。

## 2. User Experience & Functionality
- User Personas:
  - 工程维护者：需要稳定规则来控制技术债。
  - 贡献开发者：需要清晰门槛和提交前检查路径。
  - 评审者：需要可量化判断变更是否合规。
- User Scenarios & Frequency:
  - 日常提交前检查：每次提交前执行，确认格式、结构与门禁符合要求。
  - CI 失败排查：每个异常流水线触发后执行，定位脚本与规则来源。
  - 规范迭代评审：每周至少 1 次，评估误报率和治理收益。
  - 季度治理复盘：每季度 1 次，回看违规趋势与修复效率。
- User Stories:
  - PRD-ENGINEERING-001: As an 工程维护者, I want enforceable file-size and structure limits, so that maintenance cost stays bounded.
  - PRD-ENGINEERING-002: As a 开发者, I want deterministic pre-commit checks, so that regressions are caught before CI.
  - PRD-ENGINEERING-003: As a 评审者, I want auditable governance evidence, so that review decisions are defensible.
  - PRD-ENGINEERING-004: As a 文档维护者, I want legacy docs migrated with per-doc manual review, so that content intent is preserved while converging to strict schema.
  - PRD-ENGINEERING-005: As a 协调人, I want one collaboration doc with principles and owner boundaries, so that parallel migration is deterministic.
  - PRD-ENGINEERING-006: As a 迁移执行人, I want non-overlapping migration scopes, so that I can avoid merge conflicts while moving fast.
  - PRD-ENGINEERING-007: As a 质量复核人, I want measurable acceptance gates for migrated docs, so that content fidelity is auditable.
- Critical User Flows:
  1. Flow-ENG-001: `提交前执行脚本 -> 发现违规 -> 修复并复测 -> 进入 CI`
  2. Flow-ENG-002: `CI 失败 -> 定位规则来源 -> 判断误报/真实问题 -> 更新脚本或文档`
  3. Flow-ENG-003: `季度复盘 -> 汇总违规趋势 -> 调整门禁阈值 -> 发布新治理基线`
  4. Flow-ENG-004: `逐篇阅读旧文档 -> 按 strict schema 重写 -> 内容保真复核 -> 更新任务与devlog追踪`
  5. Flow-ENG-005: `冻结待迁移清单 -> 按 Owner-A/B/C/D 切分范围 -> 并行执行 -> 每日燃尽收口`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 文档治理检查 | allowlist、模块根目录规则、根目录规则 | 执行 `doc-governance-check.sh` | `pass/fail` | 按违规严重度输出 | 所有人可执行，治理维护者可更新基线 |
| 提交前检查 | 格式、静态检查、测试分层触发 | pre-commit 自动执行 | `pending -> running -> blocked/passed` | 先 required 后 full | 贡献者可触发，CI 负责人可调整策略 |
| 工程趋势统计 | 违规率、修复时长、回归率 | 周期性生成报表并复盘 | `collecting -> reported -> actioned` | 按模块与时间排序 | 评审者与维护者可读写 |
| PRD 格式迁移 | 文档路径、迁移批次ID、原文关键约束点 | 人工阅读原文后按 strict schema 重写并复核 | `inventory -> migrated -> verified` | 默认按活跃文档优先、按模块分批 | 治理维护者可冻结批次，贡献者可提交迁移 |
| 并行迁移协作 | Owner、范围、快照日期、燃尽统计 | 依据协作方案分批推进迁移 | `planned -> in_progress -> done` | 目录前缀互斥，按负载均衡调整 | 协调人分配，Owner 执行，复核人抽检 |
- Acceptance Criteria:
  - AC-1: engineering PRD 明确文件约束、脚本约束、测试分层约束。
  - AC-2: engineering project 文档维护任务拆解与状态。
  - AC-3: 与 `doc/scripts/precommit/pre-commit.md`、`testing-manual.md` 的口径一致。
  - AC-4: 每次工程规范变更有对应 devlog 记录。
  - AC-5: 文档治理脚本校验 `doc/.governance/*-allowlist.txt`，可拦截 `doc/*.md` 与 `doc/<module>/*.md` 的非预期新增。
  - AC-6: `doc/world-simulator`、`doc/p2p`、`doc/world-runtime`、`doc/testing`、`doc/site`、`doc/readme`、`doc/scripts`、`doc/game`、`doc/headless-runtime` 模块根目录仅保留 `README.md` / `prd.md` / `prd.project.md`。
  - AC-7: 每次迁移任务需附“原文关键约束点 -> 新文档章节”对照，确保内容不丢失。
  - AC-8: 并行迁移必须有公开分工表、待迁移快照和每日燃尽更新机制。
- Non-Goals:
  - 不定义 gameplay/p2p/runtime 业务规则。
  - 不替代模块内部测试策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 文档治理脚本、CI 测试脚本、静态检查脚本。
- Evaluation Strategy: 通过 required/full gate 成功率、违规项统计、回归修复时长衡量工程治理有效性。

## 4. Technical Specifications
- Architecture Overview: engineering 模块聚焦工程流程与规范，不承载业务逻辑；通过脚本与门禁把规范落地到提交链路。
- Integration Points:
  - `scripts/doc-governance-check.sh`
  - `doc/scripts/precommit/pre-commit.md`
  - `doc/scripts/precommit/fix-precommit.md`
  - `doc/.governance/doc-root-md-allowlist.txt`
  - `doc/.governance/module-root-md-allowlist.txt`
  - `doc/engineering/doc-migration/legacy-doc-migration-collaboration-2026-03-03.prd.md`
  - `doc/engineering/doc-migration/legacy-doc-migration-collaboration-2026-03-03.prd.project.md`
  - `doc/engineering/doc-migration/legacy-doc-migration-backlog-2026-03-03.md`
  - `testing-manual.md`
  - `.github/workflows/*`
- Edge Cases & Error Handling:
  - allowlist 漂移：检测到未登记新增时直接失败并提示最小修复路径。
  - 误报场景：规则误伤时保留失败证据并通过任务流程修订规则，不直接绕过。
  - 本地/CI 不一致：本地通过但 CI 失败时以 CI 结果为准并补环境对齐说明。
  - 脚本不可执行：缺依赖时给出明确安装建议与最小复现命令。
  - 并发修改冲突：同一规则多分支更新时以最新主干基线重放验证。
  - 历史文档例外：仅允许 archive/devlog 例外，不得扩散到活跃目录。
  - 新旧格式并存：迁移中允许 legacy 与 strict 共存，但每个迁移批次必须标注边界并回写追踪状态。
  - 批量迁移回归风险：结构改写可能造成引用断链，需附带路径扫描与脚本复核。
- Non-Functional Requirements:
  - NFR-ENG-1: required 门禁平均执行时长 <= 10 分钟。
  - NFR-ENG-2: 文档治理误报率 <= 5%（按周统计）。
  - NFR-ENG-3: 新增工程任务 PRD-ID 映射覆盖率 100%。
  - NFR-ENG-4: 工程治理脚本在 Linux/macOS 环境均可执行。
  - NFR-ENG-5: 规则变更需附带可追溯说明与回归证据。
  - NFR-ENG-6: 活跃文档迁移任务必须包含“原文约束点清单 + 新文档章节映射 + 回归验证结果”三件套证据。
  - NFR-ENG-7: 并行迁移阶段每工作日至少完成 16 篇迁移（4 人 * 人均 4 篇）。
- Security & Privacy: 仅涉及工程流程元信息；涉及凭据的自动化流程必须遵守最小暴露原则并避免日志泄漏。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化工程规范与门禁指标。
  - v1.1: 补齐高频违规的自动修复建议与脚本化诊断。
  - v2.0: 建立工程规范趋势看板（违规率、修复时长、回归率）。
- Technical Risks:
  - 风险-1: 规范过严导致迭代效率下降。
  - 风险-2: 新脚本引入误报造成 CI 噪声。
  - 风险-3: 老文档迁移批次过大导致评审负担与引用回归风险提升。
  - 风险-4: 多人并行对同一目录写入造成冲突与重复迁移。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-ENGINEERING-001 | TASK-ENGINEERING-001/005/006/007 | `test_tier_required` | 文档结构检查、平铺治理脚本执行 | 文档组织一致性、工程可维护性 |
| PRD-ENGINEERING-002 | TASK-ENGINEERING-002/003/007 | `test_tier_required` + `test_tier_full` | pre-commit/CI 门禁联动校验 | 提交流程稳定性、回归拦截能力 |
| PRD-ENGINEERING-003 | TASK-ENGINEERING-003/004/007 | `test_tier_required` | 趋势统计与审查模板抽样检查 | 工程治理可审计性与长期演进 |
| PRD-ENGINEERING-004 | TASK-ENGINEERING-008/009 | `test_tier_required` | 原文约束点对照、迁移后治理脚本与引用扫描 | 文档格式一致性与内容保真 |
| PRD-ENGINEERING-005 | TASK-ENGINEERING-010 | `test_tier_required` | 协作主文档结构与分工边界校验 | 并行迁移入口一致性 |
| PRD-ENGINEERING-006 | TASK-ENGINEERING-011/012/013/014 | `test_tier_required` | 按 Owner 责任域抽样检查迁移提交 | 并行效率与冲突控制 |
| PRD-ENGINEERING-007 | TASK-ENGINEERING-015 | `test_tier_required` + `test_tier_full` | 全量迁移收尾扫描、命名与引用一致性验证 | 全仓文档治理收口质量 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-ENG-001 | 以脚本门禁落实规范 | 仅依赖人工评审 | 自动化一致性更高且可复现。 |
| DEC-ENG-002 | 保留 allowlist 冻结机制 | 完全开放文档新增 | 可控制结构漂移和历史债扩散。 |
| DEC-ENG-003 | required/full 分层验证 | 单层测试策略 | 兼顾效率与风险覆盖。 |
| DEC-ENG-004 | 老格式文档按批次渐进迁移并采用逐篇人工重写 | 一次性全量改写或自动脚本改写 | 人工重写更利于保留语义细节并控制内容质量。 |
| DEC-ENG-005 | 采用四人并行、目录前缀互斥分工推进大规模迁移 | 单人串行推进或随机切片 | 可兼顾迁移速度、冲突控制与审阅可追溯性。 |
