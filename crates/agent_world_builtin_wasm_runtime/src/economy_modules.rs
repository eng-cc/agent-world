use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{
    empty_output, encode_output, ModuleCallInput, ModuleEmit, ModuleOutput,
    M4_FACTORY_ASSEMBLER_MODULE_ID, M4_FACTORY_MINER_MODULE_ID, M4_FACTORY_SMELTER_MODULE_ID,
    M4_PRODUCT_CONTROL_CHIP_MODULE_ID, M4_PRODUCT_IRON_INGOT_MODULE_ID,
    M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID, M4_PRODUCT_MOTOR_MODULE_ID,
    M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID, M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID,
    M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID, M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID,
    M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID, M4_RECIPE_SMELT_IRON_MODULE_ID,
};

const FACTORY_BUILD_DECISION_EMIT_KIND: &str = "economy.factory_build_decision";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct MaterialStackData {
    kind: String,
    amount: i64,
}

impl MaterialStackData {
    fn new(kind: impl Into<String>, amount: i64) -> Self {
        Self {
            kind: kind.into(),
            amount,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct FactoryBuildRequestData {
    factory_id: String,
    #[allow(dead_code)]
    site_id: String,
    #[allow(dead_code)]
    builder: String,
    #[serde(default)]
    available_inputs: Vec<MaterialStackData>,
    available_power: i64,
}

#[derive(Debug, Clone, Serialize)]
struct FactoryBuildDecisionData {
    accepted: bool,
    #[serde(default)]
    consume: Vec<MaterialStackData>,
    duration_ticks: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reject_reason: Option<String>,
}

impl FactoryBuildDecisionData {
    fn accepted(consume: Vec<MaterialStackData>, duration_ticks: u32) -> Self {
        Self {
            accepted: true,
            consume,
            duration_ticks,
            reject_reason: None,
        }
    }

    fn rejected(reason: impl Into<String>) -> Self {
        Self {
            accepted: false,
            consume: Vec::new(),
            duration_ticks: 0,
            reject_reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RecipeExecutionRequestData {
    recipe_id: String,
    factory_id: String,
    desired_batches: u32,
    #[serde(default)]
    available_inputs: Vec<MaterialStackData>,
    available_power: i64,
    #[allow(dead_code)]
    deterministic_seed: u64,
}

#[derive(Debug, Clone, Serialize)]
struct RecipeExecutionPlanData {
    accepted_batches: u32,
    #[serde(default)]
    consume: Vec<MaterialStackData>,
    #[serde(default)]
    produce: Vec<MaterialStackData>,
    #[serde(default)]
    byproducts: Vec<MaterialStackData>,
    power_required: i64,
    duration_ticks: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reject_reason: Option<String>,
}

impl RecipeExecutionPlanData {
    fn accepted(
        accepted_batches: u32,
        consume: Vec<MaterialStackData>,
        produce: Vec<MaterialStackData>,
        byproducts: Vec<MaterialStackData>,
        power_required: i64,
        duration_ticks: u32,
    ) -> Self {
        Self {
            accepted_batches,
            consume,
            produce,
            byproducts,
            power_required,
            duration_ticks,
            reject_reason: None,
        }
    }

    fn rejected(reason: impl Into<String>) -> Self {
        Self {
            accepted_batches: 0,
            consume: Vec::new(),
            produce: Vec::new(),
            byproducts: Vec::new(),
            power_required: 0,
            duration_ticks: 0,
            reject_reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ProductValidationData {
    product_id: String,
    accepted: bool,
    #[serde(default)]
    notes: Vec<String>,
    stack_limit: u32,
    tradable: bool,
    #[serde(default)]
    quality_levels: Vec<String>,
}

struct FactoryRule {
    factory_id: &'static str,
    consume: &'static [(&'static str, i64)],
    min_power: i64,
    duration_ticks: u32,
}

struct RecipeRule {
    recipe_id: &'static str,
    required_factory_marker: &'static str,
    consume_per_batch: &'static [(&'static str, i64)],
    produce_per_batch: &'static [(&'static str, i64)],
    byproducts_per_batch: &'static [(&'static str, i64)],
    power_per_batch: i64,
    duration_ticks: u32,
}

struct ProductRule {
    product_id: &'static str,
    stack_limit: u32,
    tradable: bool,
    quality_levels: &'static [&'static str],
}

