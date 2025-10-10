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
