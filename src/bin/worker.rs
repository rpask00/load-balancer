use std::{env, net::SocketAddr, time::Duration};

use axum::{
    extract::{Request, State},
    Router,
};
use tokio::net::TcpListener;

use tokio::io::{AsyncBufReadExt, BufReader};

async fn worker_handler(State(port): State<u16>, req: Request) -> String {
    let message = format!(
        "worker on port {} received {} {}",
        port,
        req.method(),
        req.uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
    );

    tokio::time::sleep(Duration::from_secs(1)).await;

    message
}

#[tokio::main]
async fn main() {
    let port = env::args()
        .nth(1)
        .and_then(|port| port.parse().ok())
        .or_else(|| env::var("PORT").ok().and_then(|port| port.parse().ok()))
        .unwrap_or(3000);

    let app = Router::new().fallback(worker_handler).with_state(port);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("worker listening on http://{}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind worker port");

    let shutdown_signal = async {
        loop {
            let mut stdin = BufReader::new(tokio::io::stdin());
            let mut line = String::new();
            stdin.read_line(&mut line).await.unwrap();
            if line == "shutdown\n" {
                break;
            }
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .expect("server error");
}
