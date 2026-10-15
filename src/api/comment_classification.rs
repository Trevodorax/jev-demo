use axum::Json;
use axum::extract::Query;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;

use super::Model;
use crate::ai_tasks::comment_classification::{self, Classification, Comment};
use crate::ai_tasks::{TaskError, TaskResult};

#[derive(Deserialize)]
pub struct Params {
    pub(super) model: Model,
}

// Expected format of the request body, which is also the format of the JSON file uploaded in the UI
// (see resources/example-comments.json). Also used by the bulk comment classification route. "username" and "date" are
// free text and can be null or missing, the date format doesn't matter, and unknown fields are ignored:
// {
//     "comments": [
//         { "username": "marco_b", "date": "2026-10-02T14:31:00Z", "comment": "The package arrived two weeks late." },
//         { "username": null, "date": "03/10/2026", "comment": "Great value for money, would buy again.", "likes": 4 }
//     ]
// }
#[derive(Deserialize)]
pub struct Request {
    pub comments: Vec<Comment>,
}

#[derive(Serialize)]
pub struct Response {
    classifications: Vec<ClassificationResponse>,
    input_tokens: u64,
    output_tokens: u64,
}

#[derive(Serialize)]
struct ClassificationResponse {
    category: String,
    satisfaction: f64,
    urgency: f64,
}

pub async fn classify(model: Model, request: &Request) -> Result<TaskResult<Vec<Classification>>, TaskError> {
    match model {
        Model::Jev => comment_classification::classify_with_jev(&request.comments).await,
        Model::Gpt => comment_classification::classify_with_gpt(&request.comments).await,
    }
    .inspect_err(|e| error!("Comment classification failed: {e}"))
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

impl From<TaskResult<Vec<Classification>>> for Response {
    fn from(result: TaskResult<Vec<Classification>>) -> Self {
        Self {
            classifications: result
                .value
                .into_iter()
                .map(|classification| ClassificationResponse {
                    category: classification.category,
                    satisfaction: classification.satisfaction,
                    urgency: classification.urgency,
                })
                .collect(),
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
        }
    }
}
