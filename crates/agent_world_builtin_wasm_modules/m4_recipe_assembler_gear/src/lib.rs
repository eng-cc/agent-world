#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.recipe.assembler.gear";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const RECIPE_ID: &str = "recipe.assembler.gear";
const REQUIRED_FACTORY_MARKER: &str = "assembler";
const CONSUME_PER_BATCH: &[(&str, i64)] = &[("iron_ingot", 2)];
const PRODUCE_PER_BATCH: &[(&str, i64)] = &[("gear", 1)];
const BYPRODUCTS_PER_BATCH: &[(&str, i64)] = &[];
const POWER_PER_BATCH: i64 = 4;
const DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_recipe_module_template.rs");
