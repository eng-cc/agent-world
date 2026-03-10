# Viewer 贴图查看器设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-texture-inspector.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-texture-inspector.project.md`

## 1. 设计定位
定义一个专门用于贴图风格和材质回归对比的脚本化查看器：在统一观察载体和固定镜头下批量导出截图，避免在完整世界场景里手动找角度。

## 2. 设计结构
- 脚本入口层：`viewer-texture-inspector.sh` 负责参数解析和批量执行。
- 观察载体层：通过固定 location 槽位和 automation steps 统一视角与构图。
- 贴图覆盖层：支持 base/normal/mr/emissive 的临时贴图替换与材质变体批量检查。
- 证据归档层：输出截图、日志和元数据到独立目录便于回归对比。

## 3. 关键接口 / 入口
- `scripts/viewer-texture-inspector.sh`
- `--inspect` / `--variants`
- `--base-texture` / `--normal-texture` / `--mr-texture` / `--emissive-texture`
- `output/texture_inspector/<timestamp>/...`

## 4. 约束与边界
- 本期只做脚本化闭环，不新增 Viewer 运行中 UI 面板。
- 工具用于贴图风格/对比验证，不替代最终场景验收。
- 批量截图范围必须可裁剪，避免一次执行过慢。
- 复用现有 capture 闭环，不引入新的图形运行时依赖。

## 5. 设计演进计划
- 先落脚本参数和贴图映射。
- 再补统一观察载体与输出归档。
- 最后把入口接入手册并完成回归。
