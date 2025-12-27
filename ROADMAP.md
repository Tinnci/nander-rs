# nander-rs 项目路线图

## 📊 项目状态分析

### 当前完成度概览

| 模块 | 状态 | 完成度 |
|------|------|--------|
| 项目架构 | ✅ 完成 | 100% |
| CH341A USB 驱动 | ✅ 完成 | 95% |
| SPI NAND 协议 | ✅ 完成 | 95% |
| SPI NOR 协议 | ✅ 完成 | 95% |
| CLI 命令框架 | ✅ 完成 | 95% |
| 芯片数据库 | ✅ 完成 | 100% |
| 性能优化 | ✅ 完成 | 95% |
| I2C EEPROM | ✅ 完成 | 95% |
| Microwire EEPROM | ✅ 完成 | 95% |
| SPI EEPROM | ✅ 完成 | 95% |

---

## 🔍 与 SNANDer 功能对比

### SNANDer 完整功能列表

参考 SNANDer v1.7.9，以下是原版支持的所有功能：

#### 硬件支持
- ✅ CH341A USB-SPI 编程器
- ✅ SPI-to-Microwire 适配器

#### 存储设备类型
| 类型 | SNANDer | nander-rs |
|------|---------|-----------|
| SPI NAND Flash | ✅ 85 款芯片 | ✅ ~79 款芯片 |
| SPI NOR Flash | ✅ 146 款芯片 | ✅ ~128 款芯片 |
| I2C EEPROM (24Cxx) | ✅ 11 款 | ✅ 11 款芯片 |
| Microwire EEPROM (93Cxx) | ✅ 8 款 | ✅ ~6 款芯片 |
| SPI EEPROM (25xxx) | ✅ 11 款 | ✅ 11 款芯片 |

#### 操作功能
| 功能 | SNANDer | nander-rs |
|------|---------|-----------|
| 读取芯片 ID (-i) | ✅ | ✅ |
| 读取数据 (-r) | ✅ | ✅ |
| 写入数据 (-w) | ✅ | ✅ |
| 擦除芯片 (-e) | ✅ | ✅ |
| 验证数据 (-v) | ✅ | ✅ |
| 列出芯片 (-l) | ✅ | ✅ |
| 禁用 ECC (-d) | ✅ | ✅ |
| 坏块跳过 (-k) | ✅ | ✅ |
| OOB 读写 (-o) | ✅ | ✅ |
| ECC 忽略 (-I) | ✅ | ✅ |

---

## 🎯 路线图里程碑 (架构驱动)

### Phase 0: 架构重构与结构化迁移 (v0.2.0) ✅ 已完成
**目标**: 消除 single-file 瓶颈，建立分层架构，清理技术债务。

*   **领域层定义**: ✅ 完善工具集核心逻辑抽象。
*   **基础设施解耦**: ✅ 
    *   [x] 迁移 `Programmer` trait 至 `infrastructure/programmer`。
    *   [x] 迁移 Flash 协议至 `infrastructure/flash_protocol`。
*   **应用层解构**: ✅ 为所有的 Flash 操作创建了独立的 `UseCase`。
*   **CLI 处理器拆分**: ✅ 将命令行执行逻辑迁移至 `presentation/cli/handlers/`。
*   **清理遗留代码**: ✅ 删除了所有旧版冗余模块。

### Phase 1: 数据库扩展与 NAND 深度功能 (v0.3.0) ✅ 核心完成
**目标**: 基于新架构实现 SNANDer 的完整 NAND 支持。

*   **数据库迁移**: ✅ 完成
    *   [x] 建立 `infrastructure/chip_database` 结构。
    *   [x] 批量迁移原版 200+ 芯片定义 (~79 NAND + ~128 NOR)。
*   **协议迁移**: ✅ 完成
    *   [x] 迁移 NAND 读/写/擦除逻辑到新架构
    *   [x] 迁移 NOR 读/写/擦除逻辑到新架构
    *   [x] 实现 OobMode 支持（None/Included/Only）
    *   [x] 实现 ECC 控制策略
    *   [x] 实现进度回调机制
    *   [x] 启用 NOR Flash 支持
    *   [x] 实现完整 Verify 功能
