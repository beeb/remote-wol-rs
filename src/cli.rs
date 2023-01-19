use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The MAC address of the device to wake up
    #[arg(short, long = "mac")]
    pub mac_address: Option<String>,

    /// The IP of the device to wake up (for monitoring)
    #[arg(short, long = "ip")]
    pub ip_address: Option<String>,

    /// The passphrase to use to wake up the device
    #[arg(short, long = "pass")]
    pub passphrase: Option<String>,

    /// The local port used to serve the web app
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}
