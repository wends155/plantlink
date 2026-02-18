use anyhow::Result;
use std::net::SocketAddr;
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

/// Reads data from Modbus TCP devices.
///
/// # Examples
///
/// ```no_run
/// use plantlink_core::modbus::ModbusTcpClient;
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
    ctx: Context,
}

impl ModbusTcpClient {
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let ctx = tcp::connect(addr).await?;
        Ok(Self { ctx })
    }

    pub async fn read_coils(&mut self, addr: u16, cnt: u16) -> Result<Vec<bool>> {
        let data = self.ctx.read_coils(addr, cnt).await?;
        Ok(data)
    }
}

// TODO: Implement Modbus Server
pub struct ModbusTcpServer {}
