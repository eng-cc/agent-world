# Doc 目录结构与内容时效复核（Round 3，2026-02-24）设计文档

## 目标
- 全量复核 `doc/` 目录文档，确保主题文档按目录聚类，降低顶层混放成本。
- 将已完成且被后续轮次替代的治理文档迁移到对应 `archive/` 目录。
- 维持“活跃基线文档可达、历史文档可追溯”的双层结构。

## 范围

### In Scope
- 全量读取 `doc/**/*.md`，重点复核顶层文档与新增专题文档。
- 新增目录并迁移文档：
  - `doc/nonviewer/`（non-viewer 专题）
  - `doc/engineering/`（工程治理专题）
  - `doc/nonviewer/archive/`
  - `doc/engineering/archive/`
- 将明确被后续阶段替代的文档归档，并在文件头标注归档告警与日期。
- 修复非 `doc/devlog` 文档中的路径引用并更新 `doc/README.md`。

### Out of Scope
- 不改写 `doc/devlog` 历史内容（仅追加当天任务日志）。
- 不改动业务代码语义。
- 不处理 `third_party/` 文档内容。

## 接口 / 数据
- 新目录：
  - `doc/nonviewer/`
  - `doc/nonviewer/archive/`
  - `doc/engineering/`
  - `doc/engineering/archive/`
- 文档迁移规则：
  - 顶层 nonviewer 文档迁移至 `doc/nonviewer/`
  - 顶层 Rust 超限拆分文档迁移至 `doc/engineering/`
  - 顶层 world-simulator 专题文档迁移至 `doc/world-simulator/`
- 归档标识模板：
  - `> [!WARNING]`
  - `> 该文档已归档，仅供历史追溯，不再作为当前实现依据。`
  - `> 归档日期：2026-02-24`

## 里程碑
- R3-0：输出 round3 设计/项目管理文档，冻结迁移与归档标准。
- R3-1：完成目录新增与文档迁移。
- R3-2：完成老旧文档归档与告警标注。
- R3-3：完成引用修复、目录索引更新、回归校验与 devlog 收口。

## 风险
- 风险：误归档仍可作为当前实施依据的文档。
  - 缓解：仅归档“被后续轮次替代且任务已收口”的文档；边界场景优先保留。
- 风险：路径迁移导致文档断链。
  - 缓解：迁移后执行全量路径扫描，修复非 `doc/devlog` 文档链接。
- 风险：目录迁移后入口不清晰。
  - 缓解：同步更新 `doc/README.md`，新增目录分层与专题入口说明。
