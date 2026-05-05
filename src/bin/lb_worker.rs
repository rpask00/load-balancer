use axum::{
    extract::{Request, State},
    Router,
};
use clap::Parser;
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;

use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Parser)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Number of worker threads
    #[arg(short, long, default_value_t = 1)]
    num_threads: usize,
}

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

    // tokio::time::sleep(Duration::from_secs(1)).await;
    std::thread::sleep(Duration::from_secs(5));

    message
}

fn main() {
    let args = Args::parse();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.num_threads)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let app = Router::new().fallback(worker_handler).with_state(args.port);

        let addr = SocketAddr::from(([127, 0, 0, 1], args.port));

        let listener = TcpListener::bind(addr)
            .await
            .expect(&format!("failed to bind to {}", args.port));

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
    });
}
