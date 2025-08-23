#!/usr/bin/env python3
"""
Simple test script for the eg-mcp server
"""
import json
import subprocess
import sys

def send_mcp_request(process, request):
    """Send a JSON-RPC request to the MCP server"""
    request_json = json.dumps(request) + '\n'
    process.stdin.write(request_json.encode())
    process.stdin.flush()
    
    # Read response
    response_line = process.stdout.readline()
    if response_line:
        return json.loads(response_line.decode())
    return None

def test_mcp_server():
    """Test the eg-mcp server"""
    # Start the server
    cmd = ["cargo", "run", "--bin", "eg-mcp", "--features", "mcp"]
    process = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd="/Users/nikomat/dev/eg"
    )
    
    try:
        # Initialize
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        }
        
        print("Sending initialize request...")
        response = send_mcp_request(process, init_request)
        print(f"Initialize response: {response}")
        
        # List tools
        list_tools_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }
        
        print("Sending list tools request...")
        response = send_mcp_request(process, list_tools_request)
        print(f"List tools response: {response}")
        
        # Test get_crate_source
        get_source_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "get_crate_source",
                "arguments": {"crate_name": "serde"}
            }
        }
        
        print("Sending get_crate_source request...")
        response = send_mcp_request(process, get_source_request)
        print(f"Get crate source response: {response}")
        
    finally:
        process.terminate()
        process.wait()

if __name__ == "__main__":
    test_mcp_server()
