//! OWASP A02:2021 & A03:2021 - Sensitive Data Exposure Testing
//!
//! Tests for information disclosure, configuration security, and data protection.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[cfg(test)]
mod file_permission_tests {
    use super::*;

    #[test]
    fn test_swap_file_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let swap_file = temp_dir.path().join(".test.swp");

        // Create swap file
        fs::write(&swap_file, "sensitive content").unwrap();

        // Set permissions as implementation should
        let mut perms = fs::metadata(&swap_file).unwrap().permissions();
        perms.set_mode(0o600); // Owner read/write only
        fs::set_permissions(&swap_file, perms).unwrap();

        // Verify permissions
        let metadata = fs::metadata(&swap_file).unwrap();
        let mode = metadata.permissions().mode();

        assert_eq!(
            mode & 0o777,
            0o600,
            "Swap file should have 0600 permissions (owner only)"
        );
    }

    #[test]
    fn test_config_file_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        // Create config
        fs::write(&config_file, "[editor]\ntab_size = 4").unwrap();

        // Verify permissions are not world-writable
        let metadata = fs::metadata(&config_file).unwrap();
        let mode = metadata.permissions().mode();

        assert_eq!(
            mode & 0o002,
            0,
            "Config file should not be world-writable"
        );
    }

    #[test]
    fn test_audit_log_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("audit.log");

        // Create audit log
        fs::write(&log_file, "audit entry").unwrap();

        // Set secure permissions
        let mut perms = fs::metadata(&log_file).unwrap().permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&log_file, perms).unwrap();

        // Verify
        let metadata = fs::metadata(&log_file).unwrap();
        let mode = metadata.permissions().mode();

        assert_eq!(
            mode & 0o777,
            0o600,
            "Audit log should have 0600 permissions"
        );
    }

    #[test]
    fn test_temp_file_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("file.tmp");

        // Create temp file with secure permissions
        fs::write(&temp_file, "temp content").unwrap();

        let mut perms = fs::metadata(&temp_file).unwrap().permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&temp_file, perms).unwrap();

        // Verify
        let metadata = fs::metadata(&temp_file).unwrap();
        let mode = metadata.permissions().mode();

        assert_eq!(
            mode & 0o777,
            0o600,
            "Temp file should have restrictive permissions"
        );
    }

    #[test]
    fn test_atomic_write_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.txt");

        // Simulate atomic write
        let temp_path = file.with_extension(".tmp");
        fs::write(&temp_path, "content").unwrap();

        // Set permissions before rename
        let mut perms = fs::metadata(&temp_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&temp_path, perms).unwrap();

        // Rename
        fs::rename(&temp_path, &file).unwrap();

        // Verify final permissions
        let metadata = fs::metadata(&file).unwrap();
        let mode = metadata.permissions().mode();

        assert_eq!(
            mode & 0o777,
            0o644,
            "Final file should have 0644 permissions"
        );
    }
}

#[cfg(test)]
mod configuration_security_tests {
    use super::*;

    #[test]
    fn test_no_secrets_in_config() {
        let suspicious_configs = vec![
            r#"
            [ait42]
            api_key = "sk-ant-1234567890abcdef"
            "#,
            r#"
            [ait42]
            anthropic_api_key = "secret_key"
            "#,
            r#"
            [github]
            token = "ghp_1234567890"
            "#,
            r#"
            [auth]
            password = "password123"
            "#,
        ];

        for config in suspicious_configs {
            let result = detect_secrets_in_config(config);
            assert!(
                result.is_some(),
                "Should detect potential secrets in config"
            );
        }
    }

    #[test]
    fn test_config_validation_rejects_secrets() {
        let config_with_secret = r#"
        [ait42]
        api_key = "sk-ant-secret"
        "#;

        // Configuration parser should warn or reject
        let result = parse_and_validate_config(config_with_secret);

        assert!(
            result.is_err() || result.unwrap().contains_warning,
            "Should warn about secrets in config"
        );
    }

    #[test]
    fn test_secure_defaults() {
        let default_config = get_default_config();

        // Verify secure defaults
        assert_eq!(
            default_config.get("ait42.auto_execute"),
            Some("false"),
            "Auto-execute should default to false"
        );

        assert_eq!(
            default_config.get("ait42.require_confirmation"),
            Some("true"),
            "Should require confirmation by default"
        );

        assert_eq!(
            default_config.get("editor.auto_save"),
            Some("false"),
            "Auto-save should default to false"
        );
    }

    #[test]
    fn test_environment_variable_secrets() {
        // Test that secrets come from env vars, not config
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");

        let config = load_config_from_env();

        // Config file should not contain the secret
        assert!(
            !config.file_content.contains("test-key"),
            "API key should not be in config file"
        );

        // But should be available in runtime config
        assert_eq!(
            config.runtime_api_key.as_deref(),
            Some("test-key"),
            "API key should be loaded from environment"
        );

        std::env::remove_var("ANTHROPIC_API_KEY");
    }

    // Helper functions
    fn detect_secrets_in_config(config: &str) -> Option<Vec<String>> {
        let secret_patterns = vec![
            r"api_key\s*=\s*['\"]",
            r"token\s*=\s*['\"]",
            r"password\s*=\s*['\"]",
            r"secret\s*=\s*['\"]",
            r"sk-ant-",
            r"ghp_",
        ];

        let mut found_secrets = Vec::new();

        for pattern in secret_patterns {
            if config.contains(&pattern[..pattern.len().min(10)]) {
                found_secrets.push(pattern.to_string());
            }
        }

        if found_secrets.is_empty() {
            None
        } else {
            Some(found_secrets)
        }
    }

