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
