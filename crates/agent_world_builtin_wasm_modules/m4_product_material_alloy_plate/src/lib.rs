#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.product.material.alloy_plate";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";
const PRODUCT_ID: &str = "alloy_plate";
const STACK_LIMIT: u32 = 400;
const TRADABLE: bool = true;
const QUALITY_LEVELS: &[&str] = &["industrial", "hardened"];

include!("../../_templates/m4_product_module_template.rs");
