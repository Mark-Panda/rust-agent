use crate::errors::{AgentError, AgentResult};
use crate::prompt_template::PromptRenderer;
use crate::tools::ToolRegistry;
// 这些类型在当前实现中未使用，但保留以备将来扩展
use async_openai::{
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client,
};
use futures::StreamExt;
use regex::Regex;
use std::env;
use std::io::Write;
use std::path::Path;

pub struct ReActAgent {
    tools: ToolRegistry,
    model: String,
    project_directory: String,
    client: Client<async_openai::config::OpenAIConfig>,
    prompt_renderer: PromptRenderer,
    // 添加对话历史存储
    conversation_history: Vec<ChatCompletionRequestMessage>,
}

impl ReActAgent {
    pub fn new(tools: ToolRegistry, model: String, project_directory: String) -> AgentResult<Self> {
        dotenv::dotenv().ok();

        let api_key = env::var("OPENROUTER_API_KEY")
            .map_err(|_| AgentError::EnvVarError("OPENROUTER_API_KEY".to_string()))?;

        let api_base = env::var("OPENAI_API_BASE")
            .map_err(|_| AgentError::EnvVarError("OPENAI_API_BASE".to_string()))?;

        // 配置OpenAI客户端使用OpenRouter
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(&api_base);

        let client = Client::with_config(config);

        Ok(Self {
            tools,
            model,
            project_directory,
            client,
            prompt_renderer: PromptRenderer::default(),
            conversation_history: vec![],
        })
    }

