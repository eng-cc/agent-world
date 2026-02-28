# Viewer Texture Inspector 材质可辨识增强（2026-02-28）

## 目标
- 从“截图可连通”升级到“材质可评审”，让 `default/matte/glossy` 在美术视角下具备稳定可辨识差异。
- 将预览链路拆成“场景代理预览”和“材质评审预览”，避免 location 叠层把差异冲淡。
- 支持“按实体、按变体”的材质参数与贴图配置，不再仅依赖全局 roughness/metallic 缩放。
- 建立可持续门禁（哈希 + SSIM + 人工抽检）防止后续回退到“同图不同名”。

## 范围
- **范围内**
  - 改造 `scripts/viewer-texture-inspector.sh`，新增预览模式并固化 `lookdev` 评审口径。
  - 在 Inspector 中补齐 `meta.txt` 记录（预览模式、材质评审开关、阈值口径）。
  - 逐步接入“实体直看（direct entity）”能力，减少对 `LOCATION_*` 覆盖链路依赖。
  - 为后续 per-entity/per-variant 参数表与变体贴图预留接线。
  - 增补脚本级 smoke 与视觉门禁验证流程。
- **范围外**
  - 不在本阶段重写 viewer 渲染管线（PBR 模型保持不变）。
  - 不在本阶段改 world_viewer_live 协议结构。
  - 不在本阶段引入新的截图框架（沿用 Playwright + capture-viewer-frame 链路）。

## 接口 / 数据
- 脚本：`scripts/viewer-texture-inspector.sh`
- 计划新增/增强参数：
  - `--preview-mode <mode>`：
    - `scene_proxy`：当前链路（默认），使用 location 代理槽位预览。
    - `lookdev`：评审模式，关闭 location 壳层干扰（shell/radiation/damage）。
    - `direct_entity`：实体直看模式，直接使用被检实体槽位（按里程碑接入）。
  - `--variant-ssim-threshold <f>`：变体近似门禁阈值（已存在，后续按模式分层）。
- 输出：
  - `output/texture_inspector/<ts>/<entity>/<variant>/meta.txt` 增加：
    - `preview_mode=...`
    - `lookdev_location_shell_enabled=...`
    - `lookdev_location_radiation_glow=...`
    - `lookdev_location_damage_visual=...`

## 里程碑
- **T0 建档**：设计文档 + 项目管理文档。
- **T1 预览口径增强**：新增 `--preview-mode`，落地 `lookdev`（关闭 location shell/radiation/damage 干扰）。
- **T2 直看链路**：新增 `direct_entity` 预览链路，减少 power 材质评审对 `LOCATION_*` 的依赖。
- **T3 变体参数层**：按 `entity x variant` 引入可配置参数（roughness/metallic/normal 强度等）。
- **T4 变体贴图层**：支持变体级贴图槽位（至少 metallic_roughness/normal）。
- **T5 验收与门禁**：固定评审矩阵（机位/光照），完善 SSIM 阈值与 smoke/回归脚本。

## 风险
- **模式切换兼容风险**：新增 `preview-mode` 可能影响现有脚本调用。
  - 缓解：默认保持 `scene_proxy`，新模式 opt-in。
- **场景覆盖不足**：`direct_entity` 依赖场景中存在目标实体，某些场景可能缺失。
  - 缓解：保留 `scene_proxy` 回退，元数据记录回退原因。
- **门禁误判风险**：仅靠哈希或单一 SSIM 可能误判可读性。
  - 缓解：保留双指标并增加人工抽检样例。
- **参数过拟合**：在单个场景调出的差异不一定可泛化。
  - 缓解：按实体和光照矩阵验证，不在单视角定标。
