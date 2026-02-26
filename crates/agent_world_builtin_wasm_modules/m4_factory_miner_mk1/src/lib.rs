#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.factory.miner.mk1";
const FACTORY_BUILD_DECISION_EMIT_KIND: &str = "economy.factory_build_decision";
const FACTORY_ID: &str = "factory.miner.mk1";
const FACTORY_CONSUME: &[(&str, i64)] = &[
    ("structural_frame", 8),
    ("circuit_board", 2),
    ("servo_motor", 2),
];
const FACTORY_MIN_POWER: i64 = 4;
const FACTORY_DURATION_TICKS: u32 = 1;

include!("../../_templates/m4_factory_module_template.rs");
