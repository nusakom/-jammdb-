mod lib; // 引入 lib.rs

use crate::lib::JammDB; // 引用 JammDB 结构体
use indicatif::ProgressBar; // 进度条库
use std::sync::Arc;
use std::time::Instant;
use tokio::main;
use tokio::sync::Semaphore;
use futures::future::join_all;

#[main]
async fn main() {
    let db = JammDB::new();
    let iterations: u64 = 100_000_00; // 设置为一亿
    let batch_size: u64 = 10_000; // 设置为批量大小

    let keys: Vec<_> = (0..iterations)
        .map(|i| (i.to_string(), format!("value{}", i)))
        .collect();

    let pb = ProgressBar::new(iterations);
    let start_time = Instant::now();
    
    let keys = Arc::new(keys);
    let semaphore = Arc::new(Semaphore::new(50)); // 调整并发数量

    let futures: Vec<_> = (0..(iterations / batch_size)).map(|i| {
        let semaphore_clone = Arc::clone(&semaphore);
        let db_clone = db.clone();
        let pb_clone = pb.clone();
        let keys_clone = Arc::clone(&keys);

        async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); // 获取许可
            let start = (i * batch_size) as usize;
            let end = (start + batch_size as usize).min(keys_clone.len());
            let batch = &keys_clone[start..end];

            db_clone.insert_batch(batch).await.unwrap(); // 插入数据
            pb_clone.inc(batch.len() as u64); // 更新进度条
        }
    }).collect();

    join_all(futures).await; // 等待所有插入完成

    pb.finish_with_message("Done!"); // 完成进度条

    let elapsed_time = start_time.elapsed();
    let throughput = iterations as f64 / elapsed_time.as_secs_f64();

    println!("Total inserts: {}", iterations);
    println!("Elapsed time: {:?}", elapsed_time);
    println!("Throughput: {:.2} inserts per second", throughput);

    if let Some(value) = db.get("99999").await {
        println!("Retrieved key99999: {}", value);
    } else {
        println!("Key 99999 not found");
    }
}