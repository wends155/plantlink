//! # Modbus TCP Driver
//!
//! This module provides the [`ModbusTcpClient`] implementation of the [`ModbusClient`] trait.
//! It handles industrial communication with devices over Modbus TCP.

use crate::PlantLinkError;
use crate::traits::ModbusClient;
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
/// # async fn example() -> Result<(), plantlink_core::PlantLinkError> {
/// let addr: SocketAddr = "192.168.1.100:502".parse().unwrap();
/// let client = ModbusTcpClient::connect(addr).await?;
/// let coils = client.read_coils(0, 10).await?;
/// # Ok(())
/// # }
/// ```
pub struct ModbusTcpClient {
    ctx: Mutex<tokio_modbus::client::Context>,
}

impl ModbusTcpClient {
    /// Connects to a Modbus TCP device at the specified address.
    ///
    /// # Errors
    /// Returns a [`PlantLinkError::Connection`] if the connection fails.
    #[tracing::instrument(err)]
    pub async fn connect(addr: SocketAddr) -> Result<Self, PlantLinkError> {
        let ctx = tcp::connect(addr)
            .await
            .map_err(|e| PlantLinkError::Connection(e.to_string()))?;
        Ok(Self {
            ctx: Mutex::new(ctx),
        })
    }
}

#[async_trait::async_trait]
impl ModbusClient for ModbusTcpClient {
    /// Reads the specified number of coils starting from the given address.
    ///
    /// # Errors
    /// Returns a [`PlantLinkError::Modbus`] if the read operation fails.
    #[tracing::instrument(skip(self), err)]
    async fn read_coils(&self, addr: u16, cnt: u16) -> Result<Vec<bool>, PlantLinkError> {
        let mut ctx = self.ctx.lock().await;
        let data = ctx
            .read_coils(addr, cnt)
            .await
            .map_err(|e| PlantLinkError::Modbus(e.to_string()))?;
        Ok(data)
    }
}
