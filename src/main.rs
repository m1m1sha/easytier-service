use clap::{arg, Parser};
use salvo::prelude::*;

mod constant;
mod easytier;
mod handler;
mod http;
mod model;
mod router;
mod utils;

#[derive(Parser, Debug)]
#[command(name = "EasyTier-service", author, version = constant::VERSION, about, long_about = None)]
struct Cli {
    // http port
    #[arg(short, long, default_value = "4472")]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let cli = Cli::parse();

    let acceptor = TcpListener::new(format!("127.0.0.1:{}", cli.port))
        .bind()
        .await;
    Server::new(acceptor).serve(router::router()).await;
}
