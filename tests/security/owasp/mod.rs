//! OWASP Top 10 2021 Security Test Suite
//!
//! Comprehensive security testing against OWASP Top 10 vulnerabilities.

pub mod injection;
pub mod sensitive_data;
pub mod denial_of_service;

#[cfg(test)]
mod integration {
    #[test]
    fn test_owasp_suite_loads() {
        // Ensure all modules compile
        assert!(true);
    }
}
