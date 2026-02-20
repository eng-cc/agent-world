# README 口径对齐：LLM P1/P2 生产级收口（设计文档）

## 目标
- 收口 P1：修复 LLM 工具协议与实现不一致问题，确保提示词、OpenAI tool 注册、解析与执行路径一致。
- 收口 P1：将 `module.lifecycle.status` 从“行为本地缓存视角”升级为“世界模型快照视角”，避免多主体下状态漂移。
- 收口 P1：补齐共识执行桥接对 `SimulatorAction` 的执行语义，避免提交后在 execution bridge 被静默忽略。
- 收口 P2：扩展 LLM 查询面，覆盖电力市场、模块市场、社会状态等关键制度信息，支撑完整策略决策。

## 范围
### In scope
- `crates/agent_world/src/simulator/llm_agent`
  - 补全查询 tool 注册（含 `module.lifecycle.status`）与函数名映射。
  - 扩展查询模块：电力订单簿、模块市场、社会状态。
  - 调整 `run_prompt_module`，将生命周期/市场查询改为读取 observation 快照而非本地缓存。
  - 更新 prompt 协议文案与 schema 约束描述，保持“可调用工具”与“提示词声明”一致。
- `crates/agent_world/src/simulator/kernel`
  - 扩展 `Observation` 数据结构，增加模块生命周期、模块市场、电力市场、社会状态快照。
  - 在 `observe` 路径中从 `WorldModel` 构建并返回上述快照。
- `crates/agent_world/src/bin/world_viewer_live/execution_bridge.rs`
  - 为共识执行桥接补齐 `SimulatorAction` 执行路径（新增 simulator execution mirror）。
  - 保留 RuntimeWorld 路径兼容；在有 simulator payload 时执行并落审计记录。
- 测试与回归
  - 新增/更新 `test_tier_required` 侧单测：tool 注册、query 结果、observation 快照构造。
  - 新增/更新 execution bridge 测试：`SimulatorAction` 不再静默忽略。

### Out of scope
- 不重构 runtime 与 simulator Action 体系为统一 Action。
- 不在本次改动中引入新的外部服务依赖。
- 不扩展到完整跨主机自动发现/自动组网策略。

## 接口 / 数据
### 1) LLM 查询 tool 接口
- 查询 tool（新增/补齐）：
  - `module_lifecycle_status` -> `module.lifecycle.status`
  - `power_order_book_status` -> `power.order_book.status`
  - `module_market_status` -> `module.market.status`
  - `social_state_status` -> `social.state.status`
- 保持既有：
  - `agent_modules_list`
  - `environment_current_observation`
  - `memory_short_term_recent`
  - `memory_long_term_search`
  - `agent_submit_decision`

### 2) Observation 快照扩展
- `Observation` 新增快照字段（只读视角）：
  - `module_lifecycle`
  - `module_market`
  - `power_market`
  - `social_state`
- 快照来自 `WorldKernel::observe` 当下 `WorldModel`，不依赖单个 agent 行为内缓存。

### 3) 共识执行桥接
- `NodeRuntimeExecutionDriver` 在 `on_commit` 中区分 payload：
  - `RuntimeAction`：沿用 RuntimeWorld 执行路径。
  - `SimulatorAction`：执行 simulator mirror 路径并落审计记录。
- 状态落盘：保留现有 execution state 文件；新增 simulator mirror 持久化目录和记录字段。

## 里程碑
- M1（T0）：设计文档 + 项目管理文档冻结。
- M2（T1）：LLM tool 协议收口 + `module.lifecycle.status` 可调用。
- M3（T2）：Observation 快照扩展 + LLM 查询面（module/power/social）接入。
- M4（T3）：execution bridge 支持 `SimulatorAction` 执行与审计落盘。
- M5（T4）：required-tier 回归 + 文档/devlog 收口。

## 风险
- 兼容风险：`Observation` 结构扩展可能影响测试构造体初始化；需统一 helper 默认值。
- 性能风险：快照字段增加会提升序列化成本；通过“prompt 中仍做裁剪、查询按需使用”控制。
- 执行语义风险：execution bridge 同时维护 runtime/simulator 两条路径，需防止状态来源混淆。
- 回归风险：tool 列表扩展后若函数名映射遗漏，会导致 LLM 调用失败；需单测覆盖函数名集合。
