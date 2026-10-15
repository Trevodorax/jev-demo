pub mod bulk_comment_classification;
pub mod comment_classification;
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
        .route("/api/comment-classification", post(comment_classification::handle))
        .route("/api/bulk-comment-classification", post(bulk_comment_classification::handle))
}
