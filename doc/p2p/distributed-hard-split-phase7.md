# Agent World Runtime：分布式能力彻底拆分（Phase 7）

## 目标
- 完成分布式能力从 `agent_world` 到基础 crate 的彻底拆分：`agent_world_net` / `agent_world_consensus` / `agent_world_distfs` / `agent_world_proto`。
- 删除 `agent_world` 内分布式实现文件，`agent_world` 仅保留世界内核与模拟层，不再承载分布式实现逻辑。
- 将 Viewer 协议并入 `agent_world_proto`，消除 `agent_world` 对 Viewer 协议层的承载。
- 收敛 WASM ABI 边界，消除 net 侧重复清单定义，明确 ABI 与运行时缓存归属。
- 对超 1200 行 Rust 文件完成物理拆分，满足仓库维护约束。

## 范围
### In Scope
- 新增 `agent_world_distfs` crate，并迁移分布式文件能力：CAS、分片、组装校验。
- 分布式彻底拆分：删除 `crates/agent_world/src/runtime/distributed*`、`libp2p_net.rs` 及相关分布式路径代码。
- `agent_world` facade 收敛：移除分布式导出，按领域划分导出边界。
- Viewer 协议迁移到 `agent_world_proto` 并完成 server/viewer 双端适配。
- WASM ABI 边界调整：消除 `agent_world_net` 内重复 `ModuleManifest`，统一到 ABI/proto。
- 超长文件拆分（>1200 行）。

### Out of Scope
- 新增分布式功能语义（本阶段只做拆分与归位，不扩展新协议能力）。
- 引入新的传输协议或更换现有网络后端。

## 接口 / 数据
- `agent_world_distfs`：
  - `BlobStore` / `LocalCasStore`
  - `segment_snapshot` / `segment_journal`
  - `assemble_snapshot` / `assemble_journal`
- `agent_world_proto`：
  - 新增 Viewer 协议模块（请求/响应/控制字段）
  - 继续作为跨 crate 协议与错误模型载体
- `agent_world_wasm_abi`：
  - 统一模块清单类型来源，去掉 net 侧重复结构

## 里程碑
- M13-1：`agent_world_distfs` 落地并接入现有路径
- M13-2：`agent_world` 分布式文件删除完成，调用方切换到基础 crate
- M13-3：`agent_world` facade 收敛完成
- M13-4：Viewer 协议并入 proto 并适配完成
- M13-5：WASM ABI 边界收敛完成
- M13-6：超长文件拆分与回归完成

## 风险
- 大规模路径调整会导致编译面回归；需按任务分批编译与定向测试。
- API 导出收敛会影响现有调用方；需同步修改 workspace 内依赖点。
- Viewer 协议迁移会带来序列化兼容风险；需保证 server/viewer 协议版本一致。
