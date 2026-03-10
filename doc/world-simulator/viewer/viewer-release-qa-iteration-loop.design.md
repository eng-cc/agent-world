# Viewer 发布 QA 迭代闭环设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-release-qa-iteration-loop.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-release-qa-iteration-loop.project.md`

## 1. 设计定位
定义“发现套件问题 -> 修复套件 -> 重跑验证”的发行 QA 迭代闭环，把 Web 语义断言、视觉门禁和证据归档收敛成一条可重复脚本。

## 2. 设计结构
- 脚本入口层：`viewer-release-qa-loop.sh` 统一场景、端口和输出目录参数。
- 语义断言层：围绕 `__AW_TEST__`、连接态、tick 推进和基本控制动作做硬校验。
- 多缩放视觉层：near/mid/far 截图同时检查像素质量、相机语义和视觉差异度。
- 报告归档层：输出截图、console log 和 Markdown 汇总报告。

## 3. 关键接口 / 入口
- `scripts/viewer-release-qa-loop.sh`
- `window.__AW_TEST__`
- `cameraMode` / `cameraRadius` / `cameraOrthoScale`
- `output/playwright/viewer/release-qa-summary-*.md`

## 4. 约束与边界
- 它是发布验收补充层，不替代 `test_tier_required/full`。
- 本轮以本地/agent 脚本为主，不扩成常驻 CI E2E。
- 视觉 gate 以 Playwright 截图为准，不再依赖不稳定的 canvas 读回。
- 脚本必须显式管理端口与进程，减少环境偶发误判。

## 5. 设计演进计划
- 先跑基线并识别套件缺口。
- 再实现一键化 QA 脚本和语义断言。
- 最后升级 zoom 视觉门禁并输出 PASS 报告。
