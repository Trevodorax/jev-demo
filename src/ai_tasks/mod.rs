pub mod bulk_comment_classification;
pub mod comment_classification;
pub mod rules_check;

use std::fmt;

#[derive(Debug)]
pub struct TaskResult<T> {
    pub value: T,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug)]
pub enum TaskError {
    Jev(jev_sdk::Error),
    Gpt(async_openai::error::OpenAIError),
    InvalidOutput(String),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Jev(error) => write!(f, "Jev error: {error}"),
            Self::Gpt(error) => write!(f, "GPT error: {error}"),
            Self::InvalidOutput(output) => write!(f, "Invalid output: {output}"),
        }
    }
}

impl std::error::Error for TaskError {}

impl From<jev_sdk::Error> for TaskError {
    fn from(error: jev_sdk::Error) -> Self {
        Self::Jev(error)
    }
}

impl From<async_openai::error::OpenAIError> for TaskError {
    fn from(error: async_openai::error::OpenAIError) -> Self {
        Self::Gpt(error)
    }
}
