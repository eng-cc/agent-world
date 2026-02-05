# Agent World: 测试覆盖与 CI 扩展（项目管理文档）

## 任务拆解
- [x] T1 新增离线回放 viewer 联测（snapshot/journal -> server -> client）
- [x] T1 补充 viewer server 单次运行入口（run_once）
- [x] T2 CI 增加 wasmtime 特性测试步骤
- [x] T3 文档更新（联测运行方式与 CI 覆盖说明）
- [ ] 提交到 git

## 依赖
- `ViewerServer` / `ViewerServerConfig`
- `world_viewer_demo` 生成的回放数据
- CI workflow 配置

## 状态
- 当前阶段：T3 完成
