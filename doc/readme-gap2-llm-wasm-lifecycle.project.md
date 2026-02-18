# README 缺口 2 收口：LLM 直连 WASM 生命周期（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-gap2-llm-wasm-lifecycle.md`）
- [x] T0：输出项目管理文档（本文件）
- [ ] T1：扩展 LLM 决策协议（schema/parser/prompt）支持 compile/deploy/install
- [ ] T2：实现 simulator kernel 生命周期动作 + world model 持久化 + replay 闭环
- [ ] T3：补齐 required-tier 测试、回归验证、文档/devlog 收口

## 依赖
- T2 依赖 T1 的动作协议冻结，确保 action 字段与解析一致。
- T3 依赖 T1/T2 完成后统一回归。

## 状态
- 当前阶段：T0 已完成，进入 T1。
- 阻塞项：无。
- 下一步：实现 LLM 协议扩展并补 parser 校验路径。
