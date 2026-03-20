#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.product.material.iron_ingot";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";
const PRODUCT_ID: &str = "iron_ingot";
const STACK_LIMIT: u32 = 500;
const TRADABLE: bool = true;
const QUALITY_LEVELS: &[&str] = &["industrial", "refined"];

include!("../../_templates/m4_product_module_template.rs");
