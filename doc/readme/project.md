# readme PRD Project

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-README-001 (PRD-README-001) [test_tier_required]: 完成 readme PRD 改写，建立对外口径主控入口。
- [x] TASK-README-002 (PRD-README-001/002) [test_tier_required]: 建立 README 与模块 PRD 口径一致性巡检清单。
  - 产物文件:
    - `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.prd.md`
    - `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.design.md`
    - `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.project.md`
    - `doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.md`
    - `doc/readme/governance/producer-to-qa-task-readme-002-consistency-checklist-2026-03-11.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "RC-01|RC-02|RC-03|RC-04|RC-05|RC-06|失败动作|权威源" doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.md doc/readme/governance/readme-consistency-audit-checklist-2026-03-11.prd.md`
    - `./scripts/doc-governance-check.sh`
- 模块设计总览：`doc/readme/design.md`
- [x] TASK-README-003 (PRD-README-002/003) [test_tier_required]: 增加入口链接有效性自动检查任务。
  - 产物文件:
    - `scripts/readme-link-check.sh`
    - `doc/readme/governance/readme-link-check-automation-2026-03-11.prd.md`
    - `doc/readme/governance/readme-link-check-automation-2026-03-11.design.md`
    - `doc/readme/governance/readme-link-check-automation-2026-03-11.project.md`
    - `doc/readme/governance/producer-to-qa-task-readme-003-link-check-automation-2026-03-11.md`
  - 验收命令 (`test_tier_required`):
    - `./scripts/readme-link-check.sh`
    - `./scripts/doc-governance-check.sh`
- [x] TASK-README-004 (PRD-README-003) [test_tier_required]: 形成季度口径审查与修复节奏。
  - 产物文件:
    - `doc/readme/governance/readme-quarterly-review-cycle-2026-03-11.prd.md`
    - `doc/readme/governance/readme-quarterly-review-cycle-2026-03-11.design.md`
    - `doc/readme/governance/readme-quarterly-review-cycle-2026-03-11.project.md`
    - `doc/readme/governance/readme-quarterly-review-template-2026-03-11.md`
    - `doc/readme/governance/readme-remediation-log-template-2026-03-11.md`
    - `doc/readme/governance/producer-to-qa-task-readme-004-quarterly-cycle-2026-03-11.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "Quarter|Trigger|Review Checklist|Remediation ID|状态" doc/readme/governance/readme-quarterly-review-template-2026-03-11.md doc/readme/governance/readme-remediation-log-template-2026-03-11.md`
    - `./scripts/doc-governance-check.sh`
- [x] TASK-README-005 (PRD-README-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-README-006 (PRD-README-004) [test_tier_required]: 形成版本候选对外口径简报，固化 `liveops_community` 的状态摘要、禁用表述、风险边界与回滚口径。

## 依赖
- doc/readme/prd.index.md
- `README.md`
- `world-rule.md`
- `testing-manual.md`
- `doc/readme/gap/`
- `doc/readme/production/`
- `doc/readme/governance/`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-11
- 当前状态: completed
- 下一任务: 无（当前模块主项目无未完成任务）
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- ROUND-002 进展: gap 子簇主从化已完成（gap12345 主入口，其它 gap 专题增量维护）。
- 模块进展补充（2026-03-11）: 已新增 README 口径一致性巡检清单，明确顶层叙事、状态口径、术语边界、入口链接与触发条件五类高优检查项。
- 模块进展补充（2026-03-11 / links）: 已新增 `scripts/readme-link-check.sh`，自动校验 `README.md` 与 `doc/README.md` 的本地 Markdown 入口链接。
- 模块进展补充（2026-03-11 / quarterly）: 已新增季度口径审查节奏与模板，固定季度审查、重大变更加审与修复记录闭环。
- 模块进展补充（2026-03-11 / communication）: 已新增版本候选对外口径简报，承接内部 `go/no-go` 结论并固化禁用表述、风险边界与回滚说明。
- 说明: 本文档仅维护 readme 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md` 与 `doc/devlog/2026-03-11.md`。