pub(crate) fn build_economy_module_output(input: &ModuleCallInput) -> Option<Vec<u8>> {
    match input.ctx.module_id.as_str() {
        M4_FACTORY_MINER_MODULE_ID
        | M4_FACTORY_SMELTER_MODULE_ID
        | M4_FACTORY_ASSEMBLER_MODULE_ID => Some(build_factory_output(input)),
        M4_RECIPE_SMELT_IRON_MODULE_ID
        | M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID
        | M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID
        | M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID
        | M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID
        | M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID => Some(build_recipe_output(input)),
        M4_PRODUCT_IRON_INGOT_MODULE_ID
        | M4_PRODUCT_CONTROL_CHIP_MODULE_ID
        | M4_PRODUCT_MOTOR_MODULE_ID
        | M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID => Some(build_product_output(input)),
        _ => None,
    }
}

fn build_factory_output(input: &ModuleCallInput) -> Vec<u8> {
    let Some(request) = decode_action::<FactoryBuildRequestData>(input) else {
        return encode_output(empty_output());
    };
    let Some(rule) = factory_rule_for_module(input.ctx.module_id.as_str()) else {
        return encode_output(empty_output());
    };

    if request.factory_id != rule.factory_id {
        return emit_factory_decision(FactoryBuildDecisionData::rejected(format!(
            "factory_id mismatch expected={} got={}",
            rule.factory_id, request.factory_id
        )));
    }

    if request.available_power < rule.min_power {
        return emit_factory_decision(FactoryBuildDecisionData::rejected(format!(
            "insufficient power required={} available={}",
            rule.min_power, request.available_power
        )));
    }

    let available = as_inventory(&request.available_inputs);
    if let Some(reason) = first_missing_material(rule.consume, &available) {
        return emit_factory_decision(FactoryBuildDecisionData::rejected(reason));
    }

    emit_factory_decision(FactoryBuildDecisionData::accepted(
        stacks_from_spec(rule.consume),
        rule.duration_ticks,
    ))
}

fn build_recipe_output(input: &ModuleCallInput) -> Vec<u8> {
    let Some(request) = decode_action::<RecipeExecutionRequestData>(input) else {
        return encode_output(empty_output());
    };
    let Some(rule) = recipe_rule_for_module(input.ctx.module_id.as_str()) else {
        return encode_output(empty_output());
    };

    if request.recipe_id != rule.recipe_id {
        return emit_recipe_plan(RecipeExecutionPlanData::rejected(format!(
            "recipe_id mismatch expected={} got={}",
            rule.recipe_id, request.recipe_id
        )));
    }

    if !request
        .factory_id
        .to_ascii_lowercase()
        .contains(rule.required_factory_marker)
    {
        return emit_recipe_plan(RecipeExecutionPlanData::rejected(format!(
            "factory {} incompatible with {}",
            request.factory_id, rule.recipe_id
        )));
    }

    if request.desired_batches == 0 {
        return emit_recipe_plan(RecipeExecutionPlanData::rejected(
            "desired_batches must be > 0",
        ));
    }

    let available = as_inventory(&request.available_inputs);
    let mut accepted_batches = request.desired_batches;

    for (kind, amount) in rule.consume_per_batch {
        if *amount <= 0 {
            return emit_recipe_plan(RecipeExecutionPlanData::rejected(format!(
                "invalid recipe consume amount kind={} amount={}",
                kind, amount
            )));
        }

        let available_amount = available.get(*kind).copied().unwrap_or(0).max(0);
        let material_max = (available_amount / *amount).max(0) as u32;
        accepted_batches = accepted_batches.min(material_max);
    }

    if rule.power_per_batch > 0 {
        let power_max = (request.available_power.max(0) / rule.power_per_batch) as u32;
        accepted_batches = accepted_batches.min(power_max);
    }

    if accepted_batches == 0 {
        let reason = first_recipe_bottleneck(rule, &request, &available);
        return emit_recipe_plan(RecipeExecutionPlanData::rejected(reason));
    }

    let consume = scale_stacks(rule.consume_per_batch, accepted_batches);
    let produce = scale_stacks(rule.produce_per_batch, accepted_batches);
    let byproducts = scale_stacks(rule.byproducts_per_batch, accepted_batches);
    let power_required = rule.power_per_batch.saturating_mul(accepted_batches as i64);

    emit_recipe_plan(RecipeExecutionPlanData::accepted(
        accepted_batches,
        consume,
        produce,
        byproducts,
        power_required,
        rule.duration_ticks,
    ))
}

