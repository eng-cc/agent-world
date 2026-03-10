# Viewer Fragment 元素材质渲染设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-fragment-element-rendering.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-fragment-element-rendering.project.md`

## 1. 设计定位
定义 fragment blocks 的元素着色与可控显隐方案：按主导元素为每个分块建立稳定材质表达，并通过 UI/环境变量开关控制性能成本。

## 2. 设计结构
- 数据读取层：从 `location.fragment_profile.blocks.blocks` 提取分块几何与化合物信息。
- 元素推断层：`infer_element_ppm` 负责为每个分块选取主导元素作为渲染主色。
- 材质映射层：`FragmentElementMaterialHandles` 提供元素到材质句柄的统一色板。
- 显隐控制层：`WorldOverlayConfig.show_fragment_elements` 与环境变量共同控制分块渲染开关。

## 3. 关键接口 / 入口
- `infer_element_ppm`
- `FragmentElementMaterialHandles`
- `location_fragment_render.rs`
- `WorldOverlayConfig.show_fragment_elements`

## 4. 约束与边界
- 本专题只做可视表达，不做真实体素切割、LOD 或协议扩展。
- 默认关闭是性能保护策略，不能在本轮演化成强制常开。
- 元素颜色表达以主导元素近似为准，不承担成分分析精确度职责。
- UI 开关与环境变量需要共用同一配置源，避免状态分叉。

## 5. 设计演进计划
- 先建立元素材质库与分块渲染模块。
- 再接 UI/环境变量开关与场景重建触发。
- 最后通过测试和说明文档收口元素可视化能力。
