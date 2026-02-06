# Agent World Simulator：分块世界生成与碎片元素/化合物池（设计文档）

本分册定义“按探索触发”的世界分块生成流程，并补充碎片的几何与物理量模型（体积/密度/质量）、化合物主导组成、块状分布表达。

## 目标
- 按探索进度进行世界生成：**未探索区块不生成**，降低初始化与存储开销。
- 固定分块尺寸为 **20km × 20km × 10km**，形成稳定可复现的空间切片。
- 明确碎片物理量：每个碎片可由多个长方体块组成，并具备**体积、密度、质量**。
- 化学组成以**化合物为主**（非单质），并可映射回元素统计口径。
- 区块生成时一次性计算资源 `total/remaining`，保证开采过程只扣减不重算。
- 保持同一 `world_seed` 下的确定性（同 chunk 坐标得到同结果）。

## 范围

### In Scope
- 定义 chunk 坐标、边界和生命周期状态。
- 定义碎片块（长方体）几何表达与 1cm 最小单位约束。
- 定义碎片级体积/密度/质量计算规则。
- 定义化合物池与元素映射关系（统计口径）。
- 定义 chunk 级/碎片级资源总量与剩余量数据结构。
- 定义按探索触发的 chunk 生成步骤与种子策略。

### Out of Scope
- 连续体轨道动力学、碰撞、潮汐等高精度物理。
- 多线程并行生成与跨节点分布式锁。
- 完整冶炼工艺链（本阶段只定义“资源库存生成”，不定义加工玩法）。

## 接口 / 数据

### 分块常量与坐标
- `CHUNK_SIZE_X_CM = 2_000_000`（20 km）
- `CHUNK_SIZE_Y_CM = 2_000_000`（20 km）
- `CHUNK_SIZE_Z_CM = 1_000_000`（10 km）
- `ChunkCoord { x: i32, y: i32, z: i32 }`
- `ChunkBounds { min: GeoPos, max: GeoPos }`

说明：默认空间 `100km × 100km × 10km` 切分为 `5 × 5 × 1` 个 chunk。

### Chunk 生命周期
- `Unexplored`：未知区块，仅存在索引信息。
- `Generated`：已生成碎片几何、化学组成与资源量，可观测/开采。
- `Exhausted`：区块可开采资源耗尽（可选状态，用于优化查询）。

### 几何与物理量（新增）

#### 最小空间单位
- **1 cm**（`SPACE_UNIT_CM = 1`）
- 所有 block 尺寸、block 原点、chunk 内偏移均以 cm 整数表示。

#### 单块结构（长方体）
- `FragmentBlock`
  - `origin_cm: GridPosCm`（chunk 局部坐标）
  - `size_cm: CuboidSizeCm`（`x_cm/y_cm/z_cm`）
  - `density_kg_per_m3: i64`
  - `compounds: CompoundComposition`

约束：
- `size_cm.x_cm >= 1`
- `size_cm.y_cm >= 1`
- `size_cm.z_cm >= 1`
- block 之间不重叠（同一碎片内）

#### 体积/密度/质量公式
- `volume_cm3 = x_cm * y_cm * z_cm`
- `volume_m3 = volume_cm3 / 1_000_000`
- `mass_kg = density_kg_per_m3 * volume_m3`
- 工程落地建议存储整数克：
  - `mass_g = density_kg_per_m3 * volume_cm3 / 1000`

#### 碎片聚合物理量
- `fragment_volume_cm3 = Σ(block.volume_cm3)`
- `fragment_mass_g = Σ(block.mass_g)`
- `fragment_bulk_density_kg_per_m3 = fragment_mass_kg / fragment_volume_m3`

### 化学组成：化合物主导（新增）

#### 化合物池（示例）
- `FragmentCompoundKind`
  - `SilicateMatrix`（硅酸盐基质）
  - `IronNickelAlloy`（铁镍合金）
  - `WaterIce`（水冰）
  - `HydratedMineral`（含水矿物）
  - `CarbonaceousOrganic`（碳质有机复合物）
  - `SulfideOre`（硫化物矿）
  - `RareEarthOxide`（稀土氧化物）
  - `UraniumBearingOre`（铀矿相）
  - `ThoriumBearingOre`（钍矿相）

