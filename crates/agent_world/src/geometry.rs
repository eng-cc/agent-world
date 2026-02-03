use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPos {
    pub lat_deg: f64,
    pub lon_deg: f64,
}

impl GeoPos {
    pub fn normalized(self) -> Self {
        let lat = self.lat_deg.clamp(-90.0, 90.0);
        let lon = ((self.lon_deg + 180.0) % 360.0) - 180.0;
        Self { lat_deg: lat, lon_deg: lon }
    }
}

pub const SPACE_UNIT_CM: i64 = 1;
pub const WORLD_RADIUS_KM: i64 = 10_000;
pub const WORLD_RADIUS_CM: i64 = WORLD_RADIUS_KM * 100_000;
pub const WORLD_RADIUS_M: f64 = WORLD_RADIUS_CM as f64 / 100.0;

pub fn great_circle_distance_m(a: GeoPos, b: GeoPos) -> f64 {
    great_circle_distance_m_with_radius(a, b, WORLD_RADIUS_M)
}

pub fn great_circle_distance_m_with_radius(a: GeoPos, b: GeoPos, radius_m: f64) -> f64 {
    let a = a.normalized();
    let b = b.normalized();
    let lat1 = a.lat_deg.to_radians();
    let lon1 = a.lon_deg.to_radians();
    let lat2 = b.lat_deg.to_radians();
    let lon2 = b.lon_deg.to_radians();
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let sin_dlat = (dlat / 2.0).sin();
    let sin_dlon = (dlon / 2.0).sin();
    let h = (sin_dlat * sin_dlat) + (lat1.cos() * lat2.cos() * sin_dlon * sin_dlon);
    let central_angle = (h.sqrt()).min(1.0).asin();
    2.0 * radius_m * central_angle
}

pub fn great_circle_distance_cm(a: GeoPos, b: GeoPos) -> i64 {
    great_circle_distance_cm_with_radius(a, b, WORLD_RADIUS_CM)
}

pub fn great_circle_distance_cm_with_radius(a: GeoPos, b: GeoPos, radius_cm: i64) -> i64 {
    let distance_m = great_circle_distance_m_with_radius(a, b, radius_cm as f64 / 100.0);
    (distance_m * 100.0).round().max(0.0) as i64
}
