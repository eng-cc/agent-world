# Viewer 3D 精致化与性能优化（项目管理文档）

## 任务拆解
- [x] VPP0：完成现状调研与问题归纳（渲染层次、标签密度、线框实体开销、指标缺失）
- [x] VPP1：输出设计文档（`viewer-3d-polish-performance.md`）
- [x] VPP2：输出项目管理文档（本文件）
- [x] VPP3：建立性能基线采样（triad/llm 场景，记录 avg/p95）
- [x] VPP4：设计并接入渲染质量档位（Debug/Balanced/High）
- [x] VPP5：升级选中反馈（缩放 + 视觉强调）并补齐测试
- [ ] VPP6：标签显示策略优化（距离衰减、同屏上限、遮挡降权）
- [ ] VPP7：覆盖层与网格线渲染优化（批处理/LOD/节流）
- [ ] VPP8：渲染性能摘要进入右侧总览（帧时间、对象数、标签数）
- [ ] VPP9：回归验证（2D/3D、时间轴联动、右侧面板不退化）
- [x] VPP10：截图闭环与文档收口（设计/项目/日志同步）

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `crates/agent_world_viewer/src/world_overlay.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `scripts/capture-viewer-frame.sh`
- `doc/world-simulator/viewer-3d-polish-performance.md`

## 状态
- 当前阶段：闭环验证完成首轮视觉优化（VPP0~VPP4、VPP10 已完成）
- 下一阶段：继续推进标签/覆盖层 LOD 与渲染性能摘要（VPP5~VPP9）
- 最近更新：完成 llm 场景闭环截图并落地首轮材质/光照优化（2026-02-10）

## 里程碑检查点
- Checkpoint A：完成基线采样并固化预算（VPP3）
- Checkpoint B：完成视觉精致化首轮交付（VPP4~VPP6）
- Checkpoint C：完成性能优化首轮交付（VPP7~VPP8）
- Checkpoint D：完成回归、截图闭环与文档收口（VPP9~VPP10）
