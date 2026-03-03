# Agent World Runtime：无限时长运行的序列号滚动与数值防溢出

## 目标
- 让 Runtime 在超长时间运行场景下保持可持续，不因计数器上溢导致 panic 或静默回绕。
- 对关键数值累加路径增加防溢出策略，避免 release 模式下出现不可见错误。
- 保持现有协议与数据结构兼容，优先做增量增强，不引入大规模类型迁移。

## 范围

### In Scope
- Runtime 内四类序列计数器增强：
  - `next_event_id`
  - `next_action_id`
  - `next_intent_id`
  - `next_proposal_id`
- 为序列计数增加 era（代际）状态，在 `u64::MAX` 时执行“era+1 + seq 重置”。
- snapshot 持久化增加序列 era 字段，并保持对旧快照的反序列化兼容。
- 修复关键未保护数值加法和窄化转换风险（`len as u32`、`u64 as i64`）。

### Out of Scope
- 全仓库引入 `BigInt`。
- 全量把 `u64` ID 类型升级为复合结构并改动外部协议字段。
- 改造历史 journal 结构或追加全量历史压缩系统。

## 接口/数据
- `Snapshot` 新增（默认 0）：
  - `event_id_era`
  - `action_id_era`
  - `intent_id_era`
  - `proposal_id_era`
- `World` 内部新增对应 era 状态，并在分配 ID 时执行滚动逻辑：
  - 正常：`seq += 1`
  - 边界：`seq == u64::MAX` 后下一次分配触发 `era = era + 1, seq = 1`
- 关键防溢出改造：
  - 资源与规则聚合加法改为 `saturating_add`。
  - 模块输出条目数量比较改为 `u32::try_from(len)`。
  - `snapshot.state.time -> i64` 改为 `i64::try_from` 并在越界时报错。

## 里程碑
- M0：建档与任务拆解完成。
- M1：序列滚动（era + seq）与 snapshot 兼容落地。
- M2：关键数值防溢出修复（累加、窄化转换）。
- M3：定向回归测试通过并文档收口。

## 风险
- 新增 snapshot 字段可能影响旧数据兼容。
  - 缓解：新增字段全部 `serde(default)`，旧快照按默认 era=0 读取。
- 序列跨 era 后，纯 `u64` ID 在极端远期会出现值复用。
  - 缓解：在 runtime 内持久化 era 并持续推进；当前阶段先解决“溢出失效”而非“全链路复合 ID 改造”。
- 饱和加法可能掩盖极端异常增长。
  - 缓解：先保证系统稳定运行，后续可在监控中加入“接近上限”告警。

## 当前状态
- 截至 2026-02-22：M0（建档）、M1（序列滚动与快照兼容）、M2（数值防溢出加固）、M3（回归与收口）均已完成。
