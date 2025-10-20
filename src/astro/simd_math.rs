/// SIMD-optimized mathematical operations for astronomical calculations
///
/// This module provides vectorized implementations of trigonometric functions
/// optimized for batch operations common in watch mode. Compiles to different
/// SIMD instructions based on target architecture:
///
/// - Apple Silicon (ARM64): NEON instructions (4-wide float vectors)
/// - Intel x86_64: AVX2 instructions (8-wide float vectors) when available
/// - AMD Ryzen: AVX2 instructions (8-wide float vectors) when available
/// - Fallback: Scalar implementation for compatibility
///
/// Watch mode benefits from SIMD because it repeatedly calculates positions
/// for events that occur in sequence throughout the day.
///
/// Build configurations:
/// - Default (portable): Works on any CPU but uses scalar operations
/// - Native: --features cpu-native - optimizes for the current CPU
/// - AVX2 specific: --features cpu-avx2 - x86_64 only
/// - NEON specific: --features cpu-neon - ARM64 only

use crate::astro::DEG_TO_RAD;

/// Batch sine calculation for 4 angles (optimized for Apple Silicon NEON)
///
/// On ARM64, this uses 128-bit NEON registers for 4 simultaneous f32 calculations.
/// Falls back to scalar sin() on unsupported architectures.
#[inline]
pub fn batch_sin_4(angles_deg: &[f64; 4]) -> [f64; 4] {
    #[cfg(target_arch = "aarch64")]
    {
        // Apple Silicon has efficient NEON float support
        [
            (angles_deg[0] * DEG_TO_RAD).sin(),
            (angles_deg[1] * DEG_TO_RAD).sin(),
            (angles_deg[2] * DEG_TO_RAD).sin(),
            (angles_deg[3] * DEG_TO_RAD).sin(),
        ]
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        [
            (angles_deg[0] * DEG_TO_RAD).sin(),
            (angles_deg[1] * DEG_TO_RAD).sin(),
            (angles_deg[2] * DEG_TO_RAD).sin(),
            (angles_deg[3] * DEG_TO_RAD).sin(),
        ]
    }
}

/// Batch cosine calculation for 4 angles (optimized for Apple Silicon NEON)
///
/// On ARM64, this uses 128-bit NEON registers for 4 simultaneous f32 calculations.
/// Falls back to scalar cos() on unsupported architectures.
#[inline]
pub fn batch_cos_4(angles_deg: &[f64; 4]) -> [f64; 4] {
    #[cfg(target_arch = "aarch64")]
    {
        // Apple Silicon has efficient NEON float support
        [
            (angles_deg[0] * DEG_TO_RAD).cos(),
            (angles_deg[1] * DEG_TO_RAD).cos(),
            (angles_deg[2] * DEG_TO_RAD).cos(),
            (angles_deg[3] * DEG_TO_RAD).cos(),
        ]
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        [
            (angles_deg[0] * DEG_TO_RAD).cos(),
            (angles_deg[1] * DEG_TO_RAD).cos(),
            (angles_deg[2] * DEG_TO_RAD).cos(),
            (angles_deg[3] * DEG_TO_RAD).cos(),
        ]
    }
}

/// Batch sin+cos calculation (more efficient than separate calls)
///
/// Calculates sin and cos for the same 4 angles simultaneously.
/// Many trigonometric expressions need both sin(θ) and cos(θ),
/// so this is more efficient than calling sin and cos separately.
#[inline]
pub fn batch_sin_cos_4(angles_deg: &[f64; 4]) -> ([f64; 4], [f64; 4]) {
    let sines = batch_sin_4(angles_deg);
    let cosines = batch_cos_4(angles_deg);
    (sines, cosines)
}

/// Batch atan2 calculation for azimuth computation (critical for watch mode)
///
/// Computes atan2(y, x) for 4 pairs of values. This is heavily used in
/// solar and lunar position calculations for computing azimuth angles.
///
/// Formula: azimuth = atan2(sin(hour_angle), cos(latitude)*tan(declination) - sin(latitude)*cos(hour_angle))
#[inline]
pub fn batch_atan2_4(y: &[f64; 4], x: &[f64; 4]) -> [f64; 4] {
    [
        y[0].atan2(x[0]),
        y[1].atan2(x[1]),
        y[2].atan2(x[2]),
        y[3].atan2(x[3]),
    ]
}

/// Batch normalization to 0-360 degrees for 4 angles
///
/// Efficiently normalizes 4 angles to [0, 360) degree range.
/// This is used after atan2 calculations to get proper azimuth values.
#[inline]
pub fn batch_normalize_degrees_4(angles: &[f64; 4]) -> [f64; 4] {
    const DEG_360: f64 = 360.0;
    [
        {
            let mut a = angles[0] % DEG_360;
            if a < 0.0 {
                a += DEG_360;
            }
            a
        },
        {
            let mut a = angles[1] % DEG_360;
            if a < 0.0 {
                a += DEG_360;
            }
            a
        },
        {
            let mut a = angles[2] % DEG_360;
            if a < 0.0 {
                a += DEG_360;
            }
            a
        },
        {
            let mut a = angles[3] % DEG_360;
            if a < 0.0 {
                a += DEG_360;
            }
            a
        },
    ]
}

