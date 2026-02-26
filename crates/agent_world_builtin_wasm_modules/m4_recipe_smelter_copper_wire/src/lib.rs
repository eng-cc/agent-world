#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.recipe.smelter.copper_wire";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const RECIPE_ID: &str = "recipe.smelter.copper_wire";
const REQUIRED_FACTORY_MARKER: &str = "smelter";
const CONSUME_PER_BATCH: &[(&str, i64)] = &[("copper_ore", 3)];
const PRODUCE_PER_BATCH: &[(&str, i64)] = &[("copper_wire", 4)];
const BYPRODUCTS_PER_BATCH: &[(&str, i64)] = &[];
const POWER_PER_BATCH: i64 = 6;
const DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_recipe_module_template.rs");
