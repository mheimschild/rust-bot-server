use std::{sync::{Arc, Mutex}, time::Duration};
use anyhow::Result;
use ollama_rs::generation::chat::ChatMessage;
use tokio::{net::TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::{Bytes, Message}};
use futures_util::{SinkExt, StreamExt};
use log::{info};

use crate::utils::{envs::Config, llm::{LLMAndEmbedder, OllamaLLMAndEmbedder, build_prompt}, vector_store::{RedisVectorStore, VectorStore, get_context_from_result}};

pub async fn handle_connection(
    stream: TcpStream, 
    ollama: Arc<OllamaLLMAndEmbedder>,
    redis_vector_store: Arc<RedisVectorStore>,
    config: Arc<Config>,
) -> Result<()> {
    let mut ws_stream = match accept_async(stream).await {
            Ok(stream) => stream,
            Err(error) => panic!("error handling web socket {error:?}"),
        };

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        let chat_history = Arc::new(Mutex::new(vec![ChatMessage::system(config.system_prompt.clone())]));

        loop {
            tokio::select! {
                msg = ws_stream.next() => {
                    match msg {
                        Some(msg) => {
                            let msg = msg?;
                            if msg.is_text() {
                                let received_text = msg.to_text()?;
                                
                                let parsed = json::parse(r#received_text).unwrap();
                                let payload = parsed["payload"].clone();
                                let payload_str = payload.as_str().unwrap();
                                let reply_obj = json::object!{
                                    type: "user_message",
                                    payload: payload_str,
                                };

                                ws_stream.send(Message::text(json::stringify(reply_obj))).await?;

                                info!("question: {}", payload_str);

                                let reformulated_question = if !config.use_chat_history {
                                    ollama.reformulate_question(chat_history.clone(), payload_str.to_string()).await.unwrap()
                                } else {
                                    payload_str.to_string()
                                };

                                info!("reformulated question: {}", reformulated_question);

                                let context = if config.use_index {
                                    let embeddings = ollama.embedding(reformulated_question.clone()).await;
                                    let result = redis_vector_store.query_vector(&embeddings, config.max_results).await.unwrap();
                                    let responses = result.responses;
                                    get_context_from_result(responses)
                                } else {
                                    String::from("")
                                };

                                info!("context {}", context);
                                
                                let mut stream = if config.use_chat_history {
                                    ollama.send_chat_messages_with_history_stream(
                                        chat_history.clone(),
                                        build_prompt(context, reformulated_question),
                                    ).await.unwrap() 
                                } else {
                                    ollama.send_chat_messages_stream(
                                        chat_history.clone(),
                                        build_prompt(context, reformulated_question),
                                    ).await.unwrap() 
                                };

                                let mut response_text = String::new();
                                while let Some(res) = stream.next().await {
                                    let res = res.unwrap();
                                    if !res.done {
                                        let content = res.message.content;
                                        let response_obj = json::object!{
                                            type: "bot_stream_chunk",
                                            payload: content.clone(),
                                        };
                                        response_text.push_str(content.clone().as_str());

                                        ws_stream.send(Message::text(json::stringify(response_obj))).await?;
                                    } else {
                                        let content = res.message.content;
                                        let response_obj = json::object!{
                                            type: "bot_stream_end",
                                            payload: content.clone(),
                                        };
                                        response_text.push_str(content.clone().as_str());

                                        ws_stream.send(Message::text(json::stringify(response_obj))).await?;
                                    }
                                }
                                chat_history.lock().unwrap().push(ChatMessage::assistant(response_text.clone()));
                            } else if msg.is_close() {
                                info!("client disconnected");
                                break
                            }
                        },
                        None => break,
                    }

                },
                _ = interval.tick() => {
                    ws_stream.send(Message::Ping(Bytes::from("ping"))).await?;
                }
            }
        }

    Ok(())
}