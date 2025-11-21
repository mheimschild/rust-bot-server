use std::{collections::HashMap, sync::Arc};
use redis::Client;

pub fn create_redis_client(redis_url: String) -> Arc<Client> {
    let client = Client::open(redis_url.as_str()).unwrap();
    let client = Arc::new(client);

    client
}

pub fn bulk_string_to_str(value: &redis::Value) -> Result<String, ()> {
    if let redis::Value::BulkString(bytes) = value {
        if let Ok(text) = std::str::from_utf8(bytes) {
            return Ok(String::from(text));
        }
    }

    Err(())
}

pub fn from_redis_value_to_hm(v: &redis::Value) -> Result<HashMap<String, String>, redis::ParsingError> {
    if let redis::Value::Array(items) = v {
        let mut map: HashMap<String, String> = HashMap::new();
        let mut iter = items.iter();
        while let Some(item) = iter.next() {
            if let Ok(key) = bulk_string_to_str(item) {
                if let Ok(value) = bulk_string_to_str(iter.next().unwrap()) {
                    map.insert(key, value);
                }
            }
            
        }

        return Ok(map);
    }

    Err(redis::ParsingError::from("value cannot be parsed"))
}