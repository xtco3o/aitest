#!/usr/bin/env bash

set -e
# 获取脚本所在的绝对路径
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd "$DIR"

echo "正在以 Release 模式构建 Rust MCP 服务..."
cargo build --release

# 使用 cargo metadata + jq 获取目标目录
TARGET_DIR=$(cargo metadata --format-version 1 | jq -r .target_directory)
BINARY_PATH="$TARGET_DIR/release/aitest"

if [ ! -f "$BINARY_PATH" ]; then
    echo "错误：未在 $BINARY_PATH 找到二进制文件"
    exit 1
fi

echo "正在从 Claude 中移除现有的 MCP 服务 'aitest' (如果存在)..."
claude mcp remove aitest || true

echo "正在将 MCP 服务 'aitest' 添加到 Claude..."
# 使用 'claude mcp add' 注册服务。
# 这通常会将其添加到全局的 claude_desktop_config.json 中。
claude mcp add aitest "$BINARY_PATH"

echo "完成！你现在可以使用 'aitest' MCP 服务的 'echo' 工具了。"
