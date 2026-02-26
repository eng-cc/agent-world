#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.recipe.smelter.iron_ingot";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const RECIPE_ID: &str = "recipe.smelter.iron_ingot";
const REQUIRED_FACTORY_MARKER: &str = "smelter";
const CONSUME_PER_BATCH: &[(&str, i64)] = &[("iron_ore", 4), ("carbon_fuel", 1)];
const PRODUCE_PER_BATCH: &[(&str, i64)] = &[("iron_ingot", 3)];
const BYPRODUCTS_PER_BATCH: &[(&str, i64)] = &[("slag", 1)];
const POWER_PER_BATCH: i64 = 8;
const DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_recipe_module_template.rs");
