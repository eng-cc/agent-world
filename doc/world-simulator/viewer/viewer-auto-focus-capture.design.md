# Viewer 自动聚焦与截图闭环设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-auto-focus-capture.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-auto-focus-capture.project.md`

## 1. 设计定位
定义 Viewer 面向人工调试与截图脚本共用的自动聚焦能力：通过启动配置、快捷键和脚本参数，让相机稳定对准目标并形成可重复截图闭环。

## 2. 设计结构
- 启动配置层：环境变量控制一次性自动聚焦目标、半径和 2D/3D 模式。
- 人工操作层：快捷键 `F` 对当前选中对象执行聚焦。
- 脚本透传层：`capture-viewer-frame.sh` 将参数映射到 Viewer 环境变量。
- 安全降级层：目标不存在或场景未就绪时安全不动作，不破坏默认观察行为。

## 3. 关键接口 / 入口
- `OASIS7_VIEWER_AUTO_FOCUS`
- `OASIS7_VIEWER_AUTO_FOCUS_TARGET`
- `OASIS7_VIEWER_AUTO_FOCUS_FORCE_3D`
- `OASIS7_VIEWER_AUTO_FOCUS_RADIUS`
- 快捷键 `F`
- `capture-viewer-frame.sh`

## 4. 约束与边界
- 无参数时保持现有相机逻辑，不强制改变默认行为。
- 自动切 3D 仅限启动聚焦一次，避免与人工操作冲突。
- 目标解析失败必须静默降级为不动作，但可记录诊断。
- 本轮不引入平滑镜头动画与录像导出。

## 5. 设计演进计划
- 先补环境变量与目标解析。
- 再接通快捷键与脚本参数映射。
- 最后通过截图闭环与测试固定聚焦行为基线。
