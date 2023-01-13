#[cfg(feature = "ssr")]
use anyhow::Result;

#[cfg(feature = "ssr")]
use remote_wol::server::server_start;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<()> {
    server_start().await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
}
