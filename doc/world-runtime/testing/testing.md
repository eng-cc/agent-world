# Agent World Runtime：测试与基架建议（设计分册）

本分册为 `doc/world-runtime.md` 的详细展开。

## 集成测试用例（草案）

- **register_happy_path**：artifact 写入 → propose → shadow pass → approve → apply → 注册表更新
- **shadow_fail_blocks_apply**：shadow fail → reject → 不产生模块事件
- **apply_fail_records_validation**：apply 阶段校验失败 → ModuleValidationFailed → 注册表不更新
- **upgrade_flow**：升级成功后版本更新、旧版本不可激活
- **audit_export_contains_module_events**：审计导出包含 GovernanceEvent + Module*Failed/ShadowReport

## 测试基架建议（草案）

- 文件组织：`crates/agent_world/tests/module_lifecycle.rs`
- 共享夹具：`TestWorldBuilder`（构造 world + manifest + registry 初始态）
- 伪造工件：内存内生成 dummy wasm bytes + 计算 hash
- Shadow 注入：允许在测试中强制 shadow 失败/通过
- 断言：事件流顺序、注册表内容、审计导出记录

**TestWorldBuilder（示意 API）**
```rust
struct TestWorldBuilder {
    with_registry: bool,
    with_manifest: bool,
    caps: Vec<CapabilityGrant>,
}

impl TestWorldBuilder {
    fn new() -> Self;
    fn with_registry(mut self) -> Self;
    fn with_manifest(mut self) -> Self;
    fn with_caps(mut self, caps: Vec<CapabilityGrant>) -> Self;
    fn build(self) -> World;
}
```

**Dummy WASM 工件工具（示意）**
```rust
fn dummy_wasm_bytes(label: &str) -> Vec<u8>;  // 生成确定性字节
fn wasm_hash(bytes: &[u8]) -> String;         // 计算内容哈希（sha256/hex）
```

**Shadow 注入建议（示意）**
- 在 `WorldConfig` 或测试构造器中注入 `ShadowPolicy`（`AlwaysPass` / `AlwaysFail` / `ByModuleId(HashSet)`）。
- 测试中将 `ShadowPolicy` 设置为 `AlwaysFail` 以触发 shadow reject 分支。

**审计与可回放**
- 注册/激活/升级事件进入日志，`module_registry.json` 可由事件重建。
- 任意运行时模块版本都可由 `wasm_hash` 唯一定位。
