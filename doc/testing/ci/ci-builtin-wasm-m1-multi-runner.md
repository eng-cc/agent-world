# Agent World: CI 拆分 Builtin Wasm m1 多 Runner 校验（设计文档）

## 目标
- 将 builtin wasm 构建链路校验从主测试流中独立出来，形成可单独观察和定位的 CI 工作流。
- 在不同宿主 runner 上执行同一条 `m1` 校验链路，验证多平台构建环境下的稳定性。
- 保持每个 runner 只执行 `m1` 校验，控制执行成本，同时确保链路健康可观测。

## 范围

### In Scope
- 新增独立 workflow（GitHub Actions）用于 m1 多 runner 校验。
- runner 矩阵至少覆盖：
  - `ubuntu-latest`（linux-x86_64）
  - `macos-14`（darwin-arm64）
- 每个 runner 仅执行：
  - `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
- 每个 runner 导出 m1 校验摘要（模块 hash、平台键、identity 摘要）供汇总 job 对账。
- 汇总 job 执行跨 runner 一致性对账并给出可定位失败信息。

### Out of Scope
- 不在本次新增 m4/m5 的多 runner 校验矩阵。
- 不替换现有 `required/full` 主测试流程与分层测试定义。
- 不引入 Docker/Podman 容器化构建链路。

## 接口 / 数据
- 新增 workflow：
  - `.github/workflows/builtin-wasm-m1-multi-runner.yml`
- 新增 CI 脚本：
  - `scripts/ci-m1-wasm-summary.sh`
  - `scripts/ci-verify-m1-wasm-summaries.py`
- runner 摘要输出：
  - `output/ci/m1-wasm-summary/<runner>.json`
- 摘要字段（最小集合）：
  - `runner`
  - `current_platform`
  - `manifest_path`
  - `identity_manifest_path`
  - `module_hashes`（`module_id -> hash`）
  - `identity_hashes`（`module_id -> identity_hash`）

## 里程碑
- M1：设计文档与项目管理文档落地。
- M2：CI 摘要与跨 runner 对账脚本落地。
- M3：独立 m1 多 runner workflow 接入并通过。
- M4：测试手册与项目状态收口。

## 风险
- `macos-14` 与本地开发机环境差异可能导致偶发构建抖动，需要依赖固定 toolchain 与现有 deterministic guard。
- 多 runner 任务会增加 PR 等待时间，需要把校验粒度限制在 `m1` 以控制时延。
- 摘要对账若字段定义不稳定，可能导致误报，需要固定输出 schema 并在脚本中做严格校验。
