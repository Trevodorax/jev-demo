use std::env;
use std::sync::OnceLock;

static CONFIGURATION: OnceLock<Configuration> = OnceLock::new();

#[derive(Debug)]
pub struct Configuration {
    pub jev_api_key: String,
    pub openai_api_key: String,
}

pub fn load() {
    dotenvy::dotenv().ok();
    let configuration = Configuration {
        jev_api_key: required("JEV_API_KEY"),
        openai_api_key: required("OPENAI_API_KEY"),
    };
    CONFIGURATION
        .set(configuration)
        .expect("configuration already loaded");
}

pub fn get() -> &'static Configuration {
    CONFIGURATION.get().expect("configuration not loaded")
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("missing environment variable {key}"))
}
