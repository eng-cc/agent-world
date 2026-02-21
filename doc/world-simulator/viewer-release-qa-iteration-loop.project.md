# Viewer 发行验收测试迭代闭环（项目管理）

## 任务拆解
- [x] VRQ-0 文档建档：设计文档 + 项目管理文档
- [x] VRQ-1 基线执行：跑现有套件并输出首次完成度/视觉结论
- [x] VRQ-2 套件优化：实现一键化 Web 语义闭环验收脚本
- [x] VRQ-3 复验收口：重跑并输出报告，更新手册与状态
- [x] VRQ-4 缺口修复：稳定语义 gate（连接抖动收敛）并消除字体资产错误日志
- [x] VRQ-5 缩放验收升级：三档截图像素指标 + 相机半径语义断言 + zoom 差异度门禁
- [ ] VRQ-6 发行缺口收敛：修复“zoom 生效但画面近远差异过小”问题并达成全 PASS

## 依赖
- `testing-manual.md`
- `scripts/viewer-visual-baseline.sh`
- `scripts/run-viewer-web.sh`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `/Users/scc/.codex/skills/playwright/scripts/playwright_cli.sh`

## 状态
- 当前阶段：VRQ-6 进行中
- 下一步：定位并修复 near/far zoom 视觉差异不足（当前脚本已稳定复现 `zoom visual delta too small`）
- 最近更新：2026-02-21（完成 VRQ-4/VRQ-5，新增 zoom 视觉差异硬门禁）
