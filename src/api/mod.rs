pub mod rules_check;

use axum::Router;
use axum::routing::post;
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Jev,
    Gpt,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/rules-check", post(rules_check::handle))
}