#### 组成表达
- `CompoundComposition`: `BTreeMap<FragmentCompoundKind, u32>`（ppm）
- `ElementComposition`: `BTreeMap<FragmentElementKind, u32>`（ppm，统计口径）

说明：
- 生成阶段优先采样**化合物组成**；
- 元素组成由化合物签名映射得到（用于资源统计、查询、开采规则）；
- 单质可作为极低概率特例，不作为默认主组成。

### 资源库存模型（生成即定量）
- `FragmentResourceBudget`
  - `total_by_element: BTreeMap<FragmentElementKind, i64>`
  - `remaining_by_element: BTreeMap<FragmentElementKind, i64>`
- `ChunkResourceBudget`
  - `total_by_element: BTreeMap<FragmentElementKind, i64>`
  - `remaining_by_element: BTreeMap<FragmentElementKind, i64>`

计算原则：
1. 先生成碎片及其 block 列表（长方体）；
2. 按 block 采样化合物组成（ppm）与密度；
3. 计算每个 block 的体积/质量；
4. 将化合物组成映射为元素组成；
5. 计算 `element_total = mass * composition * recoverability`；
6. 生成时写入 `total` 与 `remaining = total`，后续仅做扣减。

### 关键接口（建议）
- `fn chunk_coord_of(pos: GeoPos) -> ChunkCoord`
- `fn ensure_chunk_generated(coord: ChunkCoord, kernel: &mut WorldKernel)`
- `fn generate_chunk(seed: u64, coord: ChunkCoord, config: &WorldConfig) -> ChunkSnapshot`
- `fn chunk_seed(world_seed: u64, coord: ChunkCoord) -> u64`
- `fn infer_element_ppm(compounds: &CompoundComposition) -> ElementComposition`

### 运行时触发契约（与 observe/act 集成）
- `observe` 触发：当 Agent 进行观测时，先对“自身所在坐标 chunk”执行 `ensure_chunk_generated`，再构建 observation。
- `move` 触发：校验移动动作前，必须保证 `from_chunk` 与 `to_chunk` 已生成。
- `harvest/transfer/query` 触发：动作依赖 location 资源时，必须先确保该 location 所在 chunk 已生成。
- 统一顺序：`ensure_chunk_generated -> action validation -> action apply -> event append`。
- 一致性要求：同一 tick 内多个 Agent 命中同一未生成 chunk 时，只允许一次成功生成，其余请求复用结果。

### 持久化与回放契约（M2 对齐）

#### 快照新增字段（建议）
- `ChunkIndexSnapshot`
  - `states: BTreeMap<ChunkCoord, ChunkState>`
  - `version: u32`（chunk 生成规则版本）
  - `generated_count: u32`
- `ChunkResourceBudgetSnapshot`
  - 每个已生成 chunk 的 `total/remaining` 账本。

#### 事件新增类型（建议）
- `ChunkGenerationRequested { coord, cause }`
- `ChunkGenerated { coord, seed, fragment_count, block_count }`
- `ChunkGenerationSkipped { coord, reason }`

#### 回放规则
- 回放时以 journal 的 `ChunkGenerated` 为准，不再重新随机生成。
- 若缺失 `ChunkGenerated` 但存在后续资源扣减事件，视为数据不一致并中止回放。
- 分叉回放时保留历史 chunk 状态，新增生成事件仅追加在分叉点之后。

### 经济资源映射契约（与 M4 对齐）
为接入现有 `electricity/hardware/data` 三类核心资源，定义“化合物/元素 -> 经济资源”的最小精炼链：

