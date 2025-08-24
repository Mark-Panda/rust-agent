#!/bin/bash

# 检查参数
if [ $# -eq 0 ]; then
    echo "❌ 错误：请指定项目目录"
    echo ""
    echo "使用方法："
    echo "  ./run.sh /path/to/your/project"
    echo ""
    echo "示例："
    echo "  ./run.sh /Users/username/my_project"
    echo "  ./run.sh ."
    exit 1
fi

PROJECT_DIR="$1"

# 检查项目目录是否存在
if [ ! -d "$PROJECT_DIR" ]; then
    echo "❌ 错误：项目目录 '$PROJECT_DIR' 不存在"
    exit 1
fi

# 检查是否已经构建
if [ ! -f "./target/release/rust-agent" ]; then
    echo "⚠️  可执行文件不存在，正在构建..."
    if ! ./build.sh; then
        echo "❌ 构建失败，无法运行"
        exit 1
    fi
fi

# 检查 .env 文件
if [ ! -f .env ]; then
    echo "❌ 错误：未找到 .env 文件"
    echo "请先设置环境变量："
    echo "  cp env.example .env"
    echo "然后编辑 .env 文件设置你的 OPENROUTER_API_KEY"
    exit 1
fi

echo "🚀 启动 Rust Agent..."
echo "📁 项目目录: $PROJECT_DIR"
echo ""

# 运行 Agent
./target/release/rust-agent "$PROJECT_DIR"
