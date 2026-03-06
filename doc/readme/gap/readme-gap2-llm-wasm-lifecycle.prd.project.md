# README 缺口 2 收口：LLM 直连 WASM 生命周期（项目管理文档）

审计轮次: 4

## 审计备注
- 主项目入口统一指向 `doc/readme/gap/readme-gap-distributed-prod-hardening-gap12345.prd.project.md`，本文仅维护增量任务。

## 任务拆解
- [x] T0：输出设计文档（`doc/readme/gap/readme-gap2-llm-wasm-lifecycle.prd.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：扩展 LLM 决策协议（schema/parser/prompt）支持 compile/deploy/install
- [x] T2：实现 simulator kernel 生命周期动作 + world model 持久化 + replay 闭环
- [x] T3：补齐 required-tier 测试、回归验证、文档/devlog 收口

## 依赖
- T2 依赖 T1 的动作协议冻结，确保 action 字段与解析一致。
- T3 依赖 T1/T2 完成后统一回归。

## 状态
- 当前阶段：已完成（T0~T3 全部收口）。
- 阻塞项：无。
- 下一步：无（等待下一轮需求）。

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.prd.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
