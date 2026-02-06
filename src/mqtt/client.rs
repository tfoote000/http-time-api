use crate::config::MqttConfig;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

/// MQTT client wrapper
pub struct MqttClient {
    client: AsyncClient,
    base_topic: String,
    _event_loop_handle: JoinHandle<()>,
}

impl MqttClient {
    /// Create a new MQTT client and start event loop
    pub fn new(config: &MqttConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse broker URL
        let url = url::Url::parse(&config.broker)?;
        let host = url.host_str().ok_or("Invalid broker host")?;
        let port = url.port().unwrap_or(1883);

        // Create MQTT options
        let mut mqtt_options = MqttOptions::new("time-api", host, port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        // Set credentials if provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password);
        }

        // Create client
        let (client, mut event_loop) = AsyncClient::new(mqtt_options, 10);

        // Spawn event loop task
        let event_loop_handle = tokio::spawn(async move {
            info!("MQTT event loop started");
            loop {
                match event_loop.poll().await {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        info!("MQTT connected to broker");
                    }
                    Ok(Event::Incoming(_)) => {
                        // Ignore other incoming packets
                    }
                    Ok(Event::Outgoing(_)) => {
                        // Ignore outgoing packets
                    }
                    Err(e) => {
                        error!("MQTT event loop error: {}", e);
                        // Wait before retrying
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(Self {
            client,
            base_topic: config.base_topic.clone(),
            _event_loop_handle: event_loop_handle,
        })
    }

    /// Publish a message to a topic
    pub async fn publish(
        &self,
        subtopic: &str,
        payload: Vec<u8>,
        retain: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let topic = format!("{}/{}", self.base_topic, subtopic);
        self.client
            .publish(&topic, QoS::AtLeastOnce, retain, payload)
            .await?;
        Ok(())
    }

    /// Get the base topic
    pub fn base_topic(&self) -> &str {
        &self.base_topic
    }
}