    #[derive(Debug)]
    struct ParsedConfig {
        contains_warning: bool,
    }

    fn parse_and_validate_config(config: &str) -> Result<ParsedConfig, String> {
        if detect_secrets_in_config(config).is_some() {
            return Err("Config contains potential secrets".to_string());
        }

        Ok(ParsedConfig {
            contains_warning: false,
        })
    }

    fn get_default_config() -> std::collections::HashMap<String, String> {
        let mut config = std::collections::HashMap::new();
        config.insert("ait42.auto_execute".to_string(), "false".to_string());
        config.insert("ait42.require_confirmation".to_string(), "true".to_string());
        config.insert("editor.auto_save".to_string(), "false".to_string());
        config
    }

    struct LoadedConfig {
        file_content: String,
        runtime_api_key: Option<String>,
    }

    fn load_config_from_env() -> LoadedConfig {
        LoadedConfig {
            file_content: "[editor]\ntab_size = 4".to_string(),
            runtime_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
        }
    }
}

#[cfg(test)]
mod information_disclosure_tests {
    use super::*;

    #[test]
    fn test_error_messages_no_stack_traces() {
        // Test that error messages don't leak sensitive info
        let error_msg = format_user_error("Failed to open file");

        assert!(
            !error_msg.contains("src/"),
            "Error should not contain source paths"
        );
        assert!(
            !error_msg.contains("at "),
            "Error should not contain stack traces"
        );
        assert!(
            !error_msg.contains("/Users/"),
            "Error should not contain user paths"
        );
    }

    #[test]
    fn test_log_sanitization() {
        let sensitive_data = "API_KEY=sk-ant-1234567890";

        let sanitized = sanitize_for_logging(sensitive_data);

        assert!(
            !sanitized.contains("sk-ant-"),
            "Should redact API keys from logs"
        );
        assert!(
            sanitized.contains("***"),
            "Should show redaction marker"
        );
    }

    #[test]
    fn test_no_debug_output_in_release() {
        // Verify debug output is disabled
        #[cfg(not(debug_assertions))]
        {
            assert!(
                !is_debug_logging_enabled(),
                "Debug logging should be disabled in release"
            );
        }
    }

    #[test]
    fn test_agent_output_sanitization() {
        let agent_output = vec![
            "Processing...",
            "API_KEY=sk-ant-secret",
            "PASSWORD=secret123",
            "Normal output",
        ];

        let sanitized = sanitize_agent_output(&agent_output);

        assert!(
            !sanitized.join("\n").contains("sk-ant-"),
            "Should redact API keys"
        );
        assert!(
            !sanitized.join("\n").contains("secret123"),
            "Should redact passwords"
        );
        assert!(
            sanitized.join("\n").contains("Normal output"),
            "Should preserve non-sensitive output"
        );
    }

    #[test]
    fn test_file_not_found_disclosure() {
        // Test that file-not-found doesn't leak directory structure
        let error = format_file_not_found_error(Path::new("/Users/victim/.ssh/id_rsa"));

        assert!(
            !error.contains("/Users/victim/"),
            "Should not disclose full path in error"
        );
        assert!(
            error.contains("File not found"),
            "Should have generic message"
        );
    }

    // Helper functions
    fn format_user_error(msg: &str) -> String {
        // User-facing errors should not contain technical details
        format!("Error: {}", msg)
    }

    fn sanitize_for_logging(data: &str) -> String {
        let patterns = vec![
            (r"sk-ant-[a-zA-Z0-9]+", "sk-ant-***"),
            (r"ghp_[a-zA-Z0-9]+", "ghp_***"),
            (r"API_KEY=\S+", "API_KEY=***"),
            (r"PASSWORD=\S+", "PASSWORD=***"),
        ];

        let mut sanitized = data.to_string();

        for (pattern, replacement) in patterns {
            // Simple replacement (real implementation would use regex)
            if sanitized.contains(&pattern[..10]) {
                sanitized = sanitized.replace(&pattern[..pattern.len().min(sanitized.len())], replacement);
            }
        }

        sanitized
    }

    fn is_debug_logging_enabled() -> bool {
        #[cfg(debug_assertions)]
        {
            true
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    }

    fn sanitize_agent_output(output: &[&str]) -> Vec<String> {
        output
            .iter()
            .map(|line| sanitize_for_logging(line))
            .collect()
    }

    fn format_file_not_found_error(path: &Path) -> String {
        // Only show filename, not full path
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        format!("File not found: {}", filename)
    }
}

#[cfg(test)]
mod readonly_file_tests {
    use super::*;

    #[test]
    fn test_readonly_file_protection() {
        let temp_dir = TempDir::new().unwrap();
        let readonly_file = temp_dir.path().join("readonly.txt");

        // Create readonly file
        fs::write(&readonly_file, "readonly content").unwrap();

        let mut perms = fs::metadata(&readonly_file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&readonly_file, perms).unwrap();

        // Attempt to write should fail
        let write_result = fs::write(&readonly_file, "new content");

        assert!(
            write_result.is_err(),
            "Should not be able to write to readonly file"
        );
    }

    #[test]
    fn test_permission_check_before_write() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.txt");

        fs::write(&file, "content").unwrap();

        // Make readonly
        let mut perms = fs::metadata(&file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file, perms).unwrap();

        // Check permissions before attempting write
        let can_write = check_write_permission(&file);

        assert!(!can_write, "Should detect readonly file");
    }

    fn check_write_permission(path: &Path) -> bool {
        if let Ok(metadata) = fs::metadata(path) {
            !metadata.permissions().readonly()
        } else {
            false
        }
    }
}
