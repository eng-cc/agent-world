# oasis7 Simulator：Agent Frag 初始站位优化（项目管理文档）

- 对应设计文档: `doc/world-simulator/scenario/agent-frag-initial-spawn-position.design.md`
- 对应需求文档: `doc/world-simulator/scenario/agent-frag-initial-spawn-position.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] FSP1 设计文档与项目管理文档落地
- [x] FSP2 初始化逻辑改造（frag 优先出生 + 上方约 50m 站位）
- [x] FSP3 测试回归与文档/日志收口
- [x] FSP4 2D 遮挡修复（严格正上方出生 + Viewer 保留 standoff 渲染）

## 依赖
- doc/world-simulator/scenario/agent-frag-initial-spawn-position.prd.md
- `crates/oasis7/src/simulator/init.rs`
- `crates/oasis7/src/simulator/tests/mod.rs`
- `crates/oasis7/src/simulator/tests/*`

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段：FSP4（完成）
