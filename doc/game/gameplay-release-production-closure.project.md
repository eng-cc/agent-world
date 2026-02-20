# Gameplay 发行收口（项目管理文档）

## 任务拆解

### T0 文档与任务建模
- [x] 新建设计文档：`doc/game/gameplay-release-production-closure.md`
- [x] 新建项目管理文档：`doc/game/gameplay-release-production-closure.project.md`

### T1 社会经济治理最小闭环（对应需求 2）
- [x] 扩展 Runtime Action/DomainEvent：经济合约与治理规则动作
- [x] 扩展 Runtime State：经济合约状态、声誉账本、治理参数
- [x] 实现生命周期：合约创建/签约/结算/过期与声誉奖惩
- [x] 补充 runtime 协议测试（主路径 + 拒绝路径）

### T2 压测玩法门禁工程化（对应需求 3）
- [x] 扩展 `scripts/llm-longrun-stress.sh` 支持玩法覆盖阈值参数
- [x] 新增 `--release-gate` 发行口径开关与失败提示
- [x] 更新 `testing-manual.md` 的 S8 使用口径与示例

### T3 联盟生命周期与战争约束（对应需求 4）
- [x] 扩展 Runtime Action/DomainEvent：`join/leave/dissolve` 与战争后果字段
- [x] 扩展 Runtime 规则：联盟生命周期约束、战争成本与结算后果
- [x] 扩展 LLM 决策 schema/parser/execution controls 支持新动作
- [x] 补充 runtime + llm_agent 定向测试

### T4 收口与回写
- [ ] 回写本项目文档状态
- [ ] 追加当日 devlog 任务日志（每任务一条）
- [ ] 运行相关测试并记录

## 依赖
- Runtime Gameplay 基线：`crates/agent_world/src/runtime/world/*`
- LLM 决策协议：`crates/agent_world/src/simulator/llm_agent/*`
- 压测脚本：`scripts/llm-longrun-stress.sh`
- 测试口径手册：`testing-manual.md`

## 状态
- 当前阶段：T4（进行中）
- 阻塞项：无
