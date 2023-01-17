//! Ping a device with ICMP packets
use std::net::IpAddr;

use anyhow::Result;
use core::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence, ICMP};

/// Ping a device
pub struct Pinger {
    client_v4: Client,
    client_v6: Client,
}

impl Pinger {
    /// Create a new pinger instance
    pub fn new() -> Result<Self> {
        let config_v4 = Config::default();
        let config_v6 = Config::builder().kind(ICMP::V6).build();
        let client_v4 = Client::new(&config_v4)?;
        let client_v6 = Client::new(&config_v6)?;
        Ok(Self {
            client_v4,
            client_v6,
        })
    }

    /// Ping a device and return the result
    pub async fn ping(&self, ip_addr: IpAddr, timeout: Option<Duration>) -> Result<()> {
        let timeout = timeout.unwrap_or(Duration::from_millis(2000));
        let payload = vec![0; 56];

        let client = match ip_addr {
            IpAddr::V4(_) => &self.client_v4,
            IpAddr::V6(_) => &self.client_v6,
        };
        let mut pinger = client
            .pinger(ip_addr, PingIdentifier(fastrand::u16(..)))
            .await;
        pinger.timeout(timeout);
        let _ = pinger.ping(PingSequence(0), &payload).await?;
        Ok(())
    }
}
