# 纯 API 客户端等价玩法专题设计说明

- 对应需求文档: `doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.prd.md`
- 对应项目管理文档: `doc/game/gameplay/gameplay-pure-api-client-parity-2026-03-19.project.md`

审计轮次: 1

## 1. 设计目标

- 把纯 API 客户端从 `observer/probe` 提升为正式 `playable` 客户端。
- 明确哪些玩家语义必须下沉到协议层，哪些仍可以留在表现层。
- 为 `runtime_engineer` / `viewer_engineer` / `qa_engineer` 提供统一的等价验收基线。

## 2. 核心原则

1. 单一事实源
   - 玩家阶段、主目标、阻塞、下一步建议、可执行动作必须来自协议级 canonical 快照。
2. 表现可差异，语义不可差异
   - Web/UI 可以用卡片、颜色、布局表达；API 可以用 JSON、CLI、TUI 表达。
   - 但两边看到的核心玩法信息和能做的事情必须一致。
3. 可持续游玩优先于调试便利
   - `request_snapshot + step` 只够做探针，不够做正式客户端。
4. 观察模式与游玩模式分离
   - `observer_only` 可以降级。
   - `player_playable` 不允许降级信息和动作能力。

## 3. 协议分层建议

### 3.1 事实层

- 继续保留原始 `snapshot / event / metrics / control_feedback`。
- 该层用于审计、调试、回放和低层客户端。

### 3.2 玩家语义层

- 新增 canonical `player_gameplay_snapshot`：
  - 当前阶段
  - 当前主目标
  - 进度
  - 主阻塞
  - 下一步建议
  - 最近控制反馈
  - 当前可执行动作集合
- 该层由协议提供，UI/API 统一消费。

### 3.3 表现层

- Web/UI:
  - Mission HUD
  - 阶段切换卡
  - 阻塞卡
  - 分支推荐卡
- API:
  - JSON
  - CLI/TUI 文本块
  - 自动代理可直接消费的结构化字段

## 4. 责任边界

- `producer_system_designer`
  - 定义哪些字段属于“玩家继续玩所必需”的语义。
- `runtime_engineer`
  - 保证事实状态、事件、权限边界和恢复基线可供聚合。
- `viewer_engineer`
  - 抽离当前 UI 私有聚合逻辑，形成协议级 canonical 语义。
- `agent_engineer`
  - 确保纯 API 模式下的聊天/命令/策略驱动不因无 UI 缺失关键上下文。
- `qa_engineer`
  - 建立 parity matrix 和长玩 required/full 验收。

## 5. 推荐实现顺序

1. 先冻结 parity schema
   - 字段名、状态机、动作集合、缺失时的降级语义。
2. 再下沉现有 Viewer 私有语义
   - 先覆盖 `FirstSessionLoop`、工业引导、`PostOnboarding`。
3. 再提供纯 API 正式客户端入口
   - 至少能查看、推进、恢复、聊天/命令、完成阶段承接。
4. 最后做 parity 回归
   - required-tier 长玩
   - full-tier 长稳

## 6. 验收重点

- 纯 API 玩家是否知道自己当前处于哪个阶段。
- 纯 API 玩家是否知道下一步能做什么以及为什么被卡住。
- 纯 API 玩家是否能跨会话继续推进，而不是每次都从原始事件重新推断。
- UI 和 API 是否真正共用一份语义，而不是表面相似。

## 7. 非目标

- 不在本轮定义视觉等价。
- 不在本轮重构所有 debug/observer 通道。
- 不在本轮引入新的独立玩法系统。
