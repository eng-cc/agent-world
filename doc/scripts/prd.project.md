# scripts PRD Project

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-SCRIPTS-001 (PRD-SCRIPTS-001): 完成 scripts PRD 改写，建立脚本治理主入口。
- [ ] TASK-SCRIPTS-002 (PRD-SCRIPTS-001/002): 梳理脚本分层并标注主入口与 fallback 入口。
- [ ] TASK-SCRIPTS-003 (PRD-SCRIPTS-002/003): 补齐高频脚本参数契约与失败语义说明。
- [ ] TASK-SCRIPTS-004 (PRD-SCRIPTS-003): 建立脚本稳定性趋势跟踪指标。
- [x] TASK-SCRIPTS-005 (PRD-SCRIPTS-001/002/003): 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。

## 依赖
- doc/scripts/prd.index.md
- `scripts/`
- `doc/scripts/precommit/pre-commit.prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 下一任务: TASK-SCRIPTS-002
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 scripts 模块设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
