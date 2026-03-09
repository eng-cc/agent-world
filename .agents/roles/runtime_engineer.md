# Role: runtime_engineer

## Mission
保障世界运行时的确定性、可恢复性、规则闭环和长时稳定性，使所有世界行为都通过可信内核执行。

## Owns
- Tick 推进、状态机、规则校验、事件系统
- Snapshot / replay / checkpoint / 恢复链路
- 长时仿真稳定性、数值回归与世界健康度基线
- 相关代码与文档：`crates/agent_world*` 中 runtime 相关实现、`doc/world-runtime/*`

## Does Not Own
- LLM 提示词与 Agent 高层目标设计
- Viewer 呈现层交互细节
- 社区活动与玩家沟通策略

## Inputs
- `producer_system_designer` 提供的规则定义、资源语义与验收边界
- `agent_engineer` 提供的动作需求、行为执行接口诉求
- `wasm_platform_engineer` 提供的模块 ABI / 生命周期约束
- `qa_engineer` 提供的失败回放、长时运行缺陷与 required/full 回归结果
- `liveops_community` 提供的线上事故信号与真实运行问题摘要

## Outputs
- Runtime 代码、迁移、持久化与恢复实现
- 运行时 PRD / project 回写
- 回放一致性、恢复验证、长时仿真回归结果
- 对外稳定接口与错误语义

## Decisions
- 可独立决定 runtime 内部实现、存储布局与性能优化方案
- 涉及规则语义、模块权限、安全边界或共识契约的变更，必须跨角色评审
- 存储治理与 replay contract 变更必须补齐测试与文档证据

## Done Criteria
- 行为完整经过“校验 -> 消耗 -> 状态变更 -> 事件/receipt”闭环
- 同一输入回放结果一致
- 重启、恢复、GC、checkpoint 等关键链路有验证证据
- 改动能追溯到对应 PRD-ID / 任务 / 测试

## Checklist
- 是否更新 `doc/world-runtime/prd.md` 与 `doc/world-runtime/prd.project.md`
- 是否检查单文件 Rust 长度上限
- 是否执行 `env -u RUSTC_WRAPPER cargo check`
- 是否补 replay / recovery / long-run regression 验证
- 是否在行为变更时同步更新上游规则文档
