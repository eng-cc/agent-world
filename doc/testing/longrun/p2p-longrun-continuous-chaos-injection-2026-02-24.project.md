# Agent World：P2P 长跑持续 Chaos 注入（项目管理文档）

- 对应设计文档: `doc/testing/longrun/p2p-longrun-continuous-chaos-injection-2026-02-24.design.md`
- 对应需求文档: `doc/testing/longrun/p2p-longrun-continuous-chaos-injection-2026-02-24.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] CHAOSCNT-1 (PRD-TESTING-LONGRUN-CHAOS-001/002): 完成方案建档（设计文档 + 项目管理文档）。
- [x] CHAOSCNT-2 (PRD-TESTING-LONGRUN-CHAOS-001/003): 实现持续注入调度核心（参数解析、校验、混合模式执行）。
- [x] CHAOSCNT-3 (PRD-TESTING-LONGRUN-CHAOS-002/003): 实现证据与统计扩展（`run_config.json`、`summary.json`、`summary.md`）。
- [x] CHAOSCNT-4 (PRD-TESTING-LONGRUN-CHAOS-002): 完成 `testing-manual.md` S9 接线与使用说明。
- [x] CHAOSCNT-5 (PRD-TESTING-LONGRUN-CHAOS-003): 完成短窗 continuous chaos 实跑、任务日志与状态收口。
- [x] CHAOSCNT-6 (PRD-TESTING-004): 专题文档按 strict schema 人工重写，并切换命名到 `.prd.md/.project.md`。

## 依赖
- doc/testing/longrun/p2p-longrun-continuous-chaos-injection-2026-02-24.prd.md
- `scripts/p2p-longrun-soak.sh`
- `testing-manual.md`
- `doc/testing/longrun/p2p-storage-consensus-longrun-online-stability-2026-02-24.prd.md`
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
