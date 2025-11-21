use std::{env};

const MODEL_NAME: &str = "llama3.2";
const EMBEDDINGS_MODEL_NAME: &str = "embeddinggemma";
const INDEX_NAME: &str = "idx:cv";
const REDIS_URL: &str = "redis://127.0.0.1/";
const SYSTEM_PROMPT: &str = "";

pub struct Config {
    pub(crate) model: String,
    pub(crate) embeddings_model: String,
    pub(crate) index_name: String,
    pub(crate) use_chat_history: bool,
    pub(crate) use_index: bool,
    pub(crate) max_results: usize,
    pub(crate) redis_url: String,
    pub(crate) port: String,
    pub(crate) system_prompt: String,
}

pub fn get_config_from_envs() -> Config {
    let model = env::var("MODEL_NAME").unwrap_or(MODEL_NAME.to_string());
    let embeddings_model = env::var("EMBEDDINGS_MODEL_NAME").unwrap_or(EMBEDDINGS_MODEL_NAME.to_string());
    let index_name = env::var("INDEX_NAME").unwrap_or(INDEX_NAME.to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or(REDIS_URL.to_string());
    let port = env::var("PORT").unwrap_or(String::from("3005"));
    let use_index = env::var("USE_INDEX").unwrap_or(String::from("false")) == "true";
    let use_chat_history = env::var("USE_CHAT_HISTORY").unwrap_or(String::from("false")) == "true";
    let max_results = env::var("MAX_RESULTS").unwrap_or(String::from("3"));
    let max_results = max_results.parse().unwrap_or(3);
    let system_prompt = env::var("SYSTEM_PROMPT").unwrap_or(String::from(SYSTEM_PROMPT.to_string()));

    Config { 
        model, 
        embeddings_model, 
        index_name, 
        use_chat_history,
        use_index, 
        max_results,
        redis_url, 
        port, 
        system_prompt,
    }
}