# 启动器反馈窗口化设计（2026-03-02）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-feedback-window-2026-03-02.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-feedback-window-2026-03-02.project.md`

## 1. 设计定位
定义启动器反馈能力从主面板内嵌表单迁移到“按钮入口 + 弹窗窗口”的交互结构，以降低主面板密度并保持反馈提交能力不退化。

## 2. 设计结构
- 入口触发层：主面板新增 `反馈 / Feedback` 按钮。
- 窗口承载层：反馈表单迁入独立窗口，字段与原行为一致。
- 提交复用层：继续复用现有 `submit_feedback_with_fallback` 流程，不改协议。
- 主界面收敛层：移除内嵌反馈区域，保留简洁高频操作布局。

## 3. 关键接口 / 入口
- `反馈 / Feedback` 按钮
- 反馈窗口状态
- `kind/title/description/output_dir`
- `submit_feedback_with_fallback`

## 4. 约束与边界
- 窗口化只改变交互承载形式，不改变反馈数据结构与提交协议。
- 主面板移除内嵌区域后，反馈入口仍需易发现、可连续使用。
- 本阶段不新增反馈历史查询或附件能力。
- 回归需覆盖编译与窗口开关行为，不允许主路径退化。

## 5. 设计演进计划
- 先加入反馈按钮与窗口状态。
- 再把原表单完整迁入窗口。
- 最后移除主面板旧入口并完成回归验证。
