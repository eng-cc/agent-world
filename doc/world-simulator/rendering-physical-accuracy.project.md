# Agent World：3D 渲染物理准确性（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-simulator/rendering-physical-accuracy.md`）
- [x] 输出项目管理文档（本文件）
- [x] RPA-1：viewer 物理渲染配置结构落地（单位、光照、曝光、精度）
- [x] RPA-2：尺寸映射链路落地（cm→m、Agent/Location 尺寸统一）
- [x] RPA-3：大场景精度方案落地（floating origin + 相机裁剪）
- [ ] RPA-4：材质物理参数库接入（silicate/metal/ice/carbon/composite）
- [ ] RPA-5：小行星带光照模型接入（`1/d²` 辐照度 + 曝光回归）
- [ ] RPA-6：辐射/热状态解释性可视化（单位换算 + 阈值颜色映射）
- [ ] RPA-7：补齐回归测试（尺寸、距离、光照单调性、热态可读）
- [ ] RPA-8：截图闭环验证与文档收口

## 依赖
- `WorldSnapshot` / `WorldEvent` / `WorldConfig`（`crates/agent_world`）
- `agent_world_viewer` 3D 渲染与 UI 管线（`crates/agent_world_viewer`）
- 现有物理参数约束（`doc/world-simulator.md`，C9 参数表）
- 截图闭环脚本（`scripts/capture-viewer-frame.sh`）

## 状态
- 当前阶段：RPA-3 已完成，进入 RPA-4（材质物理参数库）准备阶段。
- 下一优先项：RPA-4（按材质类型接入密度/反照率/粗糙度等参数库）。
- 最近更新：RPA-3 完成（引入 scene root + floating origin 重锚、相机重锚同步与回归测试，2026-02-07）。
