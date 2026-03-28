use crate::traits::ModbusClient;
use anyhow::{Context as _, Result};
use std::net::SocketAddr;
use tokio::sync::Mutex;
use tokio_modbus::prelude::*;

/// Reads data from Modbus TCP devices.
///
/// # Examples
///
/// ```no_run
/// use plantlink_core::modbus::ModbusTcpClient;
/// use plantlink_core::traits::ModbusClient;
/// use std::net::SocketAddr;
///
/// # async fn example() -> anyhow::Result<()> {
/// let addr: SocketAddr = "192.168.1.100:502".parse()?;
/// let mut client = ModbusTcpClient::connect(addr).await?;
/// let coils = client.read_coils(0, 10).await?;
/// println!("Coils: {:?}", coils);
/// # Ok(())
/// # }
/// ```
pub struct ModbusTcpClient {
    ctx: Mutex<tokio_modbus::client::Context>,
}

impl ModbusTcpClient {
    ///
    /// # Errors
    /// Returns an error if the connection to the Modbus server fails.
    #[tracing::instrument(err)]
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let ctx = tcp::connect(addr)
            .await
            .context("Failed to connect to Modbus device")?;
        Ok(Self {
            ctx: Mutex::new(ctx),
        })
    }
}

#[async_trait::async_trait]
impl ModbusClient for ModbusTcpClient {
    ///
    /// # Errors
    /// Returns an error if reading from the Modbus server fails.
    #[tracing::instrument(skip(self), err)]
    async fn read_coils(&mut self, addr: u16, cnt: u16) -> Result<Vec<bool>> {
        let mut ctx = self.ctx.lock().await;
        let data = ctx
            .read_coils(addr, cnt)
            .await
            .context("Failed to read Modbus coils")?;
        Ok(data)
    }
}
