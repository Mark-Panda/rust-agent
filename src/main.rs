use clap::Parser;
use rust_agent::{create_default_tools, ReActAgent};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rust-agent")]
#[command(about = "A Rust implementation of ReAct Agent")]
#[command(version)]
struct Cli {
    /// 项目目录路径
    #[arg(value_name = "PROJECT_DIRECTORY")]
    project_directory: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // 检查项目目录是否存在
    if !cli.project_directory.exists() {
        eprintln!(
            "错误：项目目录 '{}' 不存在",
            cli.project_directory.display()
        );
        std::process::exit(1);
    }

    if !cli.project_directory.is_dir() {
        eprintln!("错误：'{}' 不是一个目录", cli.project_directory.display());
        std::process::exit(1);
    }

    let project_dir = cli.project_directory.canonicalize()?;
    println!("项目目录: {}", project_dir.display());

    // 创建工具注册表
    let tools = create_default_tools(project_dir.to_string_lossy().to_string());

    // 创建Agent
    let agent = ReActAgent::new(
        tools,
        "deepseek-v3-250324".to_string(),
        project_dir.to_string_lossy().to_string(),
    )?;

    // 获取用户输入
    print!("请输入任务：");
    io::stdout().flush()?;

    let mut task = String::new();
    io::stdin().read_line(&mut task)?;
    let task = task.trim();

    if task.is_empty() {
        println!("任务不能为空");
        return Ok(());
    }

    println!("开始执行任务: {}", task);

    // 运行Agent
    match agent.run(task).await {
        Ok(final_answer) => {
            println!("\n\n✅ Final Answer：{}", final_answer);
        }
        Err(e) => {
            eprintln!("Agent执行错误: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
