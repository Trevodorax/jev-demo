use jev_sdk::{Error, Question, SystemOneResponse, TypeSafeClient};
use serde::Serialize;

use crate::configuration;

pub struct JevClient {
    client: TypeSafeClient,
}

impl JevClient {
    pub fn new() -> Result<Self, Error> {
        let client = TypeSafeClient::new(&configuration::get().jev_api_key)?;
        Ok(Self { client })
    }

    pub async fn send<S, I, K, Q>(&self, state: S, questions: I) -> Result<SystemOneResponse, Error>
    where
        S: Serialize, // State
        I: IntoIterator<Item = (K, Q)>, // Questions array
        K: Into<String>, // Question id
        Q: Into<Question>, // Question
    {
        self.client.system_one(state, questions).await
    } 
}
