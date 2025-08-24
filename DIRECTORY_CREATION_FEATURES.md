# 目录创建和询问功能说明

## 概述

Rust Agent 现在支持智能的目录和文件创建功能，当需要创建不存在的父目录时，会询问用户是否同意创建。这提供了更好的用户控制和安全性。

## 新增工具

### 1. CreateDirectoryTool

**功能**: 创建目录，如果父目录不存在会询问是否创建

**用法**: `create_directory("path/to/directory")`

**特性**:
- 支持相对路径和绝对路径
- 自动检测父目录是否存在
- 询问用户是否创建缺失的父目录
- 递归创建整个目录结构

**示例**:
```bash
# 创建简单目录
create_directory("css")

# 创建嵌套目录
create_directory("src/components/ui")
```

### 2. CreateFileTool

**功能**: 创建文件，如果父目录不存在会询问是否创建

**用法**: `create_file("path/to/file.txt", "file content")`

**特性**:
- 支持相对路径和绝对路径
- 自动检测父目录是否存在
- 询问用户是否创建缺失的父目录
- 询问用户是否覆盖已存在的文件
- 递归创建整个目录结构

**示例**:
```bash
# 创建文件
create_file("src/main.rs", "fn main() { println!(\"Hello, world!\"); }")

# 创建嵌套文件
create_file("src/components/Button.tsx", "export const Button = () => <button>Click me</button>")
```

## 增强的现有工具

### WriteFileTool

**增强功能**: 现在在写入文件前会询问用户是否创建缺失的父目录

**行为变化**:
- 之前: 自动创建所有缺失的父目录
- 现在: 询问用户是否创建缺失的父目录

**优势**: 提供更好的用户控制，避免意外创建目录结构

## 用户交互流程

### 目录创建流程

1. **检查目录是否存在**
   - 如果目录已存在，直接返回成功
   - 如果目录不存在，继续下一步

2. **检查父目录**
   - 如果父目录存在，直接创建目标目录
   - 如果父目录不存在，询问用户

3. **用户确认**
   - 显示: `父目录 'path/to/parent' 不存在，是否创建？(Y/N): `
   - 等待用户输入
   - 如果用户输入 "Y" 或 "y"，创建父目录
   - 如果用户输入其他内容，取消操作

4. **创建目录**
   - 创建所有必要的父目录
   - 创建目标目录
   - 返回成功消息

### 文件创建流程

1. **检查文件是否存在**
   - 如果文件已存在，询问是否覆盖
   - 如果文件不存在，继续下一步

2. **检查父目录**
   - 如果父目录存在，直接创建文件
   - 如果父目录不存在，询问用户

3. **用户确认**
   - 显示: `父目录 'path/to/parent' 不存在，是否创建？(Y/N): `
   - 等待用户输入
   - 如果用户输入 "Y" 或 "y"，创建父目录
   - 如果用户输入其他内容，取消操作

4. **创建文件**
   - 创建所有必要的父目录
   - 创建目标文件
   - 写入指定内容
   - 返回成功消息

## 使用示例

### 场景 1: 创建简单目录

```
用户: 请创建一个名为 'css' 的目录

AI: 
💭 Thought: 用户要求创建一个名为 'css' 的目录。我可以使用 create_directory 工具来完成这个任务。
🔧 Action: create_directory("css")
🔍 Observation: 目录创建成功: /path/to/project/css
✅ Final Answer: 已成功创建名为 'css' 的目录。
```

### 场景 2: 创建嵌套目录（需要询问）

```
用户: 请在 css 目录下创建 components 子目录

AI:
💭 Thought: 用户要求在 css 目录下创建 components 子目录。我需要使用 create_directory 工具。
🔧 Action: create_directory("css/components")
父目录 '/path/to/project/css' 不存在，是否创建？(Y/N): y
已创建父目录: /path/to/project/css
🔍 Observation: 目录创建成功: /path/to/project/css/components
✅ Final Answer: 已在 css 目录下成功创建 components 子目录。
```

### 场景 3: 创建嵌套文件（需要询问）

```
用户: 请在 css/components 目录下创建 button.css 文件

AI:
💭 Thought: 用户要求在 css/components 目录下创建 button.css 文件。我需要使用 create_file 工具。
🔧 Action: create_file("css/components/button.css", "/* Button styles */")
父目录 '/path/to/project/css/components' 不存在，是否创建？(Y/N): y
已创建父目录: /path/to/project/css/components
🔍 Observation: 文件创建成功: /path/to/project/css/components/button.css
✅ Final Answer: 已在 css/components 目录下成功创建 button.css 文件。
```

## 安全特性

### 用户控制
- **询问确认**: 每次创建目录前都会询问用户
- **可取消操作**: 用户可以随时取消创建操作
- **透明性**: 清楚显示将要创建的目录路径

### 路径验证
- **相对路径支持**: 自动转换为项目目录下的绝对路径
- **路径安全检查**: 防止创建项目目录外的文件
- **目录存在检查**: 避免重复创建已存在的目录

## 配置选项

### 环境变量
- 支持所有现有的配置选项
- 无需额外配置即可使用新功能

### 项目目录
- 自动检测项目根目录
- 支持相对路径和绝对路径
- 智能路径解析

## 故障排除

### 常见问题

1. **询问没有响应**
   - 确保在正确的终端中运行
   - 检查输入流是否被重定向

2. **目录创建失败**
   - 检查权限设置
   - 确认路径格式正确
   - 验证项目目录存在

3. **文件创建失败**
   - 检查磁盘空间
   - 确认文件路径有效
   - 验证父目录权限

### 调试模式

启用详细日志来诊断问题：
```bash
RUST_LOG=debug cargo run -- /path/to/project
```

## 最佳实践

### 推荐用法

1. **渐进式创建**: 先创建父目录，再创建子目录
2. **路径规划**: 提前规划好目录结构
3. **用户确认**: 对于重要操作，总是确认用户意图

### 避免的问题

1. **深层嵌套**: 避免创建过深的目录结构
2. **权限问题**: 确保有足够的权限创建目录
3. **路径冲突**: 避免创建与现有文件同名的目录

## 总结

新的目录创建和询问功能显著提升了 Rust Agent 的可用性和安全性。通过智能检测缺失目录并询问用户确认，用户可以更好地控制文件系统的操作，同时保持操作的透明性和可预测性。

这些功能特别适用于：
- 项目初始化
- 目录结构重组
- 新功能开发
- 文件系统维护

通过结合流式输出和智能询问，Rust Agent 现在提供了更加友好和安全的文件系统操作体验。
