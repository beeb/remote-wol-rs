use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub mac_address: Option<String>,

    #[arg(long = "ip")]
    pub ip_address: Option<String>,

    #[arg(long = "pass")]
    pub passphrase: Option<String>,

    #[arg(long = "site")]
    pub site_addr: Option<String>,
}
