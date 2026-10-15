use axum::Json;
use axum::extract::Query;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;

use super::Model;
use crate::ai_tasks::rules_check;
use crate::ai_tasks::{TaskError, TaskResult};

#[derive(Deserialize)]
pub struct Params {
    model: Model,
}

#[derive(Deserialize)]
pub struct Request {
    rules: String,
    message: String,
}

#[derive(Serialize)]
pub struct Response {
    probability: f64,
    input_tokens: u64,
    output_tokens: u64,
}

pub async fn check(model: Model, rules: &str, message: &str) -> Result<TaskResult<f64>, TaskError> {
    match model {
        Model::Jev => rules_check::check_with_jev(rules, message).await,
        Model::Gpt => rules_check::check_with_gpt(rules, message).await,
    }
    .inspect_err(|e| error!("Rules check failed: {e}"))
}

pub async fn handle(
    Query(params): Query<Params>,
    Json(request): Json<Request>,
) -> Result<Json<Response>, StatusCode> {
    let result = check(params.model, &request.rules, &request.message)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Response {
        probability: result.value,
        input_tokens: result.input_tokens,
        output_tokens: result.output_tokens,
    }))
}
