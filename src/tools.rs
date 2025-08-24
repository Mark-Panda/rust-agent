use crate::errors::AgentResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// 验证路径是否在项目目录内
fn is_path_within_project(project_dir: &Path, target_path: &Path) -> bool {
    let project_dir = project_dir
        .canonicalize()
        .unwrap_or_else(|_| project_dir.to_path_buf());
    let target_path = target_path
        .canonicalize()
        .unwrap_or_else(|_| target_path.to_path_buf());

    target_path.starts_with(&project_dir)
}

/// 安全地解析路径，确保在项目目录内
fn safe_resolve_path(
    project_dir: &str,
    user_path: &str,
) -> Result<PathBuf, crate::errors::AgentError> {
    let project_path = PathBuf::from(project_dir);
    let user_path = Path::new(user_path);

    let final_path = if user_path.is_absolute() {
        user_path.to_path_buf()
    } else {
        // 相对路径，拼接到项目目录
        let mut resolved_path = project_path.clone();
        resolved_path.push(user_path);
        resolved_path
    };

    // 验证路径是否在项目目录内
    if !is_path_within_project(&project_path, &final_path) {
        return Err(crate::errors::AgentError::RuntimeError(format!(
            "路径 '{}' 不在项目目录 '{}' 内，操作被拒绝",
            final_path.display(),
            project_path.display()
        )));
    }

    Ok(final_path)
}

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

        // 使用安全的路径解析，确保路径在项目目录内
        let final_path = safe_resolve_path(&self.project_directory, file_path)?;

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

        // 使用安全的路径解析，确保路径在项目目录内
        let final_path = safe_resolve_path(&self.project_directory, file_path)?;

        // 确保目录存在
        if let Some(parent) = final_path.parent() {
            if !parent.exists() {
                print!("父目录 '{}' 不存在，是否创建？(Y/N): ", parent.display());
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" {
                    return Ok("用户取消写入文件".to_string());
                }

                // 创建父目录
                fs::create_dir_all(parent).await?;
                println!("已创建父目录: {}", parent.display());
            }
        }

        fs::write(&final_path, content).await?;
        Ok(format!("写入成功: {}", final_path.display()))
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
            Err(crate::errors::AgentError::RuntimeError(format!(
                "命令执行错误: {}",
                stderr
            )))
        }
    }
}

pub struct CreateDirectoryTool {
    project_directory: String,
}

impl CreateDirectoryTool {
    pub fn new(project_directory: String) -> Self {
        Self { project_directory }
    }
}

#[async_trait]
impl Tool for CreateDirectoryTool {
    fn name(&self) -> &str {
        "create_directory"
    }

    fn description(&self) -> &str {
        "创建目录，如果父目录不存在会询问是否创建"
    }

    async fn execute(&self, args: Vec<String>) -> AgentResult<String> {
        if args.len() != 1 {
            return Err(crate::errors::AgentError::RuntimeError(
                "create_directory 需要一个目录路径参数".to_string(),
            ));
        }

        let dir_path = &args[0];

        // 使用安全的路径解析，确保路径在项目目录内
        let final_path = safe_resolve_path(&self.project_directory, dir_path)?;

        let path = &final_path;

        // 如果目录已存在，直接返回
        if path.exists() && path.is_dir() {
            return Ok(format!("目录已存在: {}", final_path.display()));
        }

        // 检查父目录是否存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                print!("父目录 '{}' 不存在，是否创建？(Y/N): ", parent.display());
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" {
                    return Ok("用户取消创建目录".to_string());
                }

                // 创建父目录
                fs::create_dir_all(parent).await?;
                println!("已创建父目录: {}", parent.display());
            }
        }

        // 创建目标目录
        fs::create_dir_all(path).await?;
        Ok(format!("目录创建成功: {}", final_path.display()))
    }
}

pub struct CreateFileTool {
    project_directory: String,
}

impl CreateFileTool {
    pub fn new(project_directory: String) -> Self {
        Self { project_directory }
    }
}

#[async_trait]
impl Tool for CreateFileTool {
    fn name(&self) -> &str {
        "create_file"
    }

    fn description(&self) -> &str {
        "创建空文件，如果父目录不存在会询问是否创建。创建后可以使用 write_to_file 工具写入内容"
    }

    async fn execute(&self, args: Vec<String>) -> AgentResult<String> {
        if args.len() != 1 {
            return Err(crate::errors::AgentError::RuntimeError(
                "create_file 需要一个文件路径参数".to_string(),
            ));
        }

        let file_path = &args[0];

        // 使用安全的路径解析，确保路径在项目目录内
        let final_path = safe_resolve_path(&self.project_directory, file_path)?;

        let path = &final_path;

        // 如果文件已存在，询问是否覆盖
        if path.exists() {
            print!("文件 '{}' 已存在，是否覆盖？(Y/N): ", path.display());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                return Ok("用户取消创建文件".to_string());
            }
        }

        // 检查父目录是否存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                print!("父目录 '{}' 不存在，是否创建？(Y/N): ", parent.display());
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" {
                    return Ok("用户取消创建文件".to_string());
                }

                // 创建父目录
                fs::create_dir_all(parent).await?;
                println!("已创建父目录: {}", parent.display());
            }
        }

        // 创建空文件
        fs::write(path, "").await?;
        Ok(format!("文件创建成功: {}", final_path.display()))
    }
}

// 工具工厂函数
pub fn create_default_tools(project_directory: String) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(ReadFileTool::new(project_directory.clone()));
    registry.register(WriteFileTool::new(project_directory.clone()));
    registry.register(RunTerminalCommandTool);
    registry.register(CreateDirectoryTool::new(project_directory.clone()));
    registry.register(CreateFileTool::new(project_directory));
    registry
}
