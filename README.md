# -基于 jammdb 数据库的高性能、高可靠的异步文件系统-
### 2024/9/26
# 使用green_thread与future的方法对jammdb数据库性能基准测试
## 测试环境：
处理器型号: 11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz

总内存：12 GiB

操作系统：Ubuntu 20.04

rustc 1.83.0-nightly (04a318787 2024-09-15)

cargo 1.83.0-nightly (c1fa840a8 2024-08-29)
## 建立一个简单的jammdb数据库
### 主要特性

1. **线程安全**：使用 `Mutex` 保护数据，确保在多线程环境下的安全访问。

2. **简洁的 API**：提供简单易用的插入和获取方法。

3. **高效的性能**：在多线程插入时仍能保持较好的性能表现。
### `lib.rs`
```rust
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
```
`JammDB` 是一个简单的数据库实现，支持多线程环境下的插入和获取操作。其核心设计包括以下要点：
- **数据结构**: 
  - 使用 `HashMap` 来存储键值对，其中键为字符串类型，值为整数类型。
  - 通过 `Mutex` 来确保线程安全，保证在多线程访问时，只有一个线程能够操作数据，从而避免竞争条件和数据不一致。
- **方法**:
  - `new()`: 创建一个新的 `JammDB` 实例，初始化一个空的 `HashMap` 用于存储键值对。
  - `insert()`: 将给定的键值对插入数据库中，确保数据的存储。
  - `get()`: 根据指定的键获取对应的值，便于快速查找数据。
### `main.rs`
```rust
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
```
**模块导入**：
- 使用 `mod lib;` 引入 `lib.rs` 中定义的数据库结构。
- 引入必要的模块以支持线程、安全共享和进度条。
**创建数据库实例**：
- 通过 `let db = Arc::new(Mutex::new(lib::JammDB::new()));` 创建数据库实例，并用 `Arc` 和 `Mutex` 实现共享与安全性。
**初始化线程与进度条**：
- 使用 `Vec` 存储线程句柄，同时初始化进度条以跟踪操作进度。
**主循环创建线程**：
- 在 `for` 循环中，为每个线程克隆数据库和进度条的引用，创建线程以并发插入数据。
**数据插入**：
- 每个线程插入特定数量的数据，格式化键并实时更新进度条。
**等待所有线程完成**：
- 使用 `join()` 方法确保所有线程执行完毕。
**打印统计信息**：
- 计算并打印总插入次数、耗时及每秒插入的数量。
**读取数据并打印**：
- 尝试获取键为 "key0" 的值并进行打印，以验证插入操作的结果。
### 性能测试
```
cargo run
```
### 输出结果
```
Insertion completed [################## 100%]
Total inserts: 100000
Elapsed time: 1.5s
Throughput: 66666.67 inserts per second
Retrieved key0: 0
```
![alt text](atlas/jammdb.jpg)
### 解释
1. **进度条**：
1. **插入完成信息**：
   - `Insertion completed [################ 100%`：这表明所有插入操作已完成，并且进度条达到100%。同步操作的进度条反映了每个线程在插入数据时的实时状态，确保用户能够直观地了解操作的进展。
2. **总插入次数**：
   - `Total inserts: 100000`：表示此次操作共插入了100,000条数据。通过同步控制，确保了所有数据都被正确插入，没有丢失或重复。
3. **耗时**：
   - `Elapsed time: 14.496753666s`：插入操作所用的总时间为约14.5秒。这个时间反映了在同步访问下，数据库处理多线程插入的效率。
4. **吞吐量**：
   - `Throughput: 6898.10 inserts per second`：每秒插入约6898条数据。这一指标展示了在同步环境中，尽管使用了多个线程，但由于 `Mutex` 的存在，确保了数据的一致性与安全性，能够高效地处理插入请求。
5. **读取结果**：
   - `Retrieved key0: 0`：成功从数据库中读取键为 "key0" 的值。这个步骤确保了插入操作的有效性，表明数据在插入后可以被准确访问。
