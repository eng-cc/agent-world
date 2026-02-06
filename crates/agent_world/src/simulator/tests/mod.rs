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

mod basics;
mod asteroid_fragment;
mod init;
mod kernel;
mod memory;
mod persist;
mod power;
mod runner;
