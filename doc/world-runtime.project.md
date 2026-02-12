# Agent World Runtime：AOS 风格 world+agent 运行时（项目管理文档）

## 任务拆解
### 0. 对齐与准备
- [x] 输出设计文档（`doc/world-runtime.md`）
- [x] 输出项目管理文档（本文件）
- [x] 设计文档拆分为分册（`doc/world-runtime/`）

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
- [x] 实现 apply 阶段模块事件落盘与注册表更新
- [x] 实现 shadow 校验路径（hash/ABI/limits/caps）
- [x] 衔接 manifest/registry 的变更计划结构（ModuleChangeSet）
- [x] 补充 ModuleChangeSet 校验规则（冲突/重复/顺序约束）
- [x] 补充 ModuleChangeSet 的 manifest/patch 编码示例
- [x] 补充多补丁冲突处理规则（module_id 冲突）
- [x] 补充 ModuleChangeSet 生命周期（提案/Shadow/Apply/回放）
- [x] 补充 ShadowReport 结构与错误码（shadow 诊断）
- [x] 补充 ShadowReport 与 GovernanceEvent/Audit 导出关系
- [x] 补充模块失败事件的审计关联字段（trace_id/proposal_id）
- [x] 定义模块失败事件负载结构与错误码
- [x] 补充 GovernanceEvent 负载结构（含 ShadowReport/ModuleChangeSet）
- [x] 补充统一审计导出记录格式（AuditRecord）
- [x] 补充审计导出示例（模块注册/激活流程）
- [x] 补充审计导出示例（升级警告与失败记录）
- [x] 补充 module_registry.json / meta.json 结构示例
- [x] 补充模块注册 Happy Path（artifact → propose → shadow → apply）
- [x] 补充模块注册 Failure Path（shadow fail / apply fail）
- [x] 补充集成测试用例清单（治理闭环 + 模块生命周期事件）
- [x] 补充测试基架建议（文件组织/夹具/伪造工件/断言）
- [x] 补充 TestWorldBuilder API 草案
- [x] 补充 Dummy WASM 工件工具说明（bytes/hash）
- [x] 补充 Shadow 注入建议（测试专用策略）
- [x] 补充 ShadowPolicy 配置字段与 ShadowReport 关联
- [x] 补充模块加载与缓存设计草案
- [x] 补充沙箱执行器与资源限制设计草案
- [x] 补充 Capability/Policy 绑定设计草案
- [x] 补充事件订阅与路由设计草案
- [x] 补充模块输出校验设计草案
- [x] 补充 ModuleChangeSet 应用算法伪代码
- [x] 补充代码结构调整清单（Manifest/Applied/Audit）
- [x] 补充 Manifest 版本与迁移策略（向后兼容/向前拒绝/迁移函数）
- [x] 补充加载/恢复时迁移流程与 base_manifest_hash 行为
- [x] 补充迁移审计事件（ManifestMigrated）
- [x] 补充 ManifestMigrated 的 AuditRecord 集成示例
- [x] 补充 ManifestMigrated 的审计过滤说明
- [x] 补充 AuditFilter 示例（模块 + 迁移）
- [x] 补充审计导出分页/分片策略
- [x] 补充 cursor 格式与校验约定
- [x] 补充 cursor 与快照裁剪关系说明
- [x] 补充 cursor 失效恢复示例
- [x] 补充 cursor 编码安全建议
- [x] 补充 cursor 与 AuditFilter 的交互说明
- [x] 补充 cursor 编码示例
- [x] 补充并发导出一致性说明（start/end event_id）
- [x] 补充审计导出元信息字段建议
- [x] 补充审计导出包装格式示例（meta + records）
- [x] 实现集成测试：治理闭环 + 模块生命周期事件
- [x] 模块加载与缓存（按 wasm_hash）
- [x] 沙箱执行器（资源限制：内存/gas/调用频率）
- [x] Capability/Policy 与模块调用的绑定校验
- [x] 事件订阅与路由（事件 → 模块）
- [x] Action 订阅与路由（action → 模块）
- [x] 模块输出校验（effects/emits 限额与大小）
- [x] 单元测试/集成测试（确定性回放 + 权限拒绝）
- [x] 补充 LLM 驱动与 Agent 内部模块（memory module）设计说明

### 6. 维护
- [x] 拆分 runtime world 模块文件以满足单文件行数上限
- [x] 拆分 runtime builtin_modules（rule/body/default/power）以满足可维护性与文件行数约束

