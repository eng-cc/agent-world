# 启动器 Web 设置与反馈闭环对齐设计（2026-03-06）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-web-settings-feedback-parity-2026-03-06.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-web-settings-feedback-parity-2026-03-06.project.md`

## 1. 设计定位
定义 web 启动器在设置编辑、反馈入口与结果提示上的闭环补齐方案，使其与 native 启动器保持一致的可操作性与反馈语义。

## 2. 设计结构
- 设置编辑层：统一字段展示、修改与保存反馈。
- 反馈提交流层：web 侧补齐反馈入口、状态展示与失败重试路径。
- 状态反馈层：成功、校验失败、上游失败沿用结构化错误语义。
- 对齐治理层：native/web 在设置与反馈上的字段、按钮、错误文案保持同源。

## 3. 关键接口 / 入口
- web 设置 UI 与保存动作
- 反馈窗口 / 提交状态机
- `oasis7_web_launcher` 对应控制面接口
- native/web 共用状态与错误语义

## 4. 约束与边界
- web 不得退化为只读配置展示，必须具备与 native 对等的关键闭环。
- 反馈失败后必须允许继续重试，不锁死界面。
- 跨端文案和字段语义需保持一致，避免测试分叉。
- 本阶段不引入新的反馈业务类型。

## 5. 设计演进计划
- 先补齐设置与反馈所需的控制面接口。
- 再完成 web UI 对等入口与状态展示。
- 最后通过跨端回归校验 parity 收口。
