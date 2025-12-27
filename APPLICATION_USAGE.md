# Application Layer Use Cases - 使用示例

本文档展示如何使用应用层的用例（Use Cases）来执行 Flash 操作。

## 架构概览

```
User Code
    ↓
Application Layer (Use Cases)
    ↓
Infrastructure Layer (Flash Protocol)
    ↓
Hardware Layer (Programmer)
```

## 1. 芯片检测

```rust
use nander_rs::application::DetectChipUseCase;
use nander_rs::infrastructure::chip_database::ChipRegistry;

// 创建芯片检测用例
let detect_use_case = DetectChipUseCase::new(ChipRegistry::new());

// 执行检测
let (programmer, chip_spec) = detect_use_case.execute()?;

println!("检测到芯片: {}", chip_spec.name);
println!("制造商: {}", chip_spec.manufacturer);
println!("容量: {}", chip_spec.capacity);
```

## 2. 读取 Flash

```rust
use nander_rs::application::{ReadFlashUseCase, ReadParams};
use nander_rs::domain::OobMode;
use nander_rs::infrastructure::flash_protocol::nand::SpiNand;

// 创建 Flash 协议实例
let flash = SpiNand::new(programmer, chip_spec);

// 创建读取用例
let mut read_use_case = ReadFlashUseCase::new(flash);

// 设置读取参数
let params = ReadParams {
    address: 0x0,               // 起始地址
    length: 2048,               // 读取长度
    use_ecc: true,              // 使用 ECC
    ignore_ecc_errors: false,   // 是否忽略 ECC 错误（用于数据恢复）
    oob_mode: OobMode::None,    // 不包含 OOB 数据
    bad_block_strategy: BadBlockStrategy::Skip // 坏块处理策略
};

// `ReadRequest` 包含 `retry_count` 字段，可指定读取失败（或 ECC 错误）时的自动重试次数。
// 该参数已通过 `ReadParams` 暴露。


// 执行读取，带进度回调
let data = read_use_case.execute(params, |progress| {
    println!("读取进度: {:.1}%", progress.percentage());
})?;

println!("读取了 {} 字节", data.len());
```

## 3. 写入 Flash

```rust
use nander_rs::application::{WriteFlashUseCase, WriteParams};

// 创建写入用例
let mut write_use_case = WriteFlashUseCase::new(flash);

// 准备要写入的数据
let data = vec![0xFF; 2048];

// 设置写入参数
let params = WriteParams {
    address: 0x0,               // 起始地址
    data: &data,                // 数据
    use_ecc: true,              // 使用 ECC
    verify: true,               // 写入后验证
    ignore_ecc_errors: false,   // 验证读取时是否忽略 ECC 错误
    oob_mode: OobMode::None,    // OOB 模式
    bad_block_strategy: BadBlockStrategy::Skip, // 坏块处理策略
};

// 执行写入
write_use_case.execute(params, |progress| {
    println!("写入进度: {:.1}%", progress.percentage());
})?;

println!("写入完成");
```

## 4. 擦除 Flash

```rust
use nander_rs::application::{EraseFlashUseCase, EraseParams};

// 创建擦除用例
let mut erase_use_case = EraseFlashUseCase::new(flash);

// 设置擦除参数
let params = EraseParams {
    address: 0x0,               // 起始地址（必须块对齐）
    length: 128 * 1024,         // 擦除长度（块大小的倍数）
    bad_block_strategy: BadBlockStrategy::Skip, // 坏块处理策略
};

// 执行擦除
erase_use_case.execute(params, |progress| {
    println!("擦除进度: {:.1}%", progress.percentage());
})?;

println!("擦除完成");
```

## 5. 验证 Flash

```rust
use nander_rs::application::{VerifyFlashUseCase, VerifyParams};

// 创建验证用例
let mut verify_use_case = VerifyFlashUseCase::new(flash);

// 准备期望的数据
let expected_data = vec![0xFF; 2048];

// 设置验证参数
let params = VerifyParams {
    address: 0x0,               // 起始地址
    data: &expected_data,       // 期望数据
    use_ecc: true,              // 使用 ECC
    ignore_ecc_errors: false,   // 是否忽略 ECC 错误
    oob_mode: OobMode::None,    // OOB 模式
    bad_block_strategy: BadBlockStrategy::Skip, // 坏块处理策略
};

// 执行验证
verify_use_case.execute(params, |progress| {
    println!("验证进度: {:.1}%", progress.percentage());
})?;

println!("验证通过");

## 6. 状态寄存器与写保护 (Status & Protect)

```rust
use nander_rs::application::StatusUseCase;

// 创建状态用例
let mut status_uc = StatusUseCase::new(Box::new(flash));

// 读取状态寄存器
let status = status_uc.get_status()?;
println!("状态寄存器: {:?}", status);

// 写入状态寄存器 (例如解除保护)
status_uc.set_status(&[0x00])?;
```

## 7. 坏块表管理 (BBT)

```rust
// 扫描整个芯片寻找坏块
let bbt = flash.scan_bbt(|p| println!("扫描: {:.1}%", p.percentage()))?;

println!("找到 {} 个坏块", bbt.bad_block_count());
```
```

## 完整示例

```rust
use nander_rs::application::*;
use nander_rs::domain::OobMode;
use nander_rs::infrastructure::chip_database::ChipRegistry;
use nander_rs::infrastructure::flash_protocol::nand::SpiNand;
use anyhow::Result;

fn main() -> Result<()> {
    // 1. 检测芯片
    let registry = ChipRegistry::new();
    let detect = DetectChipUseCase::new(registry);
    let (programmer, chip) = detect.execute()?;
    
    println!("检测到: {} ({})", chip.name, chip.manufacturer);
    
    // 2. 创建 Flash 协议
    let flash = SpiNand::new(programmer, chip);
    
    // 3. 读取数据
    let mut read_uc = ReadFlashUseCase::new(flash);
    let data = read_uc.execute(
        ReadParams {
            address: 0,
            length: 2048,
            use_ecc: true,
            ignore_ecc_errors: false,
            oob_mode: OobMode::None,
            bad_block_strategy: BadBlockStrategy::Skip,
        },
        |p| println!("读取: {:.0}%", p.percentage())
    )?;
    
    // 4. 处理数据...
    println!("读取了 {} 字节", data.len());
    
    Ok(())
}
```

## OobMode 选项

- `OobMode::None` - 只读取主数据区（默认）
- `OobMode::Included` - 读取主数据+OOB区域
- `OobMode::Only` - 只读取OOB区域

## Progress 回调

所有操作都支持进度回调：

```rust
let data = use_case.execute(params, |progress| {
    let pct = progress.percentage();
    let current = progress.current;
    let total = progress.total;
    println!("进度: {current}/{total} ({pct:.1}%)");
})?;
```

## 错误处理

所有用例返回 `Result<T>`：

```rust
match use_case.execute(params, |_| {}) {
    Ok(data) => println!("成功: {} 字节", data.len()),
    Err(Error::Timeout) => println!("操作超时"),
    Err(Error::WriteFailed { address }) => {
        println!("写入失败: 0x{:08X}", address)
    }
    Err(e) => println!("错误: {}", e),
}
```

## 下一步

- 查看 `src/application/use_cases/` 了解详细实现
- 查看 `src/domain/` 了解核心类型定义
- 查看 `src/infrastructure/flash_protocol/` 了解协议实现
