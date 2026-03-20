#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.product.component.sensor_pack";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";
const PRODUCT_ID: &str = "sensor_pack";
const STACK_LIMIT: u32 = 96;
const TRADABLE: bool = true;
const QUALITY_LEVELS: &[&str] = &["precision", "survey"];

include!("../../_templates/m4_product_module_template.rs");
