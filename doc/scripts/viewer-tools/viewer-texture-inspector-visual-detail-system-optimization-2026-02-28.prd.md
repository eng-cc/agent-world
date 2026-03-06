# Viewer Texture Inspector 视觉细节系统优化（2026-02-28）

审计轮次: 4

- 对应项目管理文档: doc/scripts/viewer-tools/viewer-texture-inspector-visual-detail-system-optimization-2026-02-28.prd.project.md

## 目标
- 将“细节不足”从一次性参数调整升级为可复用的系统能力。
- 建立三段式优化链路：镜头构图策略、实体级资源映射、评审灯光预设。
- 在保持现有分层门禁（连接/语义/细节/差异）前提下，提升可评审截图的信息密度。

## 范围
- **范围内**
  - 扩展 viewer 启动层能力：支持通过环境变量在启动时隐藏右侧面板，减少构图干扰。
  - 扩展 `viewer-texture-inspector.sh`：
    - 多候选 closeup 构图重试（而非单一 fallback）；
    - 实体资源包（entity/variant 级 mesh/texture 覆盖）；
    - 新评审灯光预设（可按实体组调参）；
    - 统一记录候选构图与资源来源元数据。
  - 执行 power 场景回归，验证细节指标是否上升、框架行为是否稳定。
- **范围外**
  - 不修改资产文件本身（贴图、模型内容）。
  - 不重写 viewer 渲染管线。
  - 不改 world_viewer_live 协议。

## 接口 / 数据
- Rust：`crates/agent_world_viewer/src/app_bootstrap.rs`
  - 新增启动环境变量：`AGENT_WORLD_VIEWER_PANEL_HIDDEN`（布尔）用于设置初始右侧面板可见性。
- Shell：`scripts/viewer-texture-inspector.sh`
  - 新增参数（计划）：
    - `--resource-pack-file <path>`：加载实体资源包（env 风格 key-value）。
    - `--lighting-profile <id>`：评审灯光预设（`art_review_v1`/`art_review_v2`）。
    - `--composition-profile <id>`：构图策略（包含 closeup 候选镜头集）。
  - 新增元数据（计划）：
    - `closeup_candidate_index`、`closeup_candidate_total`、`closeup_pose_label`
    - `resource_pack_file`、`resource_pack_override_hits`
    - `panel_hidden`、`lighting_profile`
- 资源包样例：`scripts/viewer-texture-inspector-resource-pack.example.env`

## 里程碑
- **T0 建档**：设计文档 + 项目管理文档。
- **T1 Viewer 启动层改造**：支持 `AGENT_WORLD_VIEWER_PANEL_HIDDEN` 并补测试。
- **T2 Inspector 系统优化落地**：构图候选策略 + 资源包 + 灯光预设。
- **T3 回归与结项**：执行 power 回归，更新结论与日志。
- **T4 构图稳定性修复**：设施尺度归一化 + direct_entity 隔离构图 + no-crop 策略。
- **T5 材质差异增强**：按实体拉开 power 设施变体差异，压低跨变体 SSIM。

## 风险
- **回归耗时增加**：多候选构图会增加重试成本。
  - 缓解：限制候选数量并只在关键实体/失败路径触发。
- **配置复杂度上升**：资源包引入后配置面扩大。
  - 缓解：提供样例文件并设定清晰优先级（CLI > 资源包 > preset）。
- **灯光与阈值耦合**：灯光变化可能影响 edge/ssim 指标。
  - 缓解：回归固定 profile，落盘完整指标用于比较。

## T3 回归结论（2026-03-01）
- 回归命令：
  - `./scripts/viewer-texture-inspector.sh --inspect power_plant,power_storage --scenario power_bootstrap --art-capture --preview-mode direct_entity --material-profile art_review_v1 --semantic-gate-mode auto --detail-edge-threshold 0.35 --variant-ssim-threshold 0.9995 --no-prewarm --out-dir output/texture_inspector/framework_t3_visual_opt_power_scene_20260301`
- 结果：
  - `power_plant`：`status=passed`，`min_pair_ssim_initial=0.986556`，`min_edge_energy_initial=0.430453`。
  - `power_storage`：`status=passed`，`min_pair_ssim_initial=0.987376`，`min_edge_energy_initial=0.476528`。
  - 元数据链路完整：`composition_profile=art_review_v2`、`lighting_profile=art_review_v2`、`panel_hidden=1`、`selection_gate_pass=1`。
- 视觉抽检：
  - closeup 成图仍以大面积平涂块面为主，实体读形不足；仅依赖 edge/ssim 指标会产生“通过但不可评审”的误判。
  - 关键线索：`selection_gate_orbit_radius_closeup` 在 power 场景稳定落在 `0.023040`，明显偏小。

## 后续优化入口
- 回归目标从“仅阈值通过”升级为“阈值通过 + 可读性通过（至少能看清主体轮廓与纹理走向）”。
- 在 T5 引入实体级材质参数链路（`power_plant` / `power_storage` 独立 roughness/metallic/emissive 通道）以替代 `facility` 统一参数。

