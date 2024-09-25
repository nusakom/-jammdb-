mod lib; // 引入 lib.rs 中定义的 JammDB

use std::sync::{Arc, Mutex}; // 引入 Arc 和 Mutex 用于线程安全
use std::thread; // 引入 thread 模块以支持多线程
use std::time::Instant; // 引入 Instant 用于计算时间
use indicatif::{ProgressBar, ProgressStyle}; // 引入 indicatif 用于显示进度条

fn main() {
    let db = Arc::new(Mutex::new(lib::JammDB::new())); // 创建 JammDB 实例并使用 Arc 和 Mutex 进行共享
    let mut handles = vec![]; // 存储线程句柄
    let iterations = 10000; // 每个线程的插入次数
    let num_threads = 10; // 线程数量

    let bar = Arc::new(Mutex::new(ProgressBar::new(num_threads * iterations as u64))); // 创建进度条
    bar.lock().unwrap().set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} {msg} [{bar:40}] {percent:>3}%") // 设置进度条样式
        .progress_chars("##-"));

    let start_time = Instant::now(); // 记录开始时间

    for i in 0..num_threads {
        let db_clone = Arc::clone(&db); // 克隆数据库引用
        let bar_clone = Arc::clone(&bar); // 克隆进度条引用
        let handle = thread::spawn(move || { // 创建线程
            let mut db = db_clone.lock().unwrap(); // 获取锁以访问数据库
            for j in 0..iterations {
                db.insert(format!("key{}", i * iterations + j), j.try_into().unwrap()); // 插入数据
                let mut bar = bar_clone.lock().unwrap(); // 获取锁以更新进度条
                bar.inc(1); // 更新进度
            }
        });
        handles.push(handle); // 保存线程句柄
    }

    for handle in handles {
        handle.join().unwrap(); // 等待所有线程完成
    }

    bar.lock().unwrap().finish_with_message("Insertion completed"); // 完成进度条
    let duration = start_time.elapsed(); // 计算持续时间
    let total_inserts = num_threads * iterations; // 计算总插入次数
    let throughput = total_inserts as f64 / duration.as_secs_f64(); // 计算吞吐量

    println!("Total inserts: {}", total_inserts); // 打印总插入次数
    println!("Elapsed time: {:?}", duration); // 打印耗时
    println!("Throughput: {:.2} inserts per second", throughput); // 打印每秒插入次数

    let db = db.lock().unwrap(); // 获取锁以读取数据
    if let Some(value) = db.get("key0") { // 尝试获取键为 "key0" 的值
        println!("Retrieved key0: {}", value); // 打印获取的值
    }
}
