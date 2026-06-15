use anyhow::Result;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;

use crate::config::MqttConfig;

pub struct MqttPublisher {
    client: AsyncClient,
    topic: String,
    eventloop_handle: tokio::task::JoinHandle<()>,
}

impl MqttPublisher {
    pub async fn new(config: &MqttConfig) -> Result<Self> {
        let mut mqtt_options = MqttOptions::new("thoth", &config.broker, config.port);
        mqtt_options.set_credentials(&config.username, &config.password);
        mqtt_options.set_keep_alive(Duration::from_secs(30));
        mqtt_options.set_clean_session(true);

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 100);

        let eventloop_handle = tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        tracing::info!("MQTT connected successfully");
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("MQTT event loop error: {e}");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(Self {
            client,
            topic: config.topic.clone(),
            eventloop_handle,
        })
    }

    pub async fn publish_json(&self, payload: &serde_json::Value) -> Result<()> {
        let json_str = serde_json::to_string(payload)?;
        self.client
            .publish(&self.topic, QoS::AtLeastOnce, false, json_str)
            .await?;
        tracing::info!("MQTT message published to {}", self.topic);
        Ok(())
    }
}

impl Drop for MqttPublisher {
    fn drop(&mut self) {
        self.eventloop_handle.abort();
    }
}
