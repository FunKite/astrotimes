use anyhow::{anyhow, Context};
use chrono::{DateTime, Duration as ChronoDuration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::time::Duration as StdDuration;

const TIME_SERVERS: [(&str, &str); 2] = [
    ("time.google.com:123", "time.google.com (NTP)"),
    ("pool.ntp.org:123", "pool.ntp.org (NTP)"),
];
pub const PRIMARY_SOURCE_LABEL: &str = TIME_SERVERS[0].1;
const SYNC_THRESHOLD_MICROS: i64 = 50_000; // 50 ms tolerance treated as in sync

// Cache settings - 30 minutes minimum between NTP queries (pool.ntp.org ToS compliance)
const CACHE_MIN_INTERVAL_SECS: i64 = 1800; // 30 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeSyncCache {
    /// UTC timestamp when this cache entry was created
    timestamp: DateTime<Utc>,
    /// Source server that was queried
    source: String,
    /// Delta in microseconds (system time - NTP time)
    delta_micros: i64,
}

/// Default NTP servers to use when none are specified
pub fn default_servers() -> Vec<(String, String)> {
    TIME_SERVERS
        .iter()
        .map(|(server, label)| (server.to_string(), label.to_string()))
        .collect()
}

#[derive(Debug, Clone)]
pub struct TimeSyncInfo {
    pub source: &'static str,
    pub delta: Option<ChronoDuration>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSyncDirection {
    Ahead,
    Behind,
    InSync,
}

pub fn check_time_sync() -> TimeSyncInfo {
    check_time_sync_with_servers(None)
}

pub fn check_time_sync_with_servers(custom_server: Option<&str>) -> TimeSyncInfo {
    // Determine which server we're targeting (for cache key matching)
    let target_server = custom_server
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(TIME_SERVERS[0].0); // Default to primary server

    // Try to load from cache first
    if let Ok(cache) = load_cache() {
        let age = Utc::now().signed_duration_since(cache.timestamp);

        // Check if cache matches our target server and is fresh (< 30 minutes old)
        // Note: We normalize server names for comparison (strip port if present in cache)
        let cache_server_normalized = cache.source.split(':').next().unwrap_or(&cache.source);
        let target_server_normalized = target_server.split(':').next().unwrap_or(target_server);

        if cache_server_normalized == target_server_normalized
            && age.num_seconds() < CACHE_MIN_INTERVAL_SECS {
            let delta = ChronoDuration::microseconds(cache.delta_micros);
            return TimeSyncInfo {
                source: PRIMARY_SOURCE_LABEL, // Use static label for consistency
                delta: Some(delta),
                error: None,
            };
        }
    }

    // Cache is stale, missing, or for different server - perform fresh NTP query
    match fetch_delta(custom_server) {
        Ok((delta, source, server_addr)) => {
            // Save to cache for future calls
            if let Some(micros) = delta.num_microseconds() {
                let cache = TimeSyncCache {
                    timestamp: Utc::now(),
                    source: server_addr, // Store actual server address for cache matching
                    delta_micros: micros,
                };
                let _ = save_cache(&cache); // Ignore save errors
            }

            TimeSyncInfo {
                source,
                delta: Some(delta),
                error: None,
            }
        }
        Err(err) => TimeSyncInfo {
            source: PRIMARY_SOURCE_LABEL,
            delta: None,
            error: Some(err.to_string()),
        },
    }
}

pub fn format_offset(delta: ChronoDuration) -> String {
    let total_seconds = delta.num_seconds();
    let abs_seconds = total_seconds.abs();
    if abs_seconds >= 60 {
        let minutes = abs_seconds / 60;
        let seconds = abs_seconds % 60;
        let sign = if total_seconds >= 0 { "+" } else { "-" };
        return format!("{}{}m{}s", sign, minutes, seconds);
    }

    if let Some(micros) = delta.num_microseconds() {
        let seconds = micros as f64 / 1_000_000.0;
        if seconds.abs() >= 1.0 {
            format!("{:+.3}s", seconds)
        } else {
            format!("{:+.1}ms", seconds * 1000.0)
        }
    } else {
        format!("{:+}s", total_seconds)
    }
}

pub fn describe_direction(direction: TimeSyncDirection) -> &'static str {
    match direction {
        TimeSyncDirection::Ahead => "system ahead",
        TimeSyncDirection::Behind => "system behind",
        TimeSyncDirection::InSync => "system in sync",
    }
}

pub fn direction_code(direction: TimeSyncDirection) -> &'static str {
    match direction {
        TimeSyncDirection::Ahead => "ahead",
        TimeSyncDirection::Behind => "behind",
        TimeSyncDirection::InSync => "in_sync",
    }
}

