# README 生产级收口：LLM 制度动作 + DistFS 状态主路径 + 去中心化默认拓扑（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-prod-closure-llm-distfs-consensus.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：扩展 LLM 决策主链路（市场/制度动作）+ `test_tier_required` 用例
- [x] T2：DistFS 默认持久化主路径（写入+优先恢复+JSON 兜底）+ persistence 用例
- [x] T3：live 生产默认拓扑（triad）+ CLI/拓扑测试
- [x] T4：回归验证（`env -u RUSTC_WRAPPER cargo check` + 定向 tests）+ 文档/devlog 收口

## 依赖
- T2 依赖 runtime 现有 `segment_snapshot/segment_journal` 能力，优先在 world persistence 层接入。
- T3 依赖 T1/T2 无强耦合，可并行开发，但统一在 T4 回归收口。
- T4 依赖 T1/T2/T3 全部完成后统一执行。

## 状态
- 当前阶段：已完成（T0~T4 全部收口）
- 阻塞项：无
- 下一步：无（等待下一轮需求）。
