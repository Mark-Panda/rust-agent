use crate::errors::{AgentError, AgentResult};
use crate::prompt_template::PromptRenderer;
use crate::tools::ToolRegistry;
// 这些类型在当前实现中未使用，但保留以备将来扩展
use async_openai::{
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
    Client,
};
use regex::Regex;
use std::env;
use std::path::Path;

pub struct ReActAgent {
    tools: ToolRegistry,
    model: String,
    project_directory: String,
    client: Client<async_openai::config::OpenAIConfig>,
    prompt_renderer: PromptRenderer,
}

impl ReActAgent {
    pub fn new(tools: ToolRegistry, model: String, project_directory: String) -> AgentResult<Self> {
        dotenv::dotenv().ok();

        let api_key = env::var("OPENROUTER_API_KEY")
            .map_err(|_| AgentError::EnvVarError("OPENROUTER_API_KEY".to_string()))?;

        // 配置OpenAI客户端使用OpenRouter
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://ark.cn-beijing.volces.com/api/v3");

        let client = Client::with_config(config);

        Ok(Self {
            tools,
            model,
            project_directory,
            client,
            prompt_renderer: PromptRenderer::default(),
        })
    }

    pub async fn run(&self, user_input: &str) -> AgentResult<String> {
        let mut messages = vec![
            ChatCompletionRequestMessage::System(
                async_openai::types::ChatCompletionRequestSystemMessage {
                    content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text(
                        self.render_system_prompt()?,
                    ),
                    name: None,
                },
            ),
            ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        format!("<question>{}</question>", user_input),
                    ),
                    name: None,
                },
            ),
        ];

        loop {
            // 请求模型
            let content = self.call_model(&messages).await?;

            // 检测 Thought
            if let Some(thought) = self.extract_thought(&content) {
                println!("\n\n💭 Thought: {}", thought);
            }

            // 检测模型是否输出 Final Answer
            if let Some(final_answer) = self.extract_final_answer(&content) {
                return Ok(final_answer);
            }

            // 检测 Action
            let action = self.extract_action(&content)?;
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
            messages.push(ChatCompletionRequestMessage::Assistant(
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
            messages.push(ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        format!("<observation>{}</observation>", observation),
                    ),
                    name: None,
                },
            ));
        }
    }

    fn render_system_prompt(&self) -> AgentResult<String> {
        let tool_list = self.tools.get_tool_list();
        let operating_system = self.get_operating_system_name();
        let file_list = self.get_file_list()?;

        Ok(self
            .prompt_renderer
            .render(&tool_list, &operating_system, &file_list))
    }

    async fn call_model(&self, messages: &[ChatCompletionRequestMessage]) -> AgentResult<String> {
        println!("\n\n正在请求模型，请稍等...");

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages.to_vec())
            .build()
            .map_err(|e| AgentError::RuntimeError(format!("构建请求失败: {}", e)))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| AgentError::RuntimeError(format!("API调用失败: {}", e)))?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| AgentError::RuntimeError("响应中没有内容".to_string()))?;

        Ok(content.clone())
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
        let re = Regex::new(r"<action>(.*?)</action>")
            .map_err(|_| AgentError::ParseError("无法编译正则表达式".to_string()))?;

        re.captures(content)
            .map(|cap| cap[1].trim().to_string())
            .ok_or_else(|| AgentError::ParseError("模型未输出 <action>".to_string()))
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
            let inner_str = &arg_str[1..arg_str.len() - 1];
            // 处理常见的转义字符
            let processed = inner_str
                .replace("\\\"", "\"")
                .replace("\\'", "'")
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\r", "\r")
                .replace("\\\\", "\\");
            Ok(processed)
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
}
