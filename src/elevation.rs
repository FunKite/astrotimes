// Elevation estimation using ETOPO GeoTIFF data and ML-based urban correction
//
// This module implements intelligent elevation estimation that accounts for the fact
// that populated areas tend to be at lower elevations than raw terrain data suggests.
//
// Approach:
// 1. Read raw elevation from ETOPO 2022 GeoTIFF at given lat/lon
// 2. Calculate elevation corrections from nearby cities (urban_areas.json)
// 3. Use Inverse Distance Weighting (IDW) to interpolate corrections
// 4. Apply correction to raw elevation for more realistic estimate

use anyhow::{anyhow, Context, Result};
use geo_types::Coord;
use geotiff::GeoTiff;
use std::io::Cursor;
use std::sync::OnceLock;

use crate::city::City;

/// Cached GeoTIFF reader (initialized once)
static ELEVATION_DATA: OnceLock<Option<GeoTiff>> = OnceLock::new();

/// Number of nearest cities to use for elevation correction
const K_NEAREST_CITIES: usize = 5;

/// Maximum distance (km) for city influence in IDW interpolation
const MAX_DISTANCE_KM: f64 = 500.0;

/// IDW power parameter (higher = more weight to closer cities)
const IDW_POWER: f64 = 2.0;

/// Initialize the elevation data reader (called once)
fn init_elevation_data() -> Option<GeoTiff> {
    // ETOPO data is embedded in the binary at compile time
    let tif_bytes = include_bytes!("../data/ETOPO_land_ocean.tif");

    // GeoTiff::read requires Read + Seek, so wrap in Cursor
    let cursor = Cursor::new(tif_bytes.as_slice());

    match GeoTiff::read(cursor) {
        Ok(geotiff) => {
            eprintln!("✓ Loaded ETOPO elevation data ({} bytes)", tif_bytes.len());
            Some(geotiff)
        }
        Err(e) => {
            eprintln!("⚠ Warning: Could not load ETOPO elevation data: {}", e);
            None
        }
    }
}

/// Get raw elevation from ETOPO data at given lat/lon
///
/// Uses closest pixel fallback: if exact coordinate fails, tries nearby pixels
/// to ensure robustness for edge cases and coordinate precision issues.
fn get_raw_etopo_elevation(lat: f64, lon: f64) -> Result<f64> {
    let geotiff = ELEVATION_DATA.get_or_init(init_elevation_data);

    let geotiff = geotiff
        .as_ref()
        .ok_or_else(|| anyhow!("ETOPO elevation data not available"))?;

    // GeoTIFF coordinate: x = longitude, y = latitude
    // Band 0 is the first (and typically only) band in elevation data
    let coord = Coord { x: lon, y: lat };

    // Try exact coordinate first
    if let Some(elevation) = geotiff.get_value_at::<i16>(&coord, 0) {
        return Ok(elevation as f64);
    }

    // Exact lookup failed - try closest pixel fallback
    // This handles edge cases: coordinates on TIFF boundaries,
    // precision issues, or pixels marked as no-data

    // Try a small search pattern around the coordinate
    // Search within ±0.01 degrees (~1km at equator)
    let offsets = [
        (0.005, 0.0),     // East
        (-0.005, 0.0),    // West
        (0.0, 0.005),     // North
        (0.0, -0.005),    // South
        (0.005, 0.005),   // NE
        (-0.005, 0.005),  // NW
        (0.005, -0.005),  // SE
        (-0.005, -0.005), // SW
    ];

    for (dlon, dlat) in &offsets {
        let nearby_coord = Coord {
            x: lon + dlon,
            y: lat + dlat,
        };

        if let Some(elevation) = geotiff.get_value_at::<i16>(&nearby_coord, 0) {
            return Ok(elevation as f64);
        }
    }

    // If all nearby pixels failed, return error
    Err(anyhow!(
        "Failed to read elevation at ({}, {}) and nearby pixels. \
        Coordinate may be outside TIFF coverage area.",
        lat,
        lon
    ))
}

