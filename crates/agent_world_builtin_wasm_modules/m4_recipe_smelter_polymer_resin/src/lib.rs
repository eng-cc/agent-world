#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.recipe.smelter.polymer_resin";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const RECIPE_ID: &str = "recipe.smelter.polymer_resin";
const REQUIRED_FACTORY_MARKER: &str = "smelter";
const CONSUME_PER_BATCH: &[(&str, i64)] = &[("carbon_fuel", 2), ("silicate_ore", 2)];
const PRODUCE_PER_BATCH: &[(&str, i64)] = &[("polymer_resin", 2)];
const BYPRODUCTS_PER_BATCH: &[(&str, i64)] = &[("waste_resin", 1)];
const POWER_PER_BATCH: i64 = 7;
const DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_recipe_module_template.rs");
