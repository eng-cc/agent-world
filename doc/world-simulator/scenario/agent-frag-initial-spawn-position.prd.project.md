# Agent World Simulator：Agent Frag 初始站位优化（项目管理文档）

审计轮次: 3

## 任务拆解（含 PRD-ID 映射）
- [x] FSP1 设计文档与项目管理文档落地
- [x] FSP2 初始化逻辑改造（frag 优先出生 + 上方约 50m 站位）
- [x] FSP3 测试回归与文档/日志收口
- [x] FSP4 2D 遮挡修复（严格正上方出生 + Viewer 保留 standoff 渲染）

## 依赖
- doc/world-simulator/scenario/agent-frag-initial-spawn-position.prd.md
- `crates/agent_world/src/simulator/init.rs`
- `crates/agent_world/src/simulator/tests/mod.rs`
- `crates/agent_world/src/simulator/tests/*`

## 状态
- 当前阶段：FSP4（完成）
