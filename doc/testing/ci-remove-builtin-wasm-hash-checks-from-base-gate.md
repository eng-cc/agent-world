# Agent World: 基础 CI 门禁移除 Builtin Wasm Hash 校验（设计文档）

## 目标
- 将 `scripts/ci-tests.sh` 基础门禁中的 builtin wasm hash 校验（`m1/m4/m5 --check`）全部移除。
- 明确基础门禁职责只保留格式与测试编译链路，不再承担 wasm 工件 hash 校验。
- 在测试手册中同步新的覆盖边界，避免测试口径与实际 CI 行为不一致。

## 范围

### In Scope
- 修改 `scripts/ci-tests.sh`，移除：
  - `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
  - `./scripts/sync-m4-builtin-wasm-artifacts.sh --check`
  - `./scripts/sync-m5-builtin-wasm-artifacts.sh --check`
- 保留 `required/full` 的其余测试路径不变。
- 更新 `testing-manual.md` 中“当前 CI 覆盖/缺口”口径。

### Out of Scope
- 不新增 m4/m5 独立 workflow。
- 不变更 `builtin-wasm-m1-multi-runner` workflow 的现有行为。
- 不修改 wasm 构建脚本本身（`sync-m*-builtin-wasm-artifacts.sh`）。

## 接口 / 数据
- 变更脚本：`scripts/ci-tests.sh`
- 变更文档：`testing-manual.md`
- 影响入口：
  - 本地与 pre-commit 的 `./scripts/ci-tests.sh required`
  - 依赖 `scripts/ci-tests.sh` 的 workflow job

## 里程碑
- M1：设计文档与项目管理文档落地。
- M2：基础门禁脚本移除 hash 校验。
- M3：测试手册同步与口径校验。
- M4：验证与收口。

## 风险
- 移除后 `m4/m5` hash 漂移无法在基础门禁被及时拦截，需要在手册明确该盲区。
- 若团队仍依赖基础门禁发现 wasm 漂移，短期可能出现“发现时点后移”。
