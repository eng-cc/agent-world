# Builtin Wasm Canonical 构建环境固化（项目管理文档）

## 任务拆解
- [x] CBE-1 设计文档：`doc/scripts/builtin-wasm-canonical-build-environment.md`
- [x] CBE-2 项目管理文档：本文件
- [ ] CBE-3 新增 canonical builder 脚本（容器化入口）并支持 dry-run
- [ ] CBE-4 在 `sync-m1/m4` 中接入 canonical builder（默认路径）
- [ ] CBE-5 增加回退开关与安全护栏（CI 禁止 non-canonical）
- [ ] CBE-6 更新 CI workflow，required gate 统一走 canonical builder
- [ ] CBE-7 重新同步 m1/m4 hash 清单并通过 `sync --check`
- [ ] CBE-8 required tier 回归通过并在 Ubuntu runner 验证
- [ ] CBE-9 更新 devlog、收口文档与提交

## 依赖
- Rust toolchain `1.92.0`
- target `wasm32-unknown-unknown`
- 容器运行时（Docker/Podman，CI 使用 GitHub hosted runner）
- 可访问的固定 builder image（建议 ghcr 固定 tag）

## 状态
- 当前阶段：规划完成，待实现
- 最近更新：CBE-1~CBE-2 完成（2026-02-17）

