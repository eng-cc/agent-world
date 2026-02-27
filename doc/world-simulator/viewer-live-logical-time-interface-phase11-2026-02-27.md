# Viewer Live 逻辑时间与事件游标接口改造 Phase 11（2026-02-27）

## 目标
- 在保持 viewer live 事件驱动调度不回退的前提下，将对外交互从“直接依赖 tick”迁移到“事件游标 + 逻辑时间”双语义。
- 保留 runtime 内部逻辑时间（当前 tick）作为领域一致性锚点，不做错误的“去时间轴”改造。
- 为 Web 自动化/测试链路提供无 tick 依赖入口，降低外部接口与内部时间实现耦合。

## 范围

### In Scope
- `crates/agent_world_viewer/src/web_test_api.rs`：
  - `getState` 增加 `logicalTime`、`eventSeq` 字段。
  - 兼容保留 `tick`（作为 `logicalTime` 别名）。
  - `sendControl` 增加事件游标动作（`seek_event`），在前端本地转换为现有 `Seek { tick }` 请求。
- 仅改造 viewer 对外接口层，不改 runtime 规则层。
- 同步更新项目文档与 devlog。

### Out of Scope
- 不删除 `kernel.time()` / `total_ticks` 等内部逻辑时间结构。
- 不改 `agent_world_proto::ViewerControl::Seek { tick }` 协议定义（本阶段走兼容迁移）。
- 不改所有 UI 文案（timeline 的彻底术语迁移放到后续阶段）。

## 接口/数据
- `window.__AW_TEST__.getState()`：
  - 新增字段：`logicalTime: number`、`eventSeq: number`。
  - 兼容字段：`tick`（与 `logicalTime` 同值）。
- `window.__AW_TEST__.sendControl(action, payload)`：
  - 新增 `action="seek_event"`。
  - `payload` 接收 `{ eventSeq }` 或纯数字，前端基于当前缓存事件映射到 `Seek { tick }`。

## 里程碑
1. M1：完成设计/项目建档并明确“内部保留逻辑时间、外部去 tick 耦合”边界。
2. M2：完成 `web_test_api` 改造（状态字段 + 事件游标控制）。
3. M3：通过 required 测试并完成文档收口。

## 风险
- 事件窗口裁剪时，`eventSeq -> tick` 映射可能找不到目标事件；需要明确告警与降级行为。
- 外部脚本仍读取 `tick`，若直接删除会破坏兼容；本阶段仅做别名保留。
- 若误触 runtime 内核时间语义，会引入回放与规则层回归；需严格限定改造边界。
