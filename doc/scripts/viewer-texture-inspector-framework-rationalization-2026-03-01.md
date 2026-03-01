# Viewer Texture Inspector 框架合理性优化（2026-03-01）

## 目标
- 在 T5 已达成视觉差异门禁通过的基础上，继续降低可视化层的“补丁式”维护成本。
- 将关键能力从“零散分支逻辑”收敛为“可扩展结构”：
  - Rust 侧配置解析模块化；
  - Shell 侧构图策略统一入口。
- 为后续新增实体/材质通道提供稳定扩展面，避免继续堆叠 case 分支。

## 范围
- **范围内**
  - `viewer_3d_config` 解析函数拆分，降低单文件复杂度并满足文件长度约束。
  - `viewer-texture-inspector.sh` 的 power 构图策略收敛为统一解析接口，减少重复逻辑。
  - 补充对应回归验证（Rust 定向测试 + shell 语法检查 + 关键路径 smoke）。
- **范围外**
  - 不修改渲染管线（Bevy shader / render graph）。
  - 不修改业务协议（world_viewer_live）。
  - 不引入新的实体资产格式。

## 接口 / 数据
- Rust（内部模块拆分，不改外部调用接口）
  - 新增内部解析模块文件：`crates/agent_world_viewer/src/viewer_3d_config_parsing.rs`。
  - `resolve_viewer_3d_config` / `resolve_viewer_external_*` 函数签名保持不变。
- Shell
  - 保持现有 CLI 参数不变；
  - 将 power 的 hero/closeup/fallback pose 解析收敛到统一函数，retry 候选调用同一来源。

## 里程碑
- **T0 建档**：设计文档 + 项目管理文档。
- **T1 Rust 配置模块化**：抽离解析函数，`viewer_3d_config.rs` 回落至 1200 行以内，补/跑定向测试。
- **T2 Shell 构图策略结构化**：统一 power pose 解析路径，跑 syntax + 回归 smoke。

## 风险
- **解析回归风险**：模块拆分后可能误改 env 键行为。
  - 缓解：运行 `viewer_3d_config_profile_tests` 的 override/invalid 两组用例。
- **构图漂移风险**：统一函数可能引入参数误差。
  - 缓解：保持原参数字面值，回归 power 场景最小链路并比对关键 meta 字段。
