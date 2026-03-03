# Viewer 商业化发行缺口收敛 Phase 3（项目管理）

## 任务拆解
- [x] VCR3-0 文档建档：设计文档 + 项目管理文档
- [x] VCR3-1 新增材质风格覆盖配置（环境变量解析）
- [x] VCR3-2 接入 3D 场景材质构建逻辑并补测试
- [x] VCR3-3 更新手册/项目状态/devlog 并收口

## 依赖
- `crates/agent_world_viewer/src/viewer_3d_config.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/viewer_3d_config_profile_tests.rs`
- `doc/viewer-manual.md`

## 状态
- 当前阶段：VCR3（VCR3-0 ~ VCR3-3）已全部完成。
- 下一步：进入后续阶段（纹理管线/后处理资产包）前，优先在 Web 闭环与 snapshot 基线上验证主题配色的可读性与稳定性。
- 最近更新：2026-02-21。
