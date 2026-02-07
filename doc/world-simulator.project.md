# Agent World：足够真实且持久的世界模拟器（项目管理文档）

## 任务拆解
### 0. 对齐与准备
- [x] 输出设计文档（`doc/world-simulator.md`）
- [x] 输出项目管理文档（本文件）
- [x] 在 README 中给出愿景/原则/路线图与文档入口
- [x] 更新世界观设定（破碎小行星带/碎片尺寸范围/辐射能转电模块，移除辐射反作用力推进描述）
- [x] 补充小行星碎片最小间距设定（至少 500 m）
- [x] 初始化 Rust workspace 工程骨架（`Cargo.toml` + `crates/agent_world`）
- [x] 移除 Python 脚手架代码（统一以 Rust workspace 为准）

### 1. 世界内核（MVP）
- [x] 定义最小世界模型（Location/Agent/GeoPos/RobotBodySpec/Resource/Asset/Event：电力/硬件/数据为核心资源）
- [x] 定义空间坐标与距离（破碎小行星带 3D 盒状空间；长度最小单位 1 cm；欧氏距离/可见范围/移动成本）
- [x] 定义时间推进与事件队列
- [x] 定义 Action 校验与失败原因（可解释）
- [x] 实现最小闭环示例：两个 Agent 在两地点之间移动与交互
- [x] 细化行动规则（移动成本/可见性/交互约束）
- [x] 提供内核配置参数（可见半径/移动成本）
- [x] 重新定义空间坐标为 3D 破碎小行星带（x/y/z cm），替换球面距离为欧氏距离
- [x] 引入破碎小行星带空间配置（100 km × 100 km × 10 km，可配置）与位置边界校验
- [x] 扩展 Location 物理画像（材质/尺寸/辐射强度）并纳入事件/观测输出
- [x] 支持辐射采集动作（HarvestRadiation）与对应事件
- [x] 补充破碎小行星带物理细化说明（辐射/分布/热与侵蚀等）
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
- [x] 补充 LLM 驱动与模块化记忆设计（OpenAI 兼容 API、memory module 为 WASM + 受限存储）

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
- [x] Viewer 可观测性增强：Agent 活动面板与世界背景参照（边界盒 + 地板网格）
- [x] 修复 viewer 多相机渲染歧义（Camera order）并恢复 3D 交互
- [x] 输出 viewer 选中详情设计文档（`doc/world-simulator/viewer-selection-details.md`）
- [x] 输出 viewer 选中详情项目管理文档（`doc/world-simulator/viewer-selection-details.project.md`）
- [x] Viewer 选中对象详情面板（Agent/Location）+ LLM 输入输出展示
- [x] Viewer 选中对象详情扩展（Asset/PowerPlant/PowerStorage/Chunk）
- [x] 可视化设计文档补充“信息直达”原则并盘点现状缺口
- [x] 在线模式支持任意 tick seek（reset+replay，含不可达保护）
- [x] Viewer 时间轴目标 tick 控件与拖拽 seek
- [x] Viewer 时间轴关键事件标注与密度提示
- [x] Viewer 时间轴标注联动事件列表上下文跳转
- [x] Viewer 时间轴事件类别独立开关/筛选（err/llm/peak）
- [x] Viewer 事件与对象双向联动（事件定位对象、对象跳转事件）

### 6. 维护
- [x] 拆分 simulator kernel/tests 文件以满足单文件行数上限
- [x] 对齐 simulator 单元测试与新 API（memory/runner/persist/power/observe）

