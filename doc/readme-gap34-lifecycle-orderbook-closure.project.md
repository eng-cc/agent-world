# README 高优先级差距收口（二期）：模块生命周期 + 单一订单簿撮合（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-gap34-lifecycle-orderbook-closure.md`）
- [x] T0：输出项目管理文档（本文件）
- [ ] T1：Runtime 生命周期闭环 + 成本模型（动作/事件/状态/测试）
- [ ] T2：单一订单簿撮合（模块 + 电力）与一致性测试
- [ ] T3：回归验证（`cargo check` + 定向 required tests）并回写文档/devlog

## 依赖
- T2 依赖 T1 的 Runtime 事件/状态扩展。
- T3 依赖 T1/T2 完成后执行统一回归。

## 状态
- 当前阶段：进行中（T0 已完成，进入 T1）
- 阻塞项：无
- 下一步：执行 T1
