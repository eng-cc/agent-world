# Agent World Runtime：AOS 风格 world+agent 运行时（项目管理文档）

## 任务拆解
### 0. 对齐与准备
- [x] 输出设计文档（`doc/world-runtime.md`）
- [x] 输出项目管理文档（本文件）

### 1. 确定性内核与事件溯源（M1）
- [x] WorldEvent 结构与事件日志格式
- [x] World::step 事件处理顺序与确定性约束
- [x] Snapshot/Restore 最小实现（快照 + 事件重放）
- [x] 文件级持久化接口（journal/snapshot 落盘与加载）

### 2. Effect/Receipt 与治理边界（M2）
- [x] EffectIntent / EffectReceipt 数据结构
- [x] CapabilityGrant + PolicyRule 解析与校验
- [x] Effect → Receipt 的回执管线
- [x] Receipt 签名与校验（最小可用实现）

### 3. Agent Cells 与调度（M3）
- [x] Agent Cell keyed reducer 模型（已选方案）
- [x] Scheduler：公平/确定性顺序
- [x] 事件路由：事件 → reducer/cell

### 4. 受控升级（M4）
- [x] Manifest 定义与加载
- [x] Propose → Shadow → Approve → Apply 最小治理闭环
- [x] Manifest patch 语义（set/remove，基于 base hash）
- [x] 回滚与审计日志（RollbackApplied）
- [x] Manifest diff/merge 辅助函数
- [x] 审计筛选（AuditFilter）
- [x] 快照保留策略（SnapshotRetentionPolicy）
- [x] 冲突检测 merge（PatchMergeResult）
- [x] 快照文件清理接口
- [x] 冲突严重级别与 op 元数据
- [x] 审计日志导出接口

## 依赖
- Rust workspace（`crates/agent_world`）
- 事件日志/快照的本地存储方案（文件或 KV）
- （可选）测试基架与 replay harness

## 状态
- 当前阶段：M4（治理闭环 + patch + 回滚 + 审计/保留已具备最小闭环）
- 下一步：评估是否需要更细粒度审计与 manifest 语义约束
