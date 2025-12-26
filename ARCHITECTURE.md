# nander-rs 架构设计文档

## 架构原则

本项目采用**分层架构** (Layered Architecture) + **领域驱动设计** (Domain-Driven Design) 的混合模式。

### 核心原则

1. **依赖倒置** - 高层模块不依赖低层模块，都依赖抽象
2. **单一职责** - 每个模块只负责一个明确的功能域
3. **开放封闭** - 对扩展开放，对修改封闭
4. **接口隔离** - 使用 trait 定义清晰的接口边界

## 架构分层

```
┌─────────────────────────────────────┐
│      Presentation Layer             │  CLI/TUI/GUI 适配器
│      (presentation/)                │
├─────────────────────────────────────┤
│      Application Layer              │  用例编排
│      (application/)                 │  业务流程控制
├─────────────────────────────────────┤
│      Domain Layer                   │  核心业务逻辑
│      (domain/)                      │  领域模型和规则
├─────────────────────────────────────┤
│      Infrastructure Layer           │  技术实现
│      (infrastructure/)              │  硬件驱动、数据库
└─────────────────────────────────────┘
```

## 层次职责

### 1. Domain Layer (领域层)

**路径**: `src/domain/`

**职责**: 定义核心业务概念和规则，不依赖任何外部技术

**模块**:
- `chip.rs` - 芯片模型（ChipSpec, ChipCapability）
- `flash_operation.rs` - Flash 操作抽象接口
- `bad_block.rs` - 坏块管理策略（BadBlockStrategy）
- `ecc.rs` - ECC 控制策略（EccPolicy）
- `error.rs` - 领域错误定义

**示例**:
```rust
pub struct ChipSpec {
    pub id: ChipId,
    pub name: &'static str,
    pub manufacturer: Manufacturer,
    pub capacity: Capacity,
    pub layout: ChipLayout,
}

pub trait FlashOperation {
    fn read(&mut self, request: ReadRequest) -> Result<Vec<u8>>;
    fn write(&mut self, request: WriteRequest) -> Result<()>;
    fn erase(&mut self, request: EraseRequest) -> Result<()>;
}
```

### 2. Application Layer (应用层)

**路径**: `src/application/`

**职责**: 编排业务用例，协调领域对象完成具体任务

**模块**:
- `use_cases/` - 具体用例实现
  - `read_flash.rs` - 读取用例（ReadFlashUseCase）
  - `write_flash.rs` - 写入用例（WriteFlashUseCase）
  - `erase_flash.rs` - 擦除用例
  - `verify_flash.rs` - 验证用例
  - `detect_chip.rs` - 芯片检测用例
- `services/` - 应用服务
  - `chip_detector.rs` - 芯片检测服务
  - `progress_tracker.rs` - 进度跟踪服务

**示例**:
```rust
pub struct ReadFlashUseCase<P: Programmer> {
    programmer: P,
    chip_db: Arc<ChipDatabase>,
    bad_block_handler: Box<dyn BadBlockHandler>,
}

impl<P: Programmer> ReadFlashUseCase<P> {
    pub fn execute(&mut self, params: ReadParams) -> Result<Vec<u8>> {
        // 1. 检测芯片
        // 2. 应用 ECC 策略
        // 3. 处理坏块
        // 4. 执行读取
        // 5. 报告进度
    }
}
```

### 3. Infrastructure Layer (基础设施层)

**路径**: `src/infrastructure/`

**职责**: 提供技术实现，硬件驱动，数据存储

**模块**:
- `programmer/` - 硬件编程器实现
  - `traits.rs` - Programmer trait 定义
  - `ch341a/` - CH341A 驱动实现
  - `discovery.rs` - 硬件自动发现
- `flash_protocol/` - Flash 协议实现
  - `nand/` - SPI NAND 协议
  - `nor/` - SPI NOR 协议
  - `commands.rs` - SPI 命令常量
- `chip_database/` - 芯片数据库
  - `registry.rs` - 芯片注册表
  - `nand/` - NAND 芯片定义（按制造商分类）
  - `nor/` - NOR 芯片定义（按制造商分类）
  - `loader.rs` - 动态加载器

**示例**:
```rust
// programmer/traits.rs
pub trait Programmer {
    fn name(&self) -> &str;
    fn spi_transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()>;
    fn set_cs(&mut self, active: bool) -> Result<()>;
}

// chip_database/registry.rs
pub struct ChipDatabase {
    chips: Vec<ChipSpec>,
}

impl ChipDatabase {
    pub fn find_by_jedec(&self, id: &[u8; 3]) -> Option<&ChipSpec> { }
    pub fn list_by_manufacturer(&self, mfr: Manufacturer) -> Vec<&ChipSpec> { }
}
```

### 4. Presentation Layer (表现层)

**路径**: `src/presentation/`

**职责**: 用户交互，将用户输入转换为应用层调用

**模块**:
- `cli/` - 命令行接口
  - `args.rs` - 参数解析
  - `handlers/` - 命令处理器（每个命令一个文件）
    - `info_handler.rs`
    - `read_handler.rs`
    - `write_handler.rs`
  - `output/` - 输出格式化
    - `progress.rs` - 进度条
    - `formatter.rs` - 输出格式化

**示例**:
```rust
pub struct ReadHandler {
    use_case: ReadFlashUseCase<Box<dyn Programmer>>,
}

impl CommandHandler for ReadHandler {
    fn handle(&mut self, args: ReadArgs) -> Result<()> {
        let params = ReadParams::from_args(args)?;
        let data = self.use_case.execute(params)?;
        // 输出结果
    }
}
```

