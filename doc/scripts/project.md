# scripts PRD Project

审计轮次: 6

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-SCRIPTS-001 (PRD-SCRIPTS-001) [test_tier_required]: 完成 scripts PRD 改写，建立脚本治理主入口。
- [x] TASK-SCRIPTS-002 (PRD-SCRIPTS-001/002) [test_tier_required]: 梳理脚本分层并标注主入口与 fallback 入口。
  - 产物文件:
    - `doc/scripts/governance/script-entry-layering-2026-03-11.prd.md`
    - `doc/scripts/governance/script-entry-layering-2026-03-11.design.md`
    - `doc/scripts/governance/script-entry-layering-2026-03-11.project.md`
    - `doc/scripts/governance/runtime-to-qa-task-scripts-002-entry-layering-2026-03-11.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "ci-tests.sh|release-gate.sh|run-viewer-web.sh|capture-viewer-frame.sh|site-link-check.sh" doc/scripts/governance/script-entry-layering-2026-03-11.prd.md doc/scripts/governance/script-entry-layering-2026-03-11.project.md`
    - `./scripts/doc-governance-check.sh`
- [x] TASK-SCRIPTS-003 (PRD-SCRIPTS-002/003) [test_tier_required]: 补齐高频脚本参数契约与失败语义说明。
  - 产物文件:
    - `doc/scripts/governance/script-parameter-contracts-2026-03-11.prd.md`
    - `doc/scripts/governance/script-parameter-contracts-2026-03-11.design.md`
    - `doc/scripts/governance/script-parameter-contracts-2026-03-11.project.md`
    - `doc/scripts/governance/runtime-to-qa-task-scripts-003-parameter-contracts-2026-03-11.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "ci-tests.sh|release-gate.sh|build-game-launcher-bundle.sh|run-viewer-web.sh|site-link-check.sh|skip-|dry-run" doc/scripts/governance/script-parameter-contracts-2026-03-11.prd.md doc/scripts/governance/script-parameter-contracts-2026-03-11.project.md`
    - `./scripts/doc-governance-check.sh`
- [x] TASK-SCRIPTS-004 (PRD-SCRIPTS-003) [test_tier_required]: 建立脚本稳定性趋势跟踪指标。
  - 产物文件:
    - `doc/scripts/governance/script-stability-trend-tracking-2026-03-11.prd.md`
    - `doc/scripts/governance/script-stability-trend-tracking-2026-03-11.design.md`
    - `doc/scripts/governance/script-stability-trend-tracking-2026-03-11.project.md`
    - `doc/scripts/evidence/script-stability-trend-baseline-2026-03-11.md`
    - `doc/scripts/governance/runtime-to-qa-task-scripts-004-stability-trend-2026-03-11.md`
  - 验收命令 (`test_tier_required`):
    - `rg -n "主入口覆盖率|参数契约覆盖率|fallback 围栏覆盖率|100%|0d" doc/scripts/evidence/script-stability-trend-baseline-2026-03-11.md doc/scripts/governance/script-stability-trend-tracking-2026-03-11.prd.md`
    - `./scripts/doc-governance-check.sh`
