use crate::error::ApiError;
use crate::models::{TimesQuery, TimesResponse};
use crate::time::{convert_to_timezones, ChronyTracker};
use axum::{extract::Query, response::Json, Extension};
use std::sync::Arc;

/// GET /times - Get current time in requested timezones
pub async fn times(
    Query(params): Query<TimesQuery>,
    Extension(chrony_tracker): Extension<Arc<ChronyTracker>>,
) -> Result<Json<TimesResponse>, ApiError> {
    // Parse comma-separated timezone list
    let timezone_names: Vec<String> = params
        .tz
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Limit number of timezones to prevent abuse
    const MAX_TIMEZONES: usize = 50;
    if timezone_names.len() > MAX_TIMEZONES {
        return Err(ApiError::Internal(format!(
            "Too many timezones requested (max: {})",
            MAX_TIMEZONES
        )));
    }

    // Convert to timezones
    let (unix_timestamp, zones) = convert_to_timezones(&timezone_names)?;

    // Optionally get time quality metrics
    let time_quality = if params.include_quality {
        chrony_tracker.get_quality().await
    } else {
        None
    };

    Ok(Json(TimesResponse {
        unix: unix_timestamp,
        zones,
        time_quality,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_timezone_list() {
        let params = TimesQuery {
            tz: "UTC,America/Denver,Europe/London".to_string(),
            include_quality: false,
        };

        let timezone_names: Vec<String> = params
            .tz
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(timezone_names.len(), 3);
        assert_eq!(timezone_names[0], "UTC");
        assert_eq!(timezone_names[1], "America/Denver");
        assert_eq!(timezone_names[2], "Europe/London");
    }

    #[tokio::test]
    async fn test_parse_timezone_with_spaces() {
        let params = TimesQuery {
            tz: " UTC , America/Denver , Europe/London ".to_string(),
            include_quality: false,
        };

        let timezone_names: Vec<String> = params
            .tz
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(timezone_names.len(), 3);
        assert_eq!(timezone_names[0], "UTC");
    }
}
