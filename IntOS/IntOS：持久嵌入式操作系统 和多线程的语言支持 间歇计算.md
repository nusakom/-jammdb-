# INTOS: 持久嵌入式操作系统及多线程间歇性计算的语言支持

作者：Yilun Wu*，Byounguk Min†，Mohannad Ismail‡，Wenjie Xiong‡，Changhee Jung†，Dongyoon Lee*
* 纽约州立大学石溪分校，†普渡大学，‡弗吉尼亚理工学院

## 摘要

本文介绍了INTOS，一种面向无电池能量收集平台的多线程间歇性计算的嵌入式操作系统及语言支持。INTOS通过传统的“线程”和“事务”简化了编程，并在非易失性内存中对持久对象进行自动撤销日志记录。INTOS在提供使用易失性内存以提升性能和能源效率的同时，传统事务无法确保易失性寄存器和内存状态的一致性。为解决此问题，INTOS提出了一种新的重放与绕过方法，消除了用户需要检查点易失状态的需求。在电力恢复时，INTOS通过撤销因电力中断而未完成的事务来恢复非易失性状态。为了重建易失状态，INTOS通过绕过已提交的事务和系统调用，重新启动每个线程并返回记录的结果而无需重新执行。INTOS旨在构建一个支持优先级抢占式多线程并确保系统调用或线程阻塞期间即使发生断电也能保持崩溃一致性的持久嵌入式操作系统。实验结果表明，在MSP430FR5994平台上，在1毫秒的极端断电频率下，INTOS相比于采用幂等处理的先前工作展示了1.24倍的延迟降低和1.29倍的能耗减少。这一趋势在Apollo 4 Blue Plus平台上更加显著。

## 1. 引言

能量收集系统[24, 30, 40, 54, 57]利用环境能源（如太阳能[27]、射频[35]）并通过小型电容器作为储能设备，代替使用电池。无需更换电池即可提供可持续的长期部署，使其广泛应用于人体植入物[29]、可穿戴设备[61]、野生动物追踪[67]、道路监控[26]和卫星[5]。由于电容器会周期性地耗尽和充电，能量收集系统上的程序执行本质上是间歇性的，涉及重复的电力中断和恢复。间歇性计算要求崩溃一致性，以确保在频繁的电力周期中正确恢复。

操作系统（OS）为应用程序开发者（用户）提供多线程、队列、信号量、事件和定时器等基本服务，帮助创建功能丰富的应用程序。例如，广泛使用的嵌入式操作系统如FreeRTOS [3]简化了各种嵌入式应用程序的开发。然而，这种级别的操作系统/运行时支持在间歇性计算环境中是缺失的。当前的一些解决方案，如ImmortalThreads [65]，提供了一个支持伪线程的微小运行时，并采用协作调度，但功能有限，它缺乏阻塞线程的等待列表，基于轮询的事件循环浪费了微控制器（MCU）的周期。许多任务驱动的解决方案，如Ink [64]和CatNap [52]，也不支持线程。

目前，特别为间歇性计算设计的更强大操作系统的需求日益增长。硬件技术的进步，如超低功耗微控制器（如TI的MSP430FR[6]和ARM的Cortex-M4[2]）以及非易失性存储器（如FRAM[12]和MRAM[10]），使得间歇性应用能够执行更多计算。新兴的间歇性应用越来越复杂，集成了多线程、通信、同步和事件响应等功能。我们已经开始在能量收集平台上看到机器和深度学习任务的应用[16, 28, 39, 48]。然而，用户在没有足够操作系统支持的情况下，必须自行管理这些复杂性。

当前的崩溃一致性解决方案在构建持久嵌入式操作系统内核时难以采用或效率不高。一些方法[22, 31, 49, 52, 53, 56, 64]要求用户将应用程序分解为任务图，每个任务都需要固有的失败原子性和幂等性。这给程序员带来了相当大的挑战[38, 65]。拆分内核系统调用（如创建线程或在满队列上阻塞）为任务并非易事。其他基于编译器的解决方案[15, 18, 19, 37, 47, 50, 55, 62]自动划分程序（如划分为幂等区域）并结合检查点，几乎不需要用户注释，因此它们可以用于构建持久的操作系统。然而，许多解决方案（除了Chinchilla[50]）假定程序仅在非易失性存储器上执行，忽略了易失性内存的潜在优势。
先前的工作在多线程、队列、信号量、事件和定时器的支持以及编程负担和易失性内存的使用方面各不相同。下面的表格总结了这些特性，并展示了INTOS与之前间歇性计算解决方案的比较：

