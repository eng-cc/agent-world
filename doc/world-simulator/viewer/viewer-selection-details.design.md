# Viewer 选中详情统一设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-selection-details.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-selection-details.project.md`

## 1. 设计定位
定义 Viewer 各类选中对象的详情呈现和联动方案：让 location、agent、asset、fragment、module visual 等对象都走同一条可扩展的详情渲染路径。

## 2. 设计结构
- 选择类型层：统一枚举和上下文结构，区分不同实体的详情分支。
- 详情渲染层：右侧面板按选择类型输出结构化信息块。
- 事件联动层：事件列表点击和对象定位共享同一选择上下文。
- 可扩展接入层：新增实体类型时优先补选择详情分支，而不是旁路专用 UI。

## 3. 关键接口 / 入口
- `ViewerSelection`
- 右侧详情面板分支
- 事件列表对象联动
- fragment/module visual 详情入口

## 4. 约束与边界
- 不改 snapshot 协议，只重排 Viewer 侧详情表达。
- 不同对象详情字段可不同，但布局结构要稳定可预测。
- 事件联动和手动点击必须落到同一选择模型，避免状态分叉。
- 新增实体类型要复用统一详情框架，不再复制粘贴独立面板。

## 5. 设计演进计划
- 先统一选择类型与详情接口。
- 再补主要对象分支和事件联动。
- 最后通过 UI 测试收口详情一致性。
