# 启动器生命周期与就绪硬化（2026-03-01）

## 目标
- 修复 `world_game_launcher` 在启动失败与外部停止场景中的子进程清理缺陷，避免遗留 `world_viewer_live` / `world_chain_runtime`。
- 修复“就绪判定先成功、子进程随即退出”的误报窗口，降低发布门禁与人工验收中的假阳性。
- 补齐 IPv6 地址解析与 URL 构造的基础兼容，确保 CLI/GUI 启动口径一致。

## 范围
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- 对应单元测试（`test_tier_required`）

## 接口 / 数据
- 启动器信号处理：
  - 在 launcher 进程收到终止信号时，触发统一 shutdown 标记并走现有清理路径。
- 启动失败回滚：
  - 若静态 HTTP 服务绑定失败，已启动的 `world_viewer_live` / `world_chain_runtime` 必须立即终止。
- 就绪探针增强：
  - HTTP/TCP ready 等待期间追加子进程存活检查。
  - HTTP ready 仅接受有效 HTTP 响应前缀，避免非 HTTP 端口误判。
- 地址解析：
  - `parse_host_port` 支持 `[::1]:port`，拒绝未加括号的 IPv6 `::1:port`。
  - URL 输出对 IPv6 主机自动加 `[]`。

## 里程碑
- M1：建档（设计 + 项目管理）。
- M2：`world_game_launcher` 生命周期与就绪逻辑修复完成并通过定向测试。
- M3：`agent_world_client_launcher` 地址解析/URL 规则对齐并通过定向测试，文档收口。

## 风险
- 引入信号处理后，测试进程内全局 handler 可能影响并行测试行为。
  - 缓解：使用一次性安装并在测试中避免主动发信号。
- IPv6 解析收紧可能使历史非标准写法报错。
  - 缓解：错误信息明确提示 bracket 格式，避免静默兼容带来的歧义。