*   **坏块管理 (-k)**: ✅ 已完成 - BadBlockStrategy (Fail/Skip/Include) 完整实现。
*   **OOB 区域支持 (-o)**: ✅ 已完成 - OobMode (None/Included/Only) 完整实现。
*   **性能优化**: ✅ 完成
    *   [x] 扩展 Programmer trait 添加 `spi_read_bulk` 和 `spi_transaction` 方法
    *   [x] 实现 CH341A 批量传输优化 (4KB 块传输)
    *   [x] 优化 NOR Flash 连续读取 (Fast Read 命令 + 32KB 块)
    *   [x] 优化 NAND Flash 页面读取 (减少 USB 往返次数)

### Phase 2: EEPROM 支持 (v0.3.5) ✅ 部分完成
**目标**: 添加 I2C、Microwire 和 SPI EEPROM 支持。

1. **I2C EEPROM (24Cxx 系列)** ✅ 已完成
   - [x] 实现 CH341A I2C 模式切换
   - [x] 实现 I2C 读/写协议
   - [x] 支持 24c01 ~ 24c1024 全系列 (11 款芯片)
   - [x] 智能地址模式处理 (1-byte + device addr 和 2-byte)
2. **Microwire EEPROM (93Cxx 系列)** ✅ 已完成
   - [x] 实现 GPIO 位切换协议 (Bit-banging)
   - [x] 支持 93c06 ~ 93c86 全系列 (6 款芯片)
   - [x] 支持 7/9/11 位地址自动计算
3. **SPI EEPROM (25xxx 系列)** ✅ 已完成
   - [x] 实现 SPI EEPROM 协议
   - [x] 支持 25010 ~ 251024 全系列 (11 款芯片)
   - [x] 1/2/3 字节地址模式自动切换
   - [x] 9-bit 地址支持 (命令字节嵌入)

**NOR Flash 功能完善**:
- [x] 4 字节地址模式 (>16MB 芯片)
- [x] 添加状态寄存器写保护控制 (Protect/Status)
- [x] 优化连续读取性能 (Fast Read + Bulk)

---

### Phase 3: 高级功能与 UX (v0.4.0)
**目标**: 提升操作灵活性与用户交互体验。

1. **高级操作选项**
   - [x] 禁用内部 ECC 模式 (-d)
   - [x] OOB 模式支持 (-o, -O)
   - [x] ECC 错误忽略模式 (-I)
   - [x] 坏块跳过模式 (-k, -K)
   - [x] 部分读写 (地址 + 长度)
   - [x] SPI 速率动态调整 (CLI --speed)

2. **用户体验改善**
   - [x] 进度回调支持 (Progress callback)
   - [x] 详细进度条 (速度统计, 预计剩余时间)
   - [x] 彩色终端输出 (彩色 Log + 进度条)

3. **可靠性增强**
   - [x] 校验和验证 (Verify)
   - [x] 自动重试失败操作
       - [x] 在 `ReadRequest` 中新增 `retry_count` 字段（已完成）
       - [x] 暴露 `retry_count` 至 `ReadParams`/CLI 并实现自动重试逻辑 (已完成)
   - [ ] 断点续传支持

---

### Phase 4: 生态系统 (v0.5.0)
**目标**: 提供完整的工具生态系统

#### 任务列表

1. **GUI 应用**
   - [ ] 基于 egui 的跨平台 GUI
   - [ ] 可视化芯片信息
   - [ ] 拖拽文件支持
   - [ ] 操作历史记录

2. **库 API 改进**
   - [ ] 发布到 crates.io
   - [ ] 完善 API 文档
   - [ ] 添加使用示例
   - [ ] 异步 API 支持

3. **其他编程器支持**
   - [ ] CH347 支持
   - [ ] FT232H 支持
   - [ ] 树莓派 SPI 支持

---

## 🔧 技术债务清理

### 当前代码问题