/// Returns (delta, label, server_address)
fn fetch_delta(custom_server: Option<&str>) -> anyhow::Result<(ChronoDuration, &'static str, String)> {
    let mut last_err: Option<anyhow::Error> = None;

    // If custom server is specified, try it first
    if let Some(server_str) = custom_server {
        let server_trimmed = server_str.trim();
        if !server_trimmed.is_empty() {
            let server_with_port = if server_trimmed.contains(':') {
                server_trimmed.to_string()
            } else {
                format!("{}:123", server_trimmed)
            };

            match query_ntp(&server_with_port) {
                Ok(server_time) => {
                    let system_time = Utc::now();
                    let delta = system_time.signed_duration_since(server_time);
                    // Return server address for cache tracking
                    return Ok((delta, PRIMARY_SOURCE_LABEL, server_trimmed.to_string()));
                }
                Err(err) => {
                    last_err = Some(anyhow!("{} query failed: {}", server_trimmed, err));
                }
            }
        }
    }

    // Fall back to default servers
    for (server, label) in TIME_SERVERS.iter() {
        match query_ntp(server) {
            Ok(server_time) => {
                let system_time = Utc::now();
                let delta = system_time.signed_duration_since(server_time);
                // Extract server address without port for cache tracking
                let server_addr = server.split(':').next().unwrap_or(server).to_string();
                return Ok((delta, *label, server_addr));
            }
            Err(err) => {
                last_err = Some(anyhow!("{} query failed: {}", label, err));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow!("all time sources failed")))
}

fn query_ntp(server: &str) -> anyhow::Result<chrono::DateTime<Utc>> {
    let socket =
        UdpSocket::bind("0.0.0.0:0").context("failed to bind local UDP socket for time sync")?;
    socket
        .set_read_timeout(Some(StdDuration::from_secs(3)))
        .context("failed to set read timeout")?;
    socket
        .set_write_timeout(Some(StdDuration::from_secs(3)))
        .context("failed to set write timeout")?;

    let mut packet = [0u8; 48];
    packet[0] = 0b00_100_011; // LI = 0, VN = 4, Mode = 3 (client)

    socket
        .send_to(&packet, server)
        .with_context(|| format!("failed to send request to {}", server))?;

    let mut response = [0u8; 48];
    let (len, _) = socket
        .recv_from(&mut response)
        .with_context(|| format!("failed to receive response from {}", server))?;

    if len < 48 {
        return Err(anyhow!("incomplete NTP response ({} bytes)", len));
    }

    let seconds = u32::from_be_bytes([response[40], response[41], response[42], response[43]]);
    let fraction = u32::from_be_bytes([response[44], response[45], response[46], response[47]]);

    const NTP_UNIX_OFFSET: i64 = 2_208_988_800; // Seconds between 1900-01-01 and 1970-01-01
    let unix_seconds = seconds as i64 - NTP_UNIX_OFFSET;

    if unix_seconds < 0 {
        return Err(anyhow!("invalid NTP timestamp (pre-1970)"));
    }

    let nanos = ((fraction as u128) * 1_000_000_000u128 / (1u128 << 32)) as u32;

    Utc.timestamp_opt(unix_seconds, nanos)
        .single()
        .ok_or_else(|| anyhow!("invalid timestamp from {}", server))
}

impl TimeSyncInfo {
    pub fn direction(&self) -> Option<TimeSyncDirection> {
        self.delta.and_then(classify_direction)
    }

    pub fn delta_seconds(&self) -> Option<f64> {
        self.delta.and_then(|delta| {
            delta
                .num_microseconds()
                .map(|micros| micros as f64 / 1_000_000.0)
        })
    }

    pub fn error_summary(&self) -> Option<String> {
        self.error.as_ref().map(|err| summarize_error(err))
    }
}

fn summarize_error(err: &str) -> String {
    const MAX_LEN: usize = 60;
    if err.len() <= MAX_LEN {
        err.to_string()
    } else {
        let truncated = &err[..MAX_LEN];
        format!("{}…", truncated.trim_end())
    }
}

fn classify_direction(delta: ChronoDuration) -> Option<TimeSyncDirection> {
    if let Some(micros) = delta.num_microseconds() {
        if micros.abs() <= SYNC_THRESHOLD_MICROS {
            Some(TimeSyncDirection::InSync)
        } else if micros > 0 {
            Some(TimeSyncDirection::Ahead)
        } else {
            Some(TimeSyncDirection::Behind)
        }
    } else {
        None
    }
}

/// Get the cache file path
fn cache_file_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".astrotimes_ntp_cache.json"))
}

/// Load cached time sync result
fn load_cache() -> anyhow::Result<TimeSyncCache> {
    let path = cache_file_path().ok_or_else(|| anyhow!("cannot determine home directory"))?;

    if !path.exists() {
        return Err(anyhow!("cache file does not exist"));
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read cache file: {}", path.display()))?;

    let cache: TimeSyncCache = serde_json::from_str(&contents)
        .context("failed to parse cache file")?;

    Ok(cache)
}

/// Save time sync result to cache
fn save_cache(cache: &TimeSyncCache) -> anyhow::Result<()> {
    let path = cache_file_path().ok_or_else(|| anyhow!("cannot determine home directory"))?;

    let json = serde_json::to_string_pretty(cache)
        .context("failed to serialize cache")?;

    fs::write(&path, json)
        .with_context(|| format!("failed to write cache file: {}", path.display()))?;

    Ok(())
}