### 同步特性分析
- **线程安全性**：使用 `Arc` 和 `Mutex` 确保数据在多个线程之间安全共享，避免了并发插入时可能出现的数据竞争和不一致性。 
- **操作一致性**：同步的插入确保每个线程在进行数据插入时不会干扰其他线程，保证了数据的完整性和一致性。
- **性能表现**：尽管实现了同步，但吞吐量依然保持在一个较高的水平，这说明设计上有效平衡了线程安全与性能效率，适合高并发的插入场景。
### 应用场景
- **实时数据处理**：适用于对性能有要求的实时应用场景。
- **缓存系统**：作为临时存储，快速访问频繁使用的数据。
- **轻量级应用**：在简单场景中，作为复杂数据库的替代方案。
## 创建绿色线程和future方法修改的jammdb数据库
（注：以下的代码示例是使用绿色线程和future方法修改的jammdb数据库，用于比较不同并发模型下的性能。）
### 绿色线程分析
在 Rust 中，特别是使用 Tokio 这样的异步运行时时，我们可以看到一种实现“绿色线程”的概念。绿色线程是指在用户空间调度的线程，而不是由操作系统内核调度的线程。以下是对代码中绿色线程实现的分析。
#### 1. 绿色线程的定义
绿色线程是由程序运行时管理的，而不是操作系统。这使得它们能够更加轻量，且在执行时不会受到操作系统调度的限制。Tokio 使用这种模型，通过异步任务和事件循环实现绿色线程的功能。
#### 2. Tokio 的任务调度
在代码中，使用 `tokio::spawn` 创建异步任务：
```rust
let handle = tokio::spawn(async move {
    let tx = db_clone.tx(true).await.unwrap(); // 获取事务
    for (key, value) in &batch {
        tx.put(key.clone(), value.clone()).await; // 异步插入数据
    }
    bar_clone.inc(batch.len() as u64); // 更新进度条
});
```
- **异步任务**：每个任务都在 `async move` 块中定义，确保可以在未来某个时刻执行。这样的任务不会立即执行，而是被调度到 Tokio 的调度器中。
- **非阻塞行为**：`await` 关键字使得任务在等待 I/O 操作（如数据库写入）时能够释放控制权，允许其他任务继续执行。这种设计使得即使在高并发环境中，程序也能高效地利用 CPU 资源。
#### 3. 绿色线程的优势
- **高效的资源利用**：由于多个绿色线程共享同一个 OS 线程，它们的上下文切换开销大大降低。这种轻量级的线程实现适合于大量并发任务的场景。
- **简化编程模型**：程序员不需要管理线程的创建和调度，专注于编写业务逻辑。通过 `async/await` 语法，异步编程变得直观且易于维护。
- **无阻塞 I/O**：使用绿色线程的 Tokio 允许在执行 I/O 操作时不会阻塞整个程序，从而提高吞吐量和响应速度。
#### 4. 示例中的绿色线程表现
在我们的 `main` 函数中，创建了多个异步任务用于批量插入数据：
```rust
for batch_start in (0..iterations).step_by(batch_size as usize) {
    let db_clone = Arc::clone(&db);
    let bar_clone = bar.clone();
    let batch: Vec<_> = (batch_start..(batch_start + batch_size).min(iterations))
        .map(|idx| (format!("key{}", idx), format!("value{}", idx)))
        .collect(); // 生成批次

    let handle = tokio::spawn(async move {
        let tx = db_clone.tx(true).await.unwrap(); // 获取事务
        for (key, value) in &batch {
            tx.put(key.clone(), value.clone()).await; // 异步插入数据
        }
        bar_clone.inc(batch.len() as u64); // 更新进度条
    });

    handles.push(handle); // 收集任务句柄
}
```
- **批量插入**：通过将插入操作分批执行，每个批次创建一个异步任务，充分利用了绿色线程的调度优势。
- **并发控制**：`RwLock` 的使用确保在多个任务之间共享数据时的安全性，而绿色线程的非阻塞特性则允许任务在 I/O 操作时切换，避免阻塞其他任务。
### 总结
绿色线程通过轻量级的用户空间调度和非阻塞 I/O 操作为 Rust 的异步编程模型提供了强大的支持。通过 Tokio 的异步任务，我们能够高效地管理并发操作，实现高性能的数据库插入和读取。在 JammDB 示例中，绿色线程的实现展示了如何在高并发环境中保持资源利用效率，同时简化程序的复杂性。随着需求的增加，可以进一步扩展功能，提升性能。
### 分析 JammDB 中 Future 的使用
在 JammDB 的实现中，Future 的使用是实现异步编程的核心，允许非阻塞的操作和高效的并发管理。
#### 1. Future 的定义与实现
**Future 是什么？**
Future 是一个表示某个值在未来某个时刻可能可用的对象。通过使用 `async` 和 `await`，我们可以将异步代码编写得像同步代码一样直观。
#### 2. JammDB 中的 Future 方法
**插入批量数据**
```rust
pub async fn insert_batch(&self, pairs: &[(String, String)]) -> Result<(), String> {
    let mut data = self.data.write().await; // 获取写锁，返回 Future
    for (key, value) in pairs {
        data.insert(key.clone(), value.clone());
    }
    Ok(())
}
```
- **异步操作**：`insert_batch` 使用 `await` 来等待写锁的获取。这意味着在获取锁的过程中，当前任务会被挂起，其他任务可以继续执行，从而避免了阻塞。
**获取值**
```rust
pub async fn get(&self, key: &str) -> Option<String> {
    let data = self.data.read().await; // 获取读锁，返回 Future
    data.get(key).cloned()
}
```
- **非阻塞读取**：在 `get` 方法中，`await` 同样用于获取读锁。通过这种方式，可以在 I/O 操作等待期间保持高效。
#### 3. 在主函数中使用 Future
**创建异步任务**
```rust
let futures: Vec<_> = (0..(iterations / batch_size)).map(|i| {
    let db_clone = db.clone();
    let keys_clone = Arc::clone(&keys);

    async move {
        let start = (i * batch_size) as usize;
        let end = (start + batch_size as usize).min(keys_clone.len());
        let batch = &keys_clone[start..end];

        db_clone.insert_batch(batch).await.unwrap(); // 等待插入完成
    }
}).collect();
```
- **任务调度**：每个任务在 `async move` 块中定义，使得它们可以被 Tokio 的调度器调度和执行。`await` 使得任务在数据库插入时不会阻塞。
**等待所有任务完成**
```rust
join_all(futures).await; // 等待所有 Future 完成
```
- **并发管理**：使用 `join_all` 等待多个 Future 完成，确保所有数据都被插入。
#### 4. Future 的优势
- **非阻塞性**：使用 Future 使得程序在等待 I/O 操作时可以继续执行其他任务，提高了资源利用率。
- **简化代码结构**：通过 `async/await` 语法，编写异步代码变得直观，降低了复杂性。
- **高效并发**：允许创建大量并发任务而不增加线程的上下文切换开销。
### 总结
在 JammDB 的实现中，Future 的使用是核心特性，使得异步编程得以有效实现。通过合理利用 `await`，程序可以在高并发场景下保持良好的性能和响应能力，确保数据操作的效率。Future 的引入，使得 Rust 在处理异步任务时，能够有效地管理资源并提高代码的可读性。
## 对future和green_thread的基准测试
为了测量future和绿色线程的性能对比。创建任务的使用多线程并发地向 `JammDB` 数据库插入数据，并且在插入过程中实时更新进度条。
### 任务功能概述
1. **数据库初始化**：
   - 创建一个共享的 `JammDB` 实例，使用 `Arc<Mutex<>>` 来确保在多线程环境中的安全访问。
