use crate::error::ApiError;
use crate::models::ZoneInfo;
use chrono::{DateTime, Offset, Utc, TimeZone};
use chrono_tz::Tz;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Convert system time to multiple timezones
pub fn convert_to_timezones(
    timezone_names: &[String],
) -> Result<(i64, HashMap<String, ZoneInfo>), ApiError> {
    // Get current Unix timestamp
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH)?;
    let unix_timestamp = duration.as_secs() as i64;

    // Convert to UTC DateTime
    let utc_time: DateTime<Utc> = Utc.timestamp_opt(unix_timestamp, 0)
        .single()
        .ok_or_else(|| ApiError::SystemTimeError)?;

    // Convert to each requested timezone
    let mut zones = HashMap::new();
    for tz_name in timezone_names {
        let tz_name = tz_name.trim();
        if tz_name.is_empty() {
            continue;
        }

        // Parse timezone
        let tz: Tz = tz_name.parse().map_err(|_| {
            ApiError::InvalidTimezone(tz_name.to_string())
        })?;

        // Convert to local time
        let local_time = utc_time.with_timezone(&tz);

        // Format as ISO8601 without timezone suffix (YYYY-MM-DDTHH:MM:SS)
        let local_str = local_time.format("%Y-%m-%dT%H:%M:%S").to_string();

        // Calculate offset in seconds
        let offset = local_time.offset().fix().local_minus_utc();

        zones.insert(
            tz_name.to_string(),
            ZoneInfo {
                local: local_str,
                offset,
            },
        );
    }

    Ok((unix_timestamp, zones))
}

/// Get current Unix timestamp
pub fn get_unix_timestamp() -> Result<i64, ApiError> {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH)?;
    Ok(duration.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_utc() {
        let result = convert_to_timezones(&["UTC".to_string()]);
        assert!(result.is_ok());
        let (unix, zones) = result.unwrap();
        assert!(unix > 0);
        assert_eq!(zones.len(), 1);
        assert!(zones.contains_key("UTC"));

        let utc = &zones["UTC"];
        assert_eq!(utc.offset, 0);
    }

    #[test]
    fn test_convert_multiple_timezones() {
        let tzs = vec![
            "UTC".to_string(),
            "America/Denver".to_string(),
            "Europe/London".to_string(),
        ];
        let result = convert_to_timezones(&tzs);
        assert!(result.is_ok());
        let (_, zones) = result.unwrap();
        assert_eq!(zones.len(), 3);
    }

    #[test]
    fn test_invalid_timezone() {
        let result = convert_to_timezones(&["Invalid/Zone".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_timezone() {
        let result = convert_to_timezones(&["".to_string()]);
        assert!(result.is_ok());
        let (_, zones) = result.unwrap();
        assert_eq!(zones.len(), 0);
    }
}
