# Viewer 视觉外发就绪硬化设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-visual-release-readiness-hardening-2026-03-01.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-visual-release-readiness-hardening-2026-03-01.project.md`

## 1. 设计定位
定义 Viewer 从“技术演示可看”推进到“可对外展示可控”的视觉硬化方案：强化语义门禁、发布向 UI profile、主题资产质量门槛和外发样张基线。

## 2. 设计结构
- 门禁强化层：texture inspector 与相关脚本默认 strict，空画面、错误选中、错误场景一律 fail。
- 发布 profile 层：新增对外展示的 UI preset，降低调试模块干扰。
- 主题资产层：以 `industrial_v3` 为候选提升 mesh 和贴图质量，并收紧校验阈值。
- 样张基线层：固定 wide/medium/closeup 外发样张与证据路径。

## 3. 关键接口 / 入口
- `scripts/viewer-texture-inspector.sh`
- 发布向 UI profile / preset
- `industrial_v2` / `industrial_v3` 主题资产
- 外发样张基线脚本

## 4. 约束与边界
- 本轮强调展示质量与假通过消除，不重写 Viewer 核心渲染架构。
- 发布 profile 必须和调试 profile 明确分层，不能互相污染。
- 外发样张需要可复跑、可留痕，而不是一次性人工截图。
- 语义门禁不能只看脚本退出码，必须检查空选中和错误选中。

## 5. 设计演进计划
- 先强化 strict gate 和 direct_entity 构图。
- 再补发布 profile 与资产质量门槛。
- 最后固化 wide/medium/closeup 样张基线。
