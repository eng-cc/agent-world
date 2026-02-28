# 启动链路脚本迁移（2026-02-28）

## 目标
- 将仍依赖 `world_viewer_live` 旧节点参数（`--node-*` / `--topology` / `--reward-runtime-*`）的运行脚本迁移到解耦后的启动链路。
- 保证日常可执行脚本继续可用，并避免“脚本参数已失效但报错不明确”的情况。
- 对尚未完成重构的长跑脚本给出明确阻断提示与迁移方向，避免误用。

## 范围
- 迁移到 `world_game_launcher`：
  - `scripts/run-game-test.sh`
  - `scripts/viewer-release-qa-loop.sh`
- 对复杂长跑脚本增加显式阻断与提示（停止使用失效参数链路）：
  - `scripts/s10-five-node-game-soak.sh`
  - `scripts/p2p-longrun-soak.sh`
- 更新关联文档与手册中的调用口径。

## 非目标
- 本轮不重写 S10/P2P 长跑脚本为完整的 `world_chain_runtime` 多节点编排闭环。
- 本轮不恢复或兼容 `world_viewer_live` 内嵌节点参数。

## 接口 / 数据
### 新脚本调用口径
- 统一优先：`world_game_launcher`
  - `--scenario`
  - `--live-bind`
  - `--web-bind`
  - `--viewer-host` / `--viewer-port`
  - `--with-llm` / `--no-open-browser`
  - 链配置通过 `--chain-*` 参数族。

### 阻断口径
- 对仍要求旧 `world_viewer_live --node-*` 的脚本：启动前直接失败并输出迁移提示：
  - 建议改用 `world_chain_runtime`（多节点）或 `world_game_launcher`（单机一键）。

## 里程碑
- M1：建档（设计 + 项目管理）。
- M2：两条日常脚本迁移到 `world_game_launcher`。
- M3：长跑脚本阻断提示与手册收口。

## 风险
- 部分自动化链路依赖旧日志文件名或进程名。
  - 缓解：尽量保留日志产物命名兼容；文档同步说明。
- 长跑脚本功能降级为“显式阻断”后，相关测试流程暂不可执行。
  - 缓解：在提示中给出后续迁移方向与替代入口。
