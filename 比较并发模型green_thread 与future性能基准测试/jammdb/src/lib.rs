use std::collections::HashMap; // 引入 HashMap 类型，用于存储键值对
use std::sync::Mutex; // 引入 Mutex，用于在多线程中保护数据

pub struct JammDB {
    data: Mutex<HashMap<String, i32>>, // 使用 Mutex 保护 HashMap，以保证线程安全
}

impl JammDB {
    pub fn new() -> Self {
        JammDB {
            data: Mutex::new(HashMap::new()), // 初始化 JammDB，创建一个新的 HashMap
        }
    }

    pub fn insert(&self, key: String, value: i32) {
        let mut data = self.data.lock().unwrap(); // 获取锁以安全访问数据
        data.insert(key, value); // 将键值对插入 HashMap
    }

    pub fn get(&self, key: &str) -> Option<i32> {
        let data = self.data.lock().unwrap(); // 获取锁以安全访问数据
        data.get(key).cloned() // 返回指定键的值，若不存在则返回 None
    }
}
