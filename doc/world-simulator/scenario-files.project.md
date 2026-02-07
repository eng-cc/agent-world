# Agent World Simulator：场景文件化（项目管理文档）

## 任务拆解
- [x] F1 输出设计文档与项目管理文档
- [x] F2 迁移所有内置场景到 JSON 文件并接入加载逻辑
- [x] F3 更新测试、示例与文档说明
- [x] F4 world_init_demo 支持加载场景文件
- [x] F5 补充“场景 → 测试目标”覆盖矩阵并对齐现有测试命名
- [x] F6 新增 `llm_bootstrap` 场景并接入枚举/解析/矩阵/测试

## 依赖
- `WorldScenario` / `WorldInitConfig`
- `serde_json` 场景解析

## 状态
- 当前阶段：已完成
- 最近更新：新增 `llm_bootstrap` 场景并补齐覆盖测试（2026-02-06）
