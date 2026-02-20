# README 口径对齐：LLM P1/P2 生产级收口（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-llm-p1p2-production-closure.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：LLM tool 协议收口（补齐 tool 注册/映射/提示词一致性）
- [x] T2：Observation 快照扩展 + LLM 查询面扩展（module/power/social）
- [ ] T3：execution bridge 支持 `SimulatorAction` 执行与审计记录
- [ ] T4：required-tier 回归、文档与 devlog 收口

## 依赖
- T2 依赖 T1 的 tool 名称和模块名规范冻结。
- T3 与 T1/T2 可并行开发，但需在 T4 一起回归。
- T4 依赖 T1/T2/T3 完成后统一执行。

## 状态
- 当前阶段：进行中（T0/T1/T2 完成，T3 开始）。
- 阻塞项：无。
- 下一步：执行 T3（execution bridge 支持 `SimulatorAction` 执行与审计记录）。 
