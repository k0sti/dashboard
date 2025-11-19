#!/bin/bash

# MCP Server Integration Test Script
# This script tests the MCP server with actual JSON-RPC requests

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Building MCP Server...${NC}"
cd ..
cargo build --release --features mcp --bin chat-mcp-server 2>/dev/null

SERVER_BIN="./target/release/chat-mcp-server"

if [ ! -f "$SERVER_BIN" ]; then
    echo -e "${RED}Failed to build MCP server${NC}"
    exit 1
fi

echo -e "${GREEN}MCP Server built successfully${NC}"
cd tests

# Function to send a JSON-RPC request and check response
send_request() {
    local name=$1
    local request=$2
    local expected_pattern=$3

    echo -e "\n${YELLOW}Test: $name${NC}"

    response=$(echo "$request" | $SERVER_BIN 2>/dev/null)

    if echo "$response" | grep -q "$expected_pattern"; then
        echo -e "${GREEN}✓ PASSED${NC}"
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        echo "Expected pattern: $expected_pattern"
        echo "Got response: $response"
        return 1
    fi
}

echo -e "\n${YELLOW}=== Running MCP Server Integration Tests ===${NC}"

# Test 1: Initialize
send_request "Initialize" \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
    '"protocolVersion":"2024-11-05"'

# Test 2: List tools
send_request "List Tools" \
    '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
    '"name":"list_sources"'

# Test 3: Call list_sources tool
send_request "List Sources Tool" \
    '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_sources","arguments":{}}}' \
    '"content"'

# Test 4: Invalid method
send_request "Invalid Method Error" \
    '{"jsonrpc":"2.0","id":4,"method":"invalid_method"}' \
    '"error"'

# Test 5: Invalid tool name
send_request "Invalid Tool Error" \
    '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"invalid_tool","arguments":{}}}' \
    '"error"'

# Test 6: Parse error
send_request "Parse Error" \
    '{invalid json}' \
    '"error"'

echo -e "\n${GREEN}=== All MCP Server Integration Tests Completed ===${NC}"
