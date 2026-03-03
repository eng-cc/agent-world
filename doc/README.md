# 工程文档总入口（模块设计）

更新时间：2026-03-03

本文件用于导航各模块设计文档与执行文档。所有新需求与在研需求均以模块 PRD 为唯一入口。

## 快速阅读路径（推荐）
1. 先读本文件，获取导航。
2. 读 `doc/core/prd.md`，获取项目全局设计总览（模块地图、关键链路、关键分册）。
3. 进入目标模块 `doc/<module>/prd.md`，确认问题定义、方案、验收标准与技术边界。
4. 继续读 `doc/<module>/prd.project.md`，确认任务拆解、PRD-ID 映射、依赖与状态。
5. 按需下钻模块子文档（`doc/<module>/**/*.md`，含 `archive/`）。
6. 对照系统测试策略：`testing-manual.md` 与 `doc/testing/prd.md`。
7. 回溯过程记录：`doc/devlog/YYYY-MM-DD.md`。

## 根目录入口说明
- `doc/world-runtime.md` 与 `doc/world-runtime.project.md`：历史总览入口（legacy），当前模块主入口以 `doc/world-runtime/prd.md` 与 `doc/world-runtime/prd.project.md` 为准。
- `doc/world-simulator.md` 与 `doc/world-simulator.project.md`：历史总览入口（legacy），当前模块主入口以 `doc/world-simulator/prd.md` 与 `doc/world-simulator/prd.project.md` 为准。
- 以下文件为兼容跳转入口（legacy redirect），正文已迁移到模块目录：
  - `doc/viewer-manual.md` -> `doc/world-simulator/viewer/viewer-manual.md`
  - `doc/game-test.md` -> `doc/playability_test_result/game-test.md`
  - `doc/game-test.project.md` -> `doc/playability_test_result/game-test.project.md`
  - `doc/playability_test_card.md` -> `doc/playability_test_result/playability_test_card.md`
  - `doc/playability_test_manual.md` -> `doc/playability_test_result/playability_test_manual.md`

## 模块入口矩阵
| 模块 | PRD 主文档 | 项目管理文档 | 设计关注点 |
| --- | --- | --- | --- |
| core | `doc/core/prd.md` | `doc/core/prd.project.md` | 项目全局总览与跨模块治理基线 |
| engineering | `doc/engineering/prd.md` | `doc/engineering/prd.project.md` | 工程规范、质量门禁、文件治理 |
| game | `doc/game/prd.md` | `doc/game/prd.project.md` | 玩法循环、规则层、发行可玩性 |
| headless-runtime | `doc/headless-runtime/prd.md` | `doc/headless-runtime/prd.project.md` | 无界面运行链路与生产稳定性 |
| p2p | `doc/p2p/prd.md` | `doc/p2p/prd.project.md` | 网络、共识、分布式存储 |
| playability_test_result | `doc/playability_test_result/prd.md` | `doc/playability_test_result/prd.project.md` | 可玩性测试数据与收口闭环 |
| readme | `doc/readme/prd.md` | `doc/readme/prd.project.md` | 对外口径与文档入口一致性 |
| scripts | `doc/scripts/prd.md` | `doc/scripts/prd.project.md` | 自动化脚本能力与维护规范 |
| site | `doc/site/prd.md` | `doc/site/prd.project.md` | 站点体验、内容发布、SEO |
| testing | `doc/testing/prd.md` | `doc/testing/prd.project.md` | 分层测试体系与发布门禁 |
| world-runtime | `doc/world-runtime/prd.md` | `doc/world-runtime/prd.project.md` | 运行时内核、WASM、治理与审计 |
| world-simulator | `doc/world-simulator/prd.md` | `doc/world-simulator/prd.project.md` | 世界模拟、Viewer、LLM 与场景系统 |

## 目录结构说明
- `doc/<module>/prd.md`：模块设计主文档（唯一 PRD 入口）。
- `doc/<module>/prd.project.md`：模块任务拆解与执行状态。
- `doc/<module>/**/*.md`：专题设计、实现方案、复盘与归档。
- `doc/<module>/README.md`：模块目录索引（按主题子目录导航）。
- `doc/archive/root-history/`：`doc/` 根目录历史治理文档归档。
- `doc/devlog/`：按日任务日志（时刻、完成内容、遗留事项）。

## 维护约定（摘要）
- 新功能或行为变更必须先更新模块 `prd.md`，再更新 `prd.project.md`，最后实现与测试。
- 代码、测试、文档任务必须可追溯到 PRD-ID。
- 单个 Markdown 文档不超过 500 行；`prd.md` 超限时拆分为 `doc/<module>/prd/*.md`，并保留 `prd.md` 作为总览入口。
