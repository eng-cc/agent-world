# 间接控制链路 + WASM Tick 生命周期 + 长期记忆持久化（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/indirect-control-tick-lifecycle-long-term-memory.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：间接控制链路（submitter 授权 + viewer player 绑定）与测试
- [x] T2：WASM tick 生命周期（按需唤醒/挂起）与测试
- [ ] T3：长期记忆持久化（导出/恢复 + live 同步 + persist）与测试
- [ ] T4：回归验证（`cargo check` + 定向 tests）并回写文档/devlog

## 依赖
- T2 依赖 ABI/router/runtime 多模块接口同步。
- T3 依赖 `viewer/live` 与 `simulator/world_model` 数据结构联动。
- T4 依赖 T1/T2/T3 全部完成后统一回归。

## 状态
- 当前阶段：进行中（T0/T1/T2 完成，T3~T4 待完成）
- 阻塞项：无
- 下一步：执行 T3（长期记忆持久化）
