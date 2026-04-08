#!/bin/bash

# AI 经验服务 MCP 接口自动化测试脚本

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}开始集成测试...${NC}"

# 使用 cargo run -q 作为运行命令
# 我们将请求合并发送，并捕获输出
(
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
  echo '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
  echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
  echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"save_experience","arguments":{"title":"测试","content":"内容","tags":["tag"]}}}'
  echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"search_experience","arguments":{"query":"测试"}}}'
) | cargo run -q 2>&1 | while read -r line; do
    if [[ "$line" == "{"* ]]; then
        echo -e "${GREEN}响应:${NC}"
        echo "$line" | jq . || echo "$line"
    else
        # 打印日志信息（stderr 来的）
        echo "$line"
    fi
done

echo -e "\n${BLUE}测试完成。${NC}"
