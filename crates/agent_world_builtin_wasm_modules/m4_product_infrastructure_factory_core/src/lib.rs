#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.product.infrastructure.factory_core";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";
const PRODUCT_ID: &str = "factory_core";
const STACK_LIMIT: u32 = 24;
const TRADABLE: bool = false;
const QUALITY_LEVELS: &[&str] = &["critical", "governance"];

include!("../../_templates/m4_product_module_template.rs");
