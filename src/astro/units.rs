// Type-safe astronomical units
// Prevents mixing degrees/radians and enforces valid ranges

use std::f64::consts::PI;
use std::fmt;

pub const DEG_TO_RAD: f64 = PI / 180.0;
pub const RAD_TO_DEG: f64 = 180.0 / PI;

/// Angle in degrees
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Degrees(f64);

impl Degrees {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    /// Normalize to 0-360 range
    pub fn normalized(self) -> Self {
        let mut result = self.0 % 360.0;
        if result < 0.0 {
            result += 360.0;
        }
        Self(result)
    }

    /// Normalize to -180 to 180 range
    pub fn normalized_signed(self) -> Self {
        let mut result = self.0 % 360.0;
        if result > 180.0 {
            result -= 360.0;
        } else if result < -180.0 {
            result += 360.0;
        }
        Self(result)
    }

    pub fn to_radians(self) -> Radians {
        Radians::from(self)
    }

    pub fn sin(self) -> f64 {
        self.0.to_radians().sin()
    }

    pub fn cos(self) -> f64 {
        self.0.to_radians().cos()
    }

    pub fn tan(self) -> f64 {
        self.0.to_radians().tan()
    }
}

impl fmt::Display for Degrees {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}°", self.0)
    }
}

impl From<f64> for Degrees {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<Degrees> for f64 {
    fn from(deg: Degrees) -> f64 {
        deg.0
    }
}

/// Angle in radians
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radians(f64);

impl Radians {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn to_degrees(self) -> Degrees {
        Degrees::from(self)
    }

    pub fn sin(self) -> f64 {
        self.0.sin()
    }

    pub fn cos(self) -> f64 {
        self.0.cos()
    }

    pub fn tan(self) -> f64 {
        self.0.tan()
    }

    pub fn asin(value: f64) -> Self {
        Self(value.asin())
    }

    pub fn acos(value: f64) -> Self {
        Self(value.acos())
    }

    pub fn atan2(y: f64, x: f64) -> Self {
        Self(y.atan2(x))
    }
}

impl From<Degrees> for Radians {
    fn from(deg: Degrees) -> Self {
        Self(deg.0 * DEG_TO_RAD)
    }
}

impl From<Radians> for Degrees {
    fn from(rad: Radians) -> Self {
        Self(rad.0 * RAD_TO_DEG)
    }
}

/// Latitude coordinate (-90 to 90 degrees, positive North)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Latitude(f64);

impl Latitude {
    pub fn new(degrees: f64) -> Result<Self, String> {
        if !(-90.0..=90.0).contains(&degrees) {
            Err(format!("Invalid latitude: {} (must be -90 to 90)", degrees))
        } else {
            Ok(Self(degrees))
        }
    }

    /// Create without validation (use only when value is known to be valid)
    pub fn new_unchecked(degrees: f64) -> Self {
        Self(degrees)
    }

    pub fn degrees(&self) -> Degrees {
        Degrees(self.0)
    }

    pub fn radians(&self) -> Radians {
        Radians(self.0 * DEG_TO_RAD)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}° {}", self.0.abs(), if self.0 >= 0.0 { "N" } else { "S" })
    }
}

/// Longitude coordinate (-180 to 180 degrees, positive East)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Longitude(f64);

impl Longitude {
    pub fn new(degrees: f64) -> Result<Self, String> {
        if !(-180.0..=180.0).contains(&degrees) {
            Err(format!("Invalid longitude: {} (must be -180 to 180)", degrees))
        } else {
            Ok(Self(degrees))
        }
    }

    /// Create without validation (use only when value is known to be valid)
    pub fn new_unchecked(degrees: f64) -> Self {
        Self(degrees)
    }

    pub fn degrees(&self) -> Degrees {
        Degrees(self.0)
    }

    pub fn radians(&self) -> Radians {
        Radians(self.0 * DEG_TO_RAD)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Longitude {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}° {}", self.0.abs(), if self.0 >= 0.0 { "E" } else { "W" })
    }
}

/// Altitude angle (elevation above horizon, -90 to 90 degrees)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Altitude(f64);

impl Altitude {
    pub fn from_degrees(degrees: f64) -> Self {
        Self(degrees)
    }

    pub fn from_radians(radians: f64) -> Self {
        Self(radians * RAD_TO_DEG)
    }

    pub fn degrees(&self) -> Degrees {
        Degrees(self.0)
    }

    pub fn radians(&self) -> Radians {
        Radians(self.0 * DEG_TO_RAD)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for Altitude {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}°", self.0)
    }
}

/// Azimuth angle (compass bearing from North, 0 to 360 degrees)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Azimuth(f64);

impl Azimuth {
    pub fn from_degrees(degrees: f64) -> Self {
        // Normalize to 0-360
        let mut normalized = degrees % 360.0;
        if normalized < 0.0 {
            normalized += 360.0;
        }
        Self(normalized)
    }

    pub fn from_radians(radians: f64) -> Self {
        Self::from_degrees(radians * RAD_TO_DEG)
    }

    pub fn degrees(&self) -> Degrees {
        Degrees(self.0)
    }

    pub fn radians(&self) -> Radians {
        Radians(self.0 * DEG_TO_RAD)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn to_compass(&self) -> &'static str {
        let index = ((self.0 + 22.5) / 45.0).floor() as usize % 8;
        ["N", "NE", "E", "SE", "S", "SW", "W", "NW"][index]
    }
}

impl fmt::Display for Azimuth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.2}° {}", self.0, self.to_compass())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degrees_to_radians() {
        let deg = Degrees::new(180.0);
        let rad = deg.to_radians();
        assert!((rad.value() - PI).abs() < 0.0001);
    }

    #[test]
    fn test_radians_to_degrees() {
        let rad = Radians::new(PI);
        let deg = rad.to_degrees();
        assert!((deg.value() - 180.0).abs() < 0.0001);
    }

    #[test]
    fn test_degrees_normalize() {
        assert_eq!(Degrees::new(370.0).normalized().value(), 10.0);
        assert_eq!(Degrees::new(-10.0).normalized().value(), 350.0);
    }

    #[test]
    fn test_latitude_validation() {
        assert!(Latitude::new(45.0).is_ok());
        assert!(Latitude::new(-90.0).is_ok());
        assert!(Latitude::new(90.0).is_ok());
        assert!(Latitude::new(91.0).is_err());
        assert!(Latitude::new(-91.0).is_err());
    }

    #[test]
    fn test_longitude_validation() {
        assert!(Longitude::new(0.0).is_ok());
        assert!(Longitude::new(180.0).is_ok());
        assert!(Longitude::new(-180.0).is_ok());
        assert!(Longitude::new(181.0).is_err());
        assert!(Longitude::new(-181.0).is_err());
    }

    #[test]
    fn test_azimuth_normalize() {
        assert_eq!(Azimuth::from_degrees(370.0).value(), 10.0);
        assert_eq!(Azimuth::from_degrees(-10.0).value(), 350.0);
    }

    #[test]
    fn test_azimuth_compass() {
        assert_eq!(Azimuth::from_degrees(0.0).to_compass(), "N");
        assert_eq!(Azimuth::from_degrees(45.0).to_compass(), "NE");
        assert_eq!(Azimuth::from_degrees(90.0).to_compass(), "E");
        assert_eq!(Azimuth::from_degrees(270.0).to_compass(), "W");
    }
}
