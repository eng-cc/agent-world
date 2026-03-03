# engineering PRD Project

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-ENGINEERING-001 (PRD-ENGINEERING-001): 完成 engineering PRD 改写，建立工程治理主文档。
- [x] TASK-ENGINEERING-002 (PRD-ENGINEERING-001/002): 补齐高频违规（超行数/超文档长度/文档平铺新增）自动诊断与门禁建议。
- [x] TASK-ENGINEERING-005 (PRD-ENGINEERING-001/002): 执行文档平铺存量迁移批次（world-simulator/p2p），并更新 allowlist 与引用路径。
- [x] TASK-ENGINEERING-006 (PRD-ENGINEERING-001/002): 执行文档平铺存量迁移批次（world-runtime/testing/site/readme/scripts/game/headless-runtime），并更新 allowlist 与引用路径。
- [ ] TASK-ENGINEERING-003 (PRD-ENGINEERING-002/003): 建立工程门禁趋势统计（违规率、修复时长）。
- [ ] TASK-ENGINEERING-004 (PRD-ENGINEERING-003): 增加工程规范季度审查流程与记录模板。
- [x] TASK-ENGINEERING-007 (PRD-ENGINEERING-001/002/003): 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。
- [x] TASK-ENGINEERING-008 (PRD-ENGINEERING-004): 按“逐篇阅读 + 人工重写”完成老格式文档迁移试点（`oversized-rust-file-splitting-round3-2026-02-23`）并补齐内容保真映射。
- [ ] TASK-ENGINEERING-009 (PRD-ENGINEERING-004): 按模块分批推进活跃老格式文档逐篇人工迁移并持续回写 PRD-ID / project / devlog。

## 依赖
- `AGENTS.md`
- `doc/scripts/precommit/pre-commit.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`
- `doc/engineering/*.md`
- `doc/engineering/oversized-rust-file-splitting-round3-2026-02-23.prd.md`
- `doc/engineering/oversized-rust-file-splitting-round3-2026-02-23.prd.project.md`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 下一任务: TASK-ENGINEERING-009
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 engineering 设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
