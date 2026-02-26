#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.factory.assembler.mk1";
const FACTORY_BUILD_DECISION_EMIT_KIND: &str = "economy.factory_build_decision";
const FACTORY_ID: &str = "factory.assembler.mk1";
const FACTORY_CONSUME: &[(&str, i64)] = &[
    ("structural_frame", 8),
    ("iron_ingot", 10),
    ("copper_wire", 8),
];
const FACTORY_MIN_POWER: i64 = 8;
const FACTORY_DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_factory_module_template.rs");
