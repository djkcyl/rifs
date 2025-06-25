mod app_state;
mod config;
mod database;
mod entities;
mod handlers;
mod logging;
mod middleware;
mod migrations;
mod models;
mod repositories;
mod routes;
mod server;
mod services;
mod utils;

use server::run_server;

#[tokio::main]
async fn main() {
    if let Err(e) = run_server().await {
        eprintln!("服务器启动失败: {}", e);
        std::process::exit(1);
    }
}