    pub async fn run(&mut self, user_input: &str) -> AgentResult<String> {
        // 创建当前任务的消息列表，包含系统提示词和用户输入
        let mut current_messages = vec![ChatCompletionRequestMessage::System(
            async_openai::types::ChatCompletionRequestSystemMessage {
                content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text(
                    self.render_system_prompt()?,
                ),
                name: None,
            },
        )];

        // 添加对话历史（除了系统提示词）
        for message in &self.conversation_history {
            if !matches!(message, ChatCompletionRequestMessage::System(_)) {
                current_messages.push(message.clone());
            }
        }

        // 添加当前用户输入
        current_messages.push(ChatCompletionRequestMessage::User(
            async_openai::types::ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    format!("<question>{}</question>", user_input),
                ),
                name: None,
            },
        ));

        let mut retry_count = 0;
        const MAX_RETRIES: usize = 5;

        loop {
            // 检查重试次数是否超过限制
            if retry_count >= MAX_RETRIES {
                return Err(AgentError::RuntimeError(format!(
                    "同一问题调用大模型次数已达上限({}次)，请重新描述问题或检查网络连接",
                    MAX_RETRIES
                )));
            }

            // 请求模型
            let content = self.call_model_stream(&current_messages).await?;

            // 检测 Thought
            if let Some(thought) = self.extract_thought(&content) {
                println!("\n\n💭 Thought: {}", thought);
            }

            // 检测模型是否输出 Final Answer - 优先检查，如果找到立即返回
            if let Some(final_answer) = self.extract_final_answer(&content) {
                println!("\n\n✅ 检测到最终答案，任务完成！");
                println!("📝 最终答案内容: {}", final_answer);

                // 更新对话历史，包含当前对话
                current_messages.push(ChatCompletionRequestMessage::Assistant(
                    async_openai::types::ChatCompletionRequestAssistantMessage {
                        content: Some(
                            async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(
                                content,
                            ),
                        ),
                        name: None,
                        tool_calls: None,
                        function_call: None,
                        audio: None,
                        refusal: None,
                    },
                ));
                self.update_conversation_history(current_messages);

                return Ok(final_answer);
            }

            // 调试信息：显示当前内容状态
            eprintln!("\n\n🔍 内容分析:");
            eprintln!("   - 内容长度: {} 字符", content.chars().count());
            eprintln!(
                "   - 是否包含 <final_answer>: {}",
                content.contains("<final_answer>")
            );
            eprintln!(
                "   - 是否包含 </final_answer>: {}",
                content.contains("</final_answer>")
            );
            eprintln!("   - 是否包含 <action>: {}", content.contains("<action>"));
            eprintln!("   - 是否包含 </action>: {}", content.contains("</action>"));

            // 检测 Action - 只有在没有final_answer的情况下才检查
            let action_result = self.extract_action(&content);
            let action = match action_result {
                Ok(action) => action,
                Err(e) => {
                    retry_count += 1;
                    eprintln!(
                        "\n\n⚠️  模型输出不完整，尝试重新请求... (第{}次重试，最多{}次)",
                        retry_count, MAX_RETRIES
                    );
                    eprintln!("错误详情: {}", e);

                    // 检查是否真的没有final_answer（双重检查）
                    if let Some(final_answer) = self.extract_final_answer(&content) {
                        println!("\n\n✅ 重新检查发现最终答案，任务完成！");

                        // 更新对话历史，包含当前对话
                        current_messages.push(ChatCompletionRequestMessage::Assistant(
                            async_openai::types::ChatCompletionRequestAssistantMessage {
                                content: Some(
                                    async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(
                                        content,
                                    ),
                                ),
                                name: None,
                                tool_calls: None,
                                function_call: None,
                                audio: None,
                                refusal: None,
                            },
                        ));
                        self.update_conversation_history(current_messages);

                        return Ok(final_answer);
                    }

                    // 添加一个提示消息，要求模型重新输出
                    current_messages.push(ChatCompletionRequestMessage::User(
                        async_openai::types::ChatCompletionRequestUserMessage {
                            content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                                format!("请重新输出完整的action标签，格式为 <action>工具名(参数)</action>。这是第{}次重试。", retry_count),
                            ),
                            name: None,
                        },
                    ));

                    // 重新请求
                    continue;
                }
            };

            let (tool_name, args) = self.parse_action(&action)?;

            println!("\n\n🔧 Action: {}({})", tool_name, args.join(", "));

            // 只有终端命令才需要询问用户
            if tool_name == "run_terminal_command" {
                let mut input = String::new();
                println!("\n\n是否继续？（Y/N）");
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "y" {
                    println!("\n\n操作已取消。");
                    return Ok("操作被用户取消".to_string());
                }
            }

            // 执行工具
            let observation = match self.tools.get_tool(&tool_name) {
                Some(tool) => tool.execute(args).await?,
                None => format!("工具 '{}' 不存在", tool_name),
            };

            println!("\n\n🔍 Observation：{}", observation);

            // 添加观察结果到消息列表
            #[allow(deprecated)]
            current_messages.push(ChatCompletionRequestMessage::Assistant(
                async_openai::types::ChatCompletionRequestAssistantMessage {
                    content: Some(
                        async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(
                            content,
                        ),
                    ),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                    audio: None,
                    refusal: None,
                },
            ));
            current_messages.push(ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        format!("<observation>{}</observation>", observation),
                    ),
                    name: None,
                },
            ));

            // 更新对话历史
            self.update_conversation_history(current_messages.clone());
        }
    }

    // 添加一个方法来更新对话历史
    fn update_conversation_history(&mut self, messages: Vec<ChatCompletionRequestMessage>) {
        // 过滤掉系统提示词，只保留对话内容
        self.conversation_history = messages
            .into_iter()
            .filter(|msg| !matches!(msg, ChatCompletionRequestMessage::System(_)))
            .collect();
    }

    // 添加一个方法来获取对话历史长度
    pub fn get_conversation_length(&self) -> usize {
        self.conversation_history.len()
    }

    // 添加一个方法来清除对话历史
    pub fn clear_conversation_history(&mut self) {
        self.conversation_history.clear();
    }

    fn render_system_prompt(&self) -> AgentResult<String> {
        let tool_list = self.tools.get_tool_list();
        let operating_system = self.get_operating_system_name();
        let file_list = self.get_file_list()?;

        Ok(self
            .prompt_renderer
            .render(&tool_list, &operating_system, &file_list))
    }

    async fn call_model_stream(
        &self,
        messages: &[ChatCompletionRequestMessage],
    ) -> AgentResult<String> {
        println!("\n\n正在请求模型，请稍等...");

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages.to_vec())
            .stream(true)
            .build()
            .map_err(|e| AgentError::RuntimeError(format!("构建请求失败: {}", e)))?;

        let mut stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(|e| AgentError::RuntimeError(format!("API调用失败: {}", e)))?;

        let mut content = String::new();
        let mut buffer = String::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        let delta = &choice.delta;
                        if let Some(text) = &delta.content {
                            // 流式输出文本
                            print!("{}", text);
                            std::io::stdout().flush().map_err(|e| {
                                AgentError::RuntimeError(format!("输出刷新失败: {}", e))
                            })?;

                            content.push_str(text);
                            buffer.push_str(text);

                            // 检测是否包含完整的标签
                            if self.should_process_buffer(&buffer) {
                                // 如果缓冲区包含完整的标签，处理它
                                if let Some(thought) = self.extract_thought(&buffer) {
                                    println!("\n\n💭 Thought: {}", thought);
                                    buffer.clear();
                                }

                                // 如果检测到完整的 action 标签，等待更多内容确保完整性
                                if buffer.contains("</action>") {
                                    // 等待一小段时间，确保内容完整
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await;

                                    // 检查是否已经获得了完整的action内容
                                    if self.has_complete_action(&content) {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n\n流式输出错误: {}", e);
                    break;
                }
            }
        }

        println!(); // 换行

        // 调试信息：显示最终内容
        if !self.has_complete_action(&content) {
            eprintln!("\n\n⚠️  警告：模型输出可能不完整，内容：{}", content);
        }

        Ok(content)
    }

    fn should_process_buffer(&self, buffer: &str) -> bool {
        // 检查缓冲区是否包含完整的标签
        // 使用更智能的检测逻辑
        let has_complete_thought = buffer.contains("<thought>") && buffer.contains("</thought>");
        let has_complete_action = buffer.contains("<action>") && buffer.contains("</action>");
        let has_complete_final_answer =
            buffer.contains("<final_answer>") && buffer.contains("</final_answer>");

        has_complete_thought || has_complete_action || has_complete_final_answer
    }

    fn extract_thought(&self, content: &str) -> Option<String> {
        let re = Regex::new(r"<thought>(.*?)</thought>").ok()?;
        re.captures(content).map(|cap| cap[1].trim().to_string())
    }

    fn extract_final_answer(&self, content: &str) -> Option<String> {
        let re = Regex::new(r"<final_answer>(.*?)</final_answer>").ok()?;
        re.captures(content).map(|cap| cap[1].trim().to_string())
    }

    fn extract_action(&self, content: &str) -> AgentResult<String> {
        // 调试信息：显示正在解析的内容
        eprintln!(
            "\n\n🔍 正在解析内容中的action标签，内容长度: {}",
            content.len()
        );

        // 使用chars()来正确处理UTF-8字符边界
        let char_count = content.chars().count();
        let preview_length = if char_count > 100 { 100 } else { char_count };
        let preview: String = content.chars().take(preview_length).collect();

        eprintln!("🔍 内容预览 (前{}个字符): {}", preview_length, preview);

        let re = Regex::new(r"<action>(.*?)</action>")
            .map_err(|_| AgentError::ParseError("无法编译正则表达式".to_string()))?;

        match re.captures(content) {
            Some(cap) => {
                let action = cap[1].trim().to_string();
                eprintln!("✅ 成功提取action: {}", action);
                Ok(action)
            }
            None => {
                // 尝试查找部分匹配的标签
                let action_start = content.find("<action>");
                let action_end = content.find("</action>");

                eprintln!("❌ 未找到完整的action标签");
                eprintln!("   <action> 位置: {:?}", action_start);
                eprintln!("   </action> 位置: {:?}", action_end);

                if let Some(start) = action_start {
                    // 安全地处理字符串切片，避免在UTF-8字符中间截断
                    let remaining = if start < content.len() {
                        // 将字节索引转换为字符索引
                        let char_indices: Vec<(usize, char)> = content.char_indices().collect();
                        let char_start = char_indices
                            .iter()
                            .position(|(byte_pos, _)| *byte_pos >= start)
                            .unwrap_or(0);

                        if char_start < char_indices.len() {
                            let (byte_pos, _) = char_indices[char_start];
                            // 安全地截取字符串，避免在UTF-8字符中间截断
                            if byte_pos < content.len() {
                                let chars: Vec<char> = content.chars().collect();
                                let char_start = content
                                    .char_indices()
                                    .position(|(pos, _)| pos >= byte_pos)
                                    .unwrap_or(0);
                                if char_start < chars.len() {
                                    // 安全地截取字符串，避免在UTF-8字符中间截断
                                    let remaining_chars: String =
                                        chars.into_iter().skip(char_start).collect();
                                    remaining_chars
                                } else {
                                    "".to_string()
                                }
                            } else {
                                "".to_string()
                            }
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    };

                    // 安全地截取前100个字符
                    let remaining_chars: Vec<char> = remaining.chars().collect();
                    let safe_length = if remaining_chars.len() > 100 {
                        100
                    } else {
                        remaining_chars.len()
                    };
                    let safe_preview: String =
                        remaining_chars.into_iter().take(safe_length).collect();
                    eprintln!("   从 <action> 开始的内容: {}", safe_preview);
                }

                Err(AgentError::ParseError(format!(
                    "模型未输出 <action> 标签。内容长度: {} 字符, 是否包含开始标签: {}, 是否包含结束标签: {}",
                    char_count,
                    action_start.is_some(),
                    action_end.is_some()
                )))
            }
        }
    }

    fn parse_action(&self, action_str: &str) -> AgentResult<(String, Vec<String>)> {
        let re = Regex::new(r"(\w+)\((.*)\)")
            .map_err(|_| AgentError::ParseError("无法编译函数调用正则表达式".to_string()))?;

        let captures = re
            .captures(action_str)
            .ok_or_else(|| AgentError::ParseError("无效的函数调用语法".to_string()))?;

        let func_name = captures[1].to_string();
        let args_str = captures[2].trim();

        let args = self.parse_arguments(args_str)?;

        Ok((func_name, args))
    }

    fn parse_arguments(&self, args_str: &str) -> AgentResult<Vec<String>> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_string = false;
        let mut string_char = None;
        let mut paren_depth = 0;
        let mut i = 0;
        let chars: Vec<char> = args_str.chars().collect();

        while i < chars.len() {
            let char = chars[i];

            if !in_string {
                if char == '"' || char == '\'' {
                    in_string = true;
                    string_char = Some(char);
                    current_arg.push(char);
                } else if char == '(' {
                    paren_depth += 1;
                    current_arg.push(char);
                } else if char == ')' {
                    paren_depth -= 1;
                    current_arg.push(char);
                } else if char == ',' && paren_depth == 0 {
                    // 遇到顶层逗号，结束当前参数
                    args.push(self.parse_single_arg(&current_arg.trim())?);
                    current_arg.clear();
                } else {
                    current_arg.push(char);
                }
            } else {
                current_arg.push(char);
                if char == string_char.unwrap() && (i == 0 || chars[i - 1] != '\\') {
                    in_string = false;
                    string_char = None;
                }
            }

            i += 1;
        }

        // 添加最后一个参数
        if !current_arg.trim().is_empty() {
            args.push(self.parse_single_arg(&current_arg.trim())?);
        }

        Ok(args)
    }

    fn parse_single_arg(&self, arg_str: &str) -> AgentResult<String> {
        let arg_str = arg_str.trim();

        // 如果是字符串字面量
        if (arg_str.starts_with('"') && arg_str.ends_with('"'))
            || (arg_str.starts_with('\'') && arg_str.ends_with('\''))
        {
            // 安全地处理字符串切片，避免在UTF-8字符中间截断
            let chars: Vec<char> = arg_str.chars().collect();
            if chars.len() >= 2 {
                let inner_chars: String = chars[1..chars.len() - 1].iter().collect();
                // 处理常见的转义字符
                let processed = inner_chars
                    .replace("\\\"", "\"")
                    .replace("\\'", "'")
                    .replace("\\n", "\n")
                    .replace("\\t", "\t")
                    .replace("\\r", "\r")
                    .replace("\\\\", "\\");
                Ok(processed)
            } else {
                Ok("".to_string())
            }
        } else {
            // 如果不是字符串，直接返回
            Ok(arg_str.to_string())
        }
    }

    fn get_file_list(&self) -> AgentResult<String> {
        let path = Path::new(&self.project_directory);
        if !path.exists() || !path.is_dir() {
            return Err(AgentError::RuntimeError(
                "项目目录不存在或不是目录".to_string(),
            ));
        }

        let entries = std::fs::read_dir(path)?;
        let file_names: Vec<String> = entries
            .filter_map(|entry| {
                entry.ok().map(|e| {
                    e.path()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                })
            })
            .collect();

        Ok(file_names.join(", "))
    }

    fn get_operating_system_name(&self) -> String {
        match std::env::consts::OS {
            "macos" => "macOS".to_string(),
            "windows" => "Windows".to_string(),
            "linux" => "Linux".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn has_complete_action(&self, content: &str) -> bool {
        // 检查是否有完整的 <action>...</action> 标签对
        let action_start = content.find("<action>");
        let action_end = content.find("</action>");

        if let (Some(start), Some(end)) = (action_start, action_end) {
            // 确保结束标签在开始标签之后
            end > start
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_complete_action() {
        let agent = ReActAgent::new(
            ToolRegistry::new(),
            "test-model".to_string(),
            "/tmp".to_string(),
        )
        .unwrap();

        // 测试完整的action标签
        assert!(agent.has_complete_action("<action>read_file(\"test.txt\")</action>"));

        // 测试不完整的action标签
        assert!(!agent.has_complete_action("<action>read_file(\"test.txt\")"));
        assert!(!agent.has_complete_action("read_file(\"test.txt\")</action>"));
        assert!(!agent.has_complete_action(""));

        // 测试标签顺序错误
        assert!(!agent.has_complete_action("</action>read_file(\"test.txt\")<action>"));
    }

    #[test]
    fn test_extract_action() {
        let agent = ReActAgent::new(
            ToolRegistry::new(),
            "test-model".to_string(),
            "/tmp".to_string(),
        )
        .unwrap();

        // 测试成功提取
        let result = agent.extract_action("<action>read_file(\"test.txt\")</action>");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "read_file(\"test.txt\")");

        // 测试失败提取
        let result = agent.extract_action("没有action标签");
        assert!(result.is_err());
    }
}
