use std::env;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// HTTP server configuration
    pub http: HttpConfig,

    /// Optional TLS configuration
    pub tls: Option<TlsConfig>,

    /// Optional MQTT configuration
    pub mqtt: Option<MqttConfig>,

    /// Logging level
    pub log_level: String,
}

#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Bind host
    pub host: String,

    /// Bind port
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to TLS certificate file (PEM format)
    pub cert_path: PathBuf,

    /// Path to TLS private key file (PEM format)
    pub key_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MqttConfig {
    /// MQTT broker URL (e.g., "mqtt://localhost:1883")
    pub broker: String,

    /// Optional username
    pub username: Option<String>,

    /// Optional password
    pub password: Option<String>,

    /// Base topic for all publishes
    pub base_topic: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let http = HttpConfig {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8463".to_string())
                .parse()?,
        };

        let tls = if let (Ok(cert_path), Ok(key_path)) = (
            env::var("TLS_CERT_PATH"),
            env::var("TLS_KEY_PATH"),
        ) {
            Some(TlsConfig {
                cert_path: PathBuf::from(cert_path),
                key_path: PathBuf::from(key_path),
            })
        } else {
            None
        };

        let mqtt = if let Ok(broker) = env::var("MQTT_BROKER") {
            Some(MqttConfig {
                broker,
                username: env::var("MQTT_USERNAME").ok(),
                password: env::var("MQTT_PASSWORD").ok(),
                base_topic: env::var("MQTT_BASE_TOPIC")
                    .unwrap_or_else(|_| "time-api".to_string()),
            })
        } else {
            None
        };

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        Ok(Config {
            http,
            tls,
            mqtt,
            log_level,
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate port range
        if self.http.port == 0 {
            return Err("PORT must be greater than 0".to_string());
        }

        // Validate TLS paths if configured
        if let Some(ref tls) = self.tls {
            if !tls.cert_path.exists() {
                return Err(format!("TLS certificate not found: {:?}", tls.cert_path));
            }
            if !tls.key_path.exists() {
                return Err(format!("TLS private key not found: {:?}", tls.key_path));
            }
        }

        // Validate MQTT broker URL if configured
        if let Some(ref mqtt) = self.mqtt {
            if !mqtt.broker.starts_with("mqtt://") && !mqtt.broker.starts_with("mqtts://") {
                return Err("MQTT_BROKER must start with mqtt:// or mqtts://".to_string());
            }
        }

        Ok(())
    }
}
