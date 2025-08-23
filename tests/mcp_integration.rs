mod mcp_tests {
    use std::process::{Command, Stdio};
    use std::io::{Write, BufRead, BufReader};
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_mcp_server_initialization() {
        let mut child = Command::new("cargo")
            .args(&["run", "--bin", "eg-mcp"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start eg-mcp server");

        let stdin = child.stdin.as_mut().expect("Failed to get stdin");
        let stdout = child.stdout.as_mut().expect("Failed to get stdout");
        let mut reader = BufReader::new(stdout);

        // Send initialize request
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        });

        writeln!(stdin, "{}", init_request).expect("Failed to write to stdin");

        // Read response
        let mut response_line = String::new();
        reader.read_line(&mut response_line).expect("Failed to read response");

        let response: Value = serde_json::from_str(&response_line)
            .expect("Failed to parse JSON response");

        // Verify response structure
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert!(response["result"].is_object());
        
        let result = &response["result"];
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"]["tools"].is_object());
        assert!(result["serverInfo"].is_object());
        assert!(result["instructions"].is_string());

        // Cleanup
        child.kill().expect("Failed to kill child process");
    }
}
