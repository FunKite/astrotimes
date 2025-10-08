// Time utilities for astronomical calculations

use chrono::{DateTime, Duration, TimeZone};

/// Format duration as human-readable string
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.num_seconds().abs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;

    if duration.num_seconds() < 0 {
        format!("{:02}:{:02} ago", hours, minutes)
    } else {
        format!("in {:02}h {:02}m", hours, minutes)
    }
}

/// Format duration with seconds as detailed string
pub fn format_duration_detailed(duration: Duration) -> String {
    let total_secs = duration.num_seconds().abs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if duration.num_seconds() < 0 {
        format!("{:02}:{:02}:{:02} ago", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02} from now", hours, minutes, seconds)
    }
}

/// Format time as HH:MM string
pub fn format_time<T: TimeZone>(dt: &DateTime<T>) -> String
where
    T::Offset: std::fmt::Display,
{
    dt.format("%H:%M").to_string()
}

/// Format date as human-readable string
pub fn format_date<T: TimeZone>(dt: &DateTime<T>) -> String
where
    T::Offset: std::fmt::Display,
{
    dt.format("%b %d %H:%M:%S").to_string()
}

/// Calculate time until an event
pub fn time_until<T: TimeZone>(from: &DateTime<T>, to: &DateTime<T>) -> Duration
where
    T: Clone,
{
    to.clone().signed_duration_since(from.clone())
}

/// Check if a datetime is today
pub fn is_today<T: TimeZone>(dt: &DateTime<T>, reference: &DateTime<T>) -> bool {
    dt.date_naive() == reference.date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_format_duration() {
        let dur = Duration::hours(2) + Duration::minutes(30);
        assert_eq!(format_duration(dur), "in 02h 30m");

        let dur = Duration::hours(-1) + Duration::minutes(-15);
        assert_eq!(format_duration(dur), "01:15 ago");
    }
}
