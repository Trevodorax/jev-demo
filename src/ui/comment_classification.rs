use std::time::{Duration, Instant};

use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Form, Router};
use maud::{Markup, html};
use serde::Deserialize;
use tracing::error;

use super::{UseCase, components, use_case_page};
use crate::ai_tasks::comment_classification::{Classification, Comment};
use crate::ai_tasks::{TaskError, TaskResult};
use crate::api::comment_classification::Request;
use crate::api::{self, Model};

pub const USE_CASE: UseCase = UseCase {
    path: "/comment-classification",
    title: "Product comment classification",
    description: "Classifying comments on a product by category (delivery, product quality, pricing and billing, customer service, spam or irrelevant), satisfaction level (0 = very dissatisfied, 4 = very satisfied) and urgency. Each comment is sent in its own request, all at the same time. Upload a JSON file with the comments (see resources/example-comments.json). Hover a row number to see its comment.",
};

const RUN_PATH: &str = "/comment-classification/run";
const LOAD_FILE: &str = "this.files[0]?.text().then(text => this.form.request.value = text)";

#[derive(Deserialize)]
pub(super) struct Params {
    pub(super) model: Model,
}

#[derive(Deserialize)]
pub(super) struct Inputs {
    request: String,
}

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
    let result = api::comment_classification::classify(params.model, &request).await;
    classifications(params.model, result, start.elapsed(), request.comments)
}

pub(super) fn inputs() -> Markup {
    html! {
        label { "JSON file" input type="file" accept=".json,application/json" onchange=(LOAD_FILE); }
        label { "File content" textarea name="request" rows="6" readonly {} }
    }
}

pub(super) fn parse(model: Model, inputs: Inputs, start: Instant) -> Result<Request, Markup> {
    serde_json::from_str::<Request>(&inputs.request).map_err(|e| {
        error!("Invalid comment classification file: {e}");
        let result = Err(TaskError::InvalidOutput(inputs.request));
        components::result(model, result, start.elapsed(), |_: Vec<Classification>| "")
    })
}

pub(super) fn classifications(
    model: Model,
    result: Result<TaskResult<Vec<Classification>>, TaskError>,
    elapsed: Duration,
    comments: Vec<Comment>,
) -> Markup {
    components::result(model, result, elapsed, |classifications| html! {
        table.classifications {
            thead { tr { th { "#" } th { "Category" } th { "Satisfaction" } th { "Urgency" } } }
            tbody {
                @for (index, classification) in classifications.iter().enumerate() {
                    tr {
                        td title=[comments.get(index).map(|comment| &comment.comment)] { (index + 1) }
                        td { (classification.category) }
                        td { (format!("{:.1}/4", classification.satisfaction)) }
                        td { (format!("{:.0}%", classification.urgency * 100.0)) }
                    }
                }
            }
        }
    })
}
