#[cfg(feature = "ssr")]
use anyhow::Result;

#[cfg(feature = "ssr")]
use clap::Parser;
#[cfg(feature = "ssr")]
use dotenvy::dotenv;

#[cfg(feature = "ssr")]
use remote_wol::{cli::Args, server::server_start};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args = Args::parse();
    server_start(args).await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
}
