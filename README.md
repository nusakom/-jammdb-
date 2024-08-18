# -基于 jammdb 数据库的高性能、高可靠的异步文件系统-

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