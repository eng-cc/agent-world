# Agent World Simulator：Viewer Fragment 元素材质渲染与开关（设计文档）

## 目标
- 在 Viewer 中把 `location.fragment_profile.blocks` 的分块全部渲染出来，避免只看 location 外球体导致信息丢失。
- 按碎片主导元素显示不同颜色，提升“不同元素分布”可读性，支撑人工巡检与截图对比。
- 提供可控开关，默认关闭以保护性能，需要时再开启做细节分析。

## 范围

### In Scope
- 基于 `FragmentPhysicalProfile.blocks.blocks[*]` 逐块生成 3D 子实体。
- 通过 `infer_element_ppm` 推断每个分块的主导元素，并映射到固定材质色板。
- 在 Viewer 右侧 Overlay 区新增 fragment 显隐开关（人类交互）。
- 支持环境变量 `AGENT_WORLD_VIEWER_SHOW_FRAGMENT_ELEMENTS`（脚本/闭环可控）。
- 新增/更新单元测试覆盖元素材质映射、分块变换、开关配置解析。

### Out of Scope
- 不实现真实体素级布尔切割或网格重建。
- 不引入 per-frame 动态 LOD，先用“手动开关”控制性能成本。
- 不改 world server 协议，不新增 snapshot 字段。

## 接口 / 数据
- 数据来源：`WorldSnapshot.model.locations[*].fragment_profile.blocks.blocks`。
- 分块到元素：`infer_element_ppm(block.compounds)`，取 ppm 最大元素作为渲染主色。
- 元素色板：为 `FragmentElementKind` 全量建立材质句柄库（`FragmentElementMaterialHandles`）。
- 显隐控制：
  - UI 开关：`WorldOverlayConfig.show_fragment_elements`。
  - 启动默认值：`AGENT_WORLD_VIEWER_SHOW_FRAGMENT_ELEMENTS=on|off`。
- 渲染时机：`update_3d_scene` 在 snapshot 变化或开关变化时触发重建。

## 里程碑
- **FER1**：设计文档与项目管理文档。
- **FER2**：元素材质库与分块渲染模块接入。
- **FER3**：Overlay/UI + 环境变量开关接入。
- **FER4**：测试回归与文档收口。

## 风险
- 分块数量较大时，开启后可能影响帧率；通过默认关闭 + 按需启用缓解。
- 主导元素近似会丢失“混合成分”细节；后续可扩展为渐变或叠层表达。
- 不同显示设备上颜色感知有偏差；色板需持续在截图回归中校准。
