# Viewer 与节点彻底拆分设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-node-hard-decouple-2026-02-28.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-node-hard-decouple-2026-02-28.project.md`

## 1. 设计定位
定义 `world_viewer_live` 与节点运行时的硬解耦方案：把 viewer 入口收敛成纯 viewer 进程，不再注入 consensus runtime、gate 和 node 参数。

## 2. 设计结构
- 入口收敛层：`world_viewer_live` 改为纯 viewer 入口。
- 参数清理层：删除内嵌节点参数与启动链路，CLI 对旧参数给出显式报错。
- 依赖分离层：viewer 仅依赖 `ViewerLiveServer` / `ViewerWebBridge`，不再持有节点侧运行时。
- 迁移提示层：引导用户改用 `world_chain_runtime` / `world_game_launcher`。

## 3. 关键接口 / 入口
- `world_viewer_live`
- `ViewerLiveServer` / `ViewerWebBridge`
- `world_chain_runtime`
- `world_game_launcher`

## 4. 约束与边界
- 本阶段只做 viewer/node 入口硬解耦，不处理后续历史死代码归档。
- 旧脚本若仍依赖 `--node-*` 参数，必须收到明确迁移提示。
- 去耦后 viewer 不再注入 consensus runtime 和 gate。
- 新边界要在 required 测试和文档入口中保持一致。

## 5. 设计演进计划
- 先冻结纯 viewer 入口边界。
- 再清理 node 参数与运行链路。
- 最后通过回归测试和文档收口完成拆分。
