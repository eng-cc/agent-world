# Viewer Live 禁用 Seek 语义设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-disable-seek-p2p-2026-02-27.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-disable-seek-p2p-2026-02-27.project.md`

## 1. 设计定位
定义 live 模式下的“不可 seek 回退”控制语义：保证 P2P 实时链路只允许单调前进，避免测试接口与共识语义出现行为冲突。

## 2. 设计结构
- 控制白名单层：live 模式仅允许 `play/pause/step`，显式禁用 `seek`。
- 入口收敛层：viewer 玩家入口与 Web Test API 不再把 seek 暴露为可用动作。
- 协议兼容层：保留 `ViewerControl::Seek` 枚举，避免非 live 场景破坏兼容。
- 验证说明层：测试与文档同步声明“live 单调前进、不可回退”的边界。

## 3. 关键接口 / 入口
- `ViewerControl::Seek`
- `live_split_part2.rs`
- `web_test_api.rs`
- `egui_right_panel_controls.rs`

## 4. 约束与边界
- 禁用 seek 只作用于 live，不扩散到非 live viewer server。
- 协议层保留 seek 枚举，但 live 入口必须稳定拒绝或不暴露该动作。
- 测试脚本与玩家引导需要同步去 seek 化，避免出现“发送成功但无效”的错觉。
- 本阶段不重构 timeline 模块整体架构。

## 5. 设计演进计划
- 先冻结 live 无 seek 语义。
- 再收敛控制处理与前端入口动作集合。
- 最后通过测试与文档回写固化边界。
