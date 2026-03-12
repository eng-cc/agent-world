# Agent World：启动器 bundle-first 试玩入口收敛（2026-03-12）（项目管理）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] LBFP-1 (PRD-TESTING-LAUNCHER-BUNDLE-001): 建立专题 PRD / design / project，并回写 testing 索引。
- [x] LBFP-2 (PRD-TESTING-LAUNCHER-BUNDLE-002): 为 `scripts/run-game-test.sh` 增加 `--bundle-dir` 并保持源码模式兼容。
- [x] LBFP-3 (PRD-TESTING-LAUNCHER-BUNDLE-001): 同步 `testing-manual.md`、启动器人工测试清单、README 与帮助文本，明确 bundle-first 口径。
- [x] LBFP-4 (PRD-TESTING-LAUNCHER-BUNDLE-002): 完成 bundle 构建、headed/headless 对照验证、SwiftShader 阻断证据归档与 devlog 回写。
- [x] LBFP-5 (PRD-TESTING-LAUNCHER-BUNDLE-002): 为 `run-game-test-ab.sh` 增加 `headless + SwiftShader` 环境快失败与 `browser_env.json` 证据落盘，避免误把环境阻断记成 fresh Web 回归。

## 依赖
- `doc/testing/launcher/launcher-bundle-first-playtest-entry-2026-03-12.prd.md`
- `scripts/run-game-test.sh`
- `scripts/run-game-test-ab.sh`
- `scripts/build-game-launcher-bundle.sh`
- `testing-manual.md`
- `doc/testing/launcher/launcher-manual-test-checklist-2026-03-10.prd.md`
- `doc/testing/project.md`
- `doc/testing/prd.index.md`
- `doc/devlog/2026-03-12.md`

## 状态
- 更新日期：2026-03-12
- 当前阶段：已完成（入口、文档和 headless 环境 guardrail 已收敛）
- 阻塞项：无新的代码阻塞；当前已确认此前阻断主要由 `headless + SwiftShader` 环境导致。
- 下一步：后续如需真正退役源码模式，再单开专题迁移 `run-game-test-ab.sh` 与其他下游脚本；当前先保留源码模式作为开发复现入口。
