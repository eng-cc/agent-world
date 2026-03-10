# Viewer 商业化发行 Phase 6 材质变体预览设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-commercial-release-phase6-material-variant-preview.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-commercial-release-phase6-material-variant-preview.project.md`

## 1. 设计定位
定义运行时材质变体预览能力：通过预设化 roughness/metallic 参数组和热切换入口，让主题材质在不改贴图资源的情况下快速做多版本对比。

## 2. 设计结构
- 预设解析层：`MaterialVariantPreset` 收敛若干风格化参数组。
- 运行时状态层：`MaterialVariantPreviewState` 记录当前预览变体。
- 应用层：只调整目标材质的 roughness/metallic 参数，不触及贴图句柄。
- 触发层：通过功能键或环境变量启动态完成快速切换预览。

## 3. 关键接口 / 入口
- `MaterialVariantPreset`
- `MaterialVariantPreviewState`
- roughness / metallic 参数倍率
- `F8` 或环境变量启动态

## 4. 约束与边界
- 变体切换不能造成不同实体之间材质串扰。
- 参数缩放需统一 clamp 到 `[0,1]`。
- 本阶段只做材质参数预览，不新增热重载 UI 面板。
- 贴图句柄与资源映射保持不变。

## 5. 设计演进计划
- 先固定预设参数和解析逻辑。
- 再实现运行时热切换和材质应用。
- 最后通过测试与手册把变体预览能力固化为 release 工具链一环。
