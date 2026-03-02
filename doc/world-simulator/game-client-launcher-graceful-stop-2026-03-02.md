# 客户端启动器优雅退出与级联进程关闭（2026-03-02）

## 目标
- 在桌面启动器点击“停止”或直接关闭窗口时，优先让 `world_game_launcher` 走优雅退出路径。
- 提升“启动器退出后残留子进程”问题的可控性，尽量确保 `world_viewer_live` 与 `world_chain_runtime` 被一并清理。

## 范围
- 改造 `crates/agent_world_client_launcher` 的停止流程：
  - 先发送中断信号（Unix 下向 launcher 进程发送 `SIGINT`）。
  - 在限定超时时间内轮询等待 launcher 自行退出。
  - 超时后再执行强制 `kill` 兜底。
- 关闭窗口（`Drop`）复用同一停止逻辑。
- 补充单元测试覆盖停止逻辑关键路径。

## 非目标
- 不改动 `world_game_launcher` 的参数接口或核心编排逻辑。
- 不在本轮引入跨平台完整进程组管理（Windows JobObject 等）。

## 接口 / 数据
- 新增停止策略常量：
  - `GRACEFUL_STOP_TIMEOUT_MS`（优雅等待超时）。
  - `STOP_POLL_INTERVAL_MS`（轮询间隔）。
- `stop_child_process` 语义更新：
  - 优先中断信号；
  - 超时 fallback 强杀；
  - 返回值保持 `Result<(), String>`。

## 里程碑
- M1：文档建档完成。
- M2：停止逻辑改造完成并接入窗口关闭路径。
- M3：测试、文档、日志收口，项目结项。

## 风险
- 仅发送 `SIGINT` 仍可能遇到进程未响应（死锁、阻塞 IO）。
  - 缓解：保留超时后 `kill` 兜底，不阻塞 UI 无限等待。
- 跨平台信号行为差异可能导致体验不一致。
  - 缓解：Unix 实现优雅中断；非 Unix 保持安全 fallback 语义。
