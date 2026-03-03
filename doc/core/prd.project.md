# core PRD Project

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-CORE-001 (PRD-CORE-001) [test_tier_required]: 完成 core PRD 改写，固化跨模块治理基线。
- [x] TASK-CORE-002 (PRD-CORE-001/002/003) [test_tier_required]: 将 core PRD 扩展为项目全局总览入口（模块地图/关键链路/关键分册导航）。
- [ ] TASK-CORE-003 (PRD-CORE-001/002) [test_tier_required]: 建立跨模块变更影响检查清单（设计/代码/测试/发布）。
- [ ] TASK-CORE-004 (PRD-CORE-002/003) [test_tier_required]: 建立仓库级 PRD-ID 到测试证据映射模板。
- [ ] TASK-CORE-005 (PRD-CORE-003) [test_tier_required]: 对模块 PRD 进行季度一致性审查并形成审查记录。
- [x] TASK-CORE-006 (PRD-CORE-001/002) [test_tier_required]: 收敛 `doc/` 根目录 legacy redirect 入口并更新总导航。
- [x] TASK-CORE-007 (PRD-CORE-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。

## 依赖
- doc/core/prd.index.md
- `AGENTS.md`
- `doc/README.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`
- 各模块 `doc/<module>/prd.md` 与 `doc/<module>/prd.project.md`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 下一任务: TASK-CORE-003
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 core 设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
