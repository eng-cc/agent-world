# WASM Builder Image

这个目录定义 Agent World 的 canonical WASM builder image。

## 目标
- 发布级 WASM 只通过 Docker 构建。
- canonical 容器平台固定为 `linux-x86_64`（Docker platform `linux/amd64`）。
- 宿主机不再保留 native fallback。

## 本地构建
```bash
docker build \
  --platform linux/amd64 \
  -t agent-world/wasm-builder:nightly-2025-12-11 \
  -f docker/wasm-builder/Dockerfile \
  .
```

## 本地使用
默认入口仍是仓库根目录的 `scripts/build-wasm-module.sh`。该脚本会：
- 检查 Docker daemon。
- 自动构建默认 builder image（若本地不存在）。
- 把源码 workspace 绑定到容器内 `/workspace`。
- 在容器内执行 `wasm_build_suite build ...`。

示例：
```bash
./scripts/build-wasm-module.sh \
  --module-id kwt.template \
  --manifest-path tools/wasm_build_suite/templates/minimal_module/Cargo.toml \
  --out-dir .tmp/wasm-build-suite \
  --profile dev
```

## 约束
- 调用方必须保证 `manifest-path` 和 `out-dir` 位于同一个 workspace root 下。
- repo 内模块默认以仓库根目录作为 workspace root。
- `ModuleSourcePackage` 这类临时源码包应在其临时 workspace root 下调用该脚本。

## 后续
- WDBP-2 会在此基础上补 `build receipt`、builder digest 绑定和 manifest token 迁移。
