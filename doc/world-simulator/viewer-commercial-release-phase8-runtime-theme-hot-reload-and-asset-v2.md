# Viewer 商业化发行缺口收敛 Phase 8：运行中主题切换/热重载与 industrial_v2 资产包

## 目标
- 将 Phase 7 的“脚本预览链路”升级为运行中可操作能力：在 Viewer 内直接切换主题预设，并支持本地预设文件热重载。
- 交付第二代工业风资产包 `industrial_v2`，提升 mesh 细节与贴图质量，作为发行冲刺期默认美术包。
- 引入资产包可机读校验，确保主题包在提交前具备最小结构、尺寸与几何质量保障。

## 范围

### In Scope
- 新增运行中主题控制状态与应用系统（Apply / Hot Reload / 状态回显）。
- 新增右侧面板主题控制区（预设选择、自定义 preset 路径、应用按钮、热重载开关）。
- 新增 `industrial_v2` 主题包（mesh + base/normal/metallic_roughness/emissive）。
- 新增主题资产校验脚本（结构完整性、贴图尺寸、mesh 顶点阈值）。
- 更新 `doc/viewer-manual.md`、项目管理文档与 `doc/devlog/2026-02-21.md`。

### Out of Scope
- 不引入骨骼动画、角色状态机动画、VFX 粒子系统。
- 不改 world 协议与模拟语义。
- 不在本阶段实现 DCC 工具链自动导入（Maya/Blender 到仓库自动化流水线）。

## 接口 / 数据
- 新增运行时状态资源：`ThemeRuntimeState`（预设、路径、热重载状态、应用状态）。
- 新增预设约定环境变量（用于 preset 文件解析）：
  - `AGENT_WORLD_VIEWER_*_MESH_ASSET`
  - `AGENT_WORLD_VIEWER_*_{BASE,NORMAL,METALLIC_ROUGHNESS,EMISSIVE}_TEXTURE_ASSET`
  - `AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET`
- 新增主题资产目录：
  - `crates/agent_world_viewer/assets/themes/industrial_v2/meshes/*.gltf|*.bin`
  - `crates/agent_world_viewer/assets/themes/industrial_v2/textures/*.png`
  - `crates/agent_world_viewer/assets/themes/industrial_v2/presets/*.env`
- 新增资产校验入口：
  - `scripts/validate-viewer-theme-pack.py`

## 里程碑
- VCR8-0：设计文档与项目管理文档建档。
- VCR8-1：运行时主题状态、preset 解析与应用引擎落地。
- VCR8-2：右侧面板主题控制区与热重载流程落地（含回归测试）。
- VCR8-3：`industrial_v2` 资产包与预设落地。
- VCR8-4：主题资产校验脚本落地并完成 smoke 验证。
- VCR8-5：手册、项目状态、devlog 与测试收口。

## 风险
- 运行中切换导致场景对象材质/mesh 句柄串扰：
  - 缓解：统一走“资源重算 + 场景强制全量重建”路径，避免半刷新状态。
- 主题预设文件格式差异导致热重载失败：
  - 缓解：定义最小 `.env` 解析约束，失败回显明确错误并保留上一次成功配置。
- 资产体积上升影响仓库与 CI 速度：
  - 缓解：`industrial_v2` 先控制贴图分辨率与 mesh 面数阈值，并通过校验脚本守门。
