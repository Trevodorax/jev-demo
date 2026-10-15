use axum::Json;
use axum::extract::Query;
use axum::http::StatusCode;
use tracing::error;

use super::Model;
use super::comment_classification::{Params, Request, Response};
use crate::ai_tasks::bulk_comment_classification;
use crate::ai_tasks::comment_classification::Classification;
use crate::ai_tasks::{TaskError, TaskResult};

// Same request and response format as the comment classification route (see api/comment_classification.rs),
// but each model classifies all the comments in a single request.
pub async fn classify(model: Model, request: &Request) -> Result<TaskResult<Vec<Classification>>, TaskError> {
    match model {
        Model::Jev => bulk_comment_classification::classify_with_jev(&request.comments).await,
        Model::Gpt => bulk_comment_classification::classify_with_gpt(&request.comments).await,
    }
    .inspect_err(|e| error!("Bulk comment classification failed: {e}"))
}

pub async fn handle(
    Query(params): Query<Params>,
    Json(request): Json<Request>,
) -> Result<Json<Response>, StatusCode> {
    let result = classify(params.model, &request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Response::from(result)))
}
