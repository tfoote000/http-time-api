use crate::models::PpsMessage;
use crate::mqtt::MqttClient;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Instant};
use tracing::{error, info};

/// Start PPS publishing task
pub async fn start_pps_task(mqtt_client: Arc<MqttClient>) {
    info!("Starting MQTT PPS publishing task");

    loop {
        // Calculate sleep duration to align with the next second boundary
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("System time error");
        let current_nanos = duration.subsec_nanos();
        let nanos_until_next_second = 1_000_000_000 - current_nanos;
        let sleep_duration = Duration::from_nanos(nanos_until_next_second as u64);

        // Sleep until next second
        sleep(sleep_duration).await;

        // Get current Unix timestamp (should be at the top of the second)
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("System time error");
        let unix_timestamp = duration.as_secs() as i64;

        // Create PPS message
        let message = PpsMessage {
            unix: unix_timestamp,
        };

        // Serialize to JSON
        match serde_json::to_vec(&message) {
            Ok(payload) => {
                // Publish with retain flag
                if let Err(e) = mqtt_client.publish("pps", payload, true).await {
                    error!("Failed to publish PPS message: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to serialize PPS message: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pps_timing() {
        // Test that we can calculate the sleep duration correctly
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).unwrap();
        let current_nanos = duration.subsec_nanos();
        let nanos_until_next_second = 1_000_000_000 - current_nanos;

        assert!(nanos_until_next_second > 0);
        assert!(nanos_until_next_second <= 1_000_000_000);
    }
}
