use futures::future::try_join_all;
use jev_sdk::{Choice, Noul, Question, Score};
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::{TaskError, TaskResult};
use crate::ai_clients::gpt::GptClient;
use crate::ai_clients::jev::JevClient;

pub(super) const CATEGORY_ID: &str = "category";
pub(super) const SATISFACTION_ID: &str = "satisfaction";
pub(super) const URGENCY_ID: &str = "urgency";

pub(super) const CATEGORIES: [&str; 5] = [
    "Delivery",
    "Product quality",
    "Pricing and billing",
    "Customer service",
    "Spam or irrelevant",
];

pub(super) const SATISFACTION_LEVELS: [&str; 5] = [
    "Very dissatisfied",
    "Dissatisfied",
    "Neutral",
    "Satisfied",
    "Very satisfied",
];

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub username: Option<String>,
    pub date: Option<String>,
    pub comment: String,
}

#[derive(Debug, Deserialize)]
pub struct Classification {
    pub category: String,
    pub satisfaction: f64,
    pub urgency: f64,
}

#[derive(Serialize)]
pub(super) struct State<'a> {
    #[serde(rename = "Author")]
    username: Option<&'a str>,
    #[serde(rename = "Posted on")]
    date: Option<&'a str>,
    #[serde(rename = "Product comment")]
    comment: &'a str,
}

impl<'a> From<&'a Comment> for State<'a> {
    fn from(comment: &'a Comment) -> Self {
        Self {
            username: comment.username.as_deref(),
            date: comment.date.as_deref(),
            comment: &comment.comment,
        }
    }
}

pub async fn classify_with_jev(
    comments: &[Comment],
) -> Result<TaskResult<Vec<Classification>>, TaskError> {
    let results = try_join_all(comments.iter().map(|comment| classify_one_with_jev(comment))).await?;
    Ok(merge(results))
}

async fn classify_one_with_jev(
    comment: &Comment,
) -> Result<TaskResult<Classification>, TaskError> {
    let state = State::from(comment);
    let category = Choice::options("Which category does this comment belong to ?", CATEGORIES);
    let satisfaction = Score::new("How satisfied is the author with the product ?", SATISFACTION_LEVELS);
    let urgency = Noul::new("Does this comment need an urgent response ?");
    let response = JevClient::new()?
        .send(
            state,
            [
                (CATEGORY_ID, Question::from(category)),
                (SATISFACTION_ID, Question::from(satisfaction)),
                (URGENCY_ID, Question::from(urgency)),
            ],
        )
        .await?;
    let missing = |id: &str| TaskError::InvalidOutput(format!("missing answer {id}"));
    let category = response.choice(CATEGORY_ID).ok_or_else(|| missing(CATEGORY_ID))?;
    let satisfaction = response.score(SATISFACTION_ID).ok_or_else(|| missing(SATISFACTION_ID))?;
    let urgency = response.noul(URGENCY_ID).ok_or_else(|| missing(URGENCY_ID))?;
    Ok(TaskResult {
        value: Classification {
            category: category.choice.clone(),
            satisfaction: satisfaction.score,
            urgency: urgency.noul,
        },
        input_tokens: response.usage.input_tokens,
        output_tokens: response.usage.output_tokens,
    })
}

// Not using OpenAI batch mode: it takes minutes to hours, and we're looking for fast responses here.
// + it would be too much work for one use case (file upload, handling async response, etc.)
pub async fn classify_with_gpt(
    comments: &[Comment],
) -> Result<TaskResult<Vec<Classification>>, TaskError> {
    let results = try_join_all(comments.iter().map(|comment| classify_one_with_gpt(comment))).await?;
    Ok(merge(results))
}

async fn classify_one_with_gpt(
    comment: &Comment,
) -> Result<TaskResult<Classification>, TaskError> {
    let levels = SATISFACTION_LEVELS
        .iter()
        .enumerate()
        .map(|(index, level)| format!("{index} = {level}"))
        .collect::<Vec<_>>()
        .join(", ");
    let input = format!(
        "Author: {username}\nPosted on: {date}\nProduct comment: {text}\n\nExpected answer: a JSON object with\n\
         - \"category\": the category of the comment, one of: {categories}\n\
         - \"satisfaction\": how satisfied the author is with the product, a number from 0 to {max} ({levels})\n\
         - \"urgency\": a probability from 0.0 to 1.0 that this comment needs an urgent response",
        username = comment.username.as_deref().unwrap_or("unknown"),
        date = comment.date.as_deref().unwrap_or("unknown"),
        text = comment.comment,
        categories = CATEGORIES.join(", "),
        max = SATISFACTION_LEVELS.len() - 1,
    );
    let response = GptClient::new()
        .send("Answer with the JSON object only.", &input)
        .await?;
    let text = response.output_text().unwrap_or_default();
    let classification = serde_json::from_str::<Classification>(text.trim())
        .map_err(|_| TaskError::InvalidOutput(text.clone()))?;
    if !CATEGORIES.contains(&classification.category.as_str()) {
        return Err(TaskError::InvalidOutput(text));
    }
    let (input_tokens, output_tokens) = response
        .usage
        .map(|usage| (usage.input_tokens as u64, usage.output_tokens as u64))
        .unwrap_or_else(|| {
            warn!("GPT response has no token usage");
            (0, 0)
        });
    Ok(TaskResult {
        value: classification,
        input_tokens,
        output_tokens,
    })
}

fn merge(results: Vec<TaskResult<Classification>>) -> TaskResult<Vec<Classification>> {
    TaskResult {
        input_tokens: results.iter().map(|result| result.input_tokens).sum(),
        output_tokens: results.iter().map(|result| result.output_tokens).sum(),
        value: results.into_iter().map(|result| result.value).collect(),
    }
}
