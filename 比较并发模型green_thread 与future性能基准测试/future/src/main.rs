```rust
mod lib; // 引入 lib.rs 中定义的模块

use crate::lib::JammDB; // 引用 JammDB 结构体
use indicatif::ProgressBar; // 引入进度条库用于显示进度
use std::sync::Arc; // 引入 Arc 用于实现线程安全的共享
use std::time::Instant; // 引入 Instant 用于计时
use tokio::main; // 引入 Tokio 的 main 宏
use tokio::sync::Semaphore; // 引入 Semaphore 控制并发
use futures::future::join_all; // 引入 join_all 用于等待多个异步任务

#[main] // 标记为异步主函数
async fn main() {
    let db = JammDB::new(); // 创建 JammDB 实例
    let iterations: u64 = 100_000_00; // 设置插入总数为一亿
    let batch_size: u64 = 10_000; // 设置批量插入大小为一万

    // 生成键值对并收集到 Vec 中
    let keys: Vec<_> = (0..iterations)
        .map(|i| (i.to_string(), format!("value{}", i))) // 生成每个键值对
        .collect();

    let pb = ProgressBar::new(iterations); // 初始化进度条
    let start_time = Instant::now(); // 记录开始时间

    let keys = Arc::new(keys); // 将键值对放入 Arc 中以支持共享
    let semaphore = Arc::new(Semaphore::new(50)); // 创建信号量以限制并发数量为50

    // 创建异步任务列表
    let futures: Vec<_> = (0..(iterations / batch_size)).map(|i| {
        let semaphore_clone = Arc::clone(&semaphore); // 克隆信号量
        let db_clone = db.clone(); // 克隆数据库实例
        let pb_clone = pb.clone(); // 克隆进度条
        let keys_clone = Arc::clone(&keys); // 克隆键值对

        async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); // 获取许可以控制并发
            let start = (i * batch_size) as usize; // 计算当前批次的起始索引
            let end = (start + batch_size as usize).min(keys_clone.len()); // 计算结束索引
            let batch = &keys_clone[start..end]; // 取出当前批次的键值对

            db_clone.insert_batch(batch).await.unwrap(); // 执行批量插入
            pb_clone.inc(batch.len() as u64); // 更新进度条
        }
    }).collect();

    join_all(futures).await; // 等待所有插入任务完成

    pb.finish_with_message("Done!"); // 进度条完成提示

    let elapsed_time = start_time.elapsed(); // 计算耗时
    let throughput = iterations as f64 / elapsed_time.as_secs_f64(); // 计算每秒插入的数量

    // 打印插入总数、耗时和吞吐量
    println!("Total inserts: {}", iterations);
    println!("Elapsed time: {:?}", elapsed_time);
    println!("Throughput: {:.2} inserts per second", throughput);

    // 获取并打印特定键的值
    if let Some(value) = db.get("99999").await {
        println!("Retrieved key99999: {}", value);
    } else {
        println!("Key 99999 not found"); // 如果未找到键，则打印消息
    }
}
```

### 注释说明
- **模块引入**：引入了 `lib` 模块中的 `JammDB`，用于后续的数据库操作。
- **进度条**：使用 `indicatif` 库来显示插入操作的进度。
- **Arc 和 Semaphore**：使用 `Arc` 以支持线程安全的共享，使用 `Semaphore` 控制并发任务的数量，防止过多并发导致资源争用。
- **批量插入**：将键值对分批插入，利用 `join_all` 等待所有异步插入任务完成。
- **性能统计**：记录插入总数、耗时和吞吐量，并输出插入结果。