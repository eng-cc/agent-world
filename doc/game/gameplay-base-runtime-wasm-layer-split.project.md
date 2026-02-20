# Gameplay Base Runtime / WASM Layer Split（项目管理文档）

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/game/gameplay-base-runtime-wasm-layer-split.md`
- [x] 新建项目管理文档：`doc/game/gameplay-base-runtime-wasm-layer-split.project.md`

### T1 Runtime 分层改造
- [ ] 新增基础层实现文件，迁移模块通用治理校验逻辑
- [ ] 新增 gameplay 层实现文件，迁移 gameplay 契约/冲突/就绪度逻辑
- [ ] 调整 `world/mod.rs` 模块组织，形成显式分层入口

### T2 回归与收口
- [ ] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::gameplay:: -- --nocapture`
- [ ] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::modules:: -- --nocapture`
- [ ] 回写项目文档状态与 `doc/devlog/2026-02-20.md`

## 依赖
- `doc/game/gameplay-engineering-architecture.md`
- `doc/game/gameplay-runtime-governance-closure.md`
- `testing-manual.md`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2
- 阻塞项：无