- `RefineCompound`（动作）：输入 `compound_mass_g`，输出元素库存。
- `ManufactureHardware`（动作）：输入 Fe/Ni/Si/Cu 等元素库存，输出 `hardware`。
- `SynthesizeData`（动作，可选）：输入稀有元素 + 电力，输出高质量 `data`。
- `electricity` 作为加工消耗项，不直接由元素映射生成。

最小守恒规则：
- 质量守恒：输出元素总质量 `<=` 输入化合物质量。
- 能量约束：精炼与制造必须消耗 `electricity`。
- 账本约束：所有映射结果都落到 `ResourceDelta`，写入事件日志。

### 跨 chunk 边界一致性规则
- 归属规则：碎片按中心点归属 chunk；block 允许跨边界时，跨界部分记录为 `boundary_span`。
- 最小间距校验顺序：先校验本 chunk，再校验已生成邻接 chunk（26 邻域）。
- 邻块未生成时，记录 `BoundaryReservation`（边界保留信息）；邻块未来生成时必须消费该保留信息再放置碎片。
- 冲突解法：同距离冲突按 `(chunk_coord, fragment_id)` 字典序保留前者，确保确定性。

### 性能预算与降级策略
- `max_fragments_per_chunk`：单 chunk 碎片上限（默认建议 4_000）。
- `max_blocks_per_fragment`：单碎片 block 上限（默认建议 64）。
- `max_blocks_per_chunk`：单 chunk block 总量上限（默认建议 120_000）。
- 超限降级顺序：
  1. 合并相邻同成分 block；
  2. 下降 block 细分层级；
  3. 截断低价值碎片（保留高质量/高价值碎片）。
- 生成耗时预算：单 chunk 生成超时后中断并输出 `ChunkGenerationSkipped`（reason=`budget_exceeded`）。

### 验收标准（DoD）
- 同一 `world_seed + chunk_coord` 在不同运行中生成结果一致（碎片数量/资源账本一致）。
- 未访问 chunk 不进入 `Generated` 状态。
- 回放后 chunk 状态与资源账本与原运行一致。
- 精炼/制造链路满足质量守恒与电力约束。
- 在默认预算下，批量生成不出现 OOM 或极端耗时。

## 世界生成步骤
1. **初始化索引阶段**：创建 chunk 网格索引，状态置 `Unexplored`。
2. **引导区块阶段**：仅预生成 origin/初始基地所在 chunk。
3. **探索触发阶段**：观测/移动/任务访问坐标时调用 `ensure_chunk_generated`。
4. **区块种子阶段**：`chunk_seed = hash(world_seed, chunk_coord)` 派生随机源。
5. **碎片外形阶段**：在 chunk 内生成碎片骨架（位置 + 大小范围 + 最小间距）。
6. **块状离散阶段**：将碎片离散为若干长方体 block（最小单位 1cm）。
7. **化合物赋值阶段**：为每个 block 采样化合物组成（ppm）。
8. **物理量计算阶段**：计算 block 与碎片级体积/密度/质量。
9. **元素映射阶段**：由化合物组成推导元素统计分布。
10. **资源定量阶段**：写入碎片与 chunk 的 `total/remaining` 资源账本。
11. **提交与可见阶段**：写入 `WorldModel` 与 chunk 索引，状态切到 `Generated`。
12. **开采扣减阶段**：开采只减少 `remaining`，不重算 `total`。

## 里程碑
- **CG1**：完成分块生成与元素/化合物池设计文档、项目管理文档。
- **CG2**：实现 chunk 索引与按探索触发生成（最小可用闭环）。
- **CG3**：实现碎片块状物理模型（体积/密度/质量）与化合物组成。
- **CG4**：实现资源预算一次性写入与开采扣减守恒。
- **CG5**：补充回放一致性、场景联测与兼容迁移策略。

## 风险
- chunk 边界附近的最小间距约束需要考虑相邻 chunk，避免穿边重叠。
- block 粒度提升后，生成与序列化成本上升，需控制每碎片 block 数量上限。
- 化合物到元素映射若调整，会影响旧存档资源账本一致性。
- 质量公式采用整数近似时可能产生累计误差，需要统一舍入策略。
