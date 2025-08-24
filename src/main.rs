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

    // 从环境变量获取模型名称
    let model_name =
        std::env::var("OPENAI_MODEL_NAME").unwrap_or_else(|_| "kimi-k2-250711".to_string());

    // 创建Agent
    let agent = ReActAgent::new(tools, model_name, project_dir.to_string_lossy().to_string())?;

    println!("🤖 Rust Agent 已启动！输入 'quit' 或 'exit' 退出程序。");
    println!("💡 你可以继续输入新的任务，Agent会记住之前的对话上下文。\n");

    // 持续对话循环
    loop {
        // 获取用户输入
        print!("请输入任务：");
        io::stdout().flush()?;

        let mut task = String::new();
        io::stdin().read_line(&mut task)?;
        let task = task.trim();

        // 检查退出命令
        if task.is_empty() {
            println!("任务不能为空，请重新输入");
            continue;
        }

        if task.to_lowercase() == "quit" || task.to_lowercase() == "exit" {
            println!("👋 再见！");
            break;
        }

        println!("开始执行任务: {}", task);

        // 运行Agent
        match agent.run(task).await {
            Ok(final_answer) => {
                println!("\n\n✅ Final Answer：{}", final_answer);
                println!("\n{}", "=".repeat(50));
            }
            Err(e) => {
                eprintln!("Agent执行错误: {}", e);
                println!("请重新输入任务或输入 'quit' 退出程序");
                println!("\n{}", "=".repeat(50));
            }
        }
    }

    Ok(())
}
