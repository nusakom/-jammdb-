# Embassy-cn 0.1.0 第一节读后感
## 什么是异步？
异步是一种并发编程模型，他允许任务可以并发执行，当一个任务遇到I/O操作时，会将控制权交出，让其他任务继续执行，等到I/O操作完成时再继续执行。就比如你在煮面条的同时，可以去洗碗，这就是异步的一种体现。

## 同步与异步的区别
|特点 |  同步|  异步| 
 --- | ---:| :---: |
 任务执行顺序|顺序执行|并发执行|
|等待I/O操作|阻塞线程|不阻塞线程|
|代码风格|线性|非线性|
## Rust中的异步实现
Rust通过async/await关键字实现了异步编程：

async关键字： 用于定义异步函数，表示该函数可能包含异步操作。
await关键字： 用于等待异步操作完成。
## 异步的工作原理
Future： 当一个异步函数被调用时，它会返回一个Future对象，这个对象表示一个异步操作。
执行器（Executor）： 执行器负责调度和执行Future。
调度： 执行器会将就绪的Future添加到任务队列中，并从队列中取出任务执行。
await： 当一个异步函数遇到await关键字时，它会将控制权交给执行器，让执行器去调度其他任务。
 Embassy的Github库：embassy-rs/embassy: Modern embedded framework, using Rust and async. (github.com)
## Embassy的核心概念和用法
### 1. Nightly Rust和特性启用
为什么需要Nightly？ Embassy依赖于Rust Nightly版本中的一些实验性特性，这些特性能够提供更灵活和高效的编程方式。
关键特性：
#![no_std], #![no_main]: 告诉编译器不使用标准库，也不生成默认的main函数。
#![feature(type_alias_impl_trait)]: 允许使用impl Trait作为类型别名的参数。
### 2. 错误处理
defmt_rtt和panic_probe: 这些crate提供了将诊断信息输出到终端的功能，方便开发者调试程序。
建议使用defmt_rtt，它是一种高效的日志框架，特别适合嵌入式系统。
use {defmt_rtt as _, panic_probe as _};
         这行代码在Rust嵌入式开发中，特别是使用Embassy框架时非常常见。它主要用于引入两个crate：defmt_rtt 和 panic_probe，这两个crate为嵌入式开发提供了非常重要的调试和日志功能。

defmt_rtt:
高效日志框架: 专为嵌入式系统设计的日志框架。
RTT协议: 利用Real-Time Transfer协议将日志信息发送到连接的主机或调试器。
低开销: 在嵌入式系统资源有限的情况下，defmt_rtt提供了高效的日志功能，不会带来过多的性能开销。
panic_probe:
恐慌处理: 当程序发生panic（运行时错误）时，panic_probe会捕获错误信息并发送到连接的主机或调试器。
错误分析: 通过分析panic信息，开发者可以快速定位并修复程序中的错误。
```
use defmt_rtt as _;
use panic_probe as _;
 
defmt::info!("Hello, world!");
 
fn main() -> ! {
    // ... 你的主程序逻辑
    loop {}
}
```
defmt::info!("Hello, world!");：使用defmt宏输出一条日志信息。
panic_probe：在程序发生panic时，会自动捕获错误信息并发送到调试器。
### 3. 任务定义
异步任务: Embassy 中的任务都是异步的，这意味它们可以在执行过程中被暂停，以便系统去执行其他任务。这种设计使得 Embassy 可以高效地利用硬件资源，提高系统的响应性。

属性宏 #[embassy_executor::task]: 这个宏用于将一个函数标记为一个异步任务，告诉编译器这个函数需要特殊的处理。

任务参数: 任务可以接受参数，这些参数可以是各种类型，比如硬件设备、配置信息等。在下面这个例子中，led 表示要控制的 LED 引脚，interval 表示闪烁的间隔时间。

通过添加属性宏的方式，任务定义如下：
```
#[embassy_executor::task]
async fn blinker(mut led: Output<'static, P0_13>, interval: Duration) {
    loop {
        led.set_high(); // 设置 LED 为高电平（亮）
        Timer::after(interval).await; // 等待指定时间
        led.set_low();  // 设置 LED 为低电平（灭）
        Timer::after(interval).await; // 等待指定时间
    }
}
```
无限循环: loop 确保 LED 持续闪烁。
异步等待: await 关键字使得任务在等待定时器超时时可以被挂起，让其他任务获得执行机会。
没有空转等待: Embassy 采用内部计时器来实现任务的让出，避免了空转等待带来的资源浪费。
### 4. main函数
         Embassy应用的入口函数用#[embassy_executor::main]宏来定义，要求传入两个参数：Spawner、Peripherals。Spawner用于应用创建任务，Spawner是主任务创建其他任务的途径。 Peripherals来自HAL，它负责沟通可能用到的外设。

