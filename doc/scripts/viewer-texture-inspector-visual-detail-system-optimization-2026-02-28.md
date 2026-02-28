# Viewer Texture Inspector 视觉细节系统优化（2026-02-28）

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
- 在 T4 引入设施实体尺度归一（按 location 半径/世界单位缩放），修复“设施尺寸与镜头半径不一致”导致的贴脸构图。
- 回归目标从“仅阈值通过”升级为“阈值通过 + 可读性通过（至少能看清主体轮廓与纹理走向）”。