- [x] TASK-SCRIPTS-005 (PRD-SCRIPTS-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-SCRIPTS-006 (PRD-SCRIPTS-001) [test_tier_required]: 同步 `doc/scripts/README.md` 的模块入口索引，补齐近期专题、模块职责与根目录收口口径。
- [x] TASK-SCRIPTS-007 (PRD-SCRIPTS-001) [test_tier_required]: 修正 `doc/scripts/prd.index.md` 的 governance 表格格式，确保文件级索引在 Markdown 渲染中连续可读。
- [x] TASK-SCRIPTS-008 (PRD-SCRIPTS-001) [test_tier_required]: 收口 `doc/scripts/**` 治理专题标题的 `oasis7` 品牌，避免脚本治理入口继续混用旧 `Agent World` 标题。
  - 验收命令 (`test_tier_required`):
    - `rg -n "^# oasis7" doc/scripts --glob '!third_party/**'`
    - `rg -n "^# Agent World" doc/scripts --glob '!third_party/**'`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [x] TASK-SCRIPTS-009 (PRD-SCRIPTS-001) [test_tier_required]: 收口 `doc/scripts/precommit/pre-commit.{prd,project}.md` 中当前 viewer wasm 编译门禁与依赖说明的 crate 命名，统一使用 `oasis7_viewer` 口径。
  - 验收命令 (`test_tier_required`):
    - `rg -n "oasis7_viewer|cargo check -p oasis7_viewer" doc/scripts/precommit/pre-commit.prd.md doc/scripts/precommit/pre-commit.project.md`
    - `rg -n "agent_world_viewer|cargo check -p agent_world_viewer" doc/scripts/precommit/pre-commit.prd.md doc/scripts/precommit/pre-commit.project.md`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [x] TASK-SCRIPTS-010 (PRD-SCRIPTS-003) [test_tier_required]: 收口 `doc/scripts/viewer-tools/capture-viewer-frame.{prd,project}.md` 中当前 native fallback viewer 调试说明的 crate 与环境变量命名，统一使用 `oasis7_viewer` / `OASIS7_VIEWER_*` 口径。
  - 验收命令 (`test_tier_required`):
    - `rg -n "oasis7_viewer|OASIS7_VIEWER_" doc/scripts/viewer-tools/capture-viewer-frame.prd.md doc/scripts/viewer-tools/capture-viewer-frame.project.md`
    - `rg -n "agent_world_viewer|AGENT_WORLD_VIEWER_" doc/scripts/viewer-tools/capture-viewer-frame.prd.md doc/scripts/viewer-tools/capture-viewer-frame.project.md`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`
- [x] TASK-SCRIPTS-011 (PRD-SCRIPTS-001) [test_tier_required]: 收口 repo-owned OpenClaw real-play helper 文档与脚本中的当前 cargo 运行命令和入口路径，统一使用 `oasis7` / `crates/oasis7*` 口径。
  - 验收命令 (`test_tier_required`):
    - `rg -n "cargo run -p oasis7|crates/oasis7/src/bin/" .agents/skills/oasis7/SKILL.md .agents/skills/oasis7/references/real-play-config.md .agents/skills/oasis7/scripts/oasis7-run.sh`
    - `rg -n "cargo run -p agent_world|crates/agent_world/src/bin/" .agents/skills/oasis7/SKILL.md .agents/skills/oasis7/references/real-play-config.md .agents/skills/oasis7/scripts/oasis7-run.sh`
    - `./scripts/doc-governance-check.sh`
    - `git diff --check`

## 依赖
- 模块设计总览：`doc/scripts/design.md`
- doc/scripts/prd.index.md
- `scripts/`
- `doc/scripts/precommit/pre-commit.prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-20
- 当前状态: completed
- 下一任务: 无（当前模块主项目无未完成任务）
- 最新完成: `TASK-SCRIPTS-011`（repo-owned OpenClaw real-play helper 文档与脚本中的当前 cargo 运行命令和入口路径已统一切到 `oasis7` / `crates/oasis7*` 当前口径。）
- 最新完成: `TASK-SCRIPTS-010`（`capture-viewer-frame` 活跃 fallback 文档中的 viewer crate 与环境变量命名已统一切到 `oasis7_viewer` / `OASIS7_VIEWER_*` 当前口径）。
- 最新完成: `TASK-SCRIPTS-009`（pre-commit 活跃文档中的 viewer wasm 编译门禁与依赖说明已统一切到 `oasis7_viewer` 当前口径）。
- 最新完成: `TASK-SCRIPTS-006`（scripts 模块 README 入口索引同步）。
- 最新完成: `TASK-SCRIPTS-007`（scripts 文件级索引表格格式修正）。
- 最新完成: `TASK-SCRIPTS-008`（scripts 治理专题标题统一切到 `oasis7` 品牌）。
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 模块进展补充（2026-03-11）: 已新增 scripts 分层专题，明确 `ci-tests.sh`、`release-gate.sh`、`run-viewer-web.sh` 等主入口，以及 `capture-viewer-frame.sh` 的 fallback 围栏。
- 模块进展补充（2026-03-11 / contracts）: 已新增高频脚本参数契约专题，冻结 `ci-tests.sh`、`release-gate.sh`、`build-game-launcher-bundle.sh`、`run-viewer-web.sh`、`site-link-check.sh` 的最小调用、默认值与失败语义。
- 模块进展补充（2026-03-11 / trend）: 已新增 scripts 稳定性趋势专题与 baseline，建立主入口覆盖率、参数契约覆盖率、fallback 围栏覆盖率、治理修复时长四项指标。
- 说明: 本文档仅维护 scripts 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md` 与 `doc/devlog/2026-03-11.md`。
