# Agent World: 基础 CI 门禁移除 Builtin Wasm Hash 校验（项目管理文档）

## 任务拆解
- [x] T1 建档：设计文档与项目管理文档落地
- [x] T2 修改基础门禁脚本，移除 m1/m4/m5 hash 校验
- [x] T3 同步测试手册口径
- [x] T4 验证与收口

## 依赖
- `scripts/ci-tests.sh`
- `testing-manual.md`
- `.github/workflows/rust.yml`

## 状态
- 当前阶段：已完成（T1~T4）
- 最近更新：T4 完成，改造收口（2026-02-21）
