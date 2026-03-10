# 启动器优雅退出与级联进程关闭设计（2026-03-02）

- 对应需求文档: `doc/world-simulator/launcher/game-client-launcher-graceful-stop-2026-03-02.prd.md`
- 对应项目管理文档: `doc/world-simulator/launcher/game-client-launcher-graceful-stop-2026-03-02.project.md`

## 1. 设计定位
定义启动器停止与窗口关闭时的统一进程收敛策略：优先请求 `world_game_launcher` 优雅退出，在限定窗口内等待，超时后再执行强杀兜底。

## 2. 设计结构
- 停止策略层：先中断、后轮询、超时强杀。
- 入口统一层：按钮停止与窗口关闭复用同一停止逻辑。
- 超时控制层：通过 `GRACEFUL_STOP_TIMEOUT_MS` 与 `STOP_POLL_INTERVAL_MS` 固定等待窗口。
- 平台适配层：Unix 优先 `SIGINT`，非 Unix 保持安全 fallback 语义。

## 3. 关键接口 / 入口
- `stop_child_process`
- `GRACEFUL_STOP_TIMEOUT_MS`
- `STOP_POLL_INTERVAL_MS`
- 启动器窗口 `Drop` 关闭路径

## 4. 约束与边界
- UI 不能因等待子进程退出而无限阻塞。
- 优雅退出失败时必须有 kill 兜底，避免残留进程。
- 本阶段不引入完整跨平台进程组管理。
- 返回语义保持 `Result<(), String>`，便于调用方统一处理。

## 5. 设计演进计划
- 先固定优雅退出时序与超时常量。
- 再把停止逻辑接入按钮与窗口关闭路径。
- 最后补单测与文档回写，保证行为一致收口。
