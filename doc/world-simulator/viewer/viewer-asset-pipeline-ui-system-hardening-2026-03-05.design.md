# Viewer 资产管线与 UI 体系硬化设计（2026-03-05）

- 对应需求文档: `doc/world-simulator/viewer/viewer-asset-pipeline-ui-system-hardening-2026-03-05.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-asset-pipeline-ui-system-hardening-2026-03-05.project.md`

## 1. 设计定位
定义 Viewer 在主题资产门禁、3D 场景初始化去重、超长文件拆分和 legacy UI 清理上的系统性硬化方案，使资产管线、右侧面板与运行时 UI 逐步收敛到单一路径。

## 2. 设计结构
- 资产门禁层：在 `validate-viewer-theme-pack.py` 中加入 profile 级预算上限。
- Runtime 收敛层：统一 `setup_3d_scene` 与 `apply_theme_to_assets_and_scene` 的重复逻辑。
- 文件治理层：拆分超长 UI/Shell 文件，降低复杂度并满足行数约束。
- UI 单路径层：移除 legacy Bevy UI 与旧测试入口，统一到 egui 布局状态。

## 3. 关键接口 / 入口
- `validate-viewer-theme-pack.py`
- `theme_runtime.rs`
- `viewer_3d_config.rs`
- `egui_right_panel.rs`
- `scripts/viewer-texture-inspector.sh`

## 4. 约束与边界
- 预算门禁要直接复用现有发布入口，不能再分叉脚本。
- 初始化与热更新逻辑必须共源，避免长期漂移。
- legacy UI 清理后测试也要同步迁移，不能保留误导性双路径。
- 本轮不引入新的外部依赖或远程执行面。

## 5. 设计演进计划
- 先补资产预算门禁与 Runtime 去重。
- 再拆分超长文件并清理 legacy UI 路径。
- 最后以 required/full 回归和 Web 闭环验证完成硬化收口。