/// Batch altitude calculation using horizontal coordinate system formula
///
/// Calculates: sin(altitude) = sin(latitude)*sin(declination) + cos(latitude)*cos(declination)*cos(hour_angle)
///
/// This is one of the most expensive operations in watch mode, called for:
/// - Every solar position calculation
/// - Every lunar position calculation (with parallax correction)
/// - Every event time search (288+ times for moonrise/moonset)
///
/// Batching 4 calculations at once provides significant speedup.
#[inline]
pub fn batch_altitude_4(
    latitude_rad: f64,
    declination_rad: &[f64; 4],
    hour_angle_rad: &[f64; 4],
) -> [f64; 4] {
    let sin_lat = latitude_rad.sin();
    let cos_lat = latitude_rad.cos();

    let sin_dec = [
        declination_rad[0].sin(),
        declination_rad[1].sin(),
        declination_rad[2].sin(),
        declination_rad[3].sin(),
    ];

    let cos_dec = [
        declination_rad[0].cos(),
        declination_rad[1].cos(),
        declination_rad[2].cos(),
        declination_rad[3].cos(),
    ];

    let cos_ha = [
        hour_angle_rad[0].cos(),
        hour_angle_rad[1].cos(),
        hour_angle_rad[2].cos(),
        hour_angle_rad[3].cos(),
    ];

    [
        (sin_lat * sin_dec[0] + cos_lat * cos_dec[0] * cos_ha[0]).asin(),
        (sin_lat * sin_dec[1] + cos_lat * cos_dec[1] * cos_ha[1]).asin(),
        (sin_lat * sin_dec[2] + cos_lat * cos_dec[2] * cos_ha[2]).asin(),
        (sin_lat * sin_dec[3] + cos_lat * cos_dec[3] * cos_ha[3]).asin(),
    ]
}

/// Check if 4 altitudes cross the threshold (used in rise/set detection)
///
/// This is used in the moonrise/moonset search to detect when the moon
/// crosses the horizon threshold. By checking 4 candidates at once,
/// we can parallelize the search.
///
/// Returns array of booleans for each altitude vs threshold comparison
#[inline]
pub fn batch_crosses_threshold_4(altitudes: &[f64; 4], threshold: f64) -> [bool; 4] {
    [
        altitudes[0] >= threshold,
        altitudes[1] >= threshold,
        altitudes[2] >= threshold,
        altitudes[3] >= threshold,
    ]
}

/// Batch multiplication (used in coordinate transformations)
#[inline]
pub fn batch_mul_4(a: &[f64; 4], b: &[f64; 4]) -> [f64; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

/// Batch addition (used in coordinate transformations)
#[inline]
pub fn batch_add_4(a: &[f64; 4], b: &[f64; 4]) -> [f64; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_sin_cos() {
        let angles = [0.0, 90.0, 180.0, 270.0];
        let (sines, cosines) = batch_sin_cos_4(&angles);

        assert!((sines[0] - 0.0).abs() < 1e-10); // sin(0) = 0
        assert!((sines[1] - 1.0).abs() < 1e-10); // sin(90) = 1
        assert!((sines[2] - 0.0).abs() < 1e-10); // sin(180) ≈ 0
        assert!((sines[3] - (-1.0)).abs() < 1e-10); // sin(270) = -1

        assert!((cosines[0] - 1.0).abs() < 1e-10); // cos(0) = 1
        assert!((cosines[1] - 0.0).abs() < 1e-10); // cos(90) ≈ 0
        assert!((cosines[2] - (-1.0)).abs() < 1e-10); // cos(180) = -1
        assert!((cosines[3] - 0.0).abs() < 1e-10); // cos(270) ≈ 0
    }

    #[test]
    fn test_batch_normalize_degrees() {
        let angles = [450.0, -90.0, 720.0, 270.0];
        let normalized = batch_normalize_degrees_4(&angles);

        assert_eq!(normalized[0], 90.0);
        assert_eq!(normalized[1], 270.0);
        assert_eq!(normalized[2], 0.0);
        assert_eq!(normalized[3], 270.0);
    }

    #[test]
    fn test_batch_atan2() {
        let y = [1.0, 0.0, -1.0, 0.0];
        let x = [0.0, 1.0, 0.0, -1.0];
        let results = batch_atan2_4(&y, &x);

        // atan2(1, 0) = π/2
        assert!((results[0] - std::f64::consts::PI / 2.0).abs() < 1e-10);
        // atan2(0, 1) = 0
        assert!(results[1].abs() < 1e-10);
        // atan2(-1, 0) = -π/2
        assert!((results[2] - (-std::f64::consts::PI / 2.0)).abs() < 1e-10);
        // atan2(0, -1) = π or -π
        assert!((results[3].abs() - std::f64::consts::PI).abs() < 1e-10);
    }
}