| 方案             | 崩溃一致性 | 支持多线程？ | 队列支持？ | 信号量支持？ | 事件支持？ | 定时器支持？ | 编程负担 | 使用易失性内存？ |
|------------------|------------|--------------|------------|--------------|------------|--------------|----------|----------------|
| Alpaca [49], Coala [53], MayFly [31]  | 手动任务分解 | 否（任务） | 否         | 否          | 否         | 否          | 高       | 是            |
| Chain [22]       | 手动任务分解 | 否（任务）   | 有限       | 否          | 否         | 否          | 高       | 是            |
| Coati [56], Ink [64] | 手动任务分解 | 否（任务）   | 否         | 否          | 有限       | 否          | 高       | 是            |
| CatNap [52]      | 手动任务分解 | 否（任务）   | 有限       | 否          | 有限       | 否          | 高       | 是            |
| Ratchet [62], WARio [37] | 幂等处理 | 否          | 否         | 否          | 否         | 否          | 无       | 否            |
| Chinchilla [50]  | 检查点和撤销日志 | 否          | 否         | 否          | 否         | 否          | 很低     | 是            |
| HarvOS [15], RockClimb [19] | 静态能量分析 | 否          | 否         | 否          | 否         | 否          | 很低     | 否            |
| TICS [38]        | 检查点和撤销日志 | 否          | 否         | 否          | 否         | 否          | 低       | 否            |
| ImmortalThread [65] | 检查点和微延续 | 是（伪栈）  | 否         | 有限       | 有限       | 否          | 低       | 否            |
| **INTOS (本研究)** | 重放和撤销日志 | 是（栈式）  | 是         | 是          | 是         | 是          | 中（事务）| 是（重放）     |

表1：INTOS与先前间歇性计算解决方案的主要特性比较。

## 嵌入式操作系统背景
嵌入式操作系统（OS）[3, 13, 14, 25, 41, 42]是为特定嵌入式系统提供基本服务的专业软件层，能够让用户（应用程序开发人员）即便在资源受限的环境中，也能使用传统的基于线程的编程模型创建功能丰富的应用程序。INTOS的设计不仅支持常规的多线程和同步机制，还通过重放与撤销日志机制确保在频繁的断电情况下实现全系统崩溃一致性。

## 非易失性存储器的事务
事务已经被广泛采用为非易失性存储器（NVM）的编程模型，INTOS通过Rust的类型系统确保事务的一致性，提供崩溃原子性。每当非易失性对象被修改时，INTOS会自动进行撤销日志记录，而无需用户手动进行操作。这种机制借鉴了英特尔的持久内存编程工具包（PMDK）[7]，简化了开发持久嵌入式操作系统内核的任务。

## 与现有工作的比较
目前间歇性计算的解决方案中缺乏完整的操作系统功能。表1显示了INTOS相比于其他方案的多线程支持、同步机制、事务管理等方面的优势。INTOS通过优先级抢占式调度、自动事务日志等机制，简化了用户的编程负担，同时确保在系统崩溃时的全系统一致性。此外，INTOS利用重放和撤销机制，在电力恢复时通过绕过已提交的事务来重建易失性状态，从而实现能效更高的间歇性计算。

### 4.2 重放与绕过机制

为了应对上述挑战，INTOS 提出了一个创新的重放与绕过机制 (§5)，以保证整个系统在断电后的崩溃一致性。INTOS 消除了用户为了保持易失性寄存器和内存状态的一致性而进行检查点保存或自定义崩溃恢复的需求。在电源恢复后，INTOS 首先通过撤销未提交的事务来恢复非易失性状态。然后，线程从起始点重新启动，安全地恢复，而无需保留寄存器和栈状态。在执行过程中，已提交的事务和系统调用会被重放并绕过，即返回记录的结果，而不重新执行，从而实现更高效的能量恢复。这样，易失性状态会被重建，程序能够在断电点之后继续执行。

### 4.3 永久性嵌入式操作系统

系统调用 INTOS 提供了全面的多线程特性（表2），可与 FreeRTOS 相媲美。例如，线程可以通过 `sys_queue_*` 和 `sys_semaphore_*` 系统调用进行通信和/或同步。一个线程可能会因为队列为空或满而阻塞。多个线程可以通过事务内获取共享持久对象的引用访问该对象。稍后在 §6 中，我们将深入探讨 INTOS 的编程模型如何通过 Rust 的强类型系统确保强制使用互斥锁进行同步。

