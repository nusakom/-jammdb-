use std::sync::Arc;
use tokio::sync::RwLock;

pub struct JammDB {
    data: Arc<RwLock<HashMap<String, String>>>,
}
impl JammDB {
    pub fn new() -> Self {
        JammDB {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn tx(&self, _read_only: bool) -> Result<Transaction, &'static str> {
        Ok(Transaction::new(Arc::clone(&self.data)))
    }
    pub async fn insert(&self, key: String, value: String) {
        let mut data = self.data.write().await;
        data.insert(key, value);
    }
    pub async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }
}
pub struct Transaction {
    data: Arc<RwLock<HashMap<String, String>>>,
}
impl Transaction {
    pub fn new(data: Arc<RwLock<HashMap<String, String>>>) -> Self {
        Transaction { data }
    }
    pub async fn put(&self, key: String, value: String) {
        let mut data = self.data.write().await;
        data.insert(key, value);
    }
    pub fn commit(self) {
    }
