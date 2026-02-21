# Viewer 商业化发行缺口收敛 Phase 6：材质变体快速预览

## 目标
- 在不改动模拟协议与资源管线的前提下，提供运行时“材质变体快速预览”能力，缩短美术调参与观感对比回路。
- 支持在 Viewer 内快速切换预设（`default` / `matte` / `glossy`），覆盖核心实体材质的 roughness/metallic 表现。
- 保持默认兼容：未启用或未切换时与当前 Phase 5 输出一致。

## 范围

### In Scope
- 新增材质变体预览状态资源与预设枚举。
- 新增环境变量入口，支持启动时设置初始预设。
- 新增运行时热切换快捷键（`F8`）用于循环切换预设。
- 将预设系数应用到核心实体材质（agent/asset/power_plant/power_storage）的 roughness/metallic。
- 补充单元测试，覆盖预设解析、切换顺序、系数与 clamp 行为。
- 更新 `doc/viewer-manual.md` 与项目状态文档。

### Out of Scope
- 本阶段不引入运行中贴图资源热重载（文件监听/实时替换）。
- 本阶段不新增 UI 面板，仅提供环境变量 + 快捷键入口。
- 本阶段不调整 AO/clearcoat/parallax 等额外材质通道。

## 接口 / 数据
- 新增环境变量：
  - `AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET=default|matte|glossy`
- 新增运行时交互：
  - `F8`：在 `default -> matte -> glossy -> default` 之间循环切换。
- 数据结构（viewer 内部资源）：
  - `ViewerMaterialVariantPreset`
  - `MaterialVariantPreviewState`

## 里程碑
- VCR6-0：设计文档与项目管理文档建档。
- VCR6-1：预设解析 + 运行时热切换 + 材质应用 + 测试。
- VCR6-2：手册、项目状态与 devlog 收口。

## 风险
- 变体切换造成材质串扰：
  - 缓解：仅修改目标材质的 roughness/metallic 字段，不触及贴图句柄。
- 预设参数过激导致观感偏差：
  - 缓解：预设仅做温和倍率缩放并统一 clamp 到 `[0, 1]`。
- 键位冲突或误触：
  - 缓解：采用功能键 `F8`，并保留环境变量启动态以减少运行时操作依赖。