2. **多线程数据插入**：
   - 启动多个线程，每个线程负责插入一系列数据到数据库中。
   - 每个线程根据其索引和迭代次数生成唯一的键（如 `key0`, `key1` 等），并插入对应的值（简单的整数）。
3. **进度条更新**：
   - 使用 `indicatif` 库创建进度条，并在每次成功插入数据后更新进度条，以显示插入的进度。
4. **吞吐量和响应时间计算**：
   - 记录插入开始和结束的时间，计算总插入次数、耗时和吞吐量（每秒插入的数量）。
5. **示例读取操作**：
   - 在所有插入操作完成后，尝试从数据库中读取一个特定的键（如 `key0`）并输出其值。
### 代码关键点
- **线程安全**：使用 `Arc` 和 `Mutex` 确保数据库和进度条在多个线程间的安全共享。
- **性能监控**：通过 `ProgressBar` 实时显示插入进度，增加用户体验。
- **吞吐量计算**：提供了性能指标，便于评估数据库在并发插入操作下的表现。
迭代次数为10000、线程数为10：
future和green_thread:
![alt text](atlas/first.jpg)
迭代次数为10000、线程数为100：
![alt text](atlas/image.png)
迭代次数为100000、线程数为100：
![alt text](atlas/3.jpg)
迭代次数为100000、线程数为1000：
![alt text](atlas/4.jpg)
迭代次数为1000000、线程数为10000:
![alt text](atlas/30.jpg)
迭代次数为10000000、线程数为10000:
![alt text](atlas/43.jpg)
迭代次数为10000000、线程数为100000:
![alt text](atlas/9.jpg)
迭代次数为10000000、线程数为1000000:
![alt text](<atlas/2024-09-26 072203.jpg>)
迭代次数为10000000、线程数为10000000:
![alt text](<atlas/2024-09-26 072421.jpg>)
### 对比分析：Future 与 绿色线程
1. **基本概念**：
   - **Future**：代表一个可能尚未完成的计算，允许你在将来某个时刻获取其结果。在 Rust 中，`Future` 是异步编程的核心，使用 `async/await` 语法来处理异步任务。
   - **绿色线程**：由用户空间调度的轻量级线程，不依赖于操作系统的线程管理。Tokio 框架实现了这一模型，通过异步任务和事件循环来实现。
