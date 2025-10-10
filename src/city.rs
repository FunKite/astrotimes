// City database and picker

use anyhow::{Context, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub tz: String,
    pub country: String,
    pub state: Option<String>,
}

pub struct CityDatabase {
    cities: Vec<City>,
}

impl CityDatabase {
    /// Load city database from embedded JSON
    pub fn load() -> Result<Self> {
        let data = include_str!("../data/urban_areas.json");
        let cities: Vec<City> =
            serde_json::from_str(data).context("Failed to parse city database")?;

        Ok(Self { cities })
    }

    /// Get reference to all cities in the database
    #[allow(dead_code)]
    pub fn cities(&self) -> &[City] {
        &self.cities
    }

    /// Find a city by exact name match (case-insensitive)
    pub fn find_exact(&self, name: &str) -> Option<&City> {
        let name_lower = name.to_lowercase();
        self.cities
            .iter()
            .find(|c| c.name.to_lowercase() == name_lower)
    }

    /// Search cities with fuzzy matching
    pub fn search(&self, query: &str) -> Vec<(&City, i64)> {
        let matcher = SkimMatcherV2::default();
        let mut results: Vec<_> = self
            .cities
            .iter()
            .filter_map(|city| {
                let search_text = if let Some(state) = &city.state {
                    format!("{}, {}, {}", city.name, state, city.country)
                } else {
                    format!("{}, {}", city.name, city.country)
                };
                matcher
                    .fuzzy_match(&search_text, query)
                    .map(|score| (city, score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_database() {
        let db = CityDatabase::load().unwrap();
        assert!(!db.cities().is_empty());
    }

    #[test]
    fn test_find_exact() {
        let db = CityDatabase::load().unwrap();
        let city = db.find_exact("New York");
        assert!(city.is_some());
        assert_eq!(city.unwrap().country, "US");
    }

    #[test]
    fn test_search() {
        let db = CityDatabase::load().unwrap();
        let results = db.search("san");
        assert!(!results.is_empty());
    }
}