fn build_product_output(input: &ModuleCallInput) -> Vec<u8> {
    let Some(rule) = product_rule_for_module(input.ctx.module_id.as_str()) else {
        return encode_output(empty_output());
    };
    let stack = decode_action::<MaterialStackData>(input)
        .unwrap_or_else(|| MaterialStackData::new(rule.product_id, 0));

    let mut notes = Vec::new();
    let mut accepted = true;

    if stack.kind != rule.product_id {
        accepted = false;
        notes.push(format!(
            "product kind mismatch expected={} got={}",
            rule.product_id, stack.kind
        ));
    }
    if stack.amount <= 0 {
        accepted = false;
        notes.push("stack amount must be > 0".to_string());
    }
    if stack.amount > rule.stack_limit as i64 {
        accepted = false;
        notes.push(format!(
            "stack exceeds limit amount={} limit={}",
            stack.amount, rule.stack_limit
        ));
    }

    let payload = serde_json::to_value(ProductValidationData {
        product_id: rule.product_id.to_string(),
        accepted,
        notes,
        stack_limit: rule.stack_limit,
        tradable: rule.tradable,
        quality_levels: rule
            .quality_levels
            .iter()
            .map(|item| item.to_string())
            .collect(),
    })
    .unwrap_or_else(|_| serde_json::json!({ "accepted": false }));

    encode_output(ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: PRODUCT_VALIDATION_EMIT_KIND.to_string(),
            payload,
        }],
        output_bytes: 512,
    })
}

fn emit_factory_decision(decision: FactoryBuildDecisionData) -> Vec<u8> {
    let payload = serde_json::to_value(decision).unwrap_or_else(|_| {
        serde_json::json!({
            "accepted": false,
            "consume": [],
            "duration_ticks": 0,
            "reject_reason": "serialize decision failed"
        })
    });
    encode_output(ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: FACTORY_BUILD_DECISION_EMIT_KIND.to_string(),
            payload,
        }],
        output_bytes: 512,
    })
}

fn emit_recipe_plan(plan: RecipeExecutionPlanData) -> Vec<u8> {
    let payload = serde_json::to_value(plan).unwrap_or_else(|_| {
        serde_json::json!({
            "accepted_batches": 0,
            "consume": [],
            "produce": [],
            "byproducts": [],
            "power_required": 0,
            "duration_ticks": 0,
            "reject_reason": "serialize plan failed"
        })
    });
    encode_output(ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: RECIPE_EXECUTION_PLAN_EMIT_KIND.to_string(),
            payload,
        }],
        output_bytes: 512,
    })
}

fn decode_action<T: for<'de> Deserialize<'de>>(input: &ModuleCallInput) -> Option<T> {
    let bytes = input.action.as_deref()?;
    serde_cbor::from_slice(bytes).ok()
}

fn as_inventory(stacks: &[MaterialStackData]) -> BTreeMap<&str, i64> {
    let mut map = BTreeMap::new();
    for stack in stacks {
        if stack.kind.trim().is_empty() {
            continue;
        }
        *map.entry(stack.kind.as_str()).or_insert(0) += stack.amount;
    }
    map
}

fn first_missing_material(
    required: &'static [(&'static str, i64)],
    available: &BTreeMap<&str, i64>,
) -> Option<String> {
    for (kind, amount) in required {
        let available_amount = available.get(*kind).copied().unwrap_or(0);
        if available_amount < *amount {
            return Some(format!(
                "insufficient material kind={} required={} available={}",
                kind, amount, available_amount
            ));
        }
    }
    None
}

