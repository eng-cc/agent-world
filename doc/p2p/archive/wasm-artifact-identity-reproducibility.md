# World Runtime: Wasm 构建一致性与模块身份升级（设计文档）

> 归档说明（2026-02-20）：该方案已被 `doc/p2p/builtin-wasm-identity-consensus.md` 取代，不再作为现行实现依据。

## 目标
- 固定 Rust toolchain，降低不同机器（本地/CI/节点）构建结果漂移风险。
- 将模块身份从“仅 `wasm_hash`”升级为“`wasm_hash + source_hash + build_manifest_hash + artifact_signature`”的可验证组合。
- 保持去中心化执行策略：节点优先从网络拉取构建产物，拉取失败时再本地编译并验证身份，不引入单独集中 Builder 角色。

## 范围

### In Scope
- 增加仓库级 `rust-toolchain.toml`，固定 `channel/components/targets`。
- CI workflow 对齐固定 toolchain，并显式安装 `wasm32-unknown-unknown`。
- 在 wasm ABI 层新增模块身份结构（源码哈希、构建清单哈希、产物签名）。
- 在 `ModuleManifest` 增加可选身份字段；保留 `wasm_hash` 兼容。
- runtime 校验规则增强：
  - 若声明了身份字段，则要求完整且与 `wasm_hash` 语义一致。
  - 仍兼容旧 manifest（无身份字段）以保证平滑迁移。
- built-in wasm bootstrap 生成的 manifest 默认携带身份字段（至少包含与仓库可计算数据一致的 `source_hash/build_manifest_hash`）。

### Out of Scope
- 完整远程产物调度协议与 p2p 拉取实现改造（本轮仅定义策略与接口预留，不完成全链路网络实现）。
- 非 wasm 模块制品治理。
- 签名密钥分发与硬件信任根系统。

## 接口 / 数据
- toolchain 文件：`rust-toolchain.toml`
- 模块身份结构（新增）：
  - `source_hash`：源码/输入包摘要（sha256 hex）
  - `build_manifest_hash`：构建清单摘要（toolchain/target/features/profile 等）
  - `artifact_signature`：产物签名（当前允许占位策略，后续替换为正式签名）
- Manifest 兼容策略：
  - 新字段 `artifact_identity` 采用 `Option` + `serde(default)`，旧数据可直接反序列化。
- 运行时策略（本轮文档化 + 局部校验落地）：
  - 优先网络拉取 `<wasm_hash>.wasm` + identity 元信息；
  - 拉取失败回退本地编译；
  - 编译后校验 `wasm_hash` 与 identity，再进入注册/激活流程。

## 里程碑
- **WIR-1**：设计/项目文档落地。
- **WIR-2**：toolchain 固定与 CI 对齐。
- **WIR-3**：模块身份结构、manifest 字段、runtime 校验与测试落地。
- **WIR-4**：任务日志与回归收口。

## 风险
- 增加 manifest 字段后，若测试/fixture 未同步，容易出现大量编译失败；需分批回归。
- `artifact_signature` 先采用占位策略时，安全收益有限；需后续升级到真实签名体系。
- 混合模式（兼容旧 manifest）会存在过渡期双轨语义，需在文档中明确优先级。
