# Viewer 商业化发行缺口收敛 Phase 5（项目管理）

## 任务拆解
- [x] VCR5-0 文档建档：设计文档 + 项目管理文档
- [x] VCR5-1 扩展高级贴图覆盖配置（环境变量解析）
- [ ] VCR5-2 接入 3D 场景高级贴图通道并补测试
- [ ] VCR5-3 更新手册/项目状态/devlog 并收口

## 依赖
- `crates/agent_world_viewer/src/viewer_3d_config.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/tests.rs`
- `crates/agent_world_viewer/src/viewer_3d_config_profile_tests.rs`
- `doc/viewer-manual.md`

## 状态
- 当前阶段：VCR5-0、VCR5-1 已完成，VCR5-2 进行中。
- 下一步：将高级贴图通道接入 `setup_3d_scene` 材质构建流程并补行为测试。
- 最近更新：2026-02-21。
