use std::{collections::HashMap, sync::Arc};
use redis::{Client, FromRedisValue};
use anyhow::Result;
use crate::utils::{embeddings::serialize_vector, envs::Config, redis::{bulk_string_to_str, create_redis_client, from_redis_value_to_hm}};

pub(crate) struct VectorSearchResponse {
    _id: String,
    pub(crate) metadata: HashMap<String, String>,
}

pub struct VectorSearchResponses {
    pub(crate) responses: Vec<VectorSearchResponse>,
}

impl FromRedisValue for VectorSearchResponses {
    fn from_redis_value(v: redis::Value) -> std::result::Result<Self, redis::ParsingError> {
        match v {
            redis::Value::Array(items) => {
                let mut iter = items.iter();
                if let redis::Value::Int(_magic) = iter.next().unwrap() {
                    
                }

                let mut responses = vec![];

                while let Some(item) = iter.next() {
                    if let Ok(id) = bulk_string_to_str(item) {
                        if let Ok(meta) = from_redis_value_to_hm(iter.next().unwrap()) {
                            let vsr = VectorSearchResponse {
                                _id: id,
                                metadata: meta,
                            };

                            responses.push(vsr);
                        }
                    }
                }

                return Ok(VectorSearchResponses {
                    responses: responses,
                })
            },
            _ => {}
        }
        todo!()
    }
}

pub struct RedisVectorStore {
    client: Arc<Client>,
    index_name: String,
}

pub trait VectorStore {
    async fn query_vector(&self, query_vector: &Vec<f32>, count: usize) -> Result<VectorSearchResponses>;
}

impl VectorStore for RedisVectorStore {
    async fn query_vector(&self, query_vector: &Vec<f32>, count: usize) -> Result<VectorSearchResponses> {
        let knn_query = format!("@_index_name:(\"{}\")=>[KNN {} @embedding $vector AS vector_distance]", 
        self.index_name, 
        count,
    );

    let mut con = self.client.get_connection()?;
    let vector_bytes = serialize_vector(&query_vector);

    let vsrs: VectorSearchResponses = redis::cmd("FT.SEARCH")
        .arg(self.index_name.clone())
        .arg(knn_query)
        .arg("RETURN")
        .arg("4").arg("text").arg("_index_name").arg("_metadata_json").arg("vector_distance")
        .arg("SORTBY").arg("vector_distance")
        .arg("ASC")
        .arg("DIALECT").arg("2")
        .arg("LIMIT").arg("0").arg("4")
        .arg("params").arg("2").arg("vector").arg(vector_bytes)
        .query(&mut con).unwrap();

    Ok(vsrs)
    }
}

pub fn get_context_from_result(results: Vec<VectorSearchResponse>) -> String {
    let mut iter = results.iter();
    let mut context = String::from("");
    while let Some(result) = iter.next() {
        context.push_str(result.metadata["text"].as_str());
        context.push_str("\n");
    }

    println!("context: {}", context);

    context
}

pub fn create_redis_vector_store(config: &Config) -> Arc<RedisVectorStore> {
    Arc::new(RedisVectorStore {
        client: create_redis_client(config.redis_url.clone()),
        index_name: config.index_name.clone(),
    })
}