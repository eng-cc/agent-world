# headless-runtime PRD Project（原 nonviewer）

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-NONVIEWER-001 (PRD-NONVIEWER-001) [test_tier_required]: 完成 headless-runtime PRD 改写，建立无界面链路设计入口。
- [ ] TASK-NONVIEWER-002 (PRD-NONVIEWER-001/002) [test_tier_required]: 补齐生命周期与鉴权协议的一致性检查清单。
- [ ] TASK-NONVIEWER-003 (PRD-NONVIEWER-002/003) [test_tier_required]: 建立长稳归档与故障追溯证据模板。
- [ ] TASK-NONVIEWER-004 (PRD-NONVIEWER-003) [test_tier_required]: 联动 testing 模块完善 headless-runtime 长稳门禁。
- [x] TASK-NONVIEWER-005 (PRD-NONVIEWER-001/002/003) [test_tier_required]: 对齐 strict PRD schema，补齐关键流程/规格矩阵/边界异常/NFR/验证与决策记录。

## 依赖
- 模块设计总览：`doc/headless-runtime/design.md`
- doc/headless-runtime/prd.index.md
- `doc/headless-runtime/nonviewer/nonviewer-onchain-auth-protocol-hardening.prd.md`
- `doc/headless-runtime/nonviewer/nonviewer-longrun-traceable-memory-archive-hardening-2026-02-23.prd.md`
- `testing-manual.md`
- `.agents/skills/prd/check.md`

## 状态
- 更新日期: 2026-03-03
- 当前状态: active
- 下一任务: TASK-NONVIEWER-002
- PRD 质量门状态: strict schema 已对齐（含第 6 章验证与决策记录）。
- 说明: 本文档仅维护 headless-runtime（原 nonviewer）设计执行状态；过程记录在 `doc/devlog/2026-03-03.md`。
