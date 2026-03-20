#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.product.component.control_chip";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";
const PRODUCT_ID: &str = "control_chip";
const STACK_LIMIT: u32 = 200;
const TRADABLE: bool = true;
const QUALITY_LEVELS: &[&str] = &["industrial", "precision"];

include!("../../_templates/m4_product_module_template.rs");
