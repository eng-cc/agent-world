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

### 5. 自由沙盒与 WASM 模块接入（M5）
- [x] 定义 ModuleManifest/ModuleKind/ModuleSubscription/ModuleLimits 数据结构
- [x] 定义 reducer/pure module 的 ABI 签名与序列化约定
- [x] 定义模块事件 schema 与校验规则（Register/Activate/Upgrade）
- [x] 模块注册表/存储设计（哈希寻址、缓存、审计元数据）
- [x] 模块注册/激活/升级事件与治理流程接入（设计草案）
- [ ] 实现 apply 阶段模块事件落盘与注册表更新
- [ ] 实现 shadow 校验路径（hash/ABI/limits/caps）
- [x] 衔接 manifest/registry 的变更计划结构（ModuleChangeSet）
- [x] 补充 ModuleChangeSet 校验规则（冲突/重复/顺序约束）
- [ ] 集成测试：治理闭环 + 模块生命周期事件
- [ ] 模块加载与缓存（按 wasm_hash）
- [ ] 沙箱执行器（资源限制：内存/gas/调用频率）
- [ ] Capability/Policy 与模块调用的绑定校验
- [ ] 事件订阅与路由（事件 → 模块）
- [ ] 模块输出校验（effects/emits 限额与大小）
- [ ] 单元测试/集成测试（确定性回放 + 权限拒绝）

## 依赖
- Rust workspace（`crates/agent_world`）
- 事件日志/快照的本地存储方案（文件或 KV）
- （可选）测试基架与 replay harness

## 状态
- 当前阶段：M4（治理闭环 + patch + 回滚 + 审计/保留已具备最小闭环）
- 下一步：定义 WASM 模块接口与沙箱治理边界（M5 起步）
