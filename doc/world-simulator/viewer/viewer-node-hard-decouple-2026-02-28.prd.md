# Viewer 与节点彻底拆分（2026-02-28）

审计轮次: 4
- 对应项目管理文档: doc/world-simulator/viewer/viewer-node-hard-decouple-2026-02-28.prd.project.md

## 1. Executive Summary
- 将 `world_viewer_live` 从“可内嵌节点运行时”的混合进程，重构为纯 Viewer/游戏服务进程。
- 明确节点运行时唯一入口为 `world_chain_runtime`，避免双路径并存导致的配置歧义。
- 对外发行统一链路保持不变：`world_game_launcher` 负责编排 viewer + chain-runtime。

## 2. User Experience & Functionality
- 重写 `world_viewer_live` 启动入口与 CLI，仅保留 Viewer 相关参数与 Web bridge。
- 移除 `world_viewer_live` 中的节点启动、PoS/P2P、reward-runtime、execution-bridge 运行路径。
- 更新测试与文档，确保“禁止在 viewer 内启节点”的行为可验证。

## 非目标
- 本轮不删除历史节点实现源码文件（先停止编译接线，后续再做代码归档清理）。
- 本轮不改动 `world_chain_runtime` 的节点能力边界。
- 本轮不重做 S10 长跑套件（如依赖旧 viewer 内嵌节点路径，另行迁移立项）。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
### `world_viewer_live` CLI（重构后）
- 保留：
  - `world_viewer_live [scenario]`
  - `--bind <host:port>`
  - `--web-bind <host:port>`
  - `--llm` / `--no-llm`
  - `-h` / `--help`
- 删除/禁用：全部 `--node-*`、`--topology`、`--reward-runtime-*`、`--viewer-no-consensus-gate`、`--no-node`。

### 启动行为
- `world_viewer_live` 只负责：
  - 启动 `ViewerLiveServer`
  - 可选启动 `ViewerWebBridge`
- 不再注入 `consensus runtime` 和 `consensus gate`。

## 5. Risks & Roadmap
- M1：文档建档（设计 + 项目管理）。
- M2：`world_viewer_live` 重构为纯 viewer 入口。
- M3：测试回归与文档结项。

### Technical Risks
- 依赖旧 `world_viewer_live --node-*` 的脚本将失效。
  - 缓解：CLI 显式报错并提示改用 `world_chain_runtime`/`world_game_launcher`。
- 历史大体量节点代码暂存仓库，后续维护成本仍在。
  - 缓解：后续补一轮“归档/删除死代码”专项。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.prd.project.md`，保持原文约束语义不变。
