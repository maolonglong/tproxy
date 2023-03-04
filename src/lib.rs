use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;
use backon::{ExponentialBuilder, Retryable};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time;
use tracing::{error, info, warn};

const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(15);

struct Listener {
    listener: TcpListener,
    to_addr: SocketAddr,
    shutdown_complete_tx: mpsc::Sender<()>,
}

struct Handler {
    inbound: TcpStream,
    outbound: TcpStream,
    _shutdown_complete: mpsc::Sender<()>,
}

pub async fn run(listener: TcpListener, to_addr: SocketAddr, shutdown: impl Future) {
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener {
        listener,
        to_addr,
        shutdown_complete_tx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    }

    let Listener {
        shutdown_complete_tx,
        ..
    } = server;

    drop(shutdown_complete_tx);

    if time::timeout(GRACEFUL_SHUTDOWN_TIMEOUT, shutdown_complete_rx.recv())
        .await
        .is_err()
    {
        warn!("graceful shutdown timeout");
    }
}

impl Listener {
    async fn run(&mut self) -> Result<()> {
        info!("accepting inbound connections");

        let accept = || async { self.listener.accept().await };
        let backoff_builder = ExponentialBuilder::default()
            .with_jitter()
            .with_max_times(64);

        loop {
            let (inbound, addr) = accept.retry(&backoff_builder).await?;
            info!(?addr, "new connection");

            let outbound = TcpStream::connect(&self.to_addr).await;
            if let Err(err) = outbound {
                error!(cause = ?err, "connection error");
                continue;
            }

            let mut handler = Handler {
                inbound,
                outbound: outbound.unwrap(),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                match handler.run().await {
                    Ok(_) => {
                        info!(?addr, "connection closed");
                    }
                    Err(err) => {
                        error!(?addr, cause = ?err, "connection error");
                    }
                }
            });
        }
    }
}

impl Handler {
    async fn run(&mut self) -> Result<()> {
        let (mut ri, mut wi) = self.inbound.split();
        let (mut ro, mut wo) = self.outbound.split();

        let client_to_server = async {
            io::copy(&mut ri, &mut wo).await?;
            wo.shutdown().await
        };

        let server_to_client = async {
            io::copy(&mut ro, &mut wi).await?;
            wi.shutdown().await
        };

        tokio::try_join!(client_to_server, server_to_client)?;

        Ok(())
    }
}
