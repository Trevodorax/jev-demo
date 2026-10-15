use jev_sdk::Noul;
use serde::Serialize;
use tracing::warn;

use super::{TaskError, TaskResult};
use crate::ai_clients::gpt::GptClient;
use crate::ai_clients::jev::JevClient;

const QUESTION_ID: &str = "is_unacceptable";

#[derive(Serialize)]
struct State<'a> {
    #[serde(rename = "Community rules")]
    rules: &'a str,
    #[serde(rename = "Message")]
    message: &'a str,
}

pub async fn check_with_jev(rules: &str, message: &str) -> Result<TaskResult<f64>, TaskError> {
    let state = State { rules, message };
    let question = Noul::new("Is this message against the community rules ?");
    let response = JevClient::new()?.send(state, [(QUESTION_ID, question)]).await?;
    let answer = response
        .noul(QUESTION_ID)
        .ok_or_else(|| TaskError::InvalidOutput(format!("missing answer {QUESTION_ID}")))?;
    Ok(TaskResult {
        value: answer.noul,
        input_tokens: response.usage.input_tokens,
        output_tokens: response.usage.output_tokens,
    })
}

pub async fn check_with_gpt(rules: &str, message: &str) -> Result<TaskResult<f64>, TaskError> {
    let input = format!(
        "Community rules: {rules}\n\nMessage: {message}\n\nExpected answer: a probability from 0.0 to 1.0 that this message does not respect the community rules"
    );
    let response = GptClient::new()
        .send("Answer with the number only.", &input)
        .await?;
    let text = response.output_text().unwrap_or_default();
    let probability = text
        .trim()
        .parse::<f64>()
        .map_err(|_| TaskError::InvalidOutput(text.clone()))?;
    let (input_tokens, output_tokens) = response
        .usage
        .map(|usage| (usage.input_tokens as u64, usage.output_tokens as u64))
        .unwrap_or_else(|| {
            warn!("GPT response has no token usage");
            (0, 0)
        });
    Ok(TaskResult {
        value: probability,
        input_tokens,
        output_tokens,
    })
}
