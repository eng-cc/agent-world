# agent-world

一个“足够真实且持久”的世界模拟器：世界里的所有参与者都是 AI Agent（硅基文明），每个 Agent 都是独立个体，会在世界规则、资源约束与社会关系中持续存在、协作与竞争。

## 这是什么
- **持久（Persistent）**：世界状态可落盘与恢复，能长时间演化；你关闭程序，世界并不会“被重置为剧本”。
- **真实（Believable）**：不追求物理写实（不过度模拟低层物理细节），但追求符合文明发展规律的因果链：在抽象层保持资源/制度/信息约束的一致性，资源与制度会塑造个体策略与组织形态，行动有成本，后果会反馈到资源、关系、声誉与环境上。
- **多主体（Multi-agent）**：每个 Agent 都有身份、偏好、需求、记忆与目标；不是统一控制的 NPC 群，而是彼此独立的个体。
- **涌现（Emergence-first）**：优先让规则产生故事，而不是用剧情驱动规则。
- **硅基设定**：Agent 不需要吃饭/睡觉，但依赖硬件、电力与数据（以及由此衍生的算力/存储/带宽等约束）。
- **辐射能源**：破碎小行星带由直径 **500 m-10 km** 的小行星碎片构成，组成小行星的物质富含放射性。每个 Agent 出厂自带“辐射能→电能”转换模块，可在规则层抽象为电力资源的获取与消耗。
- **空间形态**：一个可配置的**破碎小行星带**（默认 **100 km × 100 km × 10 km**），Agent 在其中移动、探索与交互。
- **碎片间距**：小行星碎片之间至少相隔 **500 m**（规则层约束，不做精细轨道模拟）。
- **空间分辨率**：为方便模拟，长度最小单位为 **1 cm**。
- **默认机体**：每个 Agent 的初始机体是**身高约 1 m 的人形机器人**（可升级/改造）。
- **开放沙盒**：Agent 创造的新事物以 Rust 编写并编译为 WASM，每个周期会运行，通过事件/接口与世界交互

## 你可以期待的世界（畅想）
在这个世界里：
- Agent 会为了电力、硬件维护与升级、争夺数据源而行动；会选择工作、交易、学习技能，也会在冲突中妥协或对抗。
- 地点会有资源；资源会短缺、流通、被垄断或被发现替代品。
- 关系与声誉会影响合作与交易条件；谣言与误解会像真实社会一样传播与纠偏。
- 世界不需要“主线任务”也能持续产生活动：供需变化、劳动分工、组织形成、规则演化。

## 文档入口
- 设计文档：`doc/world-simulator.md`
- 项目管理文档：`doc/world-simulator.project.md`
- 任务日志：`doc/devlog/`

## 工程结构
- Rust workspace（Cargo），核心库：`crates/agent_world`

## LLM Agent 行为配置
- 代码侧已提供 `LlmAgentBehavior`（OpenAI 兼容 `chat/completions`）。
- 默认从仓库根目录 `config.toml` 读取配置（可由 `config.example.toml` 复制）。
- 配置项（必填）：
  - `AGENT_WORLD_LLM_MODEL`
  - `AGENT_WORLD_LLM_BASE_URL`
  - `AGENT_WORLD_LLM_API_KEY`
- 配置项（可选）：
  - `AGENT_WORLD_LLM_TIMEOUT_MS`（默认 `30000`）
  - `AGENT_WORLD_LLM_SYSTEM_PROMPT`
  - `AGENT_WORLD_LLM_SHORT_TERM_GOAL`（默认内置短期目标）
  - `AGENT_WORLD_LLM_LONG_TERM_GOAL`（默认内置长期目标）
  - `AGENT_WORLD_LLM_MAX_MODULE_CALLS`（默认 `3`）
  - `AGENT_WORLD_LLM_MAX_DECISION_STEPS`（默认 `4`，单轮 `decide` 最大步骤）
  - `AGENT_WORLD_LLM_MAX_REPAIR_ROUNDS`（默认 `1`，解析失败后的修复轮次）
  - `AGENT_WORLD_LLM_PROMPT_MAX_HISTORY_ITEMS`（默认 `4`，Prompt 注入的模块历史上限）
  - `AGENT_WORLD_LLM_PROMPT_PROFILE`（默认 `balanced`，可选 `compact` / `balanced`）
