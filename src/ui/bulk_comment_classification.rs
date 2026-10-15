use std::time::Instant;

use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Router};
use maud::Markup;

use super::comment_classification::{Inputs, Params, classifications, inputs, parse};
use super::{UseCase, use_case_page};
use crate::api;

pub const USE_CASE: UseCase = UseCase {
    path: "/bulk-comment-classification",
    title: "Product comment classification (single request)",
    description: "Same classification as \"Product comment classification\", but each model classifies all the comments in a single request: Jev gets every comment in its state with three questions per comment, GPT 5.5 gets every comment with one instruction and answers with a JSON array. Upload a JSON file with the comments (see resources/example-comments.json). Hover a row number to see its comment.",
};

const RUN_PATH: &str = "/bulk-comment-classification/run";

pub fn router() -> Router {
    Router::new()
        .route(USE_CASE.path, get(page))
        .route(RUN_PATH, post(run))
}

async fn page() -> Markup {
    use_case_page(&USE_CASE, RUN_PATH, inputs())
}

async fn run(Query(params): Query<Params>, Form(inputs): Form<Inputs>) -> Markup {
    let start = Instant::now();
    let request = match parse(params.model, inputs, start) {
        Ok(request) => request,
        Err(markup) => return markup,
    };
    let result = api::bulk_comment_classification::classify(params.model, &request).await;
    classifications(params.model, result, start.elapsed(), request.comments)
}
