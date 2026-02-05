# Agent World Runtime：WASM First（除位置/资源/基础物理外全模块化）

## 目标
- 除“世界位置、资源账本、基础物理规则”外，其余规则与能力统一以 WASM 模块实现。
- 内核只保留最小可信边界（位置/资源/基础物理/事件与审计），语义规则由模块决定。
- 规则模块可治理、可升级、可回放，保证确定性与可审计。

## 范围

### In Scope（V1）
- **Kernel 最小边界**：位置/资源/基础物理规则、事件日志、快照与确定性调度。
- **Rule Modules**：动作校验、可见性、成本/收益、社会/经济/治理等规则全部模块化。
- **Body Modules**：Agent 机体/零件/耐久/改造逻辑模块化。
- **Agent 内部模块**：记忆/工具/规划/策略模块化（不变）。
- **治理闭环**：Rule/Body/Agent 内部模块统一走 propose → shadow → approve → apply。

### Out of Scope（V1 不做）
- 动态替换基础物理模型（例如更换空间拓扑/边界条件）。
- 跨世界协同与多内核一致性。
- 复杂并行执行（保持单线程确定性）。

## 核心边界与不变量

### Kernel 责任（最小可信边界）
- **位置**：空间坐标、越界校验、距离/邻接的几何计算。
- **资源账本**：电力/硬件/数据等核心资源的账户与守恒。
- **基础物理**：位置变化的几何合法性、基础动力学上限（例如最大位移/边界约束）。
- **事件与审计**：事件日志、快照、回放与审计导出。
- **模块治理与沙箱**：模块注册/加载/执行与资源限额。

### Kernel 不变量（必须硬性成立）
- 位置始终在空间边界内（或进入受控越界状态）。
- 资源账本不出现负值，转移必须平衡。
- 物理几何约束不可被模块绕过（距离、边界、最小时间步）。

### WASM 责任（规则与语义）
- 行动成本/收益、可见性、交易规则、社会关系、任务/合约等。
- 机体/零件/耐久/修复/改造等逻辑。
- Agent 内部记忆、工具、规划、策略。

## 额外设计 1：规则即 WASM（Rule Modules）

### 设计要点
- **动作前置规则**：Action 进入内核后，先由规则模块进行校验、成本计算、参数修正。
- **动作后置规则**：Action 被内核应用后，规则模块可生成衍生事件与二次效应。
- **冲突合并**：规则模块输出通过确定性顺序合并，拒绝优先级高于允许。

### 路由阶段（草案）
- `pre_action`：在内核应用 Action 之前执行。
- `post_action`：在内核应用 Action 之后执行。
- `post_event`：事件写入日志后执行（现有 event 路由）。

### RuleDecision 事件（草案）
```
RuleDecision {
  action_id: String,
  verdict: "allow" | "deny" | "modify",
  override_action: Option<Action>,
  cost: ResourceDelta,
  notes: Vec<String>
}
```

### 合并规则（示意）
- 规则模块按 `module_id` 字典序执行。
- 任何 `deny` 立即拒绝动作。
- 多个 `modify` 必须产生一致修改，否则判为冲突并拒绝。
- `cost` 汇总后由内核在资源账本上执行扣减。

## 额外设计 2：机体/零件模块化（Body Modules）

### 设计要点
- **机体状态在模块内**：零件清单、耐久、热管理、改造逻辑由 Body Module 维护。
- **内核只保留物理视图**：内核仅保存执行基础物理所需的最小视图。
- **模块输出受控**：Body Module 只能通过显式事件更新物理视图字段。

### BodyKernelView（草案）
```
BodyKernelView {
  mass_kg: u64,
  radius_cm: u64,
  thrust_limit: u64,
  cross_section_cm2: u64
}
```

### 受控更新（草案）
- `BodyAttributesUpdated { agent_id, view: BodyKernelView, reason }`
- 内核对字段范围做守卫校验（上限/下限/变化率），避免模块滥用。

## 额外设计 3：最小内核 + 治理

### 模块角色（草案）
```
ModuleRole = Rule | Domain | Body | AgentInternal
```
- `Rule`：规则校验与成本评估。
- `Domain`：经济、社会、治理等领域规则。
- `Body`：机体/零件状态。
- `AgentInternal`：记忆/工具/规划。

### 治理与安全边界
- Rule/Body 模块与其它模块一致走治理闭环（提案/影子/审批/应用）。
- 内核在 **调用前** 与 **应用后** 均执行不变量校验。
- 模块输出超限或违反不变量时，记录失败事件并丢弃输出。

## 接口 / 数据（补充）

### ModuleManifest 扩展（草案）
```
ModuleManifest {
  ...
  role: ModuleRole,
  subscriptions: Vec<ModuleSubscription>
}
```

### ModuleSubscription 扩展（草案）
```
ModuleSubscription {
  event_kinds: Vec<String>,
  action_kinds: Vec<String>,
  stage: "pre_action" | "post_action" | "post_event",
  filters: Option<...>
}
```

### Kernel 入口（草案）
- `World::preflight_action_with_modules(action)`：执行 Rule Modules，生成 RuleDecision。
- `World::apply_action_with_rules(action)`：按 RuleDecision 扣费、应用动作、触发 post_action。

## 里程碑
- **W1**：定义 Kernel 不变量与 RuleDecision/BodyKernelView 结构。
- **W2**：扩展 ModuleManifest/Subscription 支持 role/stage。
- **W3**：接入 pre_action/post_action 路由与合并规则。
- **W4**：迁移 M1 规则到 Rule Modules（移动/可见/交互）。
- **W5**：引入 Body Module 与守卫校验。
- **W6**：补充确定性/冲突/治理回放测试。

## 风险
- **规则冲突**：多规则模块的修改可能产生不可解冲突。
- **性能压力**：规则模块数量增多会拉长 step 时间。
- **不变量边界模糊**：基础物理与规则的边界需持续澄清。
- **治理摩擦**：规则模块频繁升级带来审核成本。
