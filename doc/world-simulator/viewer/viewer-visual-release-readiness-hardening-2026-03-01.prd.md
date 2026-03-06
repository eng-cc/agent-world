# Viewer 视觉外发就绪硬化（2026-03-01）

审计轮次: 3

- 对应项目管理文档: doc/world-simulator/viewer/viewer-visual-release-readiness-hardening-2026-03-01.prd.project.md

## 1. Executive Summary
- 把当前 Viewer 从“技术演示可看”推进到“可对外展示可控”的视觉验收状态。
- 消除视觉验收链路中的假通过：空画面、未选中目标、错误场景仍被判定通过。
- 提供发布向 UI 展示配置，降低调试模块对对外展示画面的干扰。
- 交付一组可复跑、可留痕的外发样张基线（wide/medium/closeup）。

## 2. User Experience & Functionality

### In Scope
- `scripts/viewer-texture-inspector.sh`：
  - 默认主题预设切到 `industrial_v2`；
  - 语义门禁强化（默认 strict，空选中/错误选中 fail）；
  - direct_entity 构图与聚焦修复（agent/asset/power_*）。
- 新增发布向 UI profile（环境变量 preset + 运行脚本接入）。
- 工业主题资产升级一版（以 `industrial_v3` 为发行候选）：
  - mesh 细节提升；
  - 贴图生成策略去“同质条纹”。
- 更新主题资产校验脚本阈值与 profile，避免“低质资产也通过”。
- 新增外发样张脚本与基线输出目录规范。
- 同步更新 `testing-manual.md` 与相关项目状态文档。

### Out of Scope
- 不引入角色骨骼动画与复杂 VFX 管线。
- 不改 world 协议与玩法语义。
- 不接入外部 DCC 自动导入流水线（保持仓内可复现生成）。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- 新增主题包目录：
  - `crates/agent_world_viewer/assets/themes/industrial_v3/`
- 新增/调整脚本接口：
  - `scripts/viewer-texture-inspector.sh` 默认 preset 与 gate 行为调整；
  - `scripts/viewer-release-art-baseline.sh`（新）：生成外发样张。
- 新增发布 UI profile：
  - `scripts/viewer-release-ui-profile.env`（新）。
- 主题校验 profile 扩展：
  - `scripts/validate-viewer-theme-pack.py` 增加 `v3`。

## 5. Risks & Roadmap
- VVRH-0：设计文档 + 项目管理文档建档。
- VVRH-1：视觉门禁硬化（strict gate + 构图/选中修复）。
- VVRH-2：发布向 UI profile 落地并接入抓帧脚本。
- VVRH-3：`industrial_v3` 资产与校验阈值落地。
- VVRH-4：外发样张脚本 + 基线产物 + 手册收口。

### Technical Risks
- 门禁收紧后历史命令大量失败：
  - 缓解：保留显式 `--semantic-gate-mode` 兜底开关，默认收紧并给出失败原因。
- 资产升级导致渲染性能回退：
  - 缓解：限制 v3 贴图尺寸与顶点预算，保留 v2 快速回滚路径。
- UI profile 与玩家引导冲突：
  - 缓解：仅用于外发抓帧/演示命令，不改默认开发态行为。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
