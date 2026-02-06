use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query parameters for /times endpoint
#[derive(Debug, Deserialize)]
pub struct TimesQuery {
    /// Comma-separated list of IANA timezone names
    #[serde(default = "default_timezones")]
    pub tz: String,

    /// Include time quality metrics from chrony
    #[serde(default)]
    pub include_quality: bool,
}

fn default_timezones() -> String {
    "UTC".to_string()
}

/// Response for /times endpoint
#[derive(Debug, Serialize)]
pub struct TimesResponse {
    /// Unix timestamp in seconds (integer)
    pub unix: i64,

    /// Timezone information
    pub zones: HashMap<String, ZoneInfo>,

    /// Optional time quality metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_quality: Option<TimeQuality>,
}

/// Information about a specific timezone
#[derive(Debug, Serialize)]
pub struct ZoneInfo {
    /// Local time in ISO8601 format without timezone suffix (YYYY-MM-DDTHH:MM:SS)
    pub local: String,

    /// Offset from UTC in seconds
    pub offset: i32,
}

/// Time quality metrics from chrony
#[derive(Debug, Serialize, Clone)]
pub struct TimeQuality {
    /// NTP stratum level (0-16)
    pub stratum: u8,

    /// System time offset in seconds
    pub offset_seconds: f64,

    /// Reference ID (e.g., "PPS", "GPS")
    pub reference_id: String,

    /// Leap status (e.g., "Normal", "Insert second", "Delete second")
    pub leap_status: String,
}

/// Response for /health endpoint
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Overall status: "healthy", "degraded", or "unhealthy"
    pub status: String,

    /// Individual health checks
    pub checks: HealthChecks,

    /// Optional time quality details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_quality: Option<TimeQuality>,
}

#[derive(Debug, Serialize)]
pub struct HealthChecks {
    /// System clock check
    pub system_clock: CheckStatus,

    /// Chrony reachability
    pub chrony: CheckStatus,
}

#[derive(Debug, Serialize)]
pub struct CheckStatus {
    /// Check result: "ok", "warning", "error"
    pub status: String,

    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl CheckStatus {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            message: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            status: "warning".to_string(),
            message: Some(message.into()),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            message: Some(message.into()),
        }
    }
}

/// MQTT PPS message
#[derive(Debug, Serialize)]
pub struct PpsMessage {
    pub unix: i64,
}

/// MQTT Health message
#[derive(Debug, Serialize)]
pub struct MqttHealthMessage {
    pub status: String,
    pub timestamp: i64,
    pub checks: HealthChecks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_quality: Option<TimeQuality>,
}
