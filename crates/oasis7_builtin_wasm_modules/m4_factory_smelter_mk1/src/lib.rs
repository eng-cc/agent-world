#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.factory.smelter.mk1";
const FACTORY_BUILD_DECISION_EMIT_KIND: &str = "economy.factory_build_decision";
const FACTORY_ID: &str = "factory.smelter.mk1";
const FACTORY_CONSUME: &[(&str, i64)] = &[
    ("structural_frame", 12),
    ("heat_coil", 4),
    ("refractory_brick", 6),
];
const FACTORY_MIN_POWER: i64 = 6;
const FACTORY_DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_factory_module_template.rs");