内核崩溃一致性 与用户线程类似，INTOS 内核代码（包括系统调用）使用了易失性和非易失性内存。INTOS 内核采用相同的撤销日志事务，以确保在系统调用期间更新的持久性内核对象的崩溃一致性。表2列出了内核事务数量以及被内核级事务保护的持久性内核数据的示例。稍后在 §7.2 中，我们还将讨论内核如何为频繁使用的链表操作（例如，准备队列和等待队列）优化事务（不使用撤销日志）。通过相同的重放与绕过机制，INTOS 保证即使在系统调用过程中发生电源故障，并且某些线程被阻塞，也能保持崩溃一致性。

### 4.4 INTOS 程序示例

下面的代码示例展示了一个包含两个事务的 INTOS 程序。在这个示例中，创建了一个队列用于线程之间的消息传递，类似于 Linux 的管道。一个线程（`recognize`）从传感器读取数据，并将数据发送到另一个线程（未展示）进行处理。`PBox` 是指向持久性对象的智能指针。用户可以使用事务构造 `transaction::run(|j, t|{ ... })` 包裹程序片段，其中 `j` 代表日志对象，`t` 是系统调用令牌。日志对象强制执行限制，确保像 `PBox` 这样的持久性智能指针不能在事务之外被解引用，而系统调用令牌限制系统调用只能在事务内执行。相关细节将在 §6 中进一步介绍。

第一个事务（第2-9行）创建了一个大小为 `Q_SZ` 的队列。该队列包含类型为 `Result` 的对象，以及一个持久对象（`stats`），用于存储每个类别/结果的计数（`cnt`）。事务在完成一些处理后返回这些对象。第二个事务（第10-26行）使用只读引用读取一个机器学习模型，不需要撤销日志。在完成 I/O 后，它在易失性缓冲区中进行数据处理（第16-20行），如过滤、标准化和分类。这种策略比在非易失性缓冲区中进行所有中间计算的性能和能量效率更高。随后，事务获取了对持久性对象（`stats`）的可变引用，该对象是在第一个事务中创建并传递的，并对其进行更新。由于这是获取可变引用后的首次写操作，INTOS 自动应用了撤销日志。最后，事务调用 `sys_queue_send_back` 系统调用，将结果放入由 INTOS 维护的队列中。另一个线程（未展示）可以从队列中接收结果以进行后续处理。

### 5. 重放与绕过恢复机制

以下两个章节展示了 INTOS 的重放与绕过机制，并附有示例。

#### 5.1 单线程崩溃一致性

让我们使用示例程序 `recognize` 来说明 INTOS 的重放与绕过恢复机制，该程序包含两个用户级事务：TX1 和 TX2。图3a展示了在没有电源故障情况下的执行过程。为了简化，省略了第一个事务 TX1 中的系统调用，重点展示第二个事务 TX2 中的系统调用 `sys_queue_send_back`（第23行），假设该系统调用包含两个内核级事务：TX3 和 TX4。

在图3b中，我们考虑了一个场景，即 TX1 已经提交，而电源故障发生在 TX2 开始之前（事务外）。在电源恢复后，INTOS 从线程的开头开始重放（步骤1），并从空寄存器和栈状态 s0 重新启动。INTOS 的类型系统 (§6) 保证了事务之外的非易失性状态不会被更新。易失性状态在重放过程中被重建。由于非易失性状态 sc（在电源故障前）已经包含了已提交事务 TX1 的效果，重新执行 TX1 将是不正确的且不可幂等。因此，INTOS 跳过了事务 TX1（步骤2），仅返回记录的返回值，而不重新执行。重放过程中没有进行系统调用，也不需要内核级恢复。INTOS 保证程序到达与 sc 相同的状态 s1，之后可以安全恢复。

接下来，我们考虑发生在事务内部的电源故障情况。在图3c中，电源故障发生在系统调用前的用户级事务内部。INTOS 的撤销日志事务保证了事务内更新的非易失性状态的失败原子性。在电源恢复后，INTOS 应用撤销日志（步骤1）将非易失性状态从 sc 回滚到 s1，即事务开始前的状态。接着，INTOS 从初始状态 s0 开始重放（步骤2）。已提交的事务 TX1 被跳过（步骤3），INTOS 在重放过程中重建所有易失性状态，使 s1 的状态等同于故障前的 sc 状态。

