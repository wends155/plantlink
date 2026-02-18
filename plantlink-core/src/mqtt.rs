use anyhow::Result;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;

pub struct MqttDriver {
    client: AsyncClient,
}

impl MqttDriver {
    pub async fn connect(id: &str, host: &str, port: u16) -> Result<Self> {
        let mut mqttoptions = MqttOptions::new(id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Spawn event loop handler with exponential backoff retry
        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            let max_backoff = Duration::from_secs(60);

            loop {
                match eventloop.poll().await {
                    Ok(_event) => {
                        // Reset backoff on successful poll/reconnection
                        backoff = Duration::from_secs(1);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "MQTT event loop error: {}. Retrying in {:?}...",
                            e,
                            backoff
                        );
                        tokio::time::sleep(backoff).await;
                        // Exponential backoff capped at max_backoff
                        backoff = (backoff * 2).min(max_backoff);
                    }
                }
            }
        });

        Ok(Self { client })
    }

    pub async fn publish(&self, topic: &str, payload: Vec<u8>) -> Result<()> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await?;
        Ok(())
    }
}
