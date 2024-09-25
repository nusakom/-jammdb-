use tokio;
use indicatif::ProgressBar;
use std::time::Instant;
use std::sync::Arc;

mod lib; // 引入 lib.rs 中的 JammDB

#[tokio::main]
async fn main() {
    let db = Arc::new(lib::JammDB::new());
    let iterations = 10000000; // 插入的总数
    let batch_size = 10000; // 批量插入大小

    let bar = ProgressBar::new(iterations);
    let start_insertion = Instant::now();
    
    let mut handles = Vec::new(); // 存储任务句柄

    // 插入数据
    for batch_start in (0..iterations).step_by(batch_size as usize) {
        let db_clone = Arc::clone(&db);
        let bar_clone = bar.clone();
        let batch: Vec<_> = (batch_start..(batch_start + batch_size).min(iterations))
            .map(|idx| (format!("key{}", idx), format!("value{}", idx)))
            .collect(); // 生成批次

        // 使用 tokio::spawn 创建异步任务
        let handle = tokio::spawn(async move {
            let tx = db_clone.tx(true).await.unwrap(); // 获取事务
            for (key, value) in &batch {
                tx.put(key.clone(), value.clone()).await; // 异步插入数据
            }
            bar_clone.inc(batch.len() as u64); // 更新进度条
        });

        handles.push(handle); // 收集任务句柄
    }

    // 等待所有任务完成
    for handle in handles {
        let _ = handle.await; // 确保每个任务都完成
    }

    bar.finish_with_message("Insertion completed");

    let duration_insertion = start_insertion.elapsed();
    println!("Total inserts: {}", iterations);
    println!("Elapsed time: {:?}", duration_insertion);
    println!("Throughput: {:.2} inserts per second", 
        iterations as f64 / duration_insertion.as_secs_f64());

    // 获取数据并打印最后一条
    if let Some(value) = db.get(&format!("key{}", iterations - 1)).await {
        println!("Retrieved key{}: {}", iterations - 1, value);
    }