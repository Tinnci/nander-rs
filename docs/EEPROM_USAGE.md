# EEPROM 使用指南

`nander-rs` 支持三大类常见的 EEPROM 芯片。本文档介绍如何连接硬件以及如何使用 CLI 进行操作。

## 1. 支持的类型

| 类型 | 芯片系列 | 接口 | 特点 |
|------|----------|------|------|
| **SPI EEPROM** | 25xxx | SPI | 标准 SPI 接口，速度快 |
| **I2C EEPROM** | 24Cxx | I2C | 简单 2 线接口，地址可调 |
| **Microwire** | 93Cxx | Microwire | 3 线同步接口，位翻转控制 |

---

## 2. 硬件连接 (CH341A)

### SPI EEPROM (25xxx)
连接到 CH341A 的 **SPI 区域** (1-8 号插槽)。
- D0 -> CS
- D1 -> CLK
- D2 -> MISO (DIN)
- D3 -> MOSI (DOUT)

### I2C EEPROM (24Cxx)
连接到 CH341A 的 **I2C 区域** (24xx 插槽)。
- SDA -> SDA
- SCL -> SCL
- 注意：硬件地址 A0, A1, A2 默认接地。

### Microwire EEPROM (93Cxx)
连接到 CH341A 的 **SPI 引脚**，但通过位翻转驱动。
- D0 (CS) -> CS
- D1 (CLK) -> SK (Clock)
- D3 (MOSI) -> DI (Data In)
- D2 (MISO) -> DO (Data Out)
- **注意**: 建议使用 8-bit 组织模式。

---

## 3. 命令行操作示例

### 芯片识别
自动检测连接的芯片：
```bash
nander info
```

### 读取数据
读取整个 EEPROM 到文件：
```bash
nander read backup.bin
```

### 写入数据
将文件内容写入 EEPROM：
```bash
nander write firmware.bin --verify
```

### 擦除芯片
EEPROM 会被填充为 `0xFF`：
```bash
nander erase
```

---

## 4. 常见问题 (FAQ)

**Q: 为什么 93Cxx 芯片识别不出来？**
A: Microwire 协议不支持标准 JEDEC ID。`nander-rs` 使用合成 ID 进行识别。请确保连线正确，且 D2 (MISO) 引脚没有被外部上拉电阻过度干扰。

**Q: I2C 芯片写入很慢？**
A: 这是正常的。I2C EEPROM 写入每个页面后需要 ~10ms 的内部写入周期，程序已自动处理此延迟。

**Q: 25040 (512B) 读取出来的后半部分全是 0xFF？**
A: `nander-rs` 已支持 25040 的 9-bit 地址（嵌入在命令字节中）。如果仍然有问题，请检查您的芯片是否为 1-byte 地址模式。