1. **待办事项 (TODO)**
   - ~~`src/cli/commands.rs`: 实际读写逻辑未实现~~ → ✅ 已完成
   - ~~`src/flash/nand.rs`: `wait_ready` 缺少超时~~ → ✅ 已完成
   - ~~`src/flash/nor.rs`: `wait_ready` 缺少超时~~ → ✅ 已完成

2. **代码警告**
   - ~~`src/flash/nand.rs`: `set_feature` 方法未使用~~ → ✅ 已在 ECC 控制中使用
   - ~~`cargo clippy` 警告~~ → ✅ 已清理

3. **架构改进**
   - [x] 将 `ChipInfo` 的芯片类型特定字段改为枚举
   - [x] 统一错误处理机制
   - [x] 启用 NOR Flash 支持
   - [x] 添加单元测试覆盖 (MockProgrammer + NOR/NAND protocol tests)

---

## 📈 优先级排序

### 高优先级 (短期)

1. ✅ 项目可以编译
2. ✅ 实现 `read` 命令完整功能
3. ✅ 实现 `write` 命令完整功能  
4. ✅ 添加超时机制到等待循环
5. ✅ 实现坏块管理 (BadBlockStrategy)
6. ✅ 实现 OOB 支持 (OobMode)
7. ✅ 扩展芯片数据库 (~207 款芯片)

### 中优先级 (1 个月内)

1. ✅ 实现 `erase` 和 `verify` 命令
2. ✅ 实现坏块管理 (Skip/Include 策略)
3. ✅ 添加 ECC 控制 (enable/disable)
4. ✅ 添加 `list` 命令
5. ✅ 性能优化 (批量传输)
6. ✅ 单元测试覆盖

### 低优先级 (长期)

1. I2C/Microwire/SPI EEPROM 支持
2. GUI 应用
3. 其他编程器支持
4. 发布到 crates.io

---

## 📋 下一步行动

立即开始的任务：

1. ~~**实现读取功能**~~ → ✅ 已完成
   - ~~修改 `src/cli/commands.rs` 中的 `read` 函数~~
   - ~~连接到 `SpiNand` 或 `SpiNor` 的实际读取方法~~

2. ~~**添加超时机制**~~ → ✅ 已完成
   - ~~在 `wait_ready` 循环中添加超时检查~~
   - ~~使用 `std::time::Instant` 计时~~

3. ~~**扩展芯片数据库**~~ → ✅ 已完成
   - [x] 从 SNANDer 的 `spi_nand_flash.c` 提取芯片定义 (~79 款)
   - [x] 从 SNANDer 的 `spi_nor_flash.c` 提取芯片定义 (~128 款)

4. **下一步任务**
    - [x] 实现 SPI EEPROM (25xxx) 支持
    - [x] 实现 I2C EEPROM (24Cxx) 支持  
    - [x] 实现 4 字节地址模式支持
    - [x] 实现 Microwire EEPROM (93Cxx) 支持 (Phase 2)
    - [x] 实现状态寄存器写保护控制 (Protect/Status)
    - [x] 详细进度条 (速度统计, 预计剩余时间)
    - [x] 终端彩色输出优化 (Phase 3)
    - [x] 自动重试机制 (Phase 3)
        - [x] 在 `ReadRequest` 中新增 `retry_count` 字段（已完成）
        - [x] 在 `ReadParams`/CLI 中暴露 `retry_count` 并实现自动重试逻辑 (已完成)
    - [x] 坏块表 (BBT) 扫描 (Advanced) (已完成)
    - [x] 坏块表 (BBT) 更新与导出 (Advanced) (已完成)
    - [ ] 基于 egui 的 GUI 预览 (Phase 4)

---

## 🔗 参考资源

- [SNANDer 源码](https://github.com/McMCCRU/SNANDer/src) - 原始 C 实现
- [CH341A 数据手册](http://www.wch.cn/products/CH341.html)
- [nusb 库文档](https://docs.rs/nusb)
- [SPI NAND 规范](https://www.jedec.org/)

---

*最后更新: 2025-12-27*
