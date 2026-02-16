//! Tests for the runtime module.

pub(super) fn pos(x: f64, y: f64) -> crate::geometry::GeoPos {
    crate::geometry::GeoPos {
        x_cm: x,
        y_cm: y,
        z_cm: 0.0,
    }
}

mod agent_default_modules;
mod audit;
mod basic;
mod body;
mod builtin_wasm_materializer;
mod economy;
mod economy_bootstrap;
mod economy_module_requests;
mod effects;
mod governance;
mod modules;
mod persistence;
mod power_bootstrap;
mod reward_asset;
mod rules;
