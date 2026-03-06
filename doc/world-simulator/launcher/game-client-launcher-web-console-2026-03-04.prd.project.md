# 客户端启动器 Web 控制台（2026-03-04）项目管理文档

审计轮次: 3

- 对应设计文档: doc/world-simulator/launcher/game-client-launcher-web-console-2026-03-04.prd.md

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-010) [test_tier_required]: 建档并冻结 Web 控制台需求、验收标准与非目标。
- [x] T1 (PRD-WORLD_SIMULATOR-010) [test_tier_required]: 新增 `world_web_launcher` 二进制（HTTP 控制台 + 启停 API + 状态/日志接口）。
- [x] T2 (PRD-WORLD_SIMULATOR-010) [test_tier_required]: 将 Web 控制台入口接入发行打包脚本并生成运行脚本。
- [x] T3 (PRD-WORLD_SIMULATOR-010) [test_tier_required]: 补齐单元测试、模块 PRD 追溯与当日 devlog。

## 依赖
- `doc/world-simulator/prd.md`
- `doc/world-simulator/prd.project.md`
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `scripts/build-game-launcher-bundle.sh`
- `testing-manual.md`

## 状态
- 当前阶段: completed
- 当前任务: 无
- 备注: 面向“无图形会话服务器 + 浏览器远程控制”场景，T0~T3 已完成。
