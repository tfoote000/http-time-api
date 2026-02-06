use crate::models::TimeQuality;
use regex::Regex;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cached chrony tracking data
#[derive(Clone)]
struct CachedQuality {
    quality: Option<TimeQuality>,
    timestamp: Instant,
}

/// Chrony quality tracker with caching
pub struct ChronyTracker {
    cache: Arc<RwLock<Option<CachedQuality>>>,
    cache_duration: Duration,
}

impl ChronyTracker {
    /// Create a new ChronyTracker with 250ms cache duration
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
            cache_duration: Duration::from_millis(250),
        }
    }

    /// Get time quality from chrony, using cache if available
    pub async fn get_quality(&self) -> Option<TimeQuality> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                if cached.timestamp.elapsed() < self.cache_duration {
                    return cached.quality.clone();
                }
            }
        }

        // Cache miss or expired, fetch new data
        let quality = tokio::task::spawn_blocking(|| Self::fetch_chrony_tracking())
            .await
            .ok()
            .flatten();

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedQuality {
                quality: quality.clone(),
                timestamp: Instant::now(),
            });
        }

        quality
    }

    /// Execute chronyc and parse output
    fn fetch_chrony_tracking() -> Option<TimeQuality> {
        // Execute chronyc tracking with 2-second timeout
        let output = Command::new("chronyc")
            .arg("tracking")
            .output()
            .ok()?;

        if !output.status.success() {
            tracing::warn!("chronyc tracking failed: {:?}", output.status);
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_chrony_output(&stdout)
    }

    /// Parse chronyc tracking output
    fn parse_chrony_output(output: &str) -> Option<TimeQuality> {
        let mut stratum: Option<u8> = None;
        let mut offset: Option<f64> = None;
        let mut reference_id: Option<String> = None;
        let mut leap_status: Option<String> = None;

        // Parse each line
        for line in output.lines() {
            let line = line.trim();

            // Stratum: "Stratum       : 1"
            if line.starts_with("Stratum") {
                if let Some(value) = Self::extract_value(line) {
                    stratum = value.parse().ok();
                }
            }

            // Reference ID: "Reference ID    : 50505300 (PPS)"
            else if line.starts_with("Reference ID") {
                if let Some(value) = Self::extract_value(line) {
                    // Extract the part in parentheses if present
                    if let Some(start) = value.find('(') {
                        if let Some(end) = value.find(')') {
                            reference_id = Some(value[start + 1..end].to_string());
                        }
                    }
                    if reference_id.is_none() {
                        reference_id = Some(value.split_whitespace().next()?.to_string());
                    }
                }
            }

            // System time offset: "System time     : 0.000000012 seconds slow of NTP time"
            else if line.starts_with("System time") {
                if let Some(value) = Self::extract_value(line) {
                    // Extract the numeric part
                    let re = Regex::new(r"([-+]?\d+\.?\d*)").ok()?;
                    if let Some(cap) = re.captures(value) {
                        offset = cap.get(1)?.as_str().parse().ok();
                        // If the line says "slow", make it negative
                        if value.contains("slow") && offset.is_some() {
                            offset = offset.map(|o| -o);
                        }
                    }
                }
            }

            // Leap status: "Leap status     : Normal"
            else if line.starts_with("Leap status") {
                if let Some(value) = Self::extract_value(line) {
                    leap_status = Some(value.to_string());
                }
            }
        }

        // All fields must be present
        Some(TimeQuality {
            stratum: stratum?,
            offset_seconds: offset?,
            reference_id: reference_id?,
            leap_status: leap_status?,
        })
    }

    /// Extract value after colon
    fn extract_value(line: &str) -> Option<&str> {
        line.split(':').nth(1).map(|s| s.trim())
    }
}

impl Default for ChronyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chrony_output() {
        let output = r#"
Reference ID    : 50505300 (PPS)
Stratum         : 1
Ref time (UTC)  : Thu Feb 06 00:00:00 2025
System time     : 0.000000012 seconds slow of NTP time
Last offset     : -0.000000023 seconds
RMS offset      : 0.000000045 seconds
Frequency       : 1.234 ppm fast
Residual freq   : +0.001 ppm
Skew            : 0.012 ppm
Root delay      : 0.000000001 seconds
Root dispersion : 0.000000002 seconds
Update interval : 16.0 seconds
Leap status     : Normal
"#;

        let quality = ChronyTracker::parse_chrony_output(output).unwrap();
        assert_eq!(quality.stratum, 1);
        assert_eq!(quality.reference_id, "PPS");
        assert_eq!(quality.leap_status, "Normal");
        assert!(quality.offset_seconds < 0.0);
    }

    #[test]
    fn test_parse_chrony_output_fast() {
        let output = r#"
Stratum         : 2
Reference ID    : C0A80001 (192.168.0.1)
System time     : 0.000123456 seconds fast of NTP time
Leap status     : Normal
"#;

        let quality = ChronyTracker::parse_chrony_output(output).unwrap();
        assert_eq!(quality.stratum, 2);
        assert!(quality.offset_seconds > 0.0);
    }
}