2. **性能**：
   - **Future**：由于其非阻塞特性，能够在等待 I/O 操作时让出控制权，提高了并发性能。`jammdb_future` 显示出显著的吞吐量604913.05 inserts per second，说明其高效处理大量并发任务。
   - **绿色线程**：虽然也可以处理高并发，但在多线程模型中，资源管理和上下文切换可能导致性能下降。在 `jammdb_green` 中，吞吐量最高为吞吐量为 149,395.89 inserts per second，相对较低。
3. **编程模型**：
   - **Future**：使用 `async/await` 语法使得异步编程更直观。开发者可以专注于逻辑，而不需管理线程的生命周期。
   - **绿色线程**：依赖于事件循环和调度器，程序员需要理解调度的机制。虽然可以处理高并发，但相对复杂性较高。
4. **适用场景**：
   - **Future**：适合于 I/O 密集型应用，尤其是需要高并发的场景，如网络服务、数据库操作等。
   - **绿色线程**：可以用于需要轻量级任务管理的场景，但在高并发时可能不如 `Future` 高效。
### 总结
- 使用 `Future` 提供了更高的性能和更简单的编程体验，尤其在处理大量异步操作时更为显著。
- 绿色线程虽然轻量，但在高并发情况下可能受到操作系统调度的限制，导致性能下降。因此，选择合适的模型应基于应用需求和并发特性。

### 2024/9/8
经过这次会议对之前开题报告的方向进行调整，针对文件系统的异步驱动和故障还原系统。在完成这一系列的任务再针对网络进行异步处理。

再此之前对目标2和4进行修改，完成对开题报告的综诉现在详细的补充以便于后期任务的进行。

在stm32开发板上进行embassy的实践，研究在物理设备上是否具有可行性

