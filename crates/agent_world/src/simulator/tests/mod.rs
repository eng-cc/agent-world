//! Tests for the simulator module.

use super::*;
use crate::geometry::GeoPos;
use crate::models::DEFAULT_AGENT_HEIGHT_CM;
use std::fs;

fn pos(lat: f64, lon: f64) -> GeoPos {
    GeoPos {
        lat_deg: lat,
        lon_deg: lon,
    }
}

mod basics;
mod kernel;
mod memory;
mod persist;
mod power;
mod runner;
