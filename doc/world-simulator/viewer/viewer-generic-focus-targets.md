# Viewer 通用聚焦目标（可扩展实体）

## 目标
- 为 viewer 自动化聚焦/选中能力提供统一 target 语法，避免仅支持 `agent/location`。
- 支持在 Web 闭环与本地自动化脚本中稳定聚焦到更多实体（如 `asset/power/chunk/fragment/module_visual`）。
- 降低后续扩展成本：新增实体后，尽量只需在一处补充 kind 映射与实体索引绑定。

## 范围

### 范围内
- 扩展 `viewer_automation` 的 target 解析与实体解析逻辑。
- 保持兼容旧语法：`first_agent`、`first_location`、`agent:<id>`、`location:<id>`。
- 新增通用语法：
  - `first:<kind>`
  - `<kind>:<id>`
- 新增/更新单元测试，覆盖新旧语法与多实体 kind 解析。

### 范围外
- 不改 Viewer 协议字段。
- 不改 WebTest API 对外方法名（继续使用 `focus/select/runSteps`）。
- 不实现“自动遍历所有实体截图”脚本框架（该能力可后续单独任务实现）。

## 接口 / 数据

### 目标语法
- 兼容语法：
  - `first_agent`、`first_location`
  - `agent:<id>`、`location:<id>`
- 通用语法：
  - `first:<kind>`
  - `<kind>:<id>`

### kind 映射（首批）
- `agent`
- `location`
- `asset`
- `module_visual`（映射到 Asset 详情分支）
- `power_plant`
- `power_storage`
- `chunk`
- `fragment`（基于 location 索引，优先匹配 `frag-` 前缀）

### 可扩展点
- 将 kind 归一化与 scene 索引绑定集中在 `viewer_automation` 单点 resolver。
- 后续新增实体时，优先只改该 resolver，不再分散改 parser/step 执行分支。

## 里程碑
- M1：设计文档与项目管理文档完成。
- M2：`viewer_automation` 支持通用 kind target。
- M3：测试覆盖新旧语法并通过定向回归。
- M4：项目管理状态与 devlog 收口。

## 风险
- kind 写错导致目标无法解析或无法命中。
  - 缓解：保持兼容旧语法，并在测试中覆盖常见别名。
- `fragment` 与 `location` 共享底层索引，可能出现语义混淆。
  - 缓解：`fragment` 使用独立选择类型，并保持 “first_fragment 优先 `frag-` 前缀” 规则。
- 后续新增实体未接入 resolver，仍会出现“可渲染但不可聚焦”。
  - 缓解：在该文档与项目管理文档中明确“新增实体需补 resolver”作为固定检查项。
