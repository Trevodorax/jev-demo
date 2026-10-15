mod ai_clients;
mod ai_tasks;
mod api;
mod configuration;
mod ui;

use tracing::info;

const ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    configuration::load();
    let listener = tokio::net::TcpListener::bind(ADDRESS).await.unwrap();
    info!("Listening on {ADDRESS}");
    axum::serve(listener, api::router().merge(ui::router())).await.unwrap();
}
