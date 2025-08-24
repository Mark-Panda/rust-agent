# Rust Agent 项目

这是一个独立的Rust项目，实现了基于ReAct模式的智能Agent系统。

## 🚀 快速开始

### 1. 进入项目目录
```bash
cd rust-agent
```

### 2. 设置环境变量
```bash
cp env.example .env
# 编辑 .env 文件设置你的 OPENROUTER_API_KEY
```

### 3. 构建项目
```bash
./build.sh
```

### 4. 运行Agent
```bash
./run.sh /path/to/your/project
```

## 📁 项目结构

```
rust-agent/
├── Cargo.toml           # Rust项目配置
├── src/                 # 源代码目录
│   ├── main.rs          # 主程序入口
│   ├── lib.rs           # 库入口
│   ├── agent.rs         # Agent核心实现
│   ├── tools.rs         # 工具系统
│   ├── prompt_template.rs # 提示词模板
│   ├── types.rs         # 类型定义
│   ├── errors.rs        # 错误处理
│   └── tests.rs         # 测试模块
├── build.sh             # 构建脚本
├── run.sh               # 运行脚本
├── env.example          # 环境变量示例
└── README.md            # 详细说明文档
```

## 🔧 开发命令

```bash
# 开发模式构建
cargo build

# 发布模式构建
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo check

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

## 📦 依赖管理

所有依赖都在 `Cargo.toml` 中定义，使用 `cargo` 进行管理：

```bash
# 添加新依赖
cargo add package_name

# 更新依赖
cargo update

# 查看依赖树
cargo tree
```

## 🎯 这是一个完整的Rust项目

这个文件夹包含了完整的Rust项目，可以独立运行和开发。所有必要的文件都已经包含在内，包括：

- 完整的源代码结构
- 项目配置文件
- 构建和运行脚本
- 环境配置示例
- 详细的使用文档

你可以将这个文件夹复制到任何地方，作为一个独立的Rust项目使用。