### 5. 重放与绕过恢复
接下来的两节展示了 INTOS 的重放与绕过方法，并附有示例。

#### 5.1 单线程崩溃一致性
我们通过示例演示 INTOS 的重放与绕过恢复机制，如代码清单 1 中涉及的两个用户级事务 TX1 和 TX2。图 3a 展示了在没有断电情况下的 `recognize` 执行过程。为了简化，省略了 TX1 中的系统调用，仅突出显示了 TX2 中的 `sys_queue_send_back` 系统调用（第 23 行）。假设该系统调用包含两个内核级事务 TX3 和 TX4。

在图 3b 中，假设 TX1 已提交，但在 TX2 开始前发生了断电。电源恢复后，INTOS 从头开始重放线程（步骤1），并从初始状态 s0 重新启动。INTOS 的类型系统（详见 §6）确保事务外的非易失状态不会被更新，且重放过程中会重建易失状态。由于非易失状态 sc（断电前的状态）已经包含了 TX1 提交的效果，重新执行 TX1 将是不正确的且非幂等的。因此，INTOS 绕过了 TX1（步骤2），直接返回已记录的返回值而不重新执行。在绕过期间没有系统调用，也不需要进行内核级恢复。INTOS 确保程序恢复到与 sc 相同的状态 s1，并从此安全地继续执行。

接下来，我们考虑事务内发生断电的情况。在图 3c 中，用户级事务执行期间在系统调用之前发生了断电。INTOS 的撤销日志记录事务确保了非易失状态在事务内的失败原子性。电源恢复后，INTOS 应用撤销日志（步骤1）将非易失状态从 sc 回滚至 s1，即事务开始前的状态。然后，INTOS 从初始状态 s0 开始重放（步骤2）。已提交的 TX1 被绕过（步骤3），INTOS 在重建易失状态的过程中，使重放后的状态 s1 等同于断电前的状态 sc。

图 3d 展示了系统调用完成后（在用户级事务内部）发生断电时的操作。与往常一样，INTOS 应用撤销日志（步骤1）并启动重放（步骤2），绕过已提交的事务 TX1（步骤3）。特别是在这种情况下，在重放 TX2 时，INTOS 还绕过了已完成的系统调用（步骤4）。因此，INTOS 避免了更改内核状态的需求——在系统调用过程中对内核非易失状态的任何更改（在断电之前）都保持不变。INTOS 内核在系统调用完成后（断电前）缓存系统调用的返回值，并在重放时简单返回缓存的值。对于用户线程来说，系统调用可被视为嵌套的黑盒事务。

现在，我们探讨系统调用期间发生断电的情况。如前所述，INTOS 使用事务（TX3 和 TX4）来保护内核侧非易失数据。如果崩溃发生在内核级事务之前（或外部），如图 3e 所示，情况比较简单，与图 3c 展示的情况一致。此时不需要对内核做任何回滚操作。INTOS 仅需撤销调用系统调用的用户级事务（步骤1），然后启动重放与绕过恢复机制。

另一方面，如果崩溃发生在内核级事务内部，如图 3f 所示，INTOS 首先撤销内核级事务 TX4（步骤1），将内核状态回滚至 s2，然后撤销用户级事务 TX2（步骤2），将用户状态回滚至 s1。接着，INTOS 从初始状态 s0 开始重放（步骤3），绕过已提交的用户侧事务 TX1（步骤4）和内核侧事务 TX3（步骤5）。注意，INTOS 总是优先回滚内核级事务（在任何中止的用户级事务之前）。这一策略在多线程场景中具有正确性含义，我们将在下一节讨论。

#### 5.2 多线程崩溃一致性
接下来，我们讨论 INTOS 确保多线程崩溃一致性的方法。具体而言，INTOS 采用基于优先级的恢复与重放机制。电源恢复后，INTOS 总是优先恢复并重放那些已经准备就绪且优先级最高的线程。

图 4a 展示了一个两线程执行过程，没有发生断电。最初，优先级较高的线程 2 等待（例如，在队列上），低优先级线程 1 使用 `sys_queue_send_back` 将数据排入队列，使线程 1 满足等待条件。在系统调用过程中，内核事务 TX3 更新了 NVM 中的内核队列对象。线程 1 被唤醒后，由于其优先级更高，INTOS 调度器抢占了线程 1，并通过事务 TX4 修改线程相关的持久化链表（如 ready-list 和 wait-list）进行上下文切换。系统调用通常会在一个事务中更新与系统调用相关的内核数据结构（如队列），在另一个事务中修改调度相关的链表。上下文切换后，线程 2 开始运行，而线程 1 保留在 ready-list 中等待执行。

