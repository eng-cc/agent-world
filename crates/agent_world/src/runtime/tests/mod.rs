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
mod effects;
mod governance;
mod modules;
mod persistence;
mod power_bootstrap;
mod rules;
