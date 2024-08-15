#![deny(
    clippy::enum_glob_use,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used
)]

use axum::{response::Redirect, routing, Router};
use clap::Parser as _;
use color_eyre::{eyre::Context as _, Result};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[derive(clap::Parser, Debug)]
struct AppConfig {
    /// Port to run the app on
    #[arg(
        long,
        value_name = "PORT",
        default_value_t = 3779,
        env = "SUWARIYOMI_PORT"
    )]
    port: u16,

    #[arg(
        long,
        value_name = "THREADS",
        default_value_t = 0,
        env = "SUWARIYOMI_NUM_THREADS"
    )]
    num_threads: usize,
}

#[derive(Clone)]
struct AppState {}

impl From<AppConfig> for AppState {
    fn from(_value: AppConfig) -> Self {
        Self {}
    }
}

async fn inner_main(config: AppConfig) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let state: AppState = config.into();
    let app = Router::new()
        .route(
            "/",
            routing::get(|| async { Redirect::permanent("/browse/") }),
        )
        .with_state(state);

    info!("Server listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "suwariyomi=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::parse();
    info!(?config, "Starting app");

    let rt = match config.num_threads {
        0 => {
            let mut builder = tokio::runtime::Builder::new_multi_thread();
            builder.enable_all();
            builder
        }
        1 => {
            let mut builder = tokio::runtime::Builder::new_current_thread();
            builder.enable_all();
            builder
        }
        n => {
            let mut builder = tokio::runtime::Builder::new_current_thread();
            builder.enable_all().worker_threads(n);
            builder
        }
    }
    .build()
    .expect("Failed building tokio runtime");

    info!(threads = config.num_threads, "Starting runtime");
    rt.block_on(inner_main(config))
}