首先考虑一个简单的断电情况。如果断电发生在系统调用期间（例如在 TX3 或 TX4 或两者之间），这将是一个单线程的情况。恢复协议与图 3f 中的情形相同。

假设如图 4b 所示，断电发生在上下文切换后，线程 2 正在运行。这是一个多线程场景：线程 1 和线程 2 均处于可运行状态。电源恢复后，INTOS 恢复并运行线程 2——这是断电时正在运行的线程。优先级调度器总是调度准备就绪且优先级最高的线程。

在非易失性内存（NVM）中保持状态有助于推断哪些步骤已经完成，然后系统可以继续执行剩下的操作。以插入操作为例，如图5所示。链表插入操作包括六个有序步骤。首先，将新节点链接到其前后节点（步骤1-2）。接着，删除后续节点与前一个节点之间的前后链接（步骤3-4）。最后，插入新节点与前后节点之间的前后链接（步骤5-6）。如果在步骤4之前发生断电，恢复时首先会检索操作日志以确定操作类型和相关节点。此时发现新节点与前后节点的链接已存在，表明步骤1-2已完成。此外，前一个节点与后一个节点之间的前向链接已被删除，而后向链接仍然存在，这表明系统在步骤4完成之前发生了崩溃。INTOS可以通过执行步骤4-6来继续操作。

### 7.3 撤销日志优化
默认情况下，INTOS事务会在每次首次写入时自动执行撤销日志（即获得可变引用后），如清单1中的第22-23行所示。INTOS引入了另一种智能指针类型 `Ptr<T>`，它利用Rust的类型系统静态检测写后读（WAR）依赖。事务仅在存在WAR依赖时记录旧值，从而减少了日志记录。在事务中，用户应通过取消引用持久对象指针来获取引用。`Ptr<T>` 不提供原始引用，并对访问接口施加了限制，例如`r.read()`和`r.write()`。因此，使用`Ptr<T>`会增加一些额外的编码工作。

### 8 讨论
#### 部分与整体系统持久性的事务
PMDK（libpmemobj）和INTOS事务的一个关键区别在于它们的持久性保证。libpmemobj仅支持“部分”系统持久性，仅确保事务中的非易失性对象可以恢复。因此，恢复程序执行通常需要用户自定义的崩溃恢复逻辑，以实现包括易失性状态在内的一致的系统状态。而INTOS通过其回放与旁路机制，保证恢复整个系统的持久性，包括持久性和易失性状态。

#### 事务长度
为确保前进，INTOS要求事务必须在电容器完全充电的情况下能够完成。INTOS一次只处理一个就绪的最高优先级线程，并采用回放与旁路机制跳过已提交的事务和系统调用，只要每个电源周期至少有一个事务成功通过，系统就能继续前进。INTOS要求用户通过性能分析确保该属性。限制程序区域的大小是许多间歇性计算系统的常见要求，INTOS的内核事务设计为简短以满足这一要求。

#### 能源感知调度器
如果硬件能够监控电容器中的剩余能量，可以在INTOS中设计能源感知调度器。例如，当能量即将耗尽时，可以不调度线程。

#### 实时能力
INTOS在电源开启时提供与FreeRTOS相当的实时能力。然而，由于间歇性计算中固有的非确定性交能量特性，INTOS无法提供硬实时保证。

#### Rust
选择Rust是为了利用其静态正确性保证。用户可以使用C或其他语言，只要他们遵守编程规则。由于内核与用户程序之间的接口是明确的，C程序可以与Rust INTOS内核静态链接。但使用C时需要更复杂的静态程序分析来验证遵守编程规则。此外，还应进行额外的静态分析以实现自动撤销日志。

### 9 实现
我们使用Rust编程语言实现了INTOS，利用其强大的静态类型系统来确保INTOS的编程模型，并实现了与C相当的性能。INTOS内核的初始实现与FreeRTOS相似，移植到Rust并扩展了事务和崩溃一致性支持。目前，INTOS支持两种架构：ARM Cortex-M4和MSP430。INTOS的整体实现（不包括测试和基准代码）大约包含9900行Rust代码。

