use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("API调用失败: {0}")]
    ApiError(String),

    #[error("JSON序列化/反序列化错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("环境变量未设置: {0}")]
    EnvVarError(String),

    #[error("工具执行错误: {0}")]
    ToolExecutionError(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("运行时错误: {0}")]
    RuntimeError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("命令执行错误: {0}")]
    CommandExecutionError(String),
}

pub type AgentResult<T> = Result<T, AgentError>;
