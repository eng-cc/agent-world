#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.recipe.assembler.logistics_drone";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const RECIPE_ID: &str = "recipe.assembler.logistics_drone";
const REQUIRED_FACTORY_MARKER: &str = "assembler";
const CONSUME_PER_BATCH: &[(&str, i64)] =
    &[("motor_mk1", 2), ("control_chip", 1), ("iron_ingot", 2)];
const PRODUCE_PER_BATCH: &[(&str, i64)] = &[("logistics_drone", 1)];
const BYPRODUCTS_PER_BATCH: &[(&str, i64)] = &[("assembly_scrap", 1)];
const POWER_PER_BATCH: i64 = 12;
const DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_recipe_module_template.rs");
