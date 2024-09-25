use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct JammDB {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl JammDB {
    pub fn new() -> Self {
        JammDB {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // 批量插入，支持错误处理
    pub async fn insert_batch(&self, pairs: &[(String, String)]) -> Result<(), String> {
        let mut data = self.data.write().await;
        for (key, value) in pairs {
            data.insert(key.clone(), value.clone());
        }
        Ok(())
    }

    // 获取值
    pub async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }
}