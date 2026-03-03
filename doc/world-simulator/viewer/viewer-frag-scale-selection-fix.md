# Viewer Frag 实际比例与选中显示修复（设计文档）

## 目标
- 修复 Viewer 中 frag 分块显示比例失真问题，使渲染尺度与 snapshot 数据尺寸保持一致。
- 修复 frag 在“选中高亮/取消选中”流程中的显示异常，确保选中前后都保持正确几何比例。
- 去掉 frag 选中后的黄色高亮范围（selection halo），避免视觉上误判 frag 尺寸。
- 修正 Agent 几何尺度口径，使其与 `cm_to_unit` 使用同一量纲映射，避免与 frag 出现“看起来同量级”的错觉。
- 保持现有事件/详情面板与交互链路兼容，不引入协议改动。

## 范围

### In Scope
- 调整 frag 渲染尺度映射链路，去除会破坏尺寸比例的可视化钳制。
- 在 frag 分块实体上补齐高亮恢复所需的基线缩放组件（`BaseScale`）。
- 补充单测，覆盖：
  - 尺度映射按 `radius_cm * cm_to_unit` 线性生效；
  - frag 被选中/取消选中后不丢失原始比例。
  - frag 选中不显示黄色 halo；
  - Agent 几何在非 physical 模式下仍与世界尺度同量纲。

### Out of Scope
- 不改 world server 协议、不新增 snapshot 字段。
- 不调整右侧面板信息结构与多语言文案策略。
- 不引入新的渲染特效或复杂 LOD 体系。

## 接口 / 数据
- `scene_helpers::location_render_radius_units`
  - 口径：`radius_world_units = radius_cm * cm_to_unit`（线性映射）。
  - 目标：frag 父级尺度与数据半径一致，避免最小/最大钳制造成比例失真。
- `location_fragment_render::spawn_location_fragment_elements`
  - 分块实体新增 `BaseScale`，值取创建时的 `Transform.scale`。
  - 选中高亮仍复用现有 `apply_entity_highlight/reset_entity_scale`，但可正确恢复到原始分块比例。
- 测试
  - 更新 `scene_helpers` 中 location 尺度映射断言，改为验证线性比例。
  - 在场景重建测试中验证 frag 子实体带有 `BaseScale` 且与初始 `Transform.scale` 一致。
  - 新增 selection emphasis 断言：`SelectionKind::Fragment` 时不显示 halo。
  - 更新 agent 尺度断言：按 `effective_cm_to_unit` 推导期望几何尺寸。

## 里程碑
- **FSF1**：设计文档与项目管理文档落地。
- **FSF2**：完成 frag 尺度映射与选中显示 bug 修复。
- **FSF3**：补齐/更新测试并执行 `test_tier_required` 回归。
- **FSF4**：更新项目文档状态与开发日志收口。
- **FSF5**：修复误选与 frag 缩放高亮副作用。
- **FSF6**：移除 frag 选中黄色 halo；修正 Agent 尺度与世界量纲不一致问题。
- **FSF7**：修复相机缩放与裁剪口径（近远裁剪/最小缩放半径）与世界单位不一致，恢复 Agent 在放大场景下可见性。

## 风险
- 去掉钳制后，极端大尺寸 frag 可能在默认视角下占屏更明显。
- 历史测试若依赖旧钳制口径，需同步修正断言，避免误报回归。
- 高亮恢复逻辑依赖 `BaseScale`；后续若新增实体类型未补该组件，可能复发同类问题。
- Agent 尺度改为真实量纲后，在默认缩放下会更小，可能影响可见性与人工点选体验。
- 相机裁剪与缩放下限若仍使用旧单位，可能在小尺度实体场景出现“放大后仍不可见”的感知退化。
