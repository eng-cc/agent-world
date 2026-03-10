# Viewer Live runtime/world 接管 Phase 1 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase1-2026-03-04.project.md`

## 1. 设计定位
定义 runtime/world 接管 live server 的第一阶段方案：在不改 Viewer 协议的前提下引入 runtime 驱动链路，并通过兼容适配继续输出 `WorldSnapshot/WorldEvent`。

## 2. 设计结构
- 启动分流层：`world_viewer_live` 新增 `--runtime-world` 开关选择 runtime 驱动链路。
- 协议兼容层：runtime 输出继续适配成现有 `WorldSnapshot/WorldEvent`。
- 事件覆盖层：优先打通 `AgentRegistered/AgentMoved/ResourceTransferred/ActionRejected` 等关键事件。
- 回归验收层：required 测试负责验证 runtime 模式下 Viewer 基础交互不退化。

## 3. 关键接口 / 入口
- `--runtime-world`
- `runtime::World`
- `WorldSnapshot` / `WorldEvent` 兼容适配
- `world_viewer_live` 启动链路

## 4. 约束与边界
- 不在 Phase 1 改 Viewer 协议和前端 UI。
- 只做 runtime 驱动链路接入，不要求全量动作映射一次完成。
- simulator 链路暂时仍保留，直到后续阶段完成收敛。
- 适配层必须以协议兼容为先，避免 Viewer 侧感知到实现切换。

## 5. 设计演进计划
- 先建立 runtime live server 启动开关。
- 再补关键事件适配和基础快照输出。
- 最后用 required 测试冻结 phase1 接管边界。
