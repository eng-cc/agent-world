# 分布式运行时设计

- 对应需求文档: `doc/p2p/distributed/distributed-runtime.prd.md`
- 对应项目管理文档: `doc/p2p/distributed/distributed-runtime.project.md`

## 1. 设计定位
定义分布式运行时整体结构：让运行时、网络、共识、存储和执行链路形成可组合、可追踪的统一底座。

## 2. 设计结构
- 运行时主线层：定义 distributed runtime 的核心生命周期和责任分层。
- 子系统协同层：网络、共识、DistFS、执行存储围绕统一 runtime 接口协同。
- 状态与恢复层：运行态、持久态和恢复路径形成稳定模型。
- 验收总览层：各分册专题都回挂到 distributed runtime 主设计。

## 3. 关键接口 / 入口
- distributed runtime 生命周期
- 网络/共识/DistFS/执行存储接口
- 状态恢复模型
- 模块级分册回挂

## 4. 约束与边界
- 主设计文档负责总览，不展开每个子专题所有细节。
- 重点是结构和协同，不替代具体实现文档。
- 运行时恢复和状态边界必须清晰。
- 各增量专题需要回挂到该主入口，避免设计碎片化。

## 5. 设计演进计划
- 先定义 distributed runtime 总体结构。
- 再由子专题承接网络/共识/DistFS 等细化。
- 最后通过主从互链维持整体一致性。
