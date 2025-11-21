use std::{pin::Pin, sync::{Arc, Mutex}};

use futures_util::Stream;
use ollama_rs::{Ollama, error::{OllamaError}, generation::{chat::{ChatMessage, ChatMessageResponse, MessageRole, request::ChatMessageRequest}, completion::request::GenerationRequest, embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest}}, history::ChatHistory};

use crate::utils::envs::Config;

pub fn build_prompt(context: String, query: String) -> String {
    return format!("context: {}
------
question: {}
------
answer:", context, query);
}

pub(crate) struct OllamaLLMAndEmbedder {
    ollama: Ollama,
    model: String,
    embeddings_model: String,
    system_prompt: String,
}

pub(crate) trait LLMAndEmbedder {
    async fn embedding(&self, query: String) -> Vec<f32>;
    async fn send_chat_messages_with_history_stream<C: ChatHistory + Send + 'static>(&self, chat_history: Arc<Mutex<C>>, message: String) -> Result<Pin<Box<dyn Stream<Item = Result<ChatMessageResponse, ()>> + Send + 'static>>, OllamaError>;
    async fn send_chat_messages_stream<C: ChatHistory + Send + 'static>(&self, chat_history: Arc<Mutex<C>>, message: String) -> Result<Pin<Box<dyn Stream<Item = Result<ChatMessageResponse, ()>> + Send + 'static>>, OllamaError>;
    async fn reformulate_question<C: ChatHistory + Send + 'static>(&self, chat_history: Arc<Mutex<C>>, message: String) -> ollama_rs::error::Result<String>;
}

impl LLMAndEmbedder for OllamaLLMAndEmbedder {
    async fn embedding(&self, query: String) -> Vec<f32> {
        let request = GenerateEmbeddingsRequest::new(
            self.embeddings_model.clone(), 
            EmbeddingsInput::Single(query.clone()),
        );
        let res = self.ollama.generate_embeddings(request).await.unwrap();
        res.embeddings[0].clone()
    }

    async fn send_chat_messages_with_history_stream<C: ChatHistory + Send + 'static>(&self, chat_history: Arc<Mutex<C>>, message: String) -> Result<Pin<Box<dyn Stream<Item = Result<ChatMessageResponse, ()>> + Send + 'static>>, OllamaError> {
        let stream = self.ollama.send_chat_messages_with_history_stream(
            chat_history,
            ChatMessageRequest::new(
                self.model.clone(),
                vec![ChatMessage::user(
                    message.clone(),
                )]
            )
        );

        stream.await
    }

    async fn send_chat_messages_stream<C: ChatHistory + Send + 'static>(&self, chat_history: Arc<Mutex<C>>, message: String) -> Result<Pin<Box<dyn Stream<Item = Result<ChatMessageResponse, ()>> + Send + 'static>>, OllamaError> {
        {
            let mut hist = chat_history.lock().unwrap();
            hist.push(ChatMessage::user(message.clone()));
        }

        let stream = self.ollama.send_chat_messages_stream(ChatMessageRequest::new(
            self.model.clone(),
            vec![
                ChatMessage::system(self.system_prompt.clone()),
                ChatMessage::user(
                    message.clone(),
                )
            ]
        ));

        stream.await
    }
    
    async fn reformulate_question<C: ChatHistory + Send + 'static>(&self, chat_history: Arc<Mutex<C>>, message: String) -> ollama_rs::error::Result<String> {
        let chat_history_str = get_history_as_str(chat_history);
        if chat_history_str == "" {
            return Ok(message);
        }
        let prompt = format!("chat_history:
{}
----
user's question: {}
----
rephrase the user's new_question so that the context of the question is clear and fully informed by the provided chat_history. Respond in the same language as the original new_question. respond only with rephrased question without any explanation.

rephrased question:", chat_history_str, message);
  
        let res = self.ollama.generate(GenerationRequest::new(self.model.to_string(), prompt)).await.unwrap();
        println!("q: {}", res.response);
        Ok(String::from(res.response))
    }
}

fn get_history_as_str<C: ChatHistory + Send + 'static>(chat_history: Arc<Mutex<C>>) -> String {
    let mut history_as_str = String::from("");
        let mutex = chat_history.lock().unwrap();
        let messages = mutex.messages();
        
        let mut iter = messages.iter();

        while let Some(history_item) = iter.next() {
            match history_item.role {
                MessageRole::Assistant => {
                    history_as_str = format!("{}\n{}", history_as_str, history_item.content);
                },
                MessageRole::User => {
                    history_as_str = format!("{}\n{}", history_as_str, history_item.content);
                }
                _ => {}
            }
        }
        
        history_as_str
}

pub fn create_ollama(config: &Config) -> OllamaLLMAndEmbedder {
    OllamaLLMAndEmbedder {
        ollama: Ollama::default(),
        model: config.model.clone(),
        embeddings_model: config.embeddings_model.clone(),
        system_prompt: config.system_prompt.clone(),
    }
}
