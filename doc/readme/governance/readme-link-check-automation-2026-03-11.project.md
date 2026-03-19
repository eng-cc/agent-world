# oasis7: README 入口链接有效性自动检查（2026-03-11）（项目管理）

- 对应设计文档: `doc/readme/governance/readme-link-check-automation-2026-03-11.design.md`
- 对应需求文档: `doc/readme/governance/readme-link-check-automation-2026-03-11.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] RL-1 (PRD-README-LINK-001/002) [test_tier_required]: 实现 README / `doc/README.md` 本地链接检查脚本。
- [x] RL-2 (PRD-README-LINK-001/003) [test_tier_required]: 回写模块主项目 / index / handoff。
- [x] RL-3 (PRD-README-LINK-002/003) [test_tier_required]: 执行脚本验证并完成 `producer_system_designer -> qa_engineer` 交接。

## 依赖
- `scripts/readme-link-check.sh`
- `README.md`
- `doc/README.md`
- `doc/readme/project.md`

## 状态
- 更新日期：2026-03-11
- 当前阶段：已完成
- 阻塞项：无
- 下一步：转入 `TASK-README-004`，建立季度口径审查与修复节奏。
