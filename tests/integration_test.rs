use std::future;
use std::net::SocketAddr;
use std::time::Duration;

use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use reqwest::IntoUrl;
use tokio::net::TcpListener;
use tokio::time;

#[tokio::test]
async fn test() {
    let url = setup_test_server().await;

    let resp = reqwest::get(url).await.unwrap().text().await.unwrap();
    assert_eq!(resp, "Hello, tproxy!")
}

async fn setup_test_server() -> impl IntoUrl {
    async fn pick_random_addr() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        listener.local_addr().unwrap()
    }

    let from_addr = pick_random_addr().await;
    let to_addr = pick_random_addr().await;

    async fn handler() -> impl IntoResponse {
        "Hello, tproxy!"
    }

    let app = Router::new().route("/", get(handler));

    tokio::spawn(async move {
        axum::Server::bind(&to_addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    tokio::spawn(async move {
        let listener = TcpListener::bind(from_addr).await.unwrap();
        tproxy::run(listener, to_addr, future::pending::<()>()).await
    });

    time::sleep(Duration::from_secs(1)).await;
    format!("http://{}", from_addr.to_string())
}
