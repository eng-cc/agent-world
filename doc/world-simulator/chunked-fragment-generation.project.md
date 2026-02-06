# Agent World Simulator：分块世界生成与碎片元素/化合物池（项目管理文档）

## 任务拆解
- [x] CG1 输出分块生成与元素/化合物池设计文档、项目管理文档
- [x] CG2.1 实现 chunk 基础能力（坐标映射/边界/seed）
- [x] CG2.2 接入 chunk 索引与按探索触发生成（未探索不生成）
- [x] CG2.3 补充运行时触发契约（observe/move/action 前置 ensure_chunk_generated）
- [x] CG3.1 建立碎片块状物理模型骨架（长方体 block、体积/密度/质量）
- [x] CG3.2 建立化合物组成模型与元素映射骨架
- [x] CG3.3 将块状物理与化合物模型接入实际碎片生成流程
- [x] CG4 实现资源预算一次性生成（total/remaining）与开采扣减守恒
- [x] CG5 场景接入：起始 chunk 预生成 + 20km×20km×10km 分块配置
- [x] CG6.1 定义 ChunkGenerated 事件结构与触发点（init/observe/action）
- [x] CG6.2 扩展快照字段 chunk_generation_schema_version 并提供默认迁移
- [x] CG6.3 回放接入 ChunkGenerated 事件并执行摘要一致性校验
- [x] CG6.4 补充持久化/回放/迁移单元测试
- [x] CG6.5 回顾并更新设计文档/项目文档/任务日志，运行测试并提交 git
- [ ] CG7 实现跨 chunk 边界一致性（邻块校验/边界保留）
- [ ] CG8 实现经济资源映射最小闭环（RefineCompound -> hardware/electricity 约束）
- [ ] CG9 增加性能预算与降级策略（fragments/blocks 上限）
- [ ] CG10 补充回放一致性与性能回归测试

## 依赖
- `crates/agent_world/src/simulator/chunking.rs`
- `crates/agent_world/src/simulator/asteroid_fragment.rs`
- `crates/agent_world/src/simulator/init.rs`
- `crates/agent_world/src/simulator/kernel/*`
- `crates/agent_world/src/simulator/persist.rs`
- `crates/agent_world/src/simulator/world_model.rs`

## 状态
- 当前阶段：CG6（实现完成）
- 下一阶段：CG7（跨 chunk 边界一致性）
