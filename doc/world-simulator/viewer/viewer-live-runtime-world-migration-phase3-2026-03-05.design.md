# Viewer Live runtime/world 接管 Phase 3 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase3-2026-03-05.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-runtime-world-migration-phase3-2026-03-05.project.md`

## 1. 设计定位
定义 runtime/world 接管的第三阶段收口方案：扩展 action 映射覆盖并删除 `world_viewer_live` 的 simulator 启动分支，使 live server 收敛为 runtime-only 单链路。

## 2. 设计结构
- 动作映射层：扩展 `simulator_action_to_runtime` 覆盖模块工件与关键旧链路动作。
- 拒绝兜底层：不可映射动作统一返回结构化拒绝，避免 panic 或隐式 fallback。
- 启动收敛层：`world_viewer_live` 删除 simulator live 分支，只保留 runtime live server。
- 手册同步层：活跃文档与手册统一切换到 runtime-only 启动口径。

## 3. 关键接口 / 入口
- `simulator_action_to_runtime`
- `runtime_live/control_plane.rs`
- `runtime_live.rs`
- `world_viewer_live.rs`

## 4. 约束与边界
- 本阶段仍不要求 simulator action 到 runtime action 的全量 1:1 映射。
- Viewer 协议和前端 UI 不在本轮重构范围。
- 删除旧分支前必须有等价回归与拒绝语义测试兜底。
- runtime-only 收敛后，任何 legacy 参数都不应再触发 simulator server。

## 5. 设计演进计划
- 先扩关键动作映射和拒绝测试。
- 再删除 simulator 启动分支并更新手册。
- 最后通过 required 回归冻结 runtime-only 主路径。
