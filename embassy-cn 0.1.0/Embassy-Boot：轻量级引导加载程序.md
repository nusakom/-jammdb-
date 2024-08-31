# Embassy-Boot：轻量级引导加载程序
## 引言
嵌入式系统开发过程中，引导加载程序是至关重要的一环，负责管理固件的加载与更新。embassy-boot 是一个轻量级、可靠的引导加载程序，支持断电保护的固件升级以及固件回滚功能。本篇博客将深入探讨 embassy-boot 的设计原理、硬件支持及其应用场景，并结合实践经验展示如何使用该工具确保嵌入式设备的固件安全和高效更新。

## 1. embassy-boot 简介
embassy-boot 是一个精简的引导加载程序，专为嵌入式系统设计。它的主要功能包括：

断电保护的固件更新：确保即使在更新过程中断电，设备也不会损坏。
回滚机制：如果新固件启动失败，引导加载程序会自动回滚到之前的稳定版本。
内部和外部闪存支持：基于 embedded-storage 介质实现，支持多种存储配置。
数字签名验证：使用 ed25519 签名机制，确保固件的真实性和完整性。
## 2. 硬件支持
embassy-boot 适用于多种嵌入式平台，包括但不限于：

nRF52 系列：支持带或不带 SoftDevice 的版本。
STM32 系列：支持 L4、WB、WL、L1、L0、F3、F7 和 H7 等系列。
树莓派 RP2040：支持树莓派的 RP2040 微控制器。
其他支持 embedded-storage 介质的嵌入式平台也可以使用 embassy-boot，但可能需要进行适配。

## 3. 设计理念
embassy-boot 的设计中，将存储介质划分为四个主要分区：

BOOTLOADER 分区：存放引导加载程序本身。默认占用 8kB 闪存空间，但如果需要调试，建议扩展到 24kB。
ACTIVE 分区：存放主要应用程序，引导加载程序尝试从此分区启动应用。
DFU 分区：存放待更新的应用程序，该分区由应用程序写入。
BOOTLOADER STATE 分区：存储引导加载程序的当前状态，用于跟踪固件交换进度。
这种分区设计确保了引导加载程序可以安全地进行固件更新，即使在更新过程中出现断电或其他意外情况。

## 4. FirmwareUpdater 使用指南
FirmwareUpdater 是 embassy-boot 提供的一个方便工具，用于将固件写入 DFU 分区，并在设备重启时触发分区交换。以下是使用 FirmwareUpdater 的步骤：
```
use embassy_boot::FirmwareUpdater;
 
let mut updater = FirmwareUpdater::new(...);
 
// 将固件块写入 DFU 分区
updater.write_firmware(offset, &data_block).unwrap();
 
// 标记固件已准备好进行更新
updater.mark_updated().unwrap();
```
## 5. 数字签名与安全验证
为确保固件的安全性，embassy-boot 支持 ed25519 数字签名。以下是生成密钥对并对固件签名的步骤：

### 1.生成密钥对：
```
signify -G -n -p $SECRETS_DIR/key.pub -s $SECRETS_DIR/key.sec
```
### 2. 签名固件：
```
shasum -a 512 -b $FIRMWARE_DIR/myfirmware > $SECRETS_DIR/message.txt
signify -S -s $SECRETS_DIR/key.sec -m $SECRETS_DIR/message.txt -x $SECRETS_DIR/message.txt.sig
```
### 3. 嵌入式公钥并验证签名：
```
static PUBLIC_SIGNING_KEY: &[u8] = include_bytes!("key.pub");
 
let public_key = PublicKey::from_bytes(&PUBLIC_SIGNING_KEY).unwrap();
updater.verify_and_mark_updated(&public_key, &signature, firmware_length).unwrap();
```
确保私钥文件的安全存储，以防止未授权的固件签名。

## 6. 实战经验与建议
调试与开发：在调试引导加载程序时，建议使用扩展版的 BOOTLOADER 分区，并启用调试选项（例如使用 probe-rs）。
安全性：始终启用固件验证功能，并定期轮换密钥，以应对潜在的安全威胁。
兼容性：在使用不同的硬件平台时，可能需要编写特定的初始化代码来适配 embassy-boot。
## 结论
embassy-boot 为嵌入式系统提供了一个轻量级且可靠的引导加载程序解决方案。通过其分区设计和固件验证功能，开发者可以确保设备在进行固件更新时的安全性和稳定性。希望通过本篇博客，你能够更好地理解和应用 embassy-boot，为你的嵌入式项目提供坚实的基础。

这是你所需要的博客内容的完整版本，包括了代码示例和详细说明。希望这些内容能帮助你顺利撰写出一篇高质量的博客。
