#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.recipe.assembler.control_chip";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const RECIPE_ID: &str = "recipe.assembler.control_chip";
const REQUIRED_FACTORY_MARKER: &str = "assembler";
const CONSUME_PER_BATCH: &[(&str, i64)] = &[("copper_wire", 4), ("polymer_resin", 2)];
const PRODUCE_PER_BATCH: &[(&str, i64)] = &[("control_chip", 1)];
const BYPRODUCTS_PER_BATCH: &[(&str, i64)] = &[("waste_resin", 1)];
const POWER_PER_BATCH: i64 = 6;
const DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_recipe_module_template.rs");
