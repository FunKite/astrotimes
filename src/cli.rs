// Command-line argument parsing

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "astrotimes")]
#[command(version = "1.0.0")]
#[command(about = "High-precision astronomical CLI for sun and moon calculations", long_about = None)]
pub struct Args {
    /// Latitude in decimal degrees (positive North, negative South)
    #[arg(long)]
    pub lat: Option<f64>,

    /// Longitude in decimal degrees (positive East, negative West)
    #[arg(long)]
    pub lon: Option<f64>,

    /// Elevation in meters above sea level
    #[arg(long)]
    pub elev: Option<f64>,

    /// Timezone (IANA timezone name or UTC offset like -07:00)
    #[arg(long)]
    pub tz: Option<String>,

    /// Date in YYYY-MM-DD format (defaults to today)
    #[arg(long)]
    pub date: Option<String>,

    /// Select a city from the built-in database
    #[arg(long)]
    pub city: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Force watch mode (live updates)
    #[arg(long)]
    pub watch: bool,

    /// Refresh interval in seconds for watch mode (default: 60)
    #[arg(long, default_value = "60.0")]
    pub refresh: f64,

    /// Disable all interactive prompts
    #[arg(long)]
    pub no_prompt: bool,

    /// Disable saving settings to config file
    #[arg(long)]
    pub no_save: bool,

    /// Strict mode: exit with error if events don't occur (polar regions)
    #[arg(long)]
    pub strict: bool,
}

impl Args {
    #[allow(dead_code)]
    pub fn has_location(&self) -> bool {
        (self.lat.is_some() && self.lon.is_some()) || self.city.is_some()
    }

    pub fn should_watch(&self) -> bool {
        self.watch || (!self.json && !self.no_prompt)
    }
}
