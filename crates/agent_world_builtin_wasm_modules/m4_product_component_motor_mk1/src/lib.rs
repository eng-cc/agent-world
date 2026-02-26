#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.product.component.motor_mk1";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";
const PRODUCT_ID: &str = "motor_mk1";
const STACK_LIMIT: u32 = 128;
const TRADABLE: bool = true;
const QUALITY_LEVELS: &[&str] = &["industrial", "precision", "hardened"];

include!("../../_templates/m4_product_module_template.rs");
