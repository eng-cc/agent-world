# Agent World: 测试覆盖与 CI 扩展（项目管理文档）

## 任务拆解
- [x] T1 新增离线回放 viewer 联测（snapshot/journal -> server -> client）
- [x] T1 补充 viewer server 单次运行入口（run_once）
- [x] T2 CI 增加 wasmtime 特性测试步骤
- [x] T3 文档更新（联测运行方式与 CI 覆盖说明）
- [x] T4 CI 安装 viewer 系统依赖（Wayland/X11/ALSA/UDev）
- [x] T5 统一 CI/预提交测试清单脚本（`scripts/ci-tests.sh`）
- [x] T6 CI 与 pre-commit 接入统一脚本
- [x] T7 文档同步（ci-test-coverage/pre-commit/visualization）
- [ ] 提交到 git

## 依赖
- `ViewerServer` / `ViewerServerConfig`
- `world_viewer_demo` 生成的回放数据
- CI workflow 配置

## 状态
- 当前阶段：T7 完成
