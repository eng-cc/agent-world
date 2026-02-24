# Viewer Web Test API `step` 控制补齐项目管理文档（2026-02-24）

## 任务拆解
- [x] T0：建档设计文档与项目管理文档。
- [x] T1：在 `web_test_api` 补齐 `sendControl("step")` 解析与 `count` 规则。
- [x] T2：执行定向回归（`cargo check -p agent_world_viewer` + Playwright Web 闭环）。
- [x] T3：回写文档状态与 devlog，完成收口。

## 依赖
- T1 依赖 T0（先冻结接口语义）。
- T2 依赖 T1（修复后才能验证）。
- T3 依赖 T2（回归通过后收口）。

## 状态
- 当前阶段：已完成（T0~T3）。
- 阻塞项：无。
- 最近更新：2026-02-24。
