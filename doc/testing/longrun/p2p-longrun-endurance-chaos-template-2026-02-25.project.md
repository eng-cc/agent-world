# Agent World：P2P 长跑 180 分钟 Chaos 模板（项目管理文档）

- 对应设计文档: `doc/testing/longrun/p2p-longrun-endurance-chaos-template-2026-02-25.design.md`
- 对应需求文档: `doc/testing/longrun/p2p-longrun-endurance-chaos-template-2026-02-25.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] CHAOSTPL-1 (PRD-TESTING-LONGRUN-CHAOSTPL-001/002): 完成方案建档（设计文档 + 项目管理文档）。
- [x] CHAOSTPL-2 (PRD-TESTING-LONGRUN-CHAOSTPL-001/002): 新增固定 chaos 模板文件 `p2p-soak-endurance-full-chaos-v1.json` 并覆盖 180 分钟窗口。
- [x] CHAOSTPL-3 (PRD-TESTING-LONGRUN-CHAOSTPL-002/003): 更新 `testing-manual.md` S9 命令与语义边界说明。
- [x] CHAOSTPL-4 (PRD-TESTING-LONGRUN-CHAOSTPL-002): 完成短窗验证与任务日志收口。
- [x] CHAOSTPL-5 (PRD-TESTING-004): 专题文档按 strict schema 人工重写，并切换命名到 `.prd.md/.project.md`。

## 依赖
- doc/testing/longrun/p2p-longrun-endurance-chaos-template-2026-02-25.prd.md
- `doc/testing/chaos-plans/p2p-soak-endurance-full-chaos-v1.json`
- `scripts/p2p-longrun-soak.sh`
- `testing-manual.md`
- `doc/testing/longrun/p2p-longrun-continuous-chaos-injection-2026-02-24.prd.md`
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
