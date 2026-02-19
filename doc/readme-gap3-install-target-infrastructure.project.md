# README 缺口 3 收口：模块安装目标语义（自身 / 基础设施）（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-gap3-install-target-infrastructure.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：扩展数据结构（`ModuleInstallTarget` + action/event/state）并保持反序列化兼容
- [ ] T2：实现 simulator/runtime 安装目标处理与 LLM 动作入口
- [ ] T3：补齐测试并执行回归（`cargo check` + 定向 tests）后回写文档/devlog

## 依赖
- T2 依赖 T1 的类型冻结。
- T3 依赖 T1/T2 完成后统一执行。

## 状态
- 当前阶段：T0/T1 完成，T2/T3 待执行。
- 阻塞项：无。
- 下一步：执行 T2（安装目标语义与 LLM 入口实现）。
