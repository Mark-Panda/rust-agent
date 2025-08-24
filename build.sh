#!/bin/bash

echo "🚀 开始构建 Rust Agent 项目..."

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误：未找到 Cargo，请先安装 Rust"
    echo "请访问 https://rustup.rs/ 安装 Rust"
    exit 1
fi

echo "✅ Rust 已安装，版本：$(cargo --version)"

# 检查 .env 文件
if [ ! -f .env ]; then
    echo "⚠️  警告：未找到 .env 文件"
    echo "请复制 env.example 为 .env 并设置你的 OPENROUTER_API_KEY"
    echo "cp env.example .env"
    echo "然后编辑 .env 文件设置你的 API 密钥"
    echo ""
fi

# 清理之前的构建
echo "🧹 清理之前的构建..."
cargo clean

# 构建项目
echo "🔨 构建项目..."
if cargo build --release; then
    echo "✅ 构建成功！"
    echo ""
    echo "🎯 使用方法："
    echo "  ./target/release/rust-agent /path/to/your/project"
    echo ""
    echo "或者使用 Cargo 运行："
    echo "  cargo run --release -- /path/to/your/project"
    echo ""
    echo "📁 可执行文件位置：./target/release/rust-agent"
else
    echo "❌ 构建失败！"
    exit 1
fi