fn first_recipe_bottleneck(
    rule: &RecipeRule,
    request: &RecipeExecutionRequestData,
    available: &BTreeMap<&str, i64>,
) -> String {
    for (kind, amount) in rule.consume_per_batch {
        let available_amount = available.get(*kind).copied().unwrap_or(0);
        if available_amount < *amount {
            return format!(
                "insufficient material kind={} required_per_batch={} available={}",
                kind, amount, available_amount
            );
        }
    }
    if rule.power_per_batch > request.available_power {
        return format!(
            "insufficient power required_per_batch={} available={}",
            rule.power_per_batch, request.available_power
        );
    }
    "requested batches reduced to zero by constraints".to_string()
}

fn stacks_from_spec(items: &'static [(&'static str, i64)]) -> Vec<MaterialStackData> {
    items
        .iter()
        .map(|(kind, amount)| MaterialStackData::new(*kind, *amount))
        .collect()
}

fn scale_stacks(items: &'static [(&'static str, i64)], factor: u32) -> Vec<MaterialStackData> {
    items
        .iter()
        .map(|(kind, amount)| MaterialStackData::new(*kind, amount.saturating_mul(factor as i64)))
        .collect()
}

fn factory_rule_for_module(module_id: &str) -> Option<&'static FactoryRule> {
    match module_id {
        M4_FACTORY_MINER_MODULE_ID => Some(&FactoryRule {
            factory_id: "factory.miner.mk1",
            consume: &[
                ("structural_frame", 8),
                ("circuit_board", 2),
                ("servo_motor", 2),
            ],
            min_power: 4,
            duration_ticks: 1,
        }),
        M4_FACTORY_SMELTER_MODULE_ID => Some(&FactoryRule {
            factory_id: "factory.smelter.mk1",
            consume: &[
                ("structural_frame", 12),
                ("heat_coil", 4),
                ("refractory_brick", 6),
            ],
            min_power: 6,
            duration_ticks: 1,
        }),
        M4_FACTORY_ASSEMBLER_MODULE_ID => Some(&FactoryRule {
            factory_id: "factory.assembler.mk1",
            consume: &[
                ("structural_frame", 8),
                ("iron_ingot", 10),
                ("copper_wire", 8),
            ],
            min_power: 8,
            duration_ticks: 1,
        }),
        _ => None,
    }
}

fn recipe_rule_for_module(module_id: &str) -> Option<&'static RecipeRule> {
    match module_id {
        M4_RECIPE_SMELT_IRON_MODULE_ID => Some(&RecipeRule {
            recipe_id: "recipe.smelter.iron_ingot",
            required_factory_marker: "smelter",
            consume_per_batch: &[("iron_ore", 4), ("carbon_fuel", 1)],
            produce_per_batch: &[("iron_ingot", 3)],
            byproducts_per_batch: &[("slag", 1)],
            power_per_batch: 8,
            duration_ticks: 1,
        }),
        M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID => Some(&RecipeRule {
            recipe_id: "recipe.smelter.copper_wire",
            required_factory_marker: "smelter",
            consume_per_batch: &[("copper_ore", 3)],
            produce_per_batch: &[("copper_wire", 4)],
            byproducts_per_batch: &[],
            power_per_batch: 6,
            duration_ticks: 1,
        }),
        M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID => Some(&RecipeRule {
            recipe_id: "recipe.assembler.gear",
            required_factory_marker: "assembler",
            consume_per_batch: &[("iron_ingot", 2)],
            produce_per_batch: &[("gear", 1)],
            byproducts_per_batch: &[],
            power_per_batch: 4,
            duration_ticks: 1,
        }),
        M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID => Some(&RecipeRule {
            recipe_id: "recipe.assembler.control_chip",
            required_factory_marker: "assembler",
            consume_per_batch: &[("copper_wire", 4), ("polymer_resin", 2)],
            produce_per_batch: &[("control_chip", 1)],
            byproducts_per_batch: &[("waste_resin", 1)],
            power_per_batch: 6,
            duration_ticks: 1,
        }),
        M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID => Some(&RecipeRule {
            recipe_id: "recipe.assembler.motor_mk1",
            required_factory_marker: "assembler",
            consume_per_batch: &[("gear", 2), ("copper_wire", 3)],
            produce_per_batch: &[("motor_mk1", 1)],
            byproducts_per_batch: &[],
            power_per_batch: 7,
            duration_ticks: 1,
        }),
        M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID => Some(&RecipeRule {
            recipe_id: "recipe.assembler.logistics_drone",
            required_factory_marker: "assembler",
            consume_per_batch: &[("motor_mk1", 2), ("control_chip", 1), ("iron_ingot", 2)],
            produce_per_batch: &[("logistics_drone", 1)],
            byproducts_per_batch: &[("assembly_scrap", 1)],
            power_per_batch: 12,
            duration_ticks: 1,
        }),
        _ => None,
    }
}

