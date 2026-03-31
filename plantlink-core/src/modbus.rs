//! # Modbus TCP Driver
//!
//! This module provides the [`ModbusTcpClient`] implementation of the [`ModbusClient`] trait.
//! It handles industrial communication with devices over Modbus TCP.

use crate::PlantLinkError;
use crate::traits::ModbusClient;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio_modbus::prelude::{Reader, tcp};

enum ModbusCommand {
    ReadCoils {
        addr: u16,
        cnt: u16,
        resp: oneshot::Sender<Result<Vec<bool>, PlantLinkError>>,
    },
}

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
/// let addr: SocketAddr = "192.168.1.100:502".parse()
/// #    .unwrap();
/// let client = ModbusTcpClient::connect(addr).await?;
/// let coils = client.read_coils(0, 10).await?;
/// # Ok(())
/// # }
/// ```
pub struct ModbusTcpClient {
    tx: mpsc::Sender<ModbusCommand>,
}

impl ModbusTcpClient {
    /// Connects to a Modbus TCP device at the specified address.
    ///
    /// # Errors
    /// Returns a [`PlantLinkError::Connection`] if the initial connection fails.
    #[tracing::instrument(err)]
    pub async fn connect(addr: SocketAddr) -> Result<Self, PlantLinkError> {
        let (tx, mut rx) = mpsc::channel(32);

        // Initial connection check to satisfy the async return type
        let initial_ctx = tcp::connect(addr)
            .await
            .map_err(|e| PlantLinkError::Connection(Arc::new(e)))?;

        // ast-grep-ignore: raw-tokio-spawn
        tokio::spawn(async move {
            let mut ctx = Some(initial_ctx);
            let mut backoff = std::time::Duration::from_secs(1);
            let max_backoff = std::time::Duration::from_secs(60);
            let mut last_failure: Option<std::time::Instant> = None;

            while let Some(cmd) = rx.recv().await {
                // On-demand connection recovery with exponential backoff
                if ctx.is_none() {
                    // Check backoff cooldown
                    if let Some(last) = last_failure {
                        let elapsed = last.elapsed();
                        if elapsed < backoff {
                            let retry_in = backoff.checked_sub(elapsed).unwrap_or_default();
                            match cmd {
                                ModbusCommand::ReadCoils { resp, .. } => {
                                    let _ = resp.send(Err(PlantLinkError::Connection(Arc::new(
                                        crate::error::SimpleError(format!(
                                            "reconnect backoff: retry in {}s",
                                            retry_in.as_secs().max(1)
                                        )),
                                    ))));
                                }
                            }
                            continue;
                        }
                    }

                    tracing::debug!("Modbus attempting reconnect to {}", addr);
                    match tcp::connect(addr).await {
                        Ok(c) => {
                            ctx = Some(c);
                            backoff = std::time::Duration::from_secs(1);
                            last_failure = None;
                            tracing::info!("Modbus reconnected to {}", addr);
                        }
                        Err(e) => {
                            last_failure = Some(std::time::Instant::now());
                            backoff = (backoff * 2).min(max_backoff);
                            match cmd {
                                ModbusCommand::ReadCoils { resp, .. } => {
                                    let _ = resp.send(Err(PlantLinkError::Connection(Arc::new(e))));
                                }
                            }
                            continue;
                        }
                    }
                }

                // Dispatch command
                if let Some(mut c) = ctx.take() {
                    match cmd {
                        ModbusCommand::ReadCoils { addr, cnt, resp } => {
                            match c.read_coils(addr, cnt).await {
                                Ok(res) => {
                                    let _ = resp.send(Ok(res));
                                    ctx = Some(c); // Command succeeded, keep context
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Modbus command failed: {}, dropping connection",
                                        e
                                    );
                                    let _ = resp.send(Err(PlantLinkError::Modbus(Arc::new(e))));
                                    // Drop context on error to force reconnect next time
                                    ctx = None;
                                    last_failure = Some(std::time::Instant::now());
                                    // Start with initial backoff since we just lost connection
                                    backoff = std::time::Duration::from_secs(1);
                                }
                            }
                        }
                    }
                }
            }
            tracing::info!("Modbus actor task shutting down");
        });

        Ok(Self { tx })
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
        let (resp_tx, resp_rx) = oneshot::channel();
        self.tx
            .send(ModbusCommand::ReadCoils {
                addr,
                cnt,
                resp: resp_tx,
            })
            .await
            .map_err(|e| {
                PlantLinkError::Connection(Arc::new(crate::error::SimpleError(format!(
                    "actor task died: {e}"
                ))))
            })?;

        resp_rx.await.map_err(|e| {
            PlantLinkError::Connection(Arc::new(crate::error::SimpleError(format!(
                "response channel closed: {e}"
            ))))
        })?
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time;

    #[tokio::test(start_paused = true)]
    async fn test_modbus_reconnect_backoff() {
        // Bind to a random port to get a valid address
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // 1. Initial connection succeeds (port exists)
        let client = ModbusTcpClient::connect(addr)
            .await
            .expect("Initial connect failed");

        // 2. Close the listener to simulate server crash
        drop(listener);

        // 3. First command: should fail due to broken connection (connection closed by dropped listener)
        let _ = client.read_coils(0, 1).await;

        // Give the actor a moment to process the failure and set ctx = None
        time::sleep(Duration::from_millis(10)).await;

        // 4. Second command: should trigger backoff
        let res = client.read_coils(0, 1).await;

        match res {
            Err(PlantLinkError::Connection(e)) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("reconnect backoff"),
                    "Expected backoff error message, got: {msg}"
                );
            }
            other => panic!("Expected backoff connection error, got: {other:?}"),
        }
    }
}
