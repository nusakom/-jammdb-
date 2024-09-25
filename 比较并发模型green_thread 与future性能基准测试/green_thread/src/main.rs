```rust
use tokio; // 引入tokio异步运行时
use indicatif::ProgressBar; // 引入进度条库
use std::time::Instant; // 引入计时器
use std::sync::Arc; // 引入Arc以实现线程安全的共享

mod lib; // 引入lib.rs中的JammDB模块

#[tokio::main] // 主函数标记为异步
async fn main() {
    let db = Arc::new(lib::JammDB::new()); // 创建JammDB实例并用Arc包裹
    let iterations = 10000000; // 插入的总数
    let batch_size = 10000; // 批量插入的大小

    let bar = ProgressBar::new(iterations); // 初始化进度条
    let start_insertion = Instant::now(); // 记录插入开始时间
    
    let mut handles = Vec::new(); // 用于存储任务句柄

    // 插入数据
    for batch_start in (0..iterations).step_by(batch_size as usize) {
        let db_clone = Arc::clone(&db); // 克隆Arc以在任务中使用
        let bar_clone = bar.clone(); // 克隆进度条以在任务中更新
        let batch: Vec<_> = (batch_start..(batch_start + batch_size).min(iterations))
            .map(|idx| (format!("key{}", idx), format!("value{}", idx))) // 生成键值对批次
            .collect(); // 收集批次

        // 使用tokio::spawn创建异步任务
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

    bar.finish_with_message("Insertion completed"); // 完成时更新进度条消息

    let duration_insertion = start_insertion.elapsed(); // 计算插入耗时
    println!("Total inserts: {}", iterations); // 打印总插入次数
    println!("Elapsed time: {:?}", duration_insertion); // 打印耗时
    println!("Throughput: {:.2} inserts per second", 
        iterations as f64 / duration_insertion.as_secs_f64()); // 打印每秒插入次数

    // 获取并打印最后一条插入的数据
    if let Some(value) = db.get(&format!("key{}", iterations - 1)).await {
        println!("Retrieved key{}: {}", iterations - 1, value); // 打印获取的值
    }
}
```

### 注释说明
- **tokio**：用于异步编程的运行时，支持高并发操作。
- **indicatif**：用于创建用户友好的进度条。
- **Arc**：提供线程安全的共享指针，适合在多线程环境中使用。
- **ProgressBar**：用于显示插入操作的进度，提升用户体验。
- **批量插入**：通过将插入操作分为多个批次，减少上下文切换，提高效率。
- **异步任务**：使用 `tokio::spawn` 创建任务，使插入操作并发进行，提升性能。
- **时间统计**：记录插入的总时间和吞吐量，帮助评估性能。