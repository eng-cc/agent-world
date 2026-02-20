# Viewer 3D 商业化精致度收敛（项目管理文档）

## 任务拆解

### C3D-0 文档建模
- [x] C3D0.1 输出设计文档（`doc/world-simulator/viewer-3d-commercial-polish.md`）
- [x] C3D0.2 输出项目管理文档（本文件）

### C3D-1 资产层（缺口 1）
- [ ] C3D1.1 新增 `ViewerRenderProfile` 与资产分档配置（debug/balanced/cinematic）
- [ ] C3D1.2 接入几何精度分档与 Location 壳层可选渲染
- [ ] C3D1.3 补齐配置解析与场景回归测试

### C3D-2 材质层（缺口 2）
- [ ] C3D2.1 新增材质分层配置（工业风 PBR 参数组）
- [ ] C3D2.2 接入元素分块材质策略（可读性优先/质感优先）
- [ ] C3D2.3 补齐材质构建与颜色差异回归测试

### C3D-3 光照层（缺口 3）
- [ ] C3D3.1 接入三点光照（key/fill/rim）
- [ ] C3D3.2 接入按档位阴影默认策略
- [ ] C3D3.3 补齐光照参数解析与稳定性回归

### C3D-4 后处理层（缺口 4）
- [ ] C3D4.1 相机接入 Tonemapping + DebandDither + ColorGrading
- [ ] C3D4.2 接入 Bloom（含可关闭开关）
- [ ] C3D4.3 补齐后处理配置与回归测试

### C3D-5 收口
- [ ] C3D5.1 更新 `doc/viewer-manual.md` 商业化 3D 配置说明
- [ ] C3D5.2 更新项目状态与 `doc/devlog/2026-02-20.md`
- [ ] C3D5.3 执行测试：`env -u RUSTC_WRAPPER cargo test -p agent_world_viewer`

## 依赖
- `crates/agent_world_viewer/src/viewer_3d_config.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/material_library.rs`
- `doc/viewer-manual.md`

## 状态
- 当前阶段：C3D-0 完成，C3D-1 进行中。
- 下一步：实现渲染档位与资产分档配置并补齐单测。
- 最近更新：2026-02-20（完成设计建模并启动实现）。