宏展开与执行器
宏展开： 当编译器遇到 #[embassy_executor::main] 宏时，它会展开成一段 Rust 代码，这段代码负责创建一个 Embassy 执行器，初始化硬件，并将 main 函数作为主任务添加到执行器中。
执行器： 执行器是 Embassy 的核心组件，负责调度和管理所有的任务。它会根据任务的优先级、状态等信息，决定哪个任务应该获得 CPU 时间。 #[embassy_executor::main]宏会创建一个Embassy执行器，负责调度和执行任务。
```
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
 
    let led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(300))));
}
```
### 启动过程详细解析
宏展开： 编译器将 #[embassy_executor::main] 宏展开，生成相应的代码。
创建执行器： 创建一个 Embassy 执行器实例，这个实例会管理整个应用程序的任务。
初始化硬件： 调用 embassy_nrf::init() 初始化硬件，获取对硬件设备的访问权限。
创建主任务： 将 main 函数包装成一个任务，并将其添加到执行器的任务列表中。
启动执行器： 调用执行器的 run 方法，开始调度和执行任务。
Spawner 和 Peripherals 的作用 
Spawner：
Spawner 是一个任务生成器，用于在运行时创建新的任务。
通过 Spawner，我们可以动态地创建任务，实现更灵活的系统。
Peripherals:
Peripherals 提供了对硬件设备的访问接口。
通过 Peripherals，我们可以配置和控制各种外设，比如 GPIO、定时器、UART 等。
### 5. Cargo.toml文件的作用
项目依赖管理: 定义了项目所依赖的外部库或本地库。
版本控制: 指定每个依赖库的版本号。
特性配置: 通过 features 字段，可以根据需要启用或禁用依赖库中的特定功能。
```
embassy-executor = { version = "0.3.0", path = "../../../../../embassy-executor", features = ["defmt", "nightly", "integrated-timers", "arch-cortex-m", "executor-thread"] }
embassy-time = { version = "0.1.4", path = "../../../../../embassy-time", features = ["defmt", "nightly"] }
embassy-nrf = { version = "0.1.0", path = "../../../../../embassy-nrf", features = ["defmt", "nrf52840", "time-driver-rtc1", "gpiote", "nightly"] }

  ```                      
原文链接：https://blog.csdn.net/m0_63714693/article/details/141507739

配置解析
embassy-executor: 提供了任务调度、异步运行时等核心功能。
features: 启用 defmt、夜间版特性、集成定时器、Cortex-M 架构支持和线程执行器。
embassy-time: 提供了时间相关的功能，如定时器、时钟等。
features: 启用 defmt 和夜间版特性。
embassy-stm32: 专门为 STM32 微控制器设计的 HAL。
features: 启用 defmt、STM32F411 支持、RTC 作为时间驱动器、GPIO 功能和夜间版特性。

embassy-nrf 配置详解
embassy-nrf: 这个 crate 是专门为 Nordic nRF52840 微控制器设计的，提供了与该芯片相关的硬件抽象层 (HAL)。
features 字段:
defmt: 启用 defmt 调试日志功能，用于在开发过程中打印调试信息。
nrf52840: 指定目标芯片为 nRF52840。
time-driver-rtc1: 选择 RTC1 作为时间驱动器，用于提供精确的时间信息。
gpiote: 启用 GPIO 任务事件，用于异步处理 GPIO 中断。
nightly: 启用一些需要 Rust nightly 特性的功能。
硬件抽象: embassy-stm32 提供了对 STM32F411 微控制器的硬件抽象，让我们可以方便地操作 GPIO、定时器等外设，而不需要直接操作寄存器。

时间驱动: RTC 提供了精确的时间基准，用于定时任务。
调试: defmt 可以帮助我们打印调试信息，方便定位问题。
灵活性: 通过配置不同的 features，我们可以根据项目的具体需求定制 HAL。

#### 为什么选择 RTC1 作为时间驱动器？
精度: RTC1 通常比其他定时器具有更高的精度，适合用于需要精确时间测量的场合。
独立性: RTC1 通常是独立的硬件模块，不受其他外设的影响，可以提供更稳定的时间基准。
功耗: RTC1 通常具有较低的功耗，适合在低功耗模式下使用。
其他 embassy-nrf 配置选项
除了上述配置，embassy-nrf 还提供了许多其他的配置选项，例如：

选择不同的外设: 可以选择不同的外设，如 SPI、I2C、UART 等。
配置时钟: 可以配置系统的时钟频率。
启用或禁用特定功能: 可以根据需要启用或禁用某些功能，例如低功耗模式、中断等。
其他 embassy crate 的作用
embassy-executor: 提供了任务调度、异步运行时等核心功能。
embassy-time: 提供了时间相关的功能，如定时器、时钟等。
