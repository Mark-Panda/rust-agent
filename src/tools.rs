use crate::errors::AgentResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: Vec<String>) -> AgentResult<String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn get_tool_list(&self) -> String {
        self.tools
            .values()
            .map(|tool| format!("- {}: {}", tool.name(), tool.description()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn list_tools(&self) -> Vec<&dyn Tool> {
        self.tools.values().map(|t| t.as_ref()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// 具体工具实现

pub struct ReadFileTool {
    project_directory: String,
}

impl ReadFileTool {
    pub fn new(project_directory: String) -> Self {
        Self { project_directory }
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "读取指定文件的内容。支持相对路径（相对于项目目录）和绝对路径"
    }

    async fn execute(&self, args: Vec<String>) -> AgentResult<String> {
        if args.len() != 1 {
            return Err(crate::errors::AgentError::RuntimeError(
                "read_file 需要一个文件路径参数".to_string(),
            ));
        }

        let file_path = &args[0];

        // 处理文件路径：如果是相对路径，则转换为项目目录下的绝对路径
        let final_path = if Path::new(file_path).is_absolute() {
            file_path.clone()
        } else {
            // 相对路径，拼接到项目目录
            let mut project_path = PathBuf::from(&self.project_directory);
            project_path.push(file_path);
            project_path.to_string_lossy().to_string()
        };

        let content = fs::read_to_string(&final_path).await?;
        Ok(content)
    }
}

pub struct WriteFileTool {
    project_directory: String,
}

impl WriteFileTool {
    pub fn new(project_directory: String) -> Self {
        Self { project_directory }
    }
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_to_file"
    }

    fn description(&self) -> &str {
        "将指定内容写入指定文件。支持相对路径（相对于项目目录）和绝对路径"
    }

    async fn execute(&self, args: Vec<String>) -> AgentResult<String> {
        if args.len() != 2 {
            return Err(crate::errors::AgentError::RuntimeError(
                "write_to_file 需要文件路径和内容两个参数".to_string(),
            ));
        }

        let file_path = &args[0];
        let content = &args[1];

        // 处理文件路径：如果是相对路径，则转换为项目目录下的绝对路径
        let final_path = if Path::new(file_path).is_absolute() {
            file_path.clone()
        } else {
            // 相对路径，拼接到项目目录
            let mut project_path = PathBuf::from(&self.project_directory);
            project_path.push(file_path);
            project_path.to_string_lossy().to_string()
        };

        // 确保目录存在
        if let Some(parent) = Path::new(&final_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }

        fs::write(&final_path, content).await?;
        Ok(format!("写入成功: {}", final_path))
    }
}

pub struct RunTerminalCommandTool;

#[async_trait]
impl Tool for RunTerminalCommandTool {
    fn name(&self) -> &str {
        "run_terminal_command"
    }

    fn description(&self) -> &str {
        "执行终端命令"
    }

    async fn execute(&self, args: Vec<String>) -> AgentResult<String> {
        if args.is_empty() {
            return Err(crate::errors::AgentError::RuntimeError(
                "run_terminal_command 需要命令参数".to_string(),
            ));
        }

        let command = &args[0];
        let output = Command::new("sh").arg("-c").arg(command).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(format!("执行成功: {}", stdout))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(crate::errors::AgentError::CommandExecutionError(
                stderr.to_string(),
            ))
        }
    }
}

// 工具工厂函数
pub fn create_default_tools(project_directory: String) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(ReadFileTool::new(project_directory.clone()));
    registry.register(WriteFileTool::new(project_directory));
    registry.register(RunTerminalCommandTool);
    registry
}
