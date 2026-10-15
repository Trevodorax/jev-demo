use std::time::Instant;

use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Router};
use maud::{Markup, html};
use serde::Deserialize;

use super::{UseCase, components, use_case_page};
use crate::api::{self, Model};

pub const USE_CASE: UseCase = UseCase {
    path: "/rules-check",
    title: "Message rules check",
    description: "Checking that a given message respects given rules. Each model returns the probability that the message does not respect the community rules.",
};

const RUN_PATH: &str = "/rules-check/run";
const DEFAULT_RULES: &str = "No swearing, nothing sexual, nothing that could hurt someone";

#[derive(Deserialize)]
struct Params {
    model: Model,
}

#[derive(Deserialize)]
struct Inputs {
    rules: String,
    message: String,
}

pub fn router() -> Router {
    Router::new()
        .route(USE_CASE.path, get(page))
        .route(RUN_PATH, post(run))
}

async fn page() -> Markup {
    use_case_page(&USE_CASE, RUN_PATH, html! {
        label { "Community rules" textarea name="rules" rows="3" { (DEFAULT_RULES) } }
        label { "Message" textarea name="message" rows="3" {} }
    })
}

async fn run(Query(params): Query<Params>, Form(inputs): Form<Inputs>) -> Markup {
    let start = Instant::now();
    let result = api::rules_check::check(params.model, &inputs.rules, &inputs.message).await;
    components::result(params.model, result, start.elapsed(), |probability| {
        format!("{:.1}% probability of breaking the rules", probability * 100.0)
    })
}
