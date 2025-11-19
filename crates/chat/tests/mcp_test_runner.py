#!/usr/bin/env python3
"""
MCP Server Integration Test Runner
Tests the MCP server with actual JSON-RPC requests
"""

import json
import subprocess
import sys
from typing import Dict, Any

RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
NC = '\033[0m'  # No Color

class MCPTester:
    def __init__(self, server_path: str):
        self.server_path = server_path
        self.test_results = []

    def send_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Send a single JSON-RPC request to the server"""
        request_json = json.dumps(request) + '\n'

        proc = subprocess.Popen(
            [self.server_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )

        stdout, stderr = proc.communicate(input=request_json, timeout=5)

        if not stdout:
            return {'error': 'No response from server'}

        try:
            # Parse the response (first non-empty line)
            for line in stdout.strip().split('\n'):
                if line:
                    return json.loads(line)
            return {'error': 'Empty response'}
        except json.JSONDecodeError as e:
            return {'error': f'Invalid JSON response: {e}', 'raw': stdout}

    def test_initialize(self):
        """Test initialize method"""
        print(f'\n{YELLOW}Test: Initialize{NC}')

        request = {
            'jsonrpc': '2.0',
            'id': 1,
            'method': 'initialize',
            'params': {}
        }

        response = self.send_request(request)

        # Check for successful response
        if 'result' in response:
            if 'protocolVersion' in response['result']:
                print(f'{GREEN}✓ PASSED{NC}')
                print(f'  Protocol version: {response["result"]["protocolVersion"]}')
                self.test_results.append(('Initialize', True))
                return True

        print(f'{RED}✗ FAILED{NC}')
        print(f'  Response: {json.dumps(response, indent=2)}')
        self.test_results.append(('Initialize', False))
        return False

    def test_tools_list(self):
        """Test tools/list method"""
        print(f'\n{YELLOW}Test: List Tools{NC}')

        request = {
            'jsonrpc': '2.0',
            'id': 2,
            'method': 'tools/list'
        }

        response = self.send_request(request)

        if 'result' in response and 'tools' in response['result']:
            tools = response['result']['tools']
            tool_names = [tool['name'] for tool in tools]

            expected_tools = ['list_sources', 'list_chats', 'get_messages']
            if all(name in tool_names for name in expected_tools):
                print(f'{GREEN}✓ PASSED{NC}')
                print(f'  Found tools: {", ".join(tool_names)}')
                self.test_results.append(('List Tools', True))
                return True

        print(f'{RED}✗ FAILED{NC}')
        print(f'  Response: {json.dumps(response, indent=2)}')
        self.test_results.append(('List Tools', False))
        return False

    def test_list_sources_tool(self):
        """Test list_sources tool"""
        print(f'\n{YELLOW}Test: List Sources Tool{NC}')

        request = {
            'jsonrpc': '2.0',
            'id': 3,
            'method': 'tools/call',
            'params': {
                'name': 'list_sources',
                'arguments': {}
            }
        }

        response = self.send_request(request)

        if 'result' in response and 'content' in response['result']:
            print(f'{GREEN}✓ PASSED{NC}')
            print(f'  Response has content field')
            self.test_results.append(('List Sources Tool', True))
            return True

        print(f'{RED}✗ FAILED{NC}')
        print(f'  Response: {json.dumps(response, indent=2)}')
        self.test_results.append(('List Sources Tool', False))
        return False

    def test_invalid_method(self):
        """Test invalid method error handling"""
        print(f'\n{YELLOW}Test: Invalid Method Error{NC}')

        request = {
            'jsonrpc': '2.0',
            'id': 4,
            'method': 'invalid_method'
        }

        response = self.send_request(request)

        if 'error' in response:
            print(f'{GREEN}✓ PASSED{NC}')
            print(f'  Error code: {response["error"]["code"]}')
            print(f'  Error message: {response["error"]["message"]}')
            self.test_results.append(('Invalid Method Error', True))
            return True

        print(f'{RED}✗ FAILED{NC}')
        print(f'  Expected error response, got: {json.dumps(response, indent=2)}')
        self.test_results.append(('Invalid Method Error', False))
        return False

    def test_invalid_tool(self):
        """Test invalid tool name error handling"""
        print(f'\n{YELLOW}Test: Invalid Tool Error{NC}')

        request = {
            'jsonrpc': '2.0',
            'id': 5,
            'method': 'tools/call',
            'params': {
                'name': 'invalid_tool',
                'arguments': {}
            }
        }

        response = self.send_request(request)

        if 'error' in response:
            print(f'{GREEN}✓ PASSED{NC}')
            print(f'  Error code: {response["error"]["code"]}')
            print(f'  Error message: {response["error"]["message"]}')
            self.test_results.append(('Invalid Tool Error', True))
            return True

        print(f'{RED}✗ FAILED{NC}')
        print(f'  Expected error response, got: {json.dumps(response, indent=2)}')
        self.test_results.append(('Invalid Tool Error', False))
        return False

    def run_all_tests(self):
        """Run all tests"""
        print(f'{YELLOW}=== Running MCP Server Integration Tests ==={NC}')

        tests = [
            self.test_initialize,
            self.test_tools_list,
            self.test_list_sources_tool,
            self.test_invalid_method,
            self.test_invalid_tool,
        ]

        for test in tests:
            try:
                test()
            except Exception as e:
                print(f'{RED}✗ EXCEPTION: {e}{NC}')
                self.test_results.append((test.__name__, False))

        # Print summary
        print(f'\n{YELLOW}=== Test Summary ==={NC}')
        passed = sum(1 for _, result in self.test_results if result)
        total = len(self.test_results)

        for name, result in self.test_results:
            status = f'{GREEN}PASSED{NC}' if result else f'{RED}FAILED{NC}'
            print(f'  {name}: {status}')

        print(f'\n{YELLOW}Total: {passed}/{total} tests passed{NC}')

        if passed == total:
            print(f'{GREEN}All tests passed!{NC}')
            return 0
        else:
            print(f'{RED}Some tests failed{NC}')
            return 1

def main():
    # Build the server
    print(f'{YELLOW}Building MCP Server...{NC}')
    build_result = subprocess.run(
        ['cargo', 'build', '--release', '--features', 'mcp', '--bin', 'chat-mcp-server'],
        capture_output=True,
        text=True
    )

    if build_result.returncode != 0:
        print(f'{RED}Failed to build MCP server{NC}')
        print(build_result.stderr)
        return 1

    print(f'{GREEN}MCP Server built successfully{NC}')

    # Run tests
    server_path = './target/release/chat-mcp-server'
    tester = MCPTester(server_path)
    return tester.run_all_tests()

if __name__ == '__main__':
    sys.exit(main())
