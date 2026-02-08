# Agent World：3D 渲染物理准确性（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-simulator/rendering-physical-accuracy.md`）
- [x] 输出项目管理文档（本文件）
- [x] RPA-1：viewer 物理渲染配置结构落地（单位、光照、曝光、精度）
- [x] RPA-2：尺寸映射链路落地（cm→m、Agent/Location 尺寸统一）
- [x] RPA-3：大场景精度方案落地（floating origin + 相机裁剪）
- [x] RPA-4：材质物理参数库接入（silicate/metal/ice/carbon/composite）
- [x] RPA-5：小行星带光照模型接入（`1/d²` 辐照度 + 曝光回归）
- [x] RPA-6：辐射/热状态解释性可视化（单位换算 + 阈值颜色映射）
- [x] RPA-7：补齐回归测试（尺寸、距离、光照单调性、热态可读）
- [x] RPA-8：截图闭环验证与文档收口
- [x] RPA-9：世界摘要物理口径可观测性（Render Physical 状态块）

## 依赖
- `WorldSnapshot` / `WorldEvent` / `WorldConfig`（`crates/agent_world`）
- `agent_world_viewer` 3D 渲染与 UI 管线（`crates/agent_world_viewer`）
- 现有物理参数约束（`doc/world-simulator.md`，C9 参数表）
- 截图闭环脚本（`scripts/capture-viewer-frame.sh`）

## 状态
- 当前阶段：RPA-9 已完成，物理准确性分册任务全部收口。
- 下一优先项：无（本分册 RPA-1~RPA-9 全部完成）。
- 最近更新：RPA-9 完成（世界摘要新增物理渲染状态块与关键口径输出，2026-02-08）。
