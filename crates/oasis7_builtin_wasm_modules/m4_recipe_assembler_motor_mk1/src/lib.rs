#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.recipe.assembler.motor_mk1";
const RECIPE_EXECUTION_PLAN_EMIT_KIND: &str = "economy.recipe_execution_plan";
const RECIPE_ID: &str = "recipe.assembler.motor_mk1";
const REQUIRED_FACTORY_MARKER: &str = "assembler";
const CONSUME_PER_BATCH: &[(&str, i64)] = &[("gear", 2), ("copper_wire", 3)];
const PRODUCE_PER_BATCH: &[(&str, i64)] = &[("motor_mk1", 1)];
const BYPRODUCTS_PER_BATCH: &[(&str, i64)] = &[];
const POWER_PER_BATCH: i64 = 7;
const DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_recipe_module_template.rs");
