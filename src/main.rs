use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use tokio::net::TcpListener;
use tokio::signal;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Listen address
    #[arg(short, long, default_value = "127.0.0.1:6101")]
    from: SocketAddr,

    /// Address which relay to, like: 1.2.3.4:9999
    #[arg(short, long)]
    to: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let listener = TcpListener::bind(&args.from).await?;

    tproxy::run(listener, args.to, signal::ctrl_c()).await;

    Ok(())
}
