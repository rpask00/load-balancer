use bytes::Bytes;
use color_eyre::eyre::{eyre, Result};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::{body::Incoming, service::service_fn, Method, Request, Response};
use hyper_util::rt::TokioIo;
use load_balancer::load_balancer::load_balancer::LoadBalancer;
use load_balancer::load_balancer::strategy::round_robin::RoundRobinStrategy;
use load_balancer::load_balancer::strategy::LoadBalancingStrategy;
use load_balancer::tui::app::App;
use load_balancer::tui::ui::draw;
use log::LevelFilter;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use simplelog::{Config, WriteLogger};
use std::fs::File;
use std::sync::RwLock;
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use std::{io, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, task};

type BodyError = Box<dyn std::error::Error + Send + Sync>;
type BoxBodyResponse = Response<BoxBody<Bytes, BodyError>>;

async fn handle(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<BoxBodyResponse> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/strategy") => set_strategy_handler(req, load_balancer).await,
        _ => {
            let (worker, req) = {
                let lb_lock = load_balancer
                    .read()
                    .expect("Could not get read lock on load_balancer");

                let worker = lb_lock.strategy.select_worker(&lb_lock.workers)?;
                let req =
                    lb_lock.prepare_request(format!("http://localhost:{}", worker.port), req)?;
                (worker, req)
            };

            worker
                .handle(req)
                .await
                .map(|res| res.map(|body| body.boxed()))
                .map_err(|e| e.into())
        }
    }
}

async fn set_strategy_handler(
    req: Request<Incoming>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<BoxBodyResponse> {
    let body = req.collect().await?.to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();

    let strategy_name = body["strategy"]
        .as_str()
        .ok_or(eyre!("Strategy not found in request body!"))?;

    let result = load_balancer
        .write()
        .expect("Could not get write lock on load_balancer")
        .set_strategy_handler(strategy_name);

    let (status, response_msg) = match result.is_ok() {
        true => (200, "ok"),
        false => (400, "unknown strategy"),
    };

    Ok(Response::builder().status(status).body(
        Full::new(Bytes::from(response_msg))
            .map_err(|_| unreachable!())
            .boxed(),
    )?)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("tui.log")?,
    )
    .expect("Failed to setup log system");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // let default_strategy = Box::new(LeastConnectionStrategy::new());
    let default_strategy = Box::new(RoundRobinStrategy::new());

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(default_strategy).expect("failed to create load balancer"),
    ));

    let _ = std::thread::spawn({
        let load_balancer = Arc::clone(&load_balancer);
        move || {
            for t in 0..5 {
                sleep(Duration::from_secs(t));

                let num_threads: u8 = rand::random::<u8>() % 3 + 1;

                let _ = load_balancer
                    .write()
                    .expect("Could not get write lock on load_balancer")
                    .spawn_worker(num_threads, format!("Worker {}", t + 1), None);
            }
        }
    });

    let _: JoinHandle<Result<()>> = std::thread::spawn({
        let load_balancer = Arc::clone(&load_balancer);
        move || {
            let runtime = tokio::runtime::Builder::new_current_thread().build()?;

            let mut app = App::new(Arc::clone(&load_balancer));

            while !app.should_quit {
                terminal.draw(|f| draw(f, &mut app))?;

                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    if let Ok(event) = event::read() {
                        let _ = app.handle_event(event);
                    }
                }

                if let Ok(mut load_balancer) = load_balancer.try_write() {
                    runtime.block_on(async {
                        load_balancer.prune_workers().await;
                    });
                }
            }

            disable_raw_mode()?;

            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;

            Ok(())
        }
    });

    let _ = std::thread::spawn({
        let load_balancer = Arc::clone(&load_balancer);
        move || loop {
            if let Ok(load_balancer) = load_balancer.try_read() {
                load_balancer.health_check();
            }
            sleep(Duration::from_secs(5));
        }
    });

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    loop {
        let (stream, _) = listener.accept().await?;
        let load_balancer = Arc::clone(&load_balancer);

        task::spawn(async move {
            let io = TokioIo::new(stream);
            let service = service_fn(move |req| handle(req, Arc::clone(&load_balancer)));

            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                log::error!("{}", e);
            }
        });
    }
}
