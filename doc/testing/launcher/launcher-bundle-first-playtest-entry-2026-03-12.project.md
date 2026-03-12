# Agent World：启动器 bundle-first 试玩入口收敛（2026-03-12）（项目管理）

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] LBFP-1 (PRD-TESTING-LAUNCHER-BUNDLE-001): 建立专题 PRD / design / project，并回写 testing 索引。
- [x] LBFP-2 (PRD-TESTING-LAUNCHER-BUNDLE-002): 为 `scripts/run-game-test.sh` 增加 `--bundle-dir` 并保持源码模式兼容。
- [x] LBFP-3 (PRD-TESTING-LAUNCHER-BUNDLE-001): 同步 `testing-manual.md`、启动器人工测试清单、README 与帮助文本，明确 bundle-first 口径。
- [x] LBFP-4 (PRD-TESTING-LAUNCHER-BUNDLE-002): 完成 bundle 构建、bundle 模式 A/B 闭环验证、阻断证据归档与 devlog 回写。

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
- 当前阶段：已完成（入口与文档已收敛，验证已留阻断证据）
- 阻塞项：fresh bundle / fresh source Web 闭环当前停在 `connectionStatus=connecting`，需由 `viewer_engineer` 继续排查 trunk/fresh Web 构建链路。
- 下一步：后续如需真正退役源码模式，再单开专题迁移 `run-game-test-ab.sh` 与其他下游脚本；当前先保留源码模式作为开发复现入口。
