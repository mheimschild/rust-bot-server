# Chatbot backend written in Rust

## Requirements

* Rust compiler
* Ollama with
    - chat model
    - embeddings model
* Redis 8+

## Setup and running

* MODEL_NAME - name of chat model used for answering queries
* EMBEDDINGS_MODEL_NAME - name of the model producing embeddings
* REDIS_URL - connection URL for Redis
* PORT - listening port
* USE_INDEX - when set to "true", bot search for an answer in Redis Vector store
* INDEX_NAME - name of the Redis index
* MAX_RESULTS - number of results returned by index
* USE_CHAT_HISTORY - when set to true, whole chat history is passed to Ollama
* SYSTEM_PROMPT

Run `cargo run` for local test or exec `cargo build --release` to produce prod-ready binary (3.6MB only!)