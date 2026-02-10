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

        // Spawn event loop handler
        tokio::spawn(async move {
            loop {
                eventloop.poll().await.unwrap();
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
