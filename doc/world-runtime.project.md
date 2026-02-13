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
- [x] BMS-33 实现 runtime 执行路径切换（WASM 优先 + builtin fallback）并补充验证
- [x] BMS-34 逐步下线一批 builtin 注册点（先 tests/demo）并补充验证
- [x] BMS-35 回归验证与文档收口（cutover 阶段一期）
- [x] BMS-36 扩展设计与任务拆解（cutover 阶段二：逐域删除 runtime builtin fallback/实现）
- [x] BMS-37 下线 `rule/body` 相关 builtin 测试注册与默认执行路径（wasm 工件优先）
- [x] BMS-38 下线 `sensor/mobility/memory/storage/power` 相关 builtin 测试注册与默认执行路径（wasm 工件优先）
- [x] BMS-39 清理 runtime 中不再使用的 builtin 模块实现导出与冗余回退路径，并完成回归收口
- [x] BMS-40 扩展设计与任务拆解（阶段三启动：逐步物理删除 native builtin 老代码）
- [x] BMS-41 删除 `runtime/builtin_modules/*` native 实现文件，保留运行时常量与最小 sandbox 兼容层
- [x] BMS-42 下线 `BuiltinModuleSandbox` 的 builtin 注册兜底能力并清理残余引用
- [x] BMS-43 回归验证、文档与 devlog 收口（阶段三首轮）
- [x] BMS-44 扩展设计与任务拆解（阶段三第二轮：删除 `BuiltinModuleSandbox` 兼容层及导出）
- [x] BMS-45 删除 `BuiltinModuleSandbox` 类型与 `runtime` 对外导出，保留模块常量导出
- [x] BMS-46 回归验证、文档与 devlog 收口（阶段三第二轮）
- [x] BMS-47 扩展设计与任务拆解（阶段三第三轮：删除 runtime builtin 常量兼容层）
- [x] BMS-48 删除 `runtime/builtin_modules.rs` 常量层，统一引用 `agent_world_builtin_wasm` 常量导出
- [x] BMS-49 回归验证、文档与 devlog 收口（阶段三第三轮）
- [x] BMS-50 扩展设计与任务拆解（阶段四启动：产物接入收敛 + 文档去陈旧 + 工件策略决策）
- [x] BMS-51 清理过时文档描述（`BuiltinModuleSandbox`/`runtime/builtin_modules.rs` 等），统一到 wasm-only 现状
- [x] BMS-52 补齐 runtime 内嵌 wasm 工件同步机制（构建 -> 回填 -> 哈希校验）
- [x] BMS-53 收敛 bootstrap/tests 的工件引用入口，减少 `include_bytes!(m1_builtin_modules.wasm)` 分散硬编码
- [x] BMS-54 评估并决策“单聚合 wasm 工件 vs 多模块独立 wasm 工件”，输出迁移方案与分批顺序
- [x] BMS-55 回归验证、文档与 devlog 收口（阶段四）
- [x] BMS-56 扩展设计与任务拆解（阶段五启动：多模块独立 wasm 工件实施）
- [x] BMS-57 收敛内置模块清单来源（脚本/runtime 共用）并补充一致性校验
- [x] BMS-58 新增独立工件同步脚本与 hash 清单（保留单聚合兼容入口）
- [x] BMS-59 `bootstrap/runtime` 切换到“按 module_id 选择独立工件”（先规则域）
- [x] BMS-60 回归验证、文档与 devlog 收口（阶段五首轮）
- [x] BMS-61 扩展设计与任务拆解（阶段五第二轮：`body/sensor/mobility` 按 `module_id` 独立工件装载）
- [x] BMS-62 `bootstrap/runtime` 切换到“按 module_id 选择独立工件”（`body/sensor/mobility`）
- [x] BMS-63 回归验证、文档与 devlog 收口（阶段五第二轮）
- [x] BMS-64 扩展设计与任务拆解（阶段五第三轮：`memory/storage_cargo + power` 按 `module_id` 独立工件装载）
- [x] BMS-65 `bootstrap/runtime` 切换到“按 module_id 选择独立工件”（`memory/storage_cargo + power`）
- [x] BMS-66 回归验证、文档与 devlog 收口（阶段五第三轮）
- [x] BMS-67 扩展设计与任务拆解（阶段五第四轮：下线单聚合工件兼容入口）
- [x] BMS-68 删除单聚合工件兼容入口（runtime/脚本/校验路径切换到 per-module-only）
- [x] BMS-69 回归验证、文档与 devlog 收口（阶段五第四轮）
- [x] BMS-70 扩展设计与任务拆解（阶段六：`agent_world_builtin_wasm` 闭环场景联测）
- [x] BMS-71 新增 `agent_world_builtin_wasm` 单场景闭环测试，覆盖规则/身体/默认模块/状态模块协作
- [x] BMS-72 回归验证、文档与 devlog 收口（阶段六闭环联测）
- [x] BMS-73 扩展设计与任务拆解（阶段七：`agent_world` 运行时 wasmtime 闭环联测）
- [x] BMS-74 新增 `runtime::tests` 闭环场景测试，验证 `World + WasmExecutor + builtin wasm` 端到端链路
- [x] BMS-75 回归验证、文档与 devlog 收口（阶段七运行时闭环联测）

### 9. WASM 运行时激进拆分（WRS）
- [x] 输出 WRS 设计文档（`doc/world-runtime/wasm-runtime-crate-split.md`）
- [x] 输出 WRS 项目管理文档（`doc/world-runtime/wasm-runtime-crate-split.project.md`）
- [x] WRS-2 新建 ABI/Executor/Router 三 crate 并接入 `agent_world`
- [ ] WRS-3 拆分 `agent_world_builtin_wasm/src/lib.rs`（目录化）
- [ ] WRS-4 回归验证、文档与 devlog 收口

## 依赖
- Rust workspace（`crates/agent_world`）
- 事件日志/快照的本地存储方案（文件或 KV）
- （可选）测试基架与 replay harness

## 状态
- 当前阶段：M5 + ADM-S5（默认模块体系 V1 收口完成，BMS 阶段七已完成，WRS-2 已完成）
- 下一步：推进 WRS-3（`agent_world_builtin_wasm/src/lib.rs` 目录化拆分）
- 最近更新：完成 WRS-2（ABI/Executor/Router 三 crate 落地与接入）（2026-02-13）
