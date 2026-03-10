# Viewer Live 逻辑时间与事件游标接口设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-logical-time-interface-phase11-2026-02-27.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-logical-time-interface-phase11-2026-02-27.project.md`

## 1. 设计定位
定义 viewer live 对外接口从“直接依赖 tick”向“逻辑时间 + 事件游标”双语义迁移的兼容方案，降低外部自动化和内部时间实现的耦合。

## 2. 设计结构
- 状态输出层：`getState()` 同时输出 `logicalTime`、`eventSeq` 与兼容别名 `tick`。
- 控制映射层：`seek_event` 在前端本地映射到现有 `Seek { tick }` 请求。
- 兼容迁移层：内部保留逻辑时间实现，对外先去 tick 耦合而非硬删除。
- 失败降级层：事件窗口找不到映射目标时返回稳定告警与拒绝。

## 3. 关键接口 / 入口
- `window.__AW_TEST__.getState()`
- `window.__AW_TEST__.sendControl()`
- `logicalTime` / `eventSeq` / `tick`
- `seek_event` / `seek_event_seq`

## 4. 约束与边界
- 不删除 runtime 内部逻辑时间结构，也不改协议层 `ViewerControl::Seek { tick }`。
- 本轮只改 viewer 对外接口层，不做 timeline 全量术语迁移。
- 兼容字段 `tick` 必须保留到脚本迁移完成，不能直接移除。
- 事件游标映射失败要可诊断，不能静默落到错误时间点。

## 5. 设计演进计划
- 先补状态字段兼容层。
- 再补 `seek_event` 控制映射。
- 最后通过 required 测试和文档收口接口迁移。
