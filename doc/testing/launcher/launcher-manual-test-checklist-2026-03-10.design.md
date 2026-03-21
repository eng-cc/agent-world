# oasis7：启动器人工测试清单（2026-03-10）设计

- 对应需求文档: `doc/testing/launcher/launcher-manual-test-checklist-2026-03-10.prd.md`
- 对应项目管理文档: `doc/testing/launcher/launcher-manual-test-checklist-2026-03-10.project.md`

## 1. 设计定位
定义启动器人工测试清单的执行设计，统一 `P0/P1/P2` 分层、细粒度缺陷拦截规则、Web UI 闭环控制方式与证据回写口径。

## 2. 设计结构
- 测试分层：按 `P0 必测 / P1 应测 / P2 选测` 组织执行顺序与发布门槛。
- 方法论层：将 `Explorer / Transfer` 从粗粒度 smoke 拆为子能力矩阵，要求先造数、再查数、再断言字段。
- 执行链路层：Web 控制面优先使用 GUI Agent 驱动动作，再由浏览器页面验证状态与展示结果。
- 证据与归因层：统一截图、原始返回、日志、失败归因入口与发布 verdict 映射。

## 3. 关键接口 / 入口
- `testing-manual.md` 的 S6 Web UI 闭环 smoke 套件入口
- `crates/oasis7/src/bin/oasis7_web_launcher.rs`
- `crates/oasis7/src/bin/oasis7_web_launcher/gui_agent_api.rs`
- `doc/testing/launcher/launcher-manual-test-checklist-2026-03-10.prd.md`

## 4. 约束与边界
- 本专题聚焦测试设计与执行口径，不替代启动器实现代码或自动化单测。
- `Explorer / Transfer` 不允许只验证“页面能打开”或“接口返回 200”。
- 若存在真实数据依赖，必须显式声明数据前置场景与期望结果分级。
- 发布结论与细粒度执行结果冲突时，采用更保守结论。

## 5. 设计演进计划
- 先建立启动器人工测试清单基线与追溯入口。
- 再补 `Explorer / Transfer` 子能力矩阵、双证据与归因规则。
- 后续按逃逸缺陷继续补失败签名与代码入口映射。
