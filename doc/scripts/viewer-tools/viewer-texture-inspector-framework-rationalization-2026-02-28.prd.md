# Viewer Texture Inspector 框架合理性治理（2026-02-28）

## 目标
- 从“参数可调”升级为“框架可验证”：将截图链路拆为可观测、可门禁、可回退三层。
- 解决当前“连通但画面不可评审”的盲点：连接成功不代表选中正确目标，也不代表画面有足够材质/造型信息。
- 将“脚本 patch”演进为“系统机制”：把语义状态、视觉质量和变体差异统一纳入质检框架。

## 范围
- **范围内**
  - 扩展 viewer 内部 capture status，输出可用于自动门禁的语义状态。
  - 重构 texture inspector 的质检流程：连接层、语义层、视觉细节层、差异层分层判定。
  - 统一将质量指标写入产物元数据和 validation 文件，支持后续治理闭环。
  - 形成最小回归矩阵（power_bootstrap + direct_entity + art_capture）验证框架有效性。
- **范围外**
  - 不在本轮重写 viewer 渲染管线或材质资产本身。
  - 不在本轮引入新的图像处理依赖（沿用 ffmpeg）。
  - 不在本轮改 world_viewer_live 协议结构。

## 接口 / 数据
- Rust：`crates/agent_world_viewer/src/internal_capture.rs`
  - 扩展状态输出字段（示例）：
    - 选中语义：`selection_kind`、`selection_id`
    - 相机语义：`orbit_radius`、`camera_mode`
    - 场景语义：`scene_power_plant_count`、`scene_power_storage_count`
- 脚本：`scripts/viewer-texture-inspector.sh`
  - 新增或增强质检参数：
    - `--detail-edge-threshold <f>`：近景细节阈值（基于边缘能量）
    - `--semantic-gate-mode <mode>`：语义门禁策略（off, auto, strict）
  - 新增质量指标：
    - `closeup_edge_energy`、`min_edge_energy_*`
    - `selection_gate_*`
    - `variant_validation_*`（融合差异与细节）

## 里程碑
- **T0 建档**：输出设计文档与项目管理文档。
- **T1 状态可观测增强**：viewer capture status 增加语义字段，补测试。
- **T2 门禁框架落地**：脚本实现分层验证（连接/语义/细节/差异）并输出统一元数据。
- **T3 回归与收口**：执行 power 场景矩阵回归，更新文档与日志并结项。

## 风险
- **阈值误报风险**：细节阈值在不同场景下可能偏严或偏松。
  - 缓解：阈值参数化并记录指标，先以 `auto` 模式渐进启用。
- **语义字段兼容风险**：capture status 扩展可能影响旧脚本解析。
  - 缓解：保持 key-value 追加式输出，脚本按“缺省可退化”读取。
- **回归时间成本**：分层门禁会增加重试和验证开销。
  - 缓解：仅对关键实体（power）启用严格矩阵，其他实体保留轻量路径。

## T3 回归结论（2026-02-28）
- 回归命令：
  - `./scripts/viewer-texture-inspector.sh --inspect power_plant,power_storage --scenario power_bootstrap --art-capture --preview-mode direct_entity --material-profile art_review_v1 --semantic-gate-mode auto --detail-edge-threshold 0.35 --variant-ssim-threshold 0.9995 --no-prewarm --out-dir output/texture_inspector/framework_t2_power_scene_20260228_r2`
- 关键产物：
  - `output/texture_inspector/framework_t2_power_scene_20260228_r2/power_plant/variant_validation.txt`
  - `output/texture_inspector/framework_t2_power_scene_20260228_r2/power_storage/variant_validation.txt`
- 结果摘要：
  - 两类实体均 `status=failed_after_retry`，`retry_reason=low_edge_energy`；
  - `semantic_fail_count_initial/after_retry=0`，语义门禁稳定通过；
  - `unique_count=3` 且 `min_pair_ssim < threshold(0.9995)`，差异门禁通过；
  - `min_edge_energy≈0.136~0.137 < threshold(0.35)`，细节门禁持续失败。
- 框架合理性评估：
  - 结论：当前分层框架合理，能把“连接成功/目标正确”与“画面可评审”明确解耦并定位到细节层问题，避免过去仅靠 hash/SSIM 的误判。
  - 现状问题不是“门禁失效”，而是“近景构图与材质可读性不足”；后续应针对镜头构图、局部光照与实体专属资源进行系统优化。

## 原文约束点映射（内容保真）
- 约束-1（目标与问题定义）：沿用原“目标”章节约束，不改变问题定义与解决方向。
- 约束-2（范围边界）：沿用原“范围”章节的 In Scope/Out of Scope 语义，不扩散到新增范围。
- 约束-3（接口/里程碑/风险）：沿用原接口字段、阶段节奏与风险口径，并保持可追溯。