### 7. 分块世界生成（探索驱动）
- [x] 输出分块世界生成与碎片元素池设计文档（`doc/world-simulator/chunked-fragment-generation.md`）
- [x] 输出对应项目管理文档（`doc/world-simulator/chunked-fragment-generation.project.md`）
- [x] 实现 20km×20km×10km chunk 基础能力（坐标映射/边界/seed）
- [x] 明确碎片块状几何与物理量（长方体/体积/密度/质量/1cm 最小单位）
- [x] 明确化合物主导组成与元素统计映射口径
- [x] 接入未探索不生成的 chunk 索引与触发逻辑（observe/move/transfer/harvest 触发）
- [x] 接入碎片块状物理画像与化合物组成生成（体积/密度/质量 + compounds/elements）
- [x] 接入资源预算一次性生成（total/remaining）与开采扣减守恒
- [x] 场景接入起始 chunk 预生成与固定 20km×20km×10km 分块配置
- [x] 接入 ChunkGenerated 事件与持久化/回放校验（CG6：init/observe/action + 版本迁移）
- [x] 接入跨 chunk 边界一致性（CG7：邻块校验 + BoundaryReservation 保留/消费）
- [x] 接入 RefineCompound 经济资源映射最小闭环（electricity 消耗 + hardware 产出 + 回放）
- [x] 接入分块生成性能预算与确定性降级（CG9：fragments/blocks 三档上限）
- [x] 补充分块生成回放一致性与性能回归测试（CG10）

### 8. LLM Agent 行为落地
- [x] 输出 LLM Agent 落地设计文档（`doc/world-simulator/llm-agent-behavior.md`）
- [x] 输出对应项目管理文档（`doc/world-simulator/llm-agent-behavior.project.md`）
- [x] 新增 `LlmAgentBehavior`（OpenAI 兼容 chat/completions）
- [x] 新增 `config.toml` 配置读取与 `AGENT_WORLD_LLM_SYSTEM_PROMPT` 默认值
- [x] 新增 LLM 决策解析与失败降级（`Wait`）
- [x] 补充单元测试并通过 `cargo test -p agent_world`
- [x] 新增可运行 LLM demo：`world_llm_agent_demo`（`AgentRunner + LlmAgentBehavior`）
- [x] 在线 viewer 支持 LLM 决策驱动（`world_viewer_live --llm`）
- [x] 修复 viewer 3D 相机拖拽输入兼容性（支持触控板/Shift+左键平移）

### 9. 场景测试覆盖矩阵
- [x] 在 `doc/world-simulator/scenario-files.md` 补充“场景 → 测试目标”矩阵
- [x] 在 `doc/world-simulator/scenario-files.project.md` 记录矩阵任务与状态
- [x] 新增 `llm_bootstrap` 场景并接入 scenario 枚举/解析/测试矩阵

### 9. 背景故事物理一致性修订
- [x] 输出“背景故事物理一致性修订清单”并附加到设计文档（`doc/world-simulator.md`）
- [x] C1 尺寸口径统一（文档与默认配置一致）
- [x] C2 辐射源标度修订（发射强度与尺度关系可配置）
- [x] C3 采集场模型对齐（近邻源 + 背景场，含距离项）
- [x] C4 背景辐射守恒说明（`radiation_floor` 来源与边界）
- [x] C5 运动学约束补齐（最大位移/速度上限）
- [ ] C6 能耗参数重标定（`time_step_s`/`power_unit_j`/移动能耗联动）
- [ ] C7 热模型升级（与温差相关）
- [ ] C8 成分与放射性分布校准（默认占比回归可解释区间）
- [ ] C9 物理参数表固化（单位/范围/影响）
- [ ] T1-T4 物理一致性与回放回归测试补齐

## 依赖
- 基础语言与运行环境：Rust（Cargo workspace）
- 存储（本地文件、SQLite、或其他 KV/文档存储，待选）
- （可选）LLM/推理服务接入方式与预算策略（OpenAI 兼容 API、本地/远程、缓存、重试）

## 状态
- 当前阶段：M3（Agent 运行时与 SDK）**已完成**
- 下一步：M4（最小社会与经济）并行推进“背景故事物理一致性修订（C6 优先）”
- 最近更新：完成 C5 运动学约束补齐（可配置位移/速度上限 + 双路径拒绝测试，2026-02-07）
