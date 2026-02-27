#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.recipe.assembler.sensor_pack";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const RECIPE_ID: &str = "recipe.assembler.sensor_pack";
const REQUIRED_FACTORY_MARKER: &str = "assembler";
const CONSUME_PER_BATCH: &[(&str, i64)] = &[("control_chip", 1), ("copper_wire", 2)];
const PRODUCE_PER_BATCH: &[(&str, i64)] = &[("sensor_pack", 1)];
const BYPRODUCTS_PER_BATCH: &[(&str, i64)] = &[("calibration_scrap", 1)];
const POWER_PER_BATCH: i64 = 8;
const DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_recipe_module_template.rs");
