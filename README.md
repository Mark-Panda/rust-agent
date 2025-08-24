# Rust Agent - ReAct Agent的Rust实现

这是一个用Rust重新实现的ReAct Agent项目，提供了与Python版本相同的功能，但具有更好的性能和类型安全。

## 功能特性

- 🚀 **高性能**: 使用Rust的零成本抽象和内存安全
- 🔧 **工具系统**: 支持文件读写、终端命令执行等工具
- 🤖 **AI集成**: 集成OpenAI API进行智能推理
- 📝 **类型安全**: 完整的Rust类型系统保证代码质量
- 🔄 **异步支持**: 使用tokio进行异步操作

## 系统要求

- Rust 1.70+ 
- Cargo包管理器
- 网络连接（用于API调用）

## 安装和运行

### 1. 安装Rust

如果还没有安装Rust，请访问 [rustup.rs](https://rustup.rs/) 安装：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 克隆项目

```bash
git clone <your-repo-url>
cd Agent的概念、原理与构建模式
```

### 3. 配置环境变量

创建 `.env` 文件并设置你的OpenRouter API密钥：

```bash
echo "OPENROUTER_API_KEY=your_api_key_here" > .env
```

### 4. 构建项目

```bash
cargo build --release
```

### 5. 运行Agent

```bash
cargo run --release -- /path/to/your/project
```

或者使用编译后的二进制文件：

```bash
./target/release/rust-agent /path/to/your/project
```

## 项目结构

```
src/
├── main.rs          # 主程序入口
├── lib.rs           # 库入口
├── agent.rs         # ReAct Agent核心实现
├── tools.rs         # 工具系统
├── prompt_template.rs # 提示词模板
├── types.rs         # 类型定义
└── errors.rs        # 错误处理
```

## 核心组件

### ReActAgent

主要的Agent类，负责：
- 管理对话历史
- 解析AI模型的输出
- 执行工具调用
- 处理ReAct循环

### Tool System

工具系统包含：
- `ReadFileTool`: 读取文件内容
- `WriteFileTool`: 写入文件内容
- `RunTerminalCommandTool`: 执行终端命令

### Prompt Template

使用XML标签格式的提示词模板，确保AI模型按照ReAct模式工作。

## 使用示例

```bash
# 启动Agent并指定项目目录
cargo run --release -- /Users/username/my_project

# 输入任务
请输入任务：帮我查看项目中的README文件内容

# Agent会自动执行任务
💭 Thought: 用户想要查看项目中的README文件内容，我需要使用read_file工具来读取文件。
🔧 Action: read_file("/Users/username/my_project/README.md")
🔍 Observation：这是一个示例项目的README文件...
✅ Final Answer：README文件内容如下：这是一个示例项目的README文件...
```

## 配置选项

### 环境变量

- `OPENROUTER_API_KEY`: 你的OpenRouter API密钥
- `RUST_LOG`: 日志级别（可选，默认为info）

### 模型配置

在 `main.rs` 中可以修改使用的AI模型：

```rust
let agent = ReActAgent::new(
    tools,
    "openai/gpt-4o".to_string(), // 修改这里使用不同的模型
    project_dir.to_string_lossy().to_string(),
)?;
```

## 扩展工具

要添加新的工具，实现 `Tool` trait：

```rust
use async_trait::async_trait;
use crate::tools::Tool;
use crate::errors::AgentResult;

pub struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn name(&self) -> &str {
        "my_custom_tool"
    }

    fn description(&self) -> &str {
        "我的自定义工具描述"
    }

    async fn execute(&self, args: Vec<String>) -> AgentResult<String> {
        // 工具实现逻辑
        Ok("执行结果".to_string())
    }
}
```

然后在 `create_default_tools()` 函数中注册：

```rust
pub fn create_default_tools() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(ReadFileTool);
    registry.register(WriteFileTool);
    registry.register(RunTerminalCommandTool);
    registry.register(MyCustomTool); // 添加新工具
    registry
}
```

## 性能优化

- 使用 `--release` 标志进行优化编译
- 异步I/O操作提高并发性能
- 内存安全的零拷贝操作

## 故障排除

### 常见问题

1. **API密钥错误**: 确保 `.env` 文件中的 `OPENROUTER_API_KEY` 正确设置
2. **网络连接问题**: 检查网络连接和防火墙设置
3. **权限问题**: 确保有足够的权限访问项目目录和执行命令

### 调试模式

启用详细日志：

```bash
RUST_LOG=debug cargo run -- /path/to/project
```

## 贡献

欢迎提交Issue和Pull Request来改进这个项目！

## 许可证

本项目采用MIT许可证。
