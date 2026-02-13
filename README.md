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


## 路线图（TODO）

## 设计原则
- **世界是第一性**：Agent 只能通过感知得到有限信息，行动必须通过世界规则校验。
- **可审计**：关键状态变化尽量以事件形式记录，支持回放与定位问题。
- **可扩展**：先用抽象构建“像真的”系统，再逐步引入更细的规则与更大的规模。
