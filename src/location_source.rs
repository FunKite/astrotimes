// Describes where latitude/longitude coordinates originated.

#[derive(Debug, Clone, Copy)]
pub enum LocationSource {
    ManualCli,
    CityDatabase,
    SavedConfig,
    IpLookup,
}

impl LocationSource {
    pub fn short_label(self) -> &'static str {
        match self {
            LocationSource::ManualCli => "manual",
            LocationSource::CityDatabase => "city",
            LocationSource::SavedConfig => "saved",
            LocationSource::IpLookup => "ip",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ElevationSource {
    Manual,
    TerrainMl,
    Saved,
}

impl ElevationSource {
    pub fn short_label(self) -> &'static str {
        match self {
            ElevationSource::Manual => "manual",
            ElevationSource::TerrainMl => "ml",
            ElevationSource::Saved => "saved",
        }
    }
}