## T4 回归结论（2026-03-01）
- 代码面完成：
  - 设施实体按 location 半径做尺度归一；`Transform` 与 `BaseScale` 同步更新。
  - `direct_entity` 预览下隐藏 location 网格干扰，但保留 location 半径与位置数据供设施缩放使用。
  - 语义 gate 与 closeup 元数据新增 `capture_auto_focus_target`，并在 direct_entity + `--crop-window auto` 时强制 `crop_window_effective=none`。
  - 高亮开关与 Halo 联动：`AGENT_WORLD_VIEWER_HIGHLIGHT_SELECTED=0` 时不再绘制高亮圈。
- 证据（probe）：
  - `output/texture_inspector/probe_focus_radius_power_default_isolated_nocrop_20260301/power_plant/default/meta.txt`
    - `preview_mode_effective=direct_entity`
    - `capture_auto_focus_target=first_power_plant`
    - `crop_window_effective=none`
    - `selection_gate_orbit_radius_closeup=0.023040`
    - `selection_gate_pass=1`
    - `closeup_edge_energy=0.372909`
  - `output/texture_inspector/probe_power_storage_direct_entity_nocrop_20260301/power_storage/default/meta.txt`
    - `capture_auto_focus_target=first_power_storage`
    - `crop_window_effective=none`
    - `closeup_edge_energy=0.372909`
- 结论：
  - T4 已解决“direct_entity 被自动裁切吃掉边缘信息”问题，细节门禁 edge 在 power 场景恢复到可用区间（>=0.35）。
  - 当前主要阻塞转为“变体差异不足”：`min_pair_ssim` 仍接近 1（约 `0.999974`），需在 T5 做材质链路级增强。

## T5 回归结论（2026-03-01）
- 代码面完成：
  - Rust 材质链路拆分完成：
    - `ViewerMaterialConfig` 新增 `materials.power_plant` / `materials.power_storage`，支持独立 `roughness/metallic/emissive_boost`。
    - 新增环境变量：
      - `AGENT_WORLD_VIEWER_MATERIAL_POWER_PLANT_{ROUGHNESS,METALLIC,EMISSIVE_BOOST}`
      - `AGENT_WORLD_VIEWER_MATERIAL_POWER_STORAGE_{ROUGHNESS,METALLIC,EMISSIVE_BOOST}`
    - `main.rs` / `main_ui_runtime.rs` / `theme_runtime.rs` 已切换为 power 实体独立材质通道。
    - `viewer_3d_config_profile_tests.rs` 补充 env override 与 invalid fallback 单测覆盖。
  - Inspector 框架级构图修正完成（非阈值补丁）：
    - `scripts/viewer-texture-inspector.sh` 中 power direct_entity 的 hero/closeup/fallback/retry 镜头从“近距”切换到“安全半径”策略（`zoom≈3.x`），避免相机落入模型内部导致空画面。
    - 保持材质 profile 与资源包链路，继续支持 power 的独立 roughness/metallic/emissive/base_color/emissive_color 覆盖。
- 根因复盘：
  - 先前 SSIM 高并非仅“材质差异不够”，而是 closeup 构图在 power direct_entity 下常进入模型内部，导致画面主体缺失、背景占比过高，差异指标被稀释。
  - T5 将“材质链路拆分”和“构图半径策略”同时落地，恢复了框架层的可评审性。
- 证据（正式回归）：
  - 回归目录：`output/texture_inspector/framework_t5_power_material_split_zoom_profile_20260301`
  - `power_plant/variant_validation.txt`：
    - `status=passed`
    - `min_pair_ssim_initial=0.994287`（阈值 `0.9995`）
    - `min_edge_energy_initial=0.470741`
    - `retry_candidates_attempted=0`
  - `power_storage/variant_validation.txt`：
    - `status=passed`
    - `min_pair_ssim_initial=0.994574`（阈值 `0.9995`）
    - `min_edge_energy_initial=0.453686`
    - `retry_candidates_attempted=0`
  - `meta.txt` 关键值（power 两实体一致）：
    - `selection_gate_orbit_radius_closeup=0.102400`
    - `viewer_art_closeup_ssim_capture_status=cropped`
    - `ssim_metric_crop_window=760:760:220:20`
- 结论：
  - T5 已完成：power_plant / power_storage 的实体级材质差异链路可用，且在 direct_entity 构图稳定前提下通过 SSIM 与 edge 双门禁。
  - 当前这条链路具备可复用性：后续新增实体可沿同一框架接入“实体级材质通道 + 安全半径构图 profile”。

## 原文约束点映射（内容保真）
- 约束-1（目标与问题定义）：沿用原“目标”章节约束，不改变问题定义与解决方向。
- 约束-2（范围边界）：沿用原“范围”章节的 In Scope/Out of Scope 语义，不扩散到新增范围。
- 约束-3（接口/里程碑/风险）：沿用原接口字段、阶段节奏与风险口径，并保持可追溯。