之前future和绿色线程对比的代码和方案不够具体只有结论不能体现出来在什么环境下性能如何，现在重新在开发板上进行对比，这样子可以选择出更适合的方法用在嵌入式的环境下。
### 2024/9/5
完成开题报告的初稿[开题报告](https://github.com/nusakom/-jammdb-/blob/main/%E5%BC%80%E9%A2%98%E6%8A%A5%E5%91%8A/%E5%BC%80%E9%A2%98%E6%8A%A5%E5%91%8A.md)
### 2024/9/1
从基准测试结果来看，future_example 的性能优于 green_thread_example，其平均执行时间低于 green_thread_example。这可能是因为异步编程模型在处理短时间任务时更为高效，而 Rayon 在创建和管理线程时可能带来了额外的开销。

Rayon: 适用于数据并行任务，当任务的计算量大并且能够充分利用多核处理器时，Rayon 是一个不错的选择。
Tokio: 适用于需要处理大量异步 I/O 操作的任务。如果应用需要高效的异步操作来提高响应速度，Tokio 是更好的选择。
### 2024/8/31
完成 embasscy-cn阅读，写完博客上传到github

下周任务完成：绿色线程跟future性能对比
### 2024/8/24
完成embassy-cn 0.1.0 第一节阅读 [csdn链接](https://blog.csdn.net/m0_63714693/article/details/141507739?spm=1001.2014.3001.5501)
明天完成在裸机上异步
### 2024/8/23
使用cyclictest进行测试

====== cyclictest NO_STRESS_P1 begin ======

WARN: stat /dev/cpu_dma_latency failed: No such file or directory

T: 0 (    7) P:99 I:1000 C:   1000 Min:     30 Act:   54 Avg:   75 Max:     339

====== cyclictest NO_STRESS_P1 end: success ======

====== cyclictest NO_STRESS_P8 begin ======

WARN: stat /dev/cpu_dma_latency failed: No such file or directory

T: 0 (    7) P:99 I:1000 C:    997 Min:     30 Act:  120 Avg:  108 Max:    1172

T: 1 (    8) P:99 I:1500 C:    667 Min:     30 Act:  995 Avg:  121 Max:     995

T: 2 (    9) P:99 I:2000 C:    500 Min:     29 Act:  159 Avg:   95 Max:     683

T: 3 (   10) P:99 I:2500 C:    400 Min:     31 Act:  156 Avg:  123 Max:    1412

T: 4 (   11) P:99 I:3000 C:    333 Min:     29 Act: 1172 Avg:  145 Max:    1172

T: 5 (   12) P:99 I:3500 C:    286 Min:     32 Act:   42 Avg:  120 Max:     539

T: 6 (   13) P:99 I:4000 C:    250 Min:     30 Act:  486 Avg:   98 Max:    1300

T: 7 (   14) P:99 I:4500 C:    222 Min:     33 Act:  715 Avg:  166 Max:    1129

====== cyclictest NO_STRESS_P8 end: success ======

单线程测试 的延迟表现稳定，最大延迟保持在 339 微秒以内。

多线程测试 中，随着线程数量和周期的增加，系统的最大延迟显著增加，这表明在高负载条件下，系统的实时性会受到影响。最糟糕情况下的最大延迟超过了 1 毫秒（1412 微秒），这在某些实时应用中可能是不可接受的

### 2024/7/27 
1,商议确认论文题目《基于 jammdb 数据库的高性能、高可靠的异步文件系统》。

2,看陈林峰同学的论文《基于数据库的文件系统设计与实现》。
### 2024/7/28
1,《基于数据库的文件系统设计与实现》是作者编写的类 linux 操作系统 Alien_OS内核中移植了 DBFS，我选择在这个基础上将操作系统改写成异步os，将移植的DBFS改成自己写的。  

2，安装ubuntu 24.4在VM虚拟机上，配置实验环境（包括安装RUST，QUME，riscv64-linux-musl工具链等）。

3.然后将文档上传到GitHub的blog，git add . 然后 git commit -m "Describe the changes you made"最后 git push
### 2024/7/29
阅读论文在附录找到Alien_os的GitHub库[Alien]（https://github.com/nusakom/Alien ）并且克隆到本地。昨天的 riscv64-linux-musl 未安装成功，今天继续完成。
### 2024/7/30
网卡驱动掉了，改成arch_linux，用clash for windos成功连上网络。
### 2024/731
在ubuntu系统里面浏览器下载riscv64-linux-musl工具链，安装成功。
### 2024/8/1
ubuntu 24.4扩容遇到错误

 piix4_smbus 0000:00:07.3: 8HBus Host Controller not enabled!

 /dev/sda3: recovering journal

 /dev/sda3: clean,881904/2260992 files,8690642/9042944 bl0cks

 改成22.4 解决
### 2024/8/2
riscv64-unknown-linux-musl-gcc 工具链没有正确设置路径

riscv64-unknown-linux-musl-gcc:command not found

错了好几天，Gpt没有给我正确答案
### 2024/8/3
安装完riscv64-linux-mul-gcc工具链

riscv64-linux-musl-gcc --version

riscv64-linux-musl-gcc (GCC) 11.2.1 20211120

## 复现遇到 的问题

-make run- 之后
输出：
make[1]: Leaving directory '/home/zty/Alien/user/apps'

make[1]: Entering directory '/home/zty/Alien/user/c_apps'

Building C apps

riscv64-linux-musl-gcc -static -o eventfd_test eventfd_test.c;

/bin/sh: 1: riscv64-linux-musl-gcc: not found

make[1]: *** [Makefile:17: build] Error 127

make[1]: Leaving directory '/home/zty/Alien/user/c_apps'

make: *** [Makefile:122: user] Error 2
### 2024/8/8
更新一下新的库

遇到错误

error: could not compile `async_test` (bin "async_test") due to 1 previous error

make[1]: *** [Makefile:16: build] Error 101

make[1]: Leaving directory '/home/zty/Alien/user/musl'

make: *** [Makefile:122: user] Error 2

已经更改config.toml改成自己riscv-mul的地址
#### 这个错误是修改了工具链，解决方案是把MIPS的工具链都删了，然后重新安装riscv的这个然后重新创建链接就可以成功的编译
### 2024/8/12
qume我没意识到之前安装一个6.2版本的，优先权高于7.0

删除后再次编译，成功的把sysinfo，todo，slint，memory-game，printdemo这几个测试软件都通过了

手上没有星光2的开发板就没有继续后续的复现
### 2024/8/16
jammdb数据库的事务性质没有很好的测试和说明，为了实现系统发生故障或重启，这些数据也不会丢失，并且能够在系统恢复后被正确读取，采用WAL机制

WAL，即Write-Ahead Logging，中文译为预写日志。

WAL机制的核心思想是：在将数据修改写入磁盘之前，先将这些修改记录到日志中。只有当日志写入成功后，数据修改才会被提交。

事务开始： 当一个事务开始时，数据库系统会创建一个新的日志记录。

数据修改： 在事务执行过程中，对数据库中的数据进行修改时，这些修改操作都会被记录到日志中。

事务提交： 当事务提交时，数据库系统会将日志记录写入磁盘，然后才将数据修改写入数据文件。

系统崩溃： 如果系统在事务提交的过程中崩溃了，数据库系统可以通过读取日志来恢复未完成的事务，从而保证数据的完整性。

WAL日志采用Binlog日志： 记录了数据库的所有修改操作，用于主从复制和数据备份。

实现这个数据库做了一个no-std的修改，然后用数据库接口做了一个文件系统，在配合alien中的vfs接口就可以移植到内核中。

第一步预期1-2周完成。
### 2024/8/18
原先的虚拟机坏了，重新安装一个。在make run过程中无法进入静态编译，需要我手动下载才能进入。

对比sled和jammdb数据库，sled具有异步特性还有压缩算法，在长时间存储空间利用率更高。

但是sled不原生支持 no-std 环境，在移植过程中难度估计不小，估计要2周时间。