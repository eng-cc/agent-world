#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.product.finished.logistics_drone";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";
const PRODUCT_ID: &str = "logistics_drone";
const STACK_LIMIT: u32 = 32;
const TRADABLE: bool = true;
const QUALITY_LEVELS: &[&str] = &["baseline", "field_ready", "fleet_grade"];

include!("../../_templates/m4_product_module_template.rs");