#### 多线程
INTOS内核将基本数据结构（如线程控制块、线程间通信对象（如队列、信号量）和调度列表（如就绪列表、等待列表））分配到非易失性内存中。

#### 回放表
为支持回放与旁路恢复，INTOS为每个线程维护三个回放表，分别缓存用户级事务、内核级事务和系统调用的返回值。

#### 日志大小
事务完成后生成的系统调用日志会被垃圾回收，从而限制了系统调用日志的最大长度。

### 10 评估
我们在MSP430FR5994和Apollo 4 Blue Plus两个平台上评估了INTOS的性能。

优化应用：对于频繁使用系统调用的应用，链表优化显著提高了性能，提升幅度超过40%。然而，对于较简单的单线程BC、MLP和AR（仅使用内存分配系统调用）来说，提升幅度较小。撤销日志优化的效果在很大程度上取决于应用的特性。MLP、KV、SEN、EM和MQ的存储操作中，只有少部分存在写后读（WAR）依赖。因此，撤销日志优化在这些应用中展现了显著的性能提升。

### 10.4 Apollo 4实验
现在，我们将实验转移到Apollo 4 Blue Plus上进行。该平台配备了ARM Cortex-M4微控制器（MCU）、384KB的TCM（更快的SRAM）、2MB的SRAM和2MB的非易失性MRAM。不过，值得注意的是，Apollo上的MRAM目前只支持字节读取，无法字节写入。为了解决这个限制，我们通过使用（快速的）TCM作为易失性存储，并指定（较慢的）SRAM作为非易失性存储来模拟执行环境。在我们的实验中，SRAM的顺序访问速度大约是TCM的2-3倍，速度差距大于MSP430中FRAM和SRAM的差距。此次（模拟的）Apollo 4实验有两个目的。首先，它证明了INTOS能够支持不同的MCU架构：MSP430和ARM Cortex-M4。其次，它展示了一个情景，即易失性和非易失性存储之间的延迟差距更加明显。该开发板没有板载调试探针来测量能耗，因此本次实验的重点是延迟比较。

图11展示了在没有电源故障的情况下，Apollo 4 Blue Plus的延迟开销，相对于仅使用TCM的基准进行了归一化。由于Ratchet仪表化的程序崩溃，ETL和STATS条目缺失。由于易失性和非易失性存储之间的差距更大（由TCM和SRAM模拟），结果显示延迟开销高于MSP430实验（图6）。INTOS和Ratchet的延迟开销分别为2.07倍和3.44倍，Ratchet受到较慢的非易失性存储的更大影响。

图12展示了在电源故障间隔分别为1ms、500ns和200ns时的延迟开销。由于Apollo 4上的ARM Cortex-M4的时钟频率较高，故障间隔被设置得比MSP430小得多。200ns允许大约19,000次周期执行。趋势依然保持一致。即使在极端的200ns故障间隔情况下，INTOS相对于SRAM的延迟开销为2.52倍。INTOS比Ratchet低1.37倍。

### 10.5 INTOS编程开销
INTOS的编程模型要求用户在非易失性存储中分配持久性对象，并定义事务以确保对持久性对象更新的崩溃一致性。量化编程开销是一个挑战，但作为替代，表3展示了每个应用程序的源代码行数（LOC）及用于持久性对象分配和事务代码的新增/修改的LOC。通过观察四个实际的RIoTBench应用程序，表格显示修改的幅度从11%（STATS: 46/413）到26%（TRAIN: 132/511）不等。虽然这些百分比看起来较大，但需要注意的是，这些更改涉及的是持久性对象分配和事务代码，这是我们认为可以理解和管理的部分。

### 11 结论
INTOS是一个支持多线程间歇性计算的持久化嵌入式操作系统及语言支持。INTOS通过事务确保非易失性对象的崩溃一致性。与检查点易失性状态不同，INTOS提出了一种回放与旁路恢复机制，通过不重新执行已提交的事务和系统调用来重建易失性状态。通过在MSP430FR和Apollo 4上的评估显示，INTOS相比基于编译器的幂等处理，具有更低的延迟和能耗成本。

### 致谢
我们感谢匿名审稿人和报告人的宝贵反馈。这项工作部分得到了NSF资助，资助号为CNS-2135157、CCF-2153747、CCF-2153748、CCF-2153749、CNS-2314681和CNS-2214980。