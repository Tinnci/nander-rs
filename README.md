# nander-rs

🦀 **A modern SPI NAND/NOR Flash programmer written in Rust**

A complete rewrite of [SNANDer](https://github.com/McMCCRU/SNANDer) in Rust, designed for maximum portability and reliability.

## ✨ Features

- **Pure Rust USB** - Uses `nusb` for native USB communication, no `libusb` DLL required
- **Cross-platform** - Works on Windows, Linux, and macOS
- **Single binary** - No runtime dependencies, just one executable
- **Memory safe** - Rust's ownership system prevents buffer overflows and memory corruption
- **Rich CLI** - Modern command-line interface with progress bars and clear error messages
- **Cross-platform GUI** - Intuitive graphical interface for interactive flash operations
- **Extensible** - Trait-based architecture makes it easy to add new programmers or chips

## 🛠 Supported Hardware

### Programmers
- **CH341A** - The ubiquitous USB-SPI programmer (fully supported)
- More programmers can be added by implementing the `Programmer` trait

### Flash Types
- **SPI NAND** - Full support including OOB/spare area and bad block management
- **SPI NOR** - Standard JEDEC SPI NOR flash

### Chips
See [src/database/chips.rs](src/database/chips.rs) for the full list. Common chips include:
- GigaDevice: GD5F1GQ5UEYIG, GD25Q128C, etc.
- Winbond: W25N01GV, W25Q128JV, etc.
- Macronix: MX35LF1GE4AB, MX25L12833F, etc.
- Micron: MT29F1G01, etc.
- XTX, FORESEE, and more...

## 📦 Installation

### From Source
```bash
cargo install --path .
```

### Pre-built Binaries
Download from the [Releases](https://github.com/tinnci/nander-rs/releases) page.

## 🚀 Usage

### Launch Graphical User Interface
```bash
nander gui
```

### Detect Flash Chip (CLI)
```bash
nander info
```
Output includes JEDEC ID, chip details, and for NAND chips, ECC status.

### List Supported Chips
```bash
nander list
```

### Read Flash to File
```bash
nander read -o backup.bin
nander read -o partial.bin -l 0x100000 -s 0x0   # Read 1MB from start
nander read -o raw.bin -d                        # Raw read with ECC disabled (NAND)
```

### Write File to Flash
```bash
nander write -i firmware.bin
nander write -i firmware.bin --no-verify   # Skip verification
nander write -i raw.bin -d                 # Raw write with ECC disabled (NAND)
```

### Erase Flash
```bash
nander erase                    # Erase entire chip
nander erase -l 0x200000        # Erase first 2MB
```

### Verify Flash
```bash
nander verify -i firmware.bin
```

### ECC Control (NAND only)
The `-d` / `--no-ecc` flag disables internal ECC for raw operations:
- Reads full page data including ECC bytes
- Useful for complete flash dumps including OOB area
- Required for external ECC software processing

## 🔧 Development

### Building
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Linting
```bash
cargo clippy -- -D warnings
```

## 📐 Architecture

本项目采用**分层架构**，正在从遗留单体结构迁移到新架构：

```
nander-rs/
├── src/
│   ├── main.rs              # CLI 入口点
│   ├── lib.rs               # 库导出
│   ├── error.rs             # 全局错误类型
│   │
│   ├── domain/              # 💎 领域层 - 核心业务逻辑
│   │   ├── types.rs         # 核心类型 (Capacity, Address, JedecId 等)
│   │   ├── chip.rs          # 芯片规格模型
│   │   ├── flash_operation.rs # Flash 操作抽象接口
│   │   ├── bad_block.rs     # 坏块管理策略
│   │   └── ecc.rs           # ECC 控制策略
│   │
│   ├── application/         # 📦 应用层 - 用例编排
│   │   ├── use_cases/       # 具体业务用例 (待迁移)
│   │   └── services/        # 应用服务 (待迁移)
│   │
│   ├── infrastructure/      # 🔧 基础设施层 - 技术实现
│   │   ├── programmer/      # 硬件编程器驱动
│   │   │   ├── ch341a/      # CH341A USB 驱动
│   │   │   └── traits.rs    # Programmer trait 定义
│   │   ├── flash_protocol/  # Flash 协议实现
│   │   │   ├── nand/        # SPI NAND 协议
│   │   │   ├── nor/         # SPI NOR 协议
│   │   │   └── commands.rs  # SPI 命令常量
│   │   └── chip_database/   # 芯片数据库
│   │       ├── nand/        # NAND 芯片定义 (按制造商)
│   │       └── nor/         # NOR 芯片定义 (按制造商)
│   │
│   ├── presentation/        # 🖥️ 表现层 - 用户交互
│   │   └── cli/             # CLI 实现 (待迁移)
│   │       └── handlers/    # 命令处理器
│   │
│   └── [Legacy Modules]     # 遗留模块 (逐步淘汰)
│       ├── cli/             # 当前 CLI 实现
│       ├── database/        # 当前芯片数据库
│       ├── flash/           # 当前 Flash 协议
│       └── hardware/        # 当前硬件驱动
```

详见 [ARCHITECTURE.md](ARCHITECTURE.md) 了解完整架构设计。

## 📝 Adding New Chips

Edit `src/database/chips.rs` and add your chip:

```rust
ChipInfo::nand(
    "YOUR_CHIP_NAME",
    "Manufacturer",
    [0xXX, 0xYY, 0xZZ],  // JEDEC ID
    CAPACITY_BYTES,
    PAGE_SIZE,
    OOB_SIZE,
    BLOCK_SIZE,
),
```

Run `nander info` to see the JEDEC ID of your chip.

## 🔗 Related Projects

- [SNANDer](https://github.com/McMCCRU/SNANDer) - Original C implementation
- [flashrom](https://flashrom.org/) - Universal flash programmer
- [probe-rs](https://probe.rs/) - Rust embedded debugging

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Note**: This is a clean-room implementation. While inspired by SNANDer's functionality and interface design, `nander-rs` contains no GPL-licensed code. All code was written from scratch in Rust.

## 🙏 Acknowledgments

- **[SNANDer](https://github.com/McMCCRU/SNANDer)** by McMCC - The original C implementation that inspired this project's feature set and CLI design. SNANDer is GPL-licensed; `nander-rs` is an independent reimplementation.
- The OpenIPC community for testing and feedback
- The Rust embedded community for excellent libraries (`nusb`, `clap`, `indicatif`, etc.)
- JEDEC and flash chip manufacturers for public documentation
