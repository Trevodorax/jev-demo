use jev_sdk::{Choice, Noul, Question, Score};
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::comment_classification::{
    CATEGORIES, CATEGORY_ID, Classification, Comment, SATISFACTION_ID, SATISFACTION_LEVELS, State, URGENCY_ID,
};
use super::{TaskError, TaskResult};
use crate::ai_clients::gpt::GptClient;
use crate::ai_clients::jev::JevClient;

#[derive(Serialize)]
struct BulkState<'a> {
    comments: Vec<State<'a>>,
}

#[derive(Serialize)]
struct IndexedComment<'a> {
    index: usize,
    #[serde(flatten)]
    comment: State<'a>,
}

#[derive(Deserialize)]
struct IndexedClassification {
    index: usize,
    #[serde(flatten)]
    classification: Classification,
}

pub async fn classify_with_jev(comments: &[Comment]) -> Result<TaskResult<Vec<Classification>>, TaskError> {
    let state = BulkState {
        comments: comments.iter().map(State::from).collect(),
    };
    let questions = (0..comments.len()).flat_map(|index| {
        let comment = format!("the comment `comments[{index}]`");
        [
            (
                format!("{CATEGORY_ID}_{index}"),
                Question::from(Choice::options(format!("Which category does {comment} belong to ?"), CATEGORIES)),
            ),
            (
                format!("{SATISFACTION_ID}_{index}"),
                Question::from(Score::new(
                    format!("How satisfied is the author of {comment} with the product ?"),
                    SATISFACTION_LEVELS,
                )),
            ),
            (
                format!("{URGENCY_ID}_{index}"),
                Question::from(Noul::new(format!("Does {comment} need an urgent response ?"))),
            ),
        ]
    });
    let response = JevClient::new()?.send(state, questions).await?;
    let missing = |id: String| TaskError::InvalidOutput(format!("missing answer {id}"));
    let classifications = (0..comments.len())
        .map(|index| {
            let id = |question_id: &str| format!("{question_id}_{index}");
            let category = response.choice(&id(CATEGORY_ID)).ok_or_else(|| missing(id(CATEGORY_ID)))?;
            let satisfaction = response.score(&id(SATISFACTION_ID)).ok_or_else(|| missing(id(SATISFACTION_ID)))?;
            let urgency = response.noul(&id(URGENCY_ID)).ok_or_else(|| missing(id(URGENCY_ID)))?;
            Ok(Classification {
                category: category.choice.clone(),
                satisfaction: satisfaction.score,
                urgency: urgency.noul,
            })
        })
        .collect::<Result<Vec<_>, TaskError>>()?;
    Ok(TaskResult {
        value: classifications,
        input_tokens: response.usage.input_tokens,
        output_tokens: response.usage.output_tokens,
    })
}

pub async fn classify_with_gpt(comments: &[Comment]) -> Result<TaskResult<Vec<Classification>>, TaskError> {
    let levels = SATISFACTION_LEVELS
        .iter()
        .enumerate()
        .map(|(index, level)| format!("{index} = {level}"))
        .collect::<Vec<_>>()
        .join(", ");
    let instructions = format!(
        "You classify comments left on a product. The user sends a JSON array of comments, each with an \"index\", \
         an \"Author\", a \"Posted on\" date and a \"Product comment\". Author and date may be missing or messy.\n\n\
         For every comment, give:\n\
         - \"category\": exactly one of: {categories}\n\
         - \"satisfaction\": how satisfied the author is with the product, a number from 0 to {max} ({levels})\n\
         - \"urgency\": a probability from 0.0 to 1.0 that the comment needs an urgent response\n\n\
         Answer with a JSON array only, one object per comment, in this format: \
         [{{\"index\": 0, \"category\": \"...\", \"satisfaction\": 0, \"urgency\": 0.0}}]",
        categories = CATEGORIES.join(", "),
        max = SATISFACTION_LEVELS.len() - 1,
    );
    let indexed_comments = comments
        .iter()
        .enumerate()
        .map(|(index, comment)| IndexedComment {
            index,
            comment: State::from(comment),
        })
        .collect::<Vec<_>>();
    let input = serde_json::to_string(&indexed_comments).map_err(|e| TaskError::InvalidOutput(e.to_string()))?;
    let response = GptClient::new().send(&instructions, &input).await?;
    let text = response.output_text().unwrap_or_default();
    let mut answers = serde_json::from_str::<Vec<IndexedClassification>>(text.trim())
        .map_err(|_| TaskError::InvalidOutput(text.clone()))?;
    answers.sort_by_key(|answer| answer.index);
    let is_complete = answers.iter().map(|answer| answer.index).eq(0..comments.len());
    let has_valid_categories = answers
        .iter()
        .all(|answer| CATEGORIES.contains(&answer.classification.category.as_str()));
    if !is_complete || !has_valid_categories {
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
        value: answers.into_iter().map(|answer| answer.classification).collect(),
        input_tokens,
        output_tokens,
    })
}
