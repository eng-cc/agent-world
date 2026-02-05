# Agent World：足够真实且持久的世界模拟器（项目管理文档）

## 任务拆解
### 0. 对齐与准备
- [x] 输出设计文档（`doc/world-simulator.md`）
- [x] 输出项目管理文档（本文件）
- [x] 在 README 中给出愿景/原则/路线图与文档入口
- [x] 初始化 Rust workspace 工程骨架（`Cargo.toml` + `crates/agent_world`）
- [x] 移除 Python 脚手架代码（统一以 Rust workspace 为准）

### 1. 世界内核（MVP）
- [x] 定义最小世界模型（Location/Agent/GeoPos/RobotBodySpec/Resource/Asset/Event：电力/硬件/数据为核心资源）
- [x] 定义空间坐标与距离（尘埃云 3D 盒状空间；长度最小单位 1 cm；欧氏距离/可见范围/移动成本）
- [x] 定义时间推进与事件队列
- [x] 定义 Action 校验与失败原因（可解释）
- [x] 实现最小闭环示例：两个 Agent 在两地点之间移动与交互
- [x] 细化行动规则（移动成本/可见性/交互约束）
- [x] 提供内核配置参数（可见半径/移动成本）
- [x] 重新定义空间坐标为 3D 尘埃云（x/y/z cm），替换球面距离为欧氏距离
- [x] 引入尘埃云空间配置（100 km × 100 km × 10 km，可配置）与位置边界校验
- [x] 扩展 Location 物理画像（材质/尺寸/辐射强度）并纳入事件/观测输出
- [x] 支持辐射采集动作（HarvestRadiation）与对应事件
- [x] 补充尘埃云物理细化说明（辐射/推进/分布等）
- [x] 补充物理参数草案与量纲约定（时间尺度/能量单位/阈值）
- [x] 给出参数默认值/范围草案
- [x] 细化辐射采集与热管理规则（可落地）
- [x] 补充碎片分布生成器的建议草案
- [x] 扩展 WorldConfig 物理参数字段（辐射/热/侵蚀/时间尺度）
- [x] 添加 Agent 热状态与热管理规则（采集增热、tick 散热）
- [x] 实现辐射采集上限/衰减/热约束
- [x] 提供碎片分布生成器最小实现（体素+噪声+幂律）

### 2. 持久化与回放
- [x] 选择持久化策略：快照 + 增量事件（事件溯源可选）
- [x] 定义最小存储格式（snapshot.json + journal.json）
- [x] 定义版本迁移策略
- [x] 实现冷启动恢复、断点续跑
- [x] 支持回放与“从某个快照继续分叉”（可选）

### 3. Agent 运行时与 SDK
- [x] 定义 Agent 接口：`observe -> decide -> act`（AgentBehavior trait + AgentRunner）
- [x] 实现调度器：公平性、限速、配额（步数/事件/预算）
- [x] 基础可观测性：事件日志、指标（RunnerMetrics/AgentStats/RunnerLogEntry）
- [x] Agent 记忆最小实现：ShortTermMemory + LongTermMemory + AgentMemory

### 4. 社会系统（电力/硬件/数据）
- [ ] 电力供给/存储/消耗闭环（峰谷、停电、迁移成本）
- [ ] 硬件生产/维修/折旧/回收（稀缺性与供应链）
- [ ] 数据获取/存储/交易/访问控制（许可、隐私、污染与溯源）
- [ ] 交易与定价：电价、算力、存储、带宽、数据访问
- [ ] 合约/任务（算力外包、数据采集/标注）与声誉系统
- [ ] 基础治理规则（禁区、配额、税费/电费等，可极简）

### 5. 可视化与调试
- [ ] Bevy 可视化客户端（独立 crate，网络连接数据源）
- [ ] 世界状态面板（地点、人口、库存、价格等）
- [ ] 事件浏览器（筛选、回放、聚合统计）
- [ ] 运行控制（暂停/加速/单步/回滚到快照）
- [ ] 任务分册：`doc/world-simulator/visualization.project.md`

### 6. 维护
- [x] 拆分 simulator kernel/tests 文件以满足单文件行数上限
- [x] 对齐 simulator 单元测试与新 API（memory/runner/persist/power/observe）

## 依赖
- 基础语言与运行环境：Rust（Cargo workspace）
- 存储（本地文件、SQLite、或其他 KV/文档存储，待选）
- （可选）LLM/推理服务接入方式与预算策略（本地/远程、缓存、重试）

## 状态
- 当前阶段：M3（Agent 运行时与 SDK）**已完成**
- 下一步：M4（最小社会与经济；核心为 WASM 动态调用系统，Agent 创造的 Rust/WASM 模块通过事件/接口与世界交互）
- 最近更新：完成 world-simulator 设计文档细化项补全（2026-02-05）
