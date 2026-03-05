# Viewer 资产管线与 UI 体系硬化（2026-03-05）

- 对应项目管理文档: `doc/world-simulator/viewer/viewer-asset-pipeline-ui-system-hardening-2026-03-05.prd.project.md`
- 模块主追踪: `PRD-WORLD_SIMULATOR-018`

## 1. Executive Summary
- Problem Statement: 当前主题资产校验只有下限门禁，且 Viewer 在初始化与热更新阶段存在主题应用与外部配置解析重复路径，叠加超长文件导致维护成本与回归风险持续上升。
- Proposed Solution: 补齐资产预算上限门禁、收敛 Runtime 主题应用与解析逻辑到单一路径、完成 UI/Shell 超长文件模块化拆分，并清理遗留 Bevy UI 路径/合并 RightPanelLayoutState 等布局状态到 egui 单一路径，配合现有发布回归脚本闭环验证。
- Success Criteria:
  - SC-1: 主题包校验脚本支持 profile 级资产上限门禁（纹理尺寸/纹理字节/网格顶点）。
  - SC-2: Viewer 初始化与 runtime 热更新复用同一资产应用函数与同一 external config 解析函数。
  - SC-3: `egui_right_panel.rs` 与 `viewer-texture-inspector.sh` 完成结构拆分并保持行为不变。
  - SC-4: `test_tier_required` 回归全部通过，`test_tier_full` 至少完成 `viewer-release-full-coverage.sh --quick`。
  - SC-5: 旧 Bevy UI 路径移除，RightPanelLayoutState 等布局状态统一为 egui 单一路径并完成 CI required + Web 闭环截图验证。

## 2. User Experience & Functionality
- User Personas:
  - Viewer 运行时开发者：关注主题应用一致性与改动可维护性。
  - 发布工程师：关注资产质量门禁是否能提前阻断风险。
  - 视觉验收人员：关注抓帧与发布链路是否稳定复现。
- User Scenarios & Frequency:
  - 主题包更新时执行预算校验：每次资产升级必跑。
  - Runtime 主题热切换回归：每次材质/贴图逻辑改动后必跑。
  - 发布前完整验收：每个候选版本至少 1 次。
- User Stories:
  - PRD-VAPUI-001: As a 发布工程师, I want budget upper bounds in theme validation, so that oversized assets are blocked before release.
  - PRD-VAPUI-002: As a Viewer 运行时开发者, I want unified theme apply/config parsing paths, so that init and hot-reload behavior stay consistent.
  - PRD-VAPUI-003: As a 维护者, I want oversized files split into focused modules, so that future changes are safer and reviewable.
  - PRD-VAPUI-004: As a 维护者, I want legacy Bevy UI removed and RightPanelLayoutState unified in egui, so that UI state does not drift between paths.
- Critical User Flows:
  1. Flow-VAPUI-001: `更新主题资产 -> 运行 validate-viewer-theme-pack -> 若超限则阻断并输出具体阈值差异`
  2. Flow-VAPUI-002: `启动 Viewer -> 初始化主题 -> 运行 runtime apply -> 同步校验初始化与热更新使用同一路径`
  3. Flow-VAPUI-003: `修改 UI/Shell 模块 -> 跑语法与单测 -> 执行 release quick gate -> 归档结果`
  4. Flow-VAPUI-004: `启动 Viewer -> 右侧面板布局状态由 egui 资源驱动 -> legacy Bevy UI 路径不再参与`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 主题预算上限门禁 | `max_texture_size`、`max_total_texture_bytes`、`max_vertices` | `validate-viewer-theme-pack.py` 校验失败即退出非零 | `pending -> passed/failed` | 先目录完整性，再阈值检查 | 仅维护者可调整 profile 阈值 |
| Runtime 主题应用单路径 | 共享 `apply`/`parse` 函数接口 | 初始化与热更新统一调用共享函数 | `initialized -> runtime_applied` | 以配置快照为唯一输入源 | 内部模块可调用，对外接口保持不变 |
| UI/Shell 模块化拆分 | `egui_right_panel_*` 子模块、`viewer-texture-inspector-lib.sh` | 主入口文件通过 `mod/source` 复用子模块逻辑 | `legacy_single_file -> modularized` | 拆分后行为保持字面一致 | 仅代码维护者可编辑拆分边界 |
| Legacy UI 清理与布局状态迁移 | `RightPanelLayoutState`/`right_panel_layout_state.rs` | egui 右侧面板使用单一路径状态，旧 Bevy UI 移除 | `dual_path -> egui_only` | 保持默认布局不变 | 仅维护者可调整默认布局 |
- Acceptance Criteria:
  - AC-1: 资产校验脚本对 `industrial_v1/v2/v3` 均支持上限预算校验并保留现有下限校验。
  - AC-2: 初始化与 runtime apply 不再维护重复材质拼装逻辑。
  - AC-3: external mesh/material/texture 解析逻辑不再在多处重复实现。
  - AC-4: `egui_right_panel.rs` 与 `viewer-texture-inspector.sh` 行数分别回落到 1200 行以内或主文件仅保留入口转发。
  - AC-5: 必跑回归命令通过并在 devlog 留痕。
  - AC-6: `panel_layout.rs`/`panel_scroll.rs`/`setup_ui`/`update_ui` 移除，RightPanelLayoutState 迁移到 egui 侧并完成 CI required + Web 闭环截图验证。
