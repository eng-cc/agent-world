# Agent World Simulator：WASM 模块通用可视实体（项目管理文档）

## 任务拆解

### S1 文档与任务拆解
- [x] 输出设计文档（`doc/world-simulator/module-visual-entities.md`）
- [x] 输出项目管理文档（本文件）

### S2 Simulator 通用实体链路
- [x] 新增 `ModuleVisualEntity` / `ModuleVisualAnchor` 与 world model 存储
- [x] 初始化/场景配置接入 + 校验（ID、锚点、冲突）
- [x] 新增 action/event/replay 支持（upsert/remove）

### S3 Viewer 通用渲染链路
- [x] 从 snapshot 渲染模块可视实体
- [x] 处理模块可视实体增量事件（upsert/remove）
- [x] 详情面板支持模块可视实体基础信息展示

### S4 测试与回归
- [x] 新增/更新 simulator 测试（初始化、action、replay）
- [x] 新增/更新 viewer 展示测试（详情文本回归）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world`
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world_viewer`

### S5 文档回写与提交
- [x] 更新设计文档/项目管理文档状态
- [x] 追加 `doc/devlog/2026-02-07.md`
- [x] 提交 git commit

## 依赖
- simulator 初始化/内核事件链路（`init.rs` / `kernel/actions.rs` / `kernel/replay.rs`）
- 场景文件模型（`scenario.rs`）
- viewer 3D 场景构建与选择详情（`scene_helpers.rs` / `ui_text.rs`）

## 状态
- 当前阶段：S5 完成
- 下一阶段：按模块生态扩展可视实体过滤/分层渲染
- 最近更新：通用模块可视实体链路与 viewer 渲染闭环完成（2026-02-07）
