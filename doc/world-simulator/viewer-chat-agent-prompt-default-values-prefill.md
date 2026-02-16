# Agent World Simulator：Chat Panel Agent Prompt 默认值预填充输入框（设计文档）

## 目标
- 在 Chat Panel 的 `Agent Prompt Draft` 中，`system prompt`、`短期目标`、`长期目标` 输入框默认直接填入当前生效值（无 override 时填系统默认值）。
- 让用户进入后可直接编辑文本，不需要先从占位复制。
- 保持 `apply` 语义正确：未修改默认值时不产生无意义 override patch。

## 范围

### 范围内
- 草稿加载逻辑改为“生效值”加载：
  - 有 override：加载 override。
  - 无 override：加载系统默认值。
- `prompt_control.apply` patch 构造逻辑改造：
  - 当前无 override 且输入值等于默认值 -> 视为无改动。
  - 当前有 override 且输入值改回默认值 -> 发送清除 override（`Some(None)`）。
  - 输入框清空 -> 视为清除 override。
- 增加单元测试覆盖上述行为。
- 更新手册说明。

### 范围外
- 不改动后端协议结构。
- 不新增 preview / rollback 交互。
- 不引入本地持久化。

## 接口 / 数据
- 复用已有常量：
  - `DEFAULT_LLM_SYSTEM_PROMPT`
  - `DEFAULT_LLM_SHORT_TERM_GOAL`
  - `DEFAULT_LLM_LONG_TERM_GOAL`
- 新增字段 patch 辅助逻辑（按“当前 override + 默认值 + 输入值”三者计算 patch）。

## 里程碑
- M1：设计文档 + 项目管理文档。
- M2：输入框预填充与 patch 语义改造。
- M3：测试、手册、devlog、项目状态收口。

## 风险
- 风险：用户可能希望“保留 override 且值等于默认值”，但语义上通常无必要。
  - 缓解：将“输入为默认值”统一解释为“回归默认（清除 override）”，保证行为可预测。
- 风险：清空输入框与输入默认值都触发“清除 override”，两者路径重叠。
  - 缓解：文档明确该等价行为，减少认知负担。