fn product_rule_for_module(module_id: &str) -> Option<&'static ProductRule> {
    match module_id {
        M4_PRODUCT_IRON_INGOT_MODULE_ID => Some(&ProductRule {
            product_id: "iron_ingot",
            stack_limit: 500,
            tradable: true,
            quality_levels: &["industrial", "refined"],
        }),
        M4_PRODUCT_CONTROL_CHIP_MODULE_ID => Some(&ProductRule {
            product_id: "control_chip",
            stack_limit: 200,
            tradable: true,
            quality_levels: &["industrial", "precision"],
        }),
        M4_PRODUCT_MOTOR_MODULE_ID => Some(&ProductRule {
            product_id: "motor_mk1",
            stack_limit: 128,
            tradable: true,
            quality_levels: &["industrial", "precision", "hardened"],
        }),
        M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID => Some(&ProductRule {
            product_id: "logistics_drone",
            stack_limit: 32,
            tradable: true,
            quality_levels: &["baseline", "field_ready", "fleet_grade"],
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModuleContext;

    fn encode_call_input(module_id: &str, action_payload: serde_json::Value) -> ModuleCallInput {
        ModuleCallInput {
            ctx: ModuleContext {
                module_id: module_id.to_string(),
                time: 1,
            },
            event: None,
            action: Some(
                serde_cbor::to_vec(&action_payload).expect("encode action payload to cbor"),
            ),
            state: None,
        }
    }

    #[test]
    fn factory_assembler_requires_smelter_outputs() {
        let request = serde_json::json!({
            "factory_id": "factory.assembler.mk1",
            "site_id": "site-a",
            "builder": "agent-a",
            "available_inputs": [
                {"kind":"structural_frame","amount":8},
                {"kind":"iron_ingot","amount":10},
                {"kind":"copper_wire","amount":8}
            ],
            "available_power": 20
        });
        let input = encode_call_input(M4_FACTORY_ASSEMBLER_MODULE_ID, request);
        let output_bytes = build_factory_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        assert_eq!(output.emits[0].kind, FACTORY_BUILD_DECISION_EMIT_KIND);
        assert_eq!(output.emits[0].payload["accepted"], serde_json::json!(true));
    }

    #[test]
    fn recipe_drone_rejects_without_control_chip() {
        let request = serde_json::json!({
            "recipe_id": "recipe.assembler.logistics_drone",
            "factory_id": "factory.assembler.mk1",
            "desired_batches": 1,
            "available_inputs": [
                {"kind":"motor_mk1","amount":2},
                {"kind":"iron_ingot","amount":2}
            ],
            "available_power": 40,
            "deterministic_seed": 7
        });
        let input = encode_call_input(M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID, request);
        let output_bytes = build_recipe_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        assert_eq!(output.emits[0].kind, RECIPE_EXECUTION_PLAN_EMIT_KIND);
        assert_eq!(
            output.emits[0].payload["accepted_batches"],
            serde_json::json!(0)
        );
        assert_eq!(
            output.emits[0].payload["reject_reason"],
            serde_json::json!(
                "insufficient material kind=control_chip required_per_batch=1 available=0"
            )
        );
    }

    #[test]
    fn product_module_enforces_stack_limit() {
        let stack = serde_json::json!({
            "kind": "logistics_drone",
            "amount": 64
        });
        let input = encode_call_input(M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID, stack);
        let output_bytes = build_product_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        assert_eq!(output.emits[0].kind, PRODUCT_VALIDATION_EMIT_KIND);
        assert_eq!(
            output.emits[0].payload["accepted"],
            serde_json::json!(false)
        );
    }

    #[test]
    fn route_unknown_module_returns_none() {
        let input = ModuleCallInput {
            ctx: ModuleContext {
                module_id: "unknown.module".to_string(),
                time: 1,
            },
            event: None,
            action: None,
            state: None,
        };
        assert!(build_economy_module_output(&input).is_none());
    }
}
