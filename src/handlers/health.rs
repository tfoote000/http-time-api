use crate::models::{CheckStatus, HealthChecks, HealthResponse};
use crate::time::ChronyTracker;
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// GET /health - Health check endpoint
pub async fn health(
    Extension(chrony_tracker): Extension<Arc<ChronyTracker>>,
) -> impl IntoResponse {
    // Check system clock
    let system_clock = check_system_clock();

    // Check chrony and get time quality
    let (chrony_check, time_quality) = check_chrony(&chrony_tracker).await;

    // Determine overall status
    let status = determine_status(&system_clock, &chrony_check, &time_quality);

    let response = HealthResponse {
        status: status.clone(),
        checks: HealthChecks {
            system_clock,
            chrony: chrony_check,
        },
        time_quality,
    };

    // Return 503 if unhealthy, 200 otherwise
    let status_code = if status == "unhealthy" {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    (status_code, Json(response))
}

/// GET /ready - Readiness/liveness check
pub async fn ready() -> impl IntoResponse {
    // Simple check - if we can respond, we're ready
    StatusCode::OK
}

/// Check if system clock is sane (year between 2020 and 2100)
fn check_system_clock() -> CheckStatus {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let unix = duration.as_secs() as i64;
            // 2020-01-01 00:00:00 UTC = 1577836800
            // 2100-01-01 00:00:00 UTC = 4102444800
            if unix >= 1577836800 && unix <= 4102444800 {
                CheckStatus::ok()
            } else {
                CheckStatus::error(format!("System clock out of range: {}", unix))
            }
        }
        Err(e) => CheckStatus::error(format!("System clock error: {}", e)),
    }
}

/// Check chrony and get time quality
async fn check_chrony(chrony_tracker: &Arc<ChronyTracker>) -> (CheckStatus, Option<crate::models::TimeQuality>) {
    match chrony_tracker.get_quality().await {
        Some(quality) => (CheckStatus::ok(), Some(quality)),
        None => (
            CheckStatus::warning("chrony unavailable or not synchronized".to_string()),
            None,
        ),
    }
}

/// Determine overall health status
fn determine_status(
    system_clock: &CheckStatus,
    chrony: &CheckStatus,
    time_quality: &Option<crate::models::TimeQuality>,
) -> String {
    // If system clock is broken, we're unhealthy
    if system_clock.status == "error" {
        return "unhealthy".to_string();
    }

    // If chrony is unavailable, we're degraded
    if chrony.status != "ok" {
        return "degraded".to_string();
    }

    // Check stratum if we have quality data
    if let Some(ref quality) = time_quality {
        if quality.stratum >= 16 {
            return "unhealthy".to_string();
        } else if quality.stratum >= 4 {
            return "degraded".to_string();
        }
    }

    "healthy".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimeQuality;

    #[test]
    fn test_determine_status_healthy() {
        let system_clock = CheckStatus::ok();
        let chrony = CheckStatus::ok();
        let quality = Some(TimeQuality {
            stratum: 1,
            offset_seconds: 0.000001,
            reference_id: "PPS".to_string(),
            leap_status: "Normal".to_string(),
        });

        let status = determine_status(&system_clock, &chrony, &quality);
        assert_eq!(status, "healthy");
    }

    #[test]
    fn test_determine_status_degraded_stratum() {
        let system_clock = CheckStatus::ok();
        let chrony = CheckStatus::ok();
        let quality = Some(TimeQuality {
            stratum: 5,
            offset_seconds: 0.000001,
            reference_id: "NTP".to_string(),
            leap_status: "Normal".to_string(),
        });

        let status = determine_status(&system_clock, &chrony, &quality);
        assert_eq!(status, "degraded");
    }

    #[test]
    fn test_determine_status_unhealthy_stratum() {
        let system_clock = CheckStatus::ok();
        let chrony = CheckStatus::ok();
        let quality = Some(TimeQuality {
            stratum: 16,
            offset_seconds: 0.0,
            reference_id: "NONE".to_string(),
            leap_status: "Normal".to_string(),
        });

        let status = determine_status(&system_clock, &chrony, &quality);
        assert_eq!(status, "unhealthy");
    }

    #[test]
    fn test_determine_status_degraded_no_chrony() {
        let system_clock = CheckStatus::ok();
        let chrony = CheckStatus::warning("chrony unavailable");
        let quality = None;

        let status = determine_status(&system_clock, &chrony, &quality);
        assert_eq!(status, "degraded");
    }

    #[test]
    fn test_determine_status_unhealthy_clock() {
        let system_clock = CheckStatus::error("Clock error");
        let chrony = CheckStatus::ok();
        let quality = None;

        let status = determine_status(&system_clock, &chrony, &quality);
        assert_eq!(status, "unhealthy");
    }
}
