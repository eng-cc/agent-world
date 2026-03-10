# Viewer 观察导向可视优化设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-observability-visual-optimization.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-observability-visual-optimization.project.md`

## 1. 设计定位
定义右侧 EGUI 面板从“复制导向”转为“观察导向”的优化方案：通过总览指标、分组样式和脚本预校验，让用户更少依赖外部复制分析也能看清关键运行状态。

## 2. 设计结构
- 总览布局层：默认展示连接、健康、tick、事件数、选择态等关键指标。
- 分组观察层：降低折叠深度、统一分组样式，让明细更适合直接比对观察。
- 文案迁移层：把 copy panel 相关文案改为观察/明细导向。
- 脚本预校验层：`capture-viewer-frame.sh` 先做场景白名单与别名规范化。

## 3. 关键接口 / 入口
- `egui_right_panel.rs`
- `copyable_text.rs`
- `i18n.rs`
- `scripts/capture-viewer-frame.sh`

## 4. 约束与边界
- 不改 3D 渲染算法和协议模型，也不新增外部导出能力。
- 观察导向不等于全部展开，仍要靠总览 + 分组控制信息过载。
- 文案与视觉回归需要同步更新测试基线，避免优化引入漂移。
- 脚本白名单和场景实际列表需要保持可维护同步。

## 5. 设计演进计划
- 先补脚本预校验。
- 再做面板总览和观察导向布局。
- 最后接视觉回归快照与 EGUI UI 测试收口。
