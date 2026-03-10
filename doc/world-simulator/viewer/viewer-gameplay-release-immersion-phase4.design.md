# Viewer 发行体验沉浸改造 Phase 4 设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase4.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-gameplay-release-immersion-phase4.project.md`

## 1. 设计定位
定义第四阶段的场景活化与交互动效方案：通过低频、轻量、边缘锚定的氛围层与交互反馈，让 Player 模式更像游戏而非纯工具界面。

## 2. 设计结构
- 场景活化层：在不改核心场景资产的前提下增加低噪声氛围提示与状态强调。
- 交互动效层：把关键操作结果转成轻量动效或提示，提升舒适度与反馈即时性。
- 布局约束层：所有新增视觉层都保持边缘布局、可自动消退，不遮挡主视图。
- 模式保护层：Director 模式继续保留调试优先布局，Player 才启用沉浸增强。

## 3. 关键接口 / 入口
- Player 体验渲染层
- 交互动效与反馈状态入口
- `ViewerExperienceMode`
- 相关 viewer UI 测试

## 4. 约束与边界
- 不引入重资产美术升级，也不扩展外部 UI 框架。
- 动效必须低频且可读，不能演变为视觉噪音。
- 活化层只增强已有世界反馈，不制造与状态脱节的假信号。
- 所有视觉增强都需要可在 Web 闭环中验证。

## 5. 设计演进计划
- 先冻结活化/动效的低噪声原则。
- 再逐项补场景反馈与交互动效。
- 最后通过回归与截图产物收口沉浸增强。
