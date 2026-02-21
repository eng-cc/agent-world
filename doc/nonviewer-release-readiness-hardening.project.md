# Non-Viewer 发行准备加固（项目管理文档）

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/nonviewer-release-readiness-hardening.md`
- [x] 新建项目管理文档：`doc/nonviewer-release-readiness-hardening.project.md`

### T1 测试覆盖补齐
- [x] 更新 `scripts/ci-tests.sh`，补齐 `node/consensus/distfs/net` 的 non-viewer 测试门禁覆盖
- [x] 更新 `testing-manual.md` 的 CI 覆盖事实口径
- [x] 跑一轮 required 回归并记录结果

### T2 Agent/PublicKey 绑定
- [x] 扩展 viewer 协议请求：支持传入可选 `public_key`
- [x] 扩展 world model + kernel 绑定存储与查询（player_id + public_key）
- [x] 扩展 `AgentPlayerBound` 事件与 replay 兼容处理
- [x] 在 live 控制链路增加 public_key 一致性校验与错误提示
- [x] 补齐单测/集成测试

### T3 收口
- [ ] 回写设计/项目文档状态
- [ ] 更新 `doc/devlog/2026-02-21.md`
- [ ] 运行定向回归并确认通过

## 依赖
- `scripts/ci-tests.sh`
- `testing-manual.md`
- `crates/agent_world_proto/src/viewer.rs`
- `crates/agent_world/src/simulator/*`
- `crates/agent_world/src/viewer/live*`

## 状态
- 当前状态：`进行中`
- 已完成：T0、T1、T2
- 进行中：T3
- 未开始：无
- 阻塞项：无
