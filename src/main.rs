#[cfg(feature = "ssr")]
use anyhow::Result;

#[cfg(feature = "ssr")]
use remote_wol::ssr::server_start;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<()> {
    server_start().await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
