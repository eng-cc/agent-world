# README 缺口 2 收口：LLM 直连 WASM 生命周期（设计文档）

## 目标
- 收口 README 中“Agent 可编写/编译/部署/安装 WASM 模块”的主链路缺口。
- 让 `simulator` 场景下的 LLM Agent 可直接通过 `agent_submit_decision` 触发 WASM 生命周期动作：
  - `compile_module_artifact_from_source`
  - `deploy_module_artifact`
  - `install_module_from_artifact`
- 保持事件可审计、可回放，满足持久化闭环要求。

## 范围
- In scope
  - `crates/agent_world/src/simulator/llm_agent`
    - 扩展 decision schema、parser、prompt schema，支持三类 WASM 生命周期动作。
    - 增加动作参数严格校验（agent 身份、hash、hex 编码、必填字段）。
    - 提供 `module.lifecycle.status` 查询模块，用于 LLM 回合间读取已知 artifact / installed 状态。
  - `crates/agent_world/src/simulator`
    - 新增三类 `Action` 变体与对应 kernel 执行语义。
    - 新增生命周期事件（artifact deploy/install）并接入 replay。
    - 在 world model 中持久化 artifact 与 installed module 状态。
- Out of scope
  - 完整模块市场交易（挂牌/竞价/销毁）在 simulator 侧全量迁移。
  - 真实运行期模块激活执行（如自动接入 pre-action wasm sandbox）
    - 本次只完成生命周期状态闭环，不改变规则执行器绑定策略。

## 接口 / 数据
### 1) LLM 决策扩展
- 新决策枚举
  - `compile_module_artifact_from_source`
  - `deploy_module_artifact`
  - `install_module_from_artifact`
- 新增字段
  - `publisher` / `installer`: `self|agent:<id>`
  - `module_id` / `module_version`
  - `manifest_path`
  - `source_files`：`{ "path": "utf8 content" }`
  - `wasm_hash`
  - `wasm_bytes_hex`
  - `activate`
- 约束
  - `publisher/installer` 必须解析为 Agent owner，不允许 location owner。
  - `module_id/module_version/manifest_path/wasm_hash` 非空。
  - `wasm_bytes_hex` 必须可 hex 解码且非空。
  - `source_files` 非空，path/content 非空。

### 2) Simulator 动作与状态
- 新增动作
  - `Action::CompileModuleArtifactFromSource`
  - `Action::DeployModuleArtifact`
  - `Action::InstallModuleFromArtifact`
- 新增 world model 状态
  - `module_artifacts: BTreeMap<wasm_hash, ModuleArtifactState>`
  - `installed_modules: BTreeMap<module_id, InstalledModuleState>`
- 核心语义
  - Compile
    - 通过 runtime 的 source compiler 产出 wasm bytes。
    - 成功后按 hash 注册 artifact，落 `ModuleArtifactDeployed` 事件。
  - Deploy
    - 校验 `sha256(wasm_bytes) == wasm_hash`。
    - 注册 artifact，落 `ModuleArtifactDeployed` 事件。
  - Install
    - 要求 artifact 已存在，且 installer 是 artifact owner。
    - 写入/覆盖 `installed_modules[module_id]`，落 `ModuleInstalled` 事件。

### 3) 事件与回放
- 新增事件
  - `WorldEventKind::ModuleArtifactDeployed`
  - `WorldEventKind::ModuleInstalled`
- replay 要求
  - replay 时重建 artifact registry 与 installed module 状态。
  - 对 hash mismatch / owner mismatch / payload 空值保持拒绝一致性。

## 里程碑
- M1：T0 文档冻结（设计 + 项目管理）。
- M2：T1 LLM schema/parser/prompt 扩展完成。
- M3：T2 kernel 执行 + 状态持久化 + replay 闭环完成。
- M4：T3 required tests + `cargo check` + 文档/devlog 收口。

## 风险
- 编译环境风险
  - compile 动作依赖 source compiler / build script；在无工具链环境中会拒绝，需要明确错误信息。
- 事件体积风险
  - artifact deploy 事件携带 wasm bytes，可能导致 journal 体积增长。
- 行为稳定性风险
  - 新动作加入后可能增加 LLM 决策空间，需通过 prompt 约束与 status 查询降低无效尝试。