### 7. Agent 默认模块体系（ADM）
- [x] 输出默认模块设计分册（`doc/world-runtime/agent-default-modules.md`）
- [x] 输出默认模块项目管理文档（`doc/world-runtime/agent-default-modules.project.md`）
- [x] 冻结默认模块安装入口（`install_m1_agent_default_modules`）
- [x] 落地身体接口扩容动作与事件（消耗接口模块）
- [x] 落地默认 `sensor/mobility/memory/storage` 四模块最小实现
- [x] 完成默认模块回放一致性与降级策略测试

### 8. Builtin 模块独立 Crate 化（BMS）
- [x] 输出 BMS 设计文档（`doc/world-runtime/builtin-wasm-crate-split.md`）
- [x] 输出 BMS 项目管理文档（`doc/world-runtime/builtin-wasm-crate-split.project.md`）
- [x] BMS-1 新增独立 crate 并迁移首个 builtin wasm 模块（`m1.rule.move`）
- [x] BMS-2 接入构建脚本并补充验证
- [x] BMS-3 回归验证与文档收口
- [x] BMS-4 扩展设计与任务拆解（`m1.rule.visibility` / `m1.rule.transfer`）
- [x] BMS-5 迁移 `m1.rule.visibility` 到独立 wasm crate 并补充验证
- [x] BMS-6 迁移 `m1.rule.transfer` 到独立 wasm crate，扩展构建脚本并补充验证
- [x] BMS-7 回归验证与文档收口
- [x] BMS-8 扩展设计与任务拆解（`m1.body.core` 迁移阶段）
- [x] BMS-9 迁移 `m1.body.core` 到独立 wasm crate 并补充验证
- [x] BMS-10 扩展构建脚本支持 `m1.body.core` 并补充验证
- [x] BMS-11 回归验证与文档收口
- [x] BMS-12 扩展设计与任务拆解（`m1.sensor.basic` 迁移阶段）
- [x] BMS-13 迁移 `m1.sensor.basic` 到独立 wasm crate 并补充验证
- [x] BMS-14 扩展构建脚本支持 `m1.sensor.basic` 并补充验证
- [x] BMS-15 回归验证与文档收口
- [x] BMS-16 扩展设计与任务拆解（`m1.mobility.basic` 迁移阶段）
- [x] BMS-17 迁移 `m1.mobility.basic` 到独立 wasm crate 并补充验证
- [x] BMS-18 扩展构建脚本支持 `m1.mobility.basic` 并补充验证
- [x] BMS-19 回归验证与文档收口
- [x] BMS-20 扩展设计与任务拆解（`m1.memory.core` 迁移阶段）
- [x] BMS-21 迁移 `m1.memory.core` 到独立 wasm crate 并补充验证
- [x] BMS-22 扩展构建脚本支持 `m1.memory.core` 并补充验证
- [x] BMS-23 回归验证与文档收口
- [x] BMS-24 扩展设计与任务拆解（`m1.storage.cargo` 迁移阶段）
- [x] BMS-25 迁移 `m1.storage.cargo` 到独立 wasm crate 并补充验证
- [x] BMS-26 扩展构建脚本支持 `m1.storage.cargo` 并补充验证
- [x] BMS-27 回归验证与文档收口
- [x] BMS-28 扩展设计与任务拆解（`m1.power.radiation_harvest` / `m1.power.storage` 迁移阶段）
- [x] BMS-29 迁移 `m1.power.radiation_harvest` / `m1.power.storage` 到独立 wasm crate 并补充验证
- [x] BMS-30 扩展构建脚本支持 `m1.power.radiation_harvest` / `m1.power.storage` 并补充验证
- [x] BMS-31 回归验证与文档收口
- [x] BMS-32 扩展设计与任务拆解（runtime cutover：WASM 优先 + builtin fallback + 渐进下线 builtin 注册）
- [ ] BMS-33 实现 runtime 执行路径切换（WASM 优先 + builtin fallback）并补充验证
- [ ] BMS-34 逐步下线一批 builtin 注册点（先 tests/demo）并补充验证
- [ ] BMS-35 回归验证与文档收口（cutover 阶段一期）

## 依赖
- Rust workspace（`crates/agent_world`）
- 事件日志/快照的本地存储方案（文件或 KV）
- （可选）测试基架与 replay harness

## 状态
- 当前阶段：M5 + ADM-S5（默认模块体系 V1 收口完成）
- 下一步：执行 BMS-33，先在 runtime 接入 WASM 优先 + builtin fallback
- 最近更新：完成 BMS-32（runtime cutover 设计与任务拆解扩展）（2026-02-12）
