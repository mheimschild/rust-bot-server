mod utils;

use std::sync::{Arc};

use anyhow::Result;
use log::info;
use tokio::net::{TcpListener};

use utils::{
    envs::get_config_from_envs,
};

use crate::utils::llm::create_ollama;
use crate::utils::server::handle_connection;
use crate::utils::vector_store::create_redis_vector_store;

#[tokio::main]
async fn main() -> Result<()>{
    env_logger::init();

    let config = get_config_from_envs();
    let addr = ("127.0.0.1:".to_owned()+&config.port).clone();
    let listener = TcpListener::bind(&addr).await?;

    info!("server listening on {}", addr);

    let redis_vector_store = create_redis_vector_store(&config);
    let ollama = Arc::new(create_ollama(&config));
    let config = Arc::new(config);

    while let Ok((stream, _)) = listener.accept().await {
        let redis_vector_store = Arc::clone(&redis_vector_store);
        let ollama = Arc::clone(&ollama);
        let config = Arc::clone(&config);
        tokio::spawn(handle_connection(
            stream, 
            ollama, 
            redis_vector_store,
            config,
        ));
    }

    Ok(())
}