# World Runtime：Builtin Wasm 先拉取后编译回退（设计文档）

## 目标
- 在节点运行时装载 builtin wasm 时，优先从网络拉取已构建产物；拉取失败后再触发本地编译回退。
- 保持去中心化：不引入中心化 Builder，任何节点都可以在本地编译并产出可校验 wasm。
- 与既有模块身份校验对齐：最终落地的 wasm 必须匹配 `wasm_hash`，避免不同机器构建漂移导致的无声错误。

## 范围

### In Scope
- runtime builtin wasm 读取链路改造：`local distfs -> network fetch -> local compile fallback`。
- 增加可配置的 fetch / compiler 接口（通过环境变量注入），便于不同部署接入各自网络层。
- 编译回退结果写入本地 distfs（sha256），并立即复用。
- 增加 `test_tier_full` 闭环测试：覆盖“拉取失败 -> 本地编译 -> 成功装载并缓存”。

### Out of Scope
- 引入新的中心化构建服务。
- 统一的全网 artifact 发布/治理协议（本轮仅在节点内落地 fetch+fallback 行为）。
- 非 builtin wasm 模块的源码分发协议。

## 接口 / 数据
- 运行时入口：
  - `crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
  - `crates/agent_world/src/runtime/m4_builtin_wasm_artifact.rs`
- 新增内置装载辅助：
  - 本地验证读取：`sha256` + distfs `get_verified`
  - 网络拉取（可配置）：通过 fetcher/URL 配置尝试获取 `<wasm_hash>` 对应 bytes
  - 回退编译（可配置）：按 `module_id` 在节点本地编译 wasm，写回 distfs
- 关键环境变量（设计约定）：
  - `AGENT_WORLD_BUILTIN_WASM_FETCHER`
  - `AGENT_WORLD_BUILTIN_WASM_FETCH_URLS`
  - `AGENT_WORLD_BUILTIN_WASM_COMPILER`
  - `AGENT_WORLD_BUILTIN_WASM_DISTFS_ROOT`

## 里程碑
- **BFC-1**：设计文档与项目管理文档落地。
- **BFC-2**：runtime builtin wasm 拉取/回退编译链路实现。
- **BFC-3**：`test_tier_full` 闭环测试落地。
- **BFC-4**：回归、项目文档状态更新、任务日志收口。

## 风险
- 本地编译回退依赖节点编译环境（toolchain/target）可用；需清晰报错。
- 远端返回错误产物时必须严格 hash 校验，避免污染本地缓存。
- 测试若直接依赖真实网络可能不稳定，需使用可控闭环场景。
