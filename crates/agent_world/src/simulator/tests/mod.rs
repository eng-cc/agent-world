//! Tests for the simulator module.

use super::*;
use crate::geometry::GeoPos;
use crate::models::DEFAULT_AGENT_HEIGHT_CM;
use std::fs;

fn pos(x: f64, y: f64) -> GeoPos {
    GeoPos {
        x_cm: x,
        y_cm: y,
        z_cm: 0.0,
    }
}

mod asteroid_fragment;
mod basics;
mod boundary_extremes;
mod chunking;
mod conservation;
mod consistency;
mod fragment_physics;
mod init;
mod kernel;
mod kernel_rule_decisions;
mod kernel_rule_invariants;
mod kernel_wasm_rule_bridge;
mod kernel_wasm_sandbox_bridge;
mod memory;
mod module_visual;
mod monotonicity;
mod persist;
mod physics_parameters;
mod power;
mod runner;
