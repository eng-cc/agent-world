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
- 扩展 `llm_bootstrap` 为多 Agent（5 Agent）场景，提升提案/投票/合约交互触发面。
- 在 `world_llm_agent_demo` 引入 runtime gameplay bridge：对 simulator 中 runtime-only 的 gameplay/economic 动作，接入 runtime `World` 执行路径，减少“非预期拒绝”并形成更闭环的压测链路。
- 增加“阶段基线世界”状态落盘/加载能力：支持先跑工业建基线，再从同一状态继续治理/危机测试，降低随机起步噪声。
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
- 新增多阶段切换输入（用于千 tick 级长周期）：
  - `--prompt-switches-json`：按 tick 序列声明多次切换（每项至少包含一个 `llm_*` 覆盖字段）。
  - 与单次切换参数互斥，避免配置歧义。
- 新增游戏发展测试 prompt 套件（stress 脚本内置 `--prompt-pack`）：
  - `story_balanced`（默认推荐）
  - `frontier_builder`
  - `industrial_baseline`（工业建基线专用）
  - `civic_operator`
  - `resilience_drill`
- 保持已有环境变量兼容：
  - `AGENT_WORLD_LLM_SYSTEM_PROMPT`
  - `AGENT_WORLD_LLM_SHORT_TERM_GOAL`
  - `AGENT_WORLD_LLM_LONG_TERM_GOAL`
- 默认起步 prompt 增强：
  - 明确“规则/观察 -> 资源稳态 -> 工业闭环 -> 治理协作”阶段推进。
  - 若前置条件不明确，优先调用规则查询工具再决策。

### 世界规则查询工具（新增）
- 新增查询工具：`world.rules.guide`
  - OpenAI tool 名称：`world_rules_guide`
  - prompt module 名称：`world.rules.guide`
  - 支持 `topic`：
    - `quickstart`
    - `resources`
    - `industry`
    - `governance`
    - `economic`
    - `social`
    - `recovery`
    - `all`
- 目标：让 Agent 在开局/切阶段时可主动读取“玩法规则 + 前置动作链 + 拒绝恢复策略”，减少盲目重试与动作循环。

### LLM 决策协议扩展
- 在 `agent.submit_decision` schema 中新增：
  - `open_economic_contract`
  - `accept_economic_contract`
  - `settle_economic_contract`
- parser 映射到对应 `Action::*`，并补齐参数边界校验。

### 压测 Gate 扩展
- 在 `llm-longrun-stress.sh` 增加 release gate profile（工业 / gameplay / 混合）选择。
- gate 输出中明确 profile 与缺失动作统计。

### Runtime Gameplay Bridge（demo/stress）
- `world_llm_agent_demo` 支持启用 runtime gameplay bridge：
  - 当动作属于 gameplay/economic 且 simulator 内核为 runtime-only 拒绝域时，通过 runtime `World` 执行并回传结果到 LLM 行为循环。
  - 保持 simulator 观察链路不变，避免对既有工业动作闭环造成破坏。
- `scripts/llm-longrun-stress.sh` 增加对应透传参数与默认策略，确保长稳测试可复现实战化交互。

### Baseline State IO（demo/stress）
- `world_llm_agent_demo` 支持：
  - `--save-state-dir <path>`：将当前 simulator kernel 状态落盘（`snapshot.json` + `journal.json`）。
  - `--load-state-dir <path>`：从已落盘状态加载并继续运行 LLM 闭环。
- `scripts/llm-longrun-stress.sh` 支持 state dir 透传，形成两阶段脚本链路：
  - 阶段 A：工业建基线并保存。
  - 阶段 B：从基线加载，重点验证治理/危机/元进度动作覆盖。
- 新增 `--llm-execute-until-auto-reenter-ticks <n>` 参数透传：
  - 映射环境变量 `AGENT_WORLD_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS`，用于长周期中减少重复动作的 LLM 往返。
  - `industrial_baseline` 默认设置 `24`，可显式覆盖。

### Tracked Baseline Fixture Smoke（full tier）
- 基线状态从临时目录转存到 git 跟踪路径：
  - `fixtures/llm_baseline/state_01/snapshot.json`
  - `fixtures/llm_baseline/state_01/journal.json`
- 新增 smoke 脚本：`scripts/llm-baseline-fixture-smoke.sh`
  - 先校验 fixture 文件存在，再执行 `test_tier_full` 定向回归：
    - 验证基线可加载且结构完整；
    - 验证从基线加载后，通过 runtime gameplay bridge 可离线继续推进治理/经济动作链（提案/投票/元进度/合约）。
- `scripts/ci-tests.sh full` 接入该脚本，形成“可追踪基线 + 可重复加载校验”的发布前基础保障。

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
- M5：长周期（>=1000 ticks）多阶段 prompt 切换能力落地并通过脚本/解析回归。
- M6：`llm_bootstrap` 5 Agent + runtime gameplay bridge 落地，完成非 viewer 的 gameplay 交互闭环回归。
- M7：阶段基线世界（落盘/加载）闭环落地，支持“同一起点多玩法口径”对比测试。
- M8：基线 fixture 入库并接入 full-tier smoke，确保状态工件可持续复用。
- M9：补齐基线加载后的离线治理续跑 smoke，降低在线 LLM 依赖对门禁稳定性的影响。

## 当前回归结论（T7）
- bridge 生效性已验证：在 `llm_bootstrap` 下，`open_governance_proposal` 等 runtime-only gameplay 动作可通过 runtime bridge 成功执行，不再被 simulator 内核直接拒绝。
- 场景规模已验证：`llm_bootstrap` 已从 1 Agent 提升至 5 Agent，可形成提案/协作类交互基础。
- 覆盖稳定性结论：短中程 run 仍可能出现“动作集中于采集/工业尝试”的分布，玩法链路覆盖（vote/crisis/meta）存在波动；后续需通过预设世界事件或阶段目标进一步稳定。

## 风险
- Prompt 调整可能引入动作漂移，导致既有工业闭环退化。
  - 缓解：保留工业目标约束，新增覆盖统计回归。
- Gate 口径增强会提高失败率。
  - 缓解：提供 profile 分层与清晰失败诊断信息。
- 规则增强可能触发历史测试断言漂移。
  - 缓解：先补测试再改规则，保持协议字段兼容。
- 长周期切换配置可能出现参数冲突（单次切换与多次切换混用）。
  - 缓解：CLI 显式互斥校验，脚本默认仅选择一种切换路径。
- runtime bridge 与 simulator 状态可能出现观测偏差（双状态源）。
  - 缓解：bridge 仅接管 runtime-only gameplay/economic 动作，工业动作保持 simulator 执行；同时输出 bridge 指标用于诊断。
- 1000+ tick 长程工业基线对 LLM 往返延迟敏感，wall-clock 成本高，影响当次任务内产出速度。
  - 缓解：提供 `industrial_baseline` 专用 prompt pack 与 `--llm-execute-until-auto-reenter-ticks` 透传；推荐后台长跑并沉淀可复用 state dir。
