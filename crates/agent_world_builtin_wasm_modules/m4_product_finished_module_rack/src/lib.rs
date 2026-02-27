#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = "m4.product.finished.module_rack";
const PRODUCT_VALIDATION_EMIT_KIND: &str = "economy.product_validation";
const PRODUCT_ID: &str = "module_rack";
const STACK_LIMIT: u32 = 48;
const TRADABLE: bool = true;
const QUALITY_LEVELS: &[&str] = &["industrial", "governance_ready"];

include!("../../_templates/m4_product_module_template.rs");
