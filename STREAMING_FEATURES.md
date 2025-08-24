# 流式输出功能说明

## 概述

Rust Agent 现在支持流式输出，这意味着AI模型的响应会实时显示，而不是等待完整响应后一次性显示。这提供了更好的用户体验和交互性。

## 主要特性

### 1. 实时输出
- AI模型的每个token都会立即显示
- 用户可以实时看到AI的思考过程
- 无需等待完整响应

### 2. 智能标签检测
- 自动检测 `<thought>`, `<action>`, `<final_answer>` 等标签
- 当检测到完整标签时，会立即处理和显示
- 支持缓冲区管理，确保标签完整性

### 3. 流式处理
- 使用 `async-openai` 的流式API
- 支持 `stream(true)` 参数
- 异步处理每个响应块

## 技术实现

### 核心方法
```rust
async fn call_model_stream(&self, messages: &[ChatCompletionRequestMessage]) -> AgentResult<String>
```

### 流式处理流程
1. 创建流式请求 (`stream(true)`)
2. 使用 `create_stream()` 方法
3. 逐块处理响应 (`StreamExt::next()`)
4. 实时输出文本内容
5. 检测完整标签并处理

### 缓冲区管理
```rust
fn should_process_buffer(&self, buffer: &str) -> bool {
    buffer.contains("</thought>") || 
    buffer.contains("</action>") || 
    buffer.contains("</final_answer>")
}
```

## 使用示例

### 基本用法
```bash
cargo run -- /path/to/project
# 输入任务后，AI会实时显示思考过程
```

### 输出示例
```
正在请求模型，请稍等...
<thought>用户想要了解项目结构，我需要使用list_files工具来查看目录内容。</thought>

💭 Thought: 用户想要了解项目结构，我需要使用list_files工具来查看目录内容。
<action>list_files(".")</action>

🔧 Action: list_files(.)
🔍 Observation：项目包含以下文件：src/, Cargo.toml, README.md...
<final_answer>项目结构如下：src/目录包含源代码，Cargo.toml是项目配置文件...</final_answer>

✅ Final Answer：项目结构如下：src/目录包含源代码，Cargo.toml是项目配置文件...
```

## 优势

### 用户体验
- **即时反馈**: 用户立即看到AI开始工作
- **透明性**: 可以观察AI的思考过程
- **交互性**: 更好的参与感

### 技术优势
- **响应性**: 减少感知延迟
- **可观察性**: 便于调试和监控
- **扩展性**: 为未来功能（如中断、实时编辑）奠定基础

## 配置选项

### 环境变量
- `OPENROUTER_API_KEY`: API密钥（必需）
- 支持所有现有的配置选项

### 模型兼容性
- 支持所有支持流式输出的模型
- 自动回退到非流式模式（如果模型不支持）

## 未来扩展

### 计划功能
- [ ] 支持用户中断流式输出
- [ ] 实时编辑和修改
- [ ] 流式工具调用
- [ ] 多模型并行流式输出

### 性能优化
- [ ] 智能缓冲区大小调整
- [ ] 流式输出的内存优化
- [ ] 并发流式处理

## 故障排除

### 常见问题
1. **流式输出中断**: 检查网络连接和API配额
2. **标签检测问题**: 确保模型输出格式正确
3. **性能问题**: 考虑调整缓冲区大小

### 调试模式
启用详细日志来诊断流式输出问题：
```bash
RUST_LOG=debug cargo run -- /path/to/project
```

## 总结

流式输出功能显著提升了Rust Agent的用户体验，使AI助手更加智能和友好。通过实时显示思考过程，用户可以更好地理解AI的决策逻辑，并享受更流畅的交互体验。
