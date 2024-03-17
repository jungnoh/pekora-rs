use log::debug;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ClientSet<K: Clone, T> {
    lock: tokio::sync::Mutex<HashMap<String, Arc<T>>>,
    client_factory: Box<dyn Fn(K, String) -> T + Send + Sync>,
    initial_data: K,
}

impl<K: Clone, T> ClientSet<K, T> {
    pub fn new(initial_data: K, client_factory: Box<dyn Fn(K, String) -> T + Send + Sync>) -> Self {
        Self {
            lock: tokio::sync::Mutex::new(HashMap::new()),
            client_factory,
            initial_data,
        }
    }

    pub async fn get(&self, key: &str) -> Arc<T> {
        let mut lock = self.lock.lock().await;
        if let Some(client) = lock.get(&key.to_string()) {
            return client.clone();
        }
        debug!("ClientSet: Creating new client for {}", key);
        let client = Arc::new((self.client_factory)(
            self.initial_data.clone(),
            key.to_string(),
        ));
        lock.insert(key.to_string(), client.clone());
        client
    }
}