- Non-Goals:
  - 不改变主题包资产内容风格与命名规范。
  - 不重构 Viewer 功能行为与交互设计。
  - 不引入新的发布脚本入口。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: N/A（本专题不新增 AI 侧工具链要求）。
- Evaluation Strategy: N/A。

## 4. Technical Specifications
- Architecture Overview: 以“校验门禁前移 + 运行时逻辑收敛 + 文件结构治理”三层硬化现有 Viewer 资产与 UI 体系；外部接口保持稳定，内部实现去重。
- Integration Points:
  - `scripts/validate-viewer-theme-pack.py`
  - `scripts/viewer-release-full-coverage.sh`
  - `scripts/viewer-texture-inspector.sh`
  - `crates/agent_world_viewer/src/main_ui_runtime.rs`
  - `crates/agent_world_viewer/src/right_panel_layout_state.rs`
  - `crates/agent_world_viewer/src/panel_layout.rs`（legacy 清理）
  - `crates/agent_world_viewer/src/panel_scroll.rs`（legacy 清理）
  - `crates/agent_world_viewer/src/theme_runtime.rs`
  - `crates/agent_world_viewer/src/viewer_3d_config.rs`
  - `crates/agent_world_viewer/src/egui_right_panel.rs`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 主题目录缺失或 profile 不匹配时必须返回结构化错误并退出非零。
  - 单通道纹理字节异常膨胀（压缩失效）时必须触发上限门禁。
- 运行时主题热更新失败时不得破坏现有场景状态，应回退并输出失败原因。
- Shell 拆分后若 `source` 文件缺失，入口脚本应快速失败并给出缺失路径。
- Web 端闭环 `console error=0` 门禁需要静态根提供 `favicon.ico`，避免 404 噪音导致 gate 失败。
- 解析逻辑收敛后必须覆盖空值、非法色值、非法路径输入。
- Non-Functional Requirements:
  - NFR-VAPUI-1: `validate-viewer-theme-pack.py` 单包校验 `p95 <= 2s`（本地开发机）。
  - NFR-VAPUI-2: 重构后 `theme_runtime` 与 `viewer_3d_config` 定向测试全部通过。
  - NFR-VAPUI-3: 拆分后主入口文件复杂度下降且行为回归保持 100%。
  - NFR-VAPUI-4: 脚本拆分不得引入额外外部依赖。
- Security & Privacy:
  - 维持现有本地脚本执行权限模型，不新增远程执行面。
  - 输出日志不得泄露敏感路径外的运行时凭据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP: TASK-WORLD_SIMULATOR-041（预算上限门禁）
  - v1.1: TASK-WORLD_SIMULATOR-042（Runtime 去重）
  - v2.0: TASK-WORLD_SIMULATOR-043（文件模块化 + full gate 回归）
  - v2.1: TASK-WORLD_SIMULATOR-044（Legacy UI 清理 + 右侧面板布局状态迁移 + CI/Web 闭环）
- Technical Risks:
  - 风险-1: 去重过程可能改变细节行为。
  - 风险-2: Shell 拆分可能破坏历史执行环境。
  - 风险-3: 上限门禁阈值过严导致误阻断。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-VAPUI-001 | TASK-WORLD_SIMULATOR-041 | `test_tier_required` | `python3 scripts/validate-viewer-theme-pack.py --theme-dir crates/agent_world_viewer/assets/themes/industrial_v1 --profile v1` + 同 v2/v3 | 主题资产门禁与发布阻断准确性 |
| PRD-VAPUI-002 | TASK-WORLD_SIMULATOR-042 | `test_tier_required` | `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer theme_runtime` + `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer viewer_3d_config` | Viewer 初始化/热更新一致性 |
| PRD-VAPUI-003 | TASK-WORLD_SIMULATOR-043 | `test_tier_required` + `test_tier_full` | `bash -n scripts/viewer-texture-inspector.sh` + `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer egui_right_panel_tests` + `./scripts/viewer-release-full-coverage.sh --quick` | UI 模块维护性、发布回归稳定性 |
| PRD-VAPUI-004 | TASK-WORLD_SIMULATOR-044 | `test_tier_required` + `test_tier_full` | `./scripts/ci-tests.sh required` + `./scripts/viewer-release-qa-loop.sh` + `./scripts/viewer-release-art-baseline.sh` | Legacy UI 清理与 Web 端闭环验证 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-VAPUI-001 | 在现有校验脚本直接补充上限预算门禁 | 新增独立预算检查脚本 | 复用现有入口可避免发布流程分叉。 |
| DEC-VAPUI-002 | 初始化与热更新统一复用函数 | 继续双实现并靠测试兜底 | 双实现长期会漂移，维护成本不可控。 |
| DEC-VAPUI-003 | 主文件保留入口并拆到子模块文件 | 单文件内继续堆叠逻辑 | 需要满足行数治理并降低审查复杂度。 |
| DEC-VAPUI-004 | 移除 legacy Bevy UI 路径并统一 egui 布局状态 | 同时保留双 UI 路径 | 双路径增加漂移与测试成本，不符合治理目标。 |
