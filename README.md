# tproxy

Minimal TCP relay (proxy).

## Lib

```rust
use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    tproxy::run(
        TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 6101))).await?,
        SocketAddr::from(([127, 0, 0, 1], 8080)),
        signal::ctrl_c(),
    )
    .await;

    Ok(())
}
```

## CLI

```bash
$ tproxy --help
Minimal TCP relay (proxy).

Usage: tproxy [OPTIONS] --to <TO>

Options:
  -f, --from <FROM>  Listen address [default: 127.0.0.1:6101]
  -t, --to <TO>      Address which relay to, like: 1.2.3.4:9999
  -h, --help         Print help
  -V, --version      Print version

$ tproxy --to 127.0.0.1:8080
2023-03-04T11:50:44.624316Z  INFO tproxy: accepting inbound connections
...
```
