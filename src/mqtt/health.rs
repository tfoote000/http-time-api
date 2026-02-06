use crate::models::{CheckStatus, HealthChecks, MqttHealthMessage};
use crate::mqtt::MqttClient;
use crate::time::ChronyTracker;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{error, info};

/// Start health publishing task
pub async fn start_health_task(mqtt_client: Arc<MqttClient>, chrony_tracker: Arc<ChronyTracker>) {
    info!("Starting MQTT health publishing task");

    let mut last_status: Option<String> = None;
    let mut last_publish = Instant::now();
    const MIN_PUBLISH_INTERVAL: Duration = Duration::from_secs(5);
    const POLL_INTERVAL: Duration = Duration::from_secs(1);

    loop {
        // Poll health status
        let (status, checks, time_quality) = check_health(&chrony_tracker).await;

        // Check if status changed
        let status_changed = last_status.as_ref() != Some(&status);

        // Publish if status changed and enough time has passed since last publish
        if status_changed && last_publish.elapsed() >= MIN_PUBLISH_INTERVAL {
            // Get current Unix timestamp
            let now = SystemTime::now();
            let timestamp = now
                .duration_since(UNIX_EPOCH)
                .expect("System time error")
                .as_secs() as i64;

            // Create health message
            let message = MqttHealthMessage {
                status: status.clone(),
                timestamp,
                checks,
                time_quality,
            };

            // Serialize to JSON
            match serde_json::to_vec(&message) {
                Ok(payload) => {
                    // Publish with retain flag
                    if let Err(e) = mqtt_client.publish("health", payload, true).await {
                        error!("Failed to publish health message: {}", e);
                    } else {
                        info!("Published health status: {}", status);
                        last_status = Some(status.clone());
                        last_publish = Instant::now();
                    }
                }
                Err(e) => {
                    error!("Failed to serialize health message: {}", e);
                }
            }
        } else if status_changed {
            // Status changed but rate limited
            info!("Health status changed to {}, but rate limited", status);
        }

        // Sleep before next poll
        sleep(POLL_INTERVAL).await;
    }
}

/// Check health status (similar to health endpoint)
async fn check_health(
    chrony_tracker: &Arc<ChronyTracker>,
) -> (String, HealthChecks, Option<crate::models::TimeQuality>) {
    // Check system clock
    let system_clock = check_system_clock();

    // Check chrony and get time quality
    let (chrony_check, time_quality) = check_chrony(chrony_tracker).await;

    // Determine overall status
    let status = determine_status(&system_clock, &chrony_check, &time_quality);

    let checks = HealthChecks {
        system_clock,
        chrony: chrony_check,
    };

    (status, checks, time_quality)
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
async fn check_chrony(
    chrony_tracker: &Arc<ChronyTracker>,
) -> (CheckStatus, Option<crate::models::TimeQuality>) {
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