/// Calculate Haversine distance between two points in kilometers
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a =
        (dlat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Calculate elevation correction using Inverse Distance Weighting from nearby cities
///
/// This accounts for the urban bias: people tend to live in valleys and plains,
/// not on mountain peaks, so raw terrain elevation tends to overestimate actual
/// settlement elevation.
fn calculate_urban_correction(lat: f64, lon: f64, cities: &[City]) -> Result<f64> {
    // Find K nearest cities with their distances
    let mut city_distances: Vec<(f64, &City)> = cities
        .iter()
        .map(|city| {
            let dist = haversine_distance(lat, lon, city.lat, city.lon);
            (dist, city)
        })
        .collect();

    // Sort by distance and take K nearest
    city_distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let nearest_cities = &city_distances[..K_NEAREST_CITIES.min(city_distances.len())];

    // If all cities are very far away, don't apply correction
    if nearest_cities.is_empty() || nearest_cities[0].0 > MAX_DISTANCE_KM {
        return Ok(0.0);
    }

    // Calculate IDW-weighted correction
    let mut weighted_correction = 0.0;
    let mut weight_sum = 0.0;

    for (distance, city) in nearest_cities {
        if *distance > MAX_DISTANCE_KM {
            continue;
        }

        // Get ETOPO elevation at this city's location
        let city_etopo = match get_raw_etopo_elevation(city.lat, city.lon) {
            Ok(elev) => elev,
            Err(_) => continue, // Skip cities where we can't read ETOPO
        };

        // Calculate correction: actual city elevation - ETOPO elevation
        // Negative correction means city is lower than terrain (common for settlements)
        let correction = city.elev - city_etopo;

        // IDW weight: 1 / distance^power
        // Add small epsilon to avoid division by zero for exact matches
        let weight = 1.0 / (distance + 0.1).powf(IDW_POWER);

        weighted_correction += correction * weight;
        weight_sum += weight;
    }

    if weight_sum > 0.0 {
        Ok(weighted_correction / weight_sum)
    } else {
        Ok(0.0)
    }
}

/// Estimate elevation at given lat/lon using ETOPO data and urban correction
///
/// This combines raw terrain elevation with learned patterns from the city database
/// to provide more realistic estimates for populated areas.
///
/// # Arguments
/// * `lat` - Latitude in decimal degrees
/// * `lon` - Longitude in decimal degrees
/// * `cities` - Reference to city database for urban correction
///
/// # Returns
/// Estimated elevation in meters, or error if data unavailable
pub fn estimate_elevation(lat: f64, lon: f64, cities: &[City]) -> Result<f64> {
    // Get raw ETOPO elevation
    let raw_elevation =
        get_raw_etopo_elevation(lat, lon).context("Failed to read ETOPO elevation data")?;

    // OCEAN DETECTION FIX:
    // ETOPO encodes elevations < 1m as 0 (includes oceans, below sea level)
    // If ETOPO shows ~0 and we're far from any city, this is likely ocean
    if raw_elevation.abs() < 1.0 {
        // Find distance to nearest city
        let nearest_distance = cities
            .iter()
            .map(|city| haversine_distance(lat, lon, city.lat, city.lon))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(f64::INFINITY);

        // If nearest city is >50km away, assume this is ocean
        // Return 0m directly without urban correction (avoids ±40m artifacts)
        if nearest_distance > 50.0 {
            return Ok(0.0);
        }
    }

    // Calculate urban correction based on nearby cities
    let correction = calculate_urban_correction(lat, lon, cities).unwrap_or(0.0); // If correction fails, use raw elevation

    // Apply correction
    let estimated_elevation = raw_elevation + correction;

    // Clamp to reasonable values (Death Valley to Everest)
    let clamped_elevation = estimated_elevation.max(-100.0).min(9000.0);

    Ok(clamped_elevation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        // New York to Los Angeles: ~3944 km
        let dist = haversine_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!(
            (dist - 3944.0).abs() < 50.0,
            "Distance should be ~3944 km, got {}",
            dist
        );
    }

    #[test]
    fn test_elevation_estimation_bounds() {
        // Test that clamping works
        let cities = vec![];

        // This should not panic even with no cities
        let result = estimate_elevation(0.0, 0.0, &cities);

        if let Ok(elev) = result {
            assert!(
                elev >= -100.0 && elev <= 9000.0,
                "Elevation {} should be within bounds",
                elev
            );
        }
    }
}
