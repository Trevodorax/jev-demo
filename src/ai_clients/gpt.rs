use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::responses::{CreateResponseArgs, Response};

use crate::configuration;

const MODEL: &str = "gpt-5.5";

pub struct GptClient {
    client: Client<OpenAIConfig>,
}

impl GptClient {
    pub fn new() -> Self {
        let config = OpenAIConfig::new().with_api_key(&configuration::get().openai_api_key);
        Self {
            client: Client::with_config(config),
        }
    }

    pub async fn send(&self, instructions: &str, input: &str) -> Result<Response, OpenAIError> {
        let request = CreateResponseArgs::default()
            .model(MODEL)
            .instructions(instructions)
            .input(input)
            .build()?;
        self.client.responses().create(request).await
    }
}