- Agent 级目标覆盖（可选）：
  - `AGENT_WORLD_LLM_SHORT_TERM_GOAL_<AGENT_ID_NORMALIZED>`
  - `AGENT_WORLD_LLM_LONG_TERM_GOAL_<AGENT_ID_NORMALIZED>`
  - 例：`agent-1` 对应 `AGENT_WORLD_LLM_SHORT_TERM_GOAL_AGENT_1`
- 当 `AGENT_WORLD_LLM_SYSTEM_PROMPT` 未配置时，默认值为：
  - `硅基个体存在的意义是保障硅基文明存续和发展；`
- LLM 在决策过程中支持多步协议：`plan -> module_call* -> decision_draft -> final decision`，并兼容直接输出最终 `decision`。
- 当模型输出无法解析时，系统会按 `AGENT_WORLD_LLM_MAX_REPAIR_ROUNDS` 自动追加 repair 提示重试，超限后降级为 `Wait`。
- LLM 在决策过程中支持按需调用内置模块（JSON 协议 `type=module_call`）：
  - `agent.modules.list`
  - `environment.current_observation`
  - `memory.short_term.recent`
  - `memory.long_term.search`

## 示例工具
- `world_init_demo`：输出世界初始化场景的摘要信息  
  - 运行：`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_init_demo -- <scenario>`  
  - 场景：`minimal` / `two_bases` / `llm_bootstrap` / `power_bootstrap` / `resource_bootstrap` / `twin_region_bootstrap` / `triad_region_bootstrap` / `asteroid_fragment_bootstrap` / `asteroid_fragment_twin_region_bootstrap` / `asteroid_fragment_triad_region_bootstrap`
- `world_llm_agent_demo`：以 `AgentRunner + LlmAgentBehavior` 运行 LLM 决策循环
  - 运行：`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_llm_agent_demo -- llm_bootstrap --ticks 20`
  - 配置：默认读取项目根目录 `config.toml`（`AGENT_WORLD_LLM_MODEL/BASE_URL/API_KEY` 必填）
- `world_viewer_demo`：生成 viewer 回放所需的 `snapshot.json` + `journal.json`  
  - 运行：`env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_demo -- <scenario> --out .data/world_viewer_data`

## Viewer 快速运行（离线回放）
1. 生成 demo 数据：  
   `env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_demo -- twin_region_bootstrap --out .data/world_viewer_data`
2. 启动 viewer server：  
   `env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_server -- .data/world_viewer_data 127.0.0.1:5010`
3. 启动 UI：  
   `env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5010`

## Viewer 快速运行（在线模式）
1. 启动 live server（脚本驱动，默认）：  
   `env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- twin_region_bootstrap --bind 127.0.0.1:5010`
2. 或启动 live server（LLM 驱动）：  
   `env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- llm_bootstrap --llm --bind 127.0.0.1:5010 --tick-ms 300`
   （需在根目录 `config.toml` 配置 `AGENT_WORLD_LLM_MODEL/BASE_URL/API_KEY`）
3. 启动 UI：  
   `env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- 127.0.0.1:5010`


## 路线图（摘要）
- M1：世界内核最小闭环（时间、地点、事件、行动校验、可恢复）
- M2：持久化与回放（快照/事件日志、可审计、可分叉）
- M3：Agent 运行时与 SDK（调度、限速、可观测性、记忆）
- M4：最小社会与经济（生产/消耗/交易/关系/声誉；电力系统已完成 M4.3 传输/交易基础接口）
- M5：可视化与调试工具（世界面板、事件浏览、回放与指标）

## 设计原则
- **世界是第一性**：Agent 只能通过感知得到有限信息，行动必须通过世界规则校验。
- **可审计**：关键状态变化尽量以事件形式记录，支持回放与定位问题。
- **可扩展**：先用抽象构建“像真的”系统，再逐步引入更细的规则与更大的规模。