## 目录结构

```
src/
├── main.rs                         # 程序入口
├── lib.rs                          # 库导出
│
├── domain/                         # 领域层
│   ├── mod.rs
│   ├── chip.rs
│   ├── flash_operation.rs
│   ├── bad_block.rs
│   ├── ecc.rs
│   └── types.rs
│
├── application/                    # 应用层
│   ├── mod.rs
│   ├── use_cases/
│   │   ├── mod.rs
│   │   ├── read_flash.rs
│   │   ├── write_flash.rs
│   │   ├── erase_flash.rs
│   │   ├── verify_flash.rs
│   │   └── detect_chip.rs
│   └── services/
│       ├── mod.rs
│       ├── chip_detector.rs
│       └── progress_tracker.rs
│
├── infrastructure/                 # 基础设施层
│   ├── mod.rs
│   ├── programmer/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   ├── ch341a/
│   │   │   ├── mod.rs
│   │   │   ├── usb.rs
│   │   │   └── protocol.rs
│   │   └── discovery.rs
│   ├── flash_protocol/
│   │   ├── mod.rs
│   │   ├── commands.rs
│   │   ├── nand/
│   │   │   ├── mod.rs
│   │   │   ├── protocol.rs
│   │   │   └── ecc_handler.rs
│   │   └── nor/
│   │       ├── mod.rs
│   │       └── protocol.rs
│   └── chip_database/
│       ├── mod.rs
│       ├── registry.rs
│       ├── loader.rs
│       ├── nand/
│       │   ├── mod.rs
│       │   ├── gigadevice.rs
│       │   ├── winbond.rs
│       │   ├── macronix.rs
│       │   ├── micron.rs
│       │   └── xtx.rs
│       └── nor/
│           ├── mod.rs
│           ├── gigadevice.rs
│           ├── winbond.rs
│           └── macronix.rs
│
├── presentation/                   # 表现层
│   ├── mod.rs
│   └── cli/
│       ├── mod.rs
│       ├── args.rs
│       ├── handlers/
│       │   ├── mod.rs
│       │   ├── info_handler.rs
│       │   ├── list_handler.rs
│       │   ├── read_handler.rs
│       │   ├── write_handler.rs
│       │   ├── erase_handler.rs
│       │   └── verify_handler.rs
│       └── output/
│           ├── mod.rs
│           ├── progress.rs
│           └── formatter.rs
│
├── config/                         # 配置
│   ├── mod.rs
│   └── constants.rs
│
└── error.rs                        # 全局错误类型

```

## 依赖关系

```
presentation → application → domain
                    ↓
              infrastructure
```

- **Presentation** 依赖 **Application** 和 **Infrastructure**
- **Application** 依赖 **Domain** 和 **Infrastructure**
- **Domain** 不依赖任何其他层（纯业务逻辑）
- **Infrastructure** 实现 **Domain** 定义的接口

## 扩展指南

### 添加新的硬件编程器

1. 在 `infrastructure/programmer/` 创建新文件夹
2. 实现 `Programmer` trait
3. 在 `discovery.rs` 中添加检测逻辑

### 添加新的芯片

1. 在 `infrastructure/chip_database/nand/` 或 `nor/` 对应制造商文件中添加
2. 如果是新制造商，创建新文件
3. 芯片会在启动时自动加载

### 添加新的命令

1. 在 `application/use_cases/` 创建用例
2. 在 `presentation/cli/handlers/` 创建处理器
3. 在 `args.rs` 添加命令参数定义

## 测试策略

- **单元测试**: 每层独立测试
  - Domain: 纯逻辑测试，无 mock
  - Application: 使用 mock Programmer
  - Infrastructure: 使用真实硬件或 mock USB
  - Presentation: 测试参数解析和格式化

- **集成测试**: 跨层测试
  - 端到端测试完整用例

## 重构路线图

### Phase 1: 建立新架构骨架 ✅ 已完成
- [x] 创建新的目录结构
- [x] 定义领域层核心类型 (`domain/types.rs`)
- [x] 定义芯片规格模型 (`domain/chip.rs`)
- [x] 定义 Flash 操作 trait (`domain/flash_operation.rs`)
- [x] 定义坏块管理策略 (`domain/bad_block.rs`)
- [x] 定义 ECC 策略 (`domain/ecc.rs`)
- [x] 创建基础设施层结构
- [x] 迁移 Programmer trait 到新位置
- [x] 创建芯片数据库按制造商分类结构

### Phase 2: 协议与数据库迁移 ✅ 已完成
- [x] 创建 `flash_protocol/nand` 基础框架
- [x] 创建 `flash_protocol/nor` 基础框架
- [x] 创建芯片数据库分类结构 (NAND/NOR)
- [x] 迁移完整的 NAND 读/写/擦除逻辑
- [x] 迁移完整的 NOR 读/写/擦除逻辑
- [x] 实现 OobMode 支持（None/Included/Only）
- [x] 实现 ECC 控制（enable/disable）
- [x] 实现进度报告回调
- [ ] 从旧版芯片数据库迁移所有芯片定义

### Phase 3: 应用层与表现层 🔶 进行中
- [ ] 实现 `application/use_cases/detect_chip.rs`
- [ ] 实现 `application/use_cases/read_flash.rs`
- [ ] 实现 `application/use_cases/write_flash.rs`
- [ ] 实现 `application/use_cases/erase_flash.rs`
- [ ] 迁移 CLI 处理器到 `presentation/cli/handlers/`
- [ ] 移除遗留模块


