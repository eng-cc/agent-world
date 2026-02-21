# Gameplay 发行差距收口（Prompt 触发 + Gate 覆盖 + 经济动作 + 规则深度）

## 目标
- 修复当前“游戏性可发行（非 viewer）”的 5 个关键差距：
  - 默认 LLM prompt 难以触发多类玩法事件。
  - 缺少基于优化 prompt 的闭环复验。
  - `release-gate` 只覆盖工业动作，未覆盖 gameplay 发行口径。
  - LLM 决策协议缺失经济治理动作解析路径。
  - m5 gameplay 模块规则仍偏 MVP，策略深度不足。
- 保持既有 runtime 协议与测试分层口径（`test_tier_required` / `test_tier_full`）兼容。

## 范围

### In Scope
- LLM 默认 prompt 优化：提高在 `llm_bootstrap` 下的动作覆盖与玩法事件触发概率。
- 增加“中途切换 prompt”能力：支持 long-run 场景下阶段性目标收敛。
- 扩展 `scripts/llm-longrun-stress.sh` gate 口径：新增 gameplay 覆盖 profile，并与 release gate 对齐。
- 补齐 LLM 经济治理动作 schema + parser + 测试：
  - `open_economic_contract`
  - `accept_economic_contract`
  - `settle_economic_contract`
- 优化 m5 gameplay 模块规则：在不破坏协议兼容前提下增强战争/危机/经济叠加规则表达。

### Out of Scope
- Viewer/UI 改造。
- 新增链上协议与网络协议。
- 大规模数值平衡重构（只做可发行必要增强，不做赛季级重平衡）。

## 接口/数据

### LLM Prompt 控制
- 新增运行期 prompt 切换输入（world_llm_agent_demo / stress 脚本透传）：
  - 切换触发 tick。
  - 切换后的 `system_prompt` / `short_term_goal` / `long_term_goal`。
- 保持已有环境变量兼容：
  - `AGENT_WORLD_LLM_SYSTEM_PROMPT`
  - `AGENT_WORLD_LLM_SHORT_TERM_GOAL`
  - `AGENT_WORLD_LLM_LONG_TERM_GOAL`

### LLM 决策协议扩展
- 在 `agent.submit_decision` schema 中新增：
  - `open_economic_contract`
  - `accept_economic_contract`
  - `settle_economic_contract`
- parser 映射到对应 `Action::*`，并补齐参数边界校验。

### 压测 Gate 扩展
- 在 `llm-longrun-stress.sh` 增加 release gate profile（工业 / gameplay / 混合）选择。
- gate 输出中明确 profile 与缺失动作统计。

### m5 模块规则增强
- `m5_gameplay_war_core`：增强结算输出（更丰富 participant outcomes 语义）。
- `m5_gameplay_crisis_cycle`：危机生成/超时规则加入动态性（非固定模板）。
- `m5_gameplay_economic_overlay`：引入经济合约事件联动奖励/惩罚。

## 里程碑
- M0：文档与任务拆解建档完成。
- M1：默认 prompt + 中途切换能力落地并通过定向测试。
- M2：release gate profile 与玩法覆盖口径落地并通过脚本回归。
- M3：LLM 经济动作 schema/parser/tests 收口。
- M4：m5 规则增强 + runtime/模块测试回归通过。

## 风险
- Prompt 调整可能引入动作漂移，导致既有工业闭环退化。
  - 缓解：保留工业目标约束，新增覆盖统计回归。
- Gate 口径增强会提高失败率。
  - 缓解：提供 profile 分层与清晰失败诊断信息。
- 规则增强可能触发历史测试断言漂移。
  - 缓解：先补测试再改规则，保持协议字段兼容。
