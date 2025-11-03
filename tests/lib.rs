//! AIT42 Editor Comprehensive Test Suite
//!
//! This test suite provides extensive coverage including:
//! - Unit tests (in src/ modules with #[cfg(test)])
//! - Integration tests (tests/integration/)
//! - Property-based tests (tests/property/)
//!
//! # Test Organization
//!
//! ```text
//! tests/
//! ├── unit/                 # Additional unit tests (edge cases)
//! │   ├── buffer_edge_cases.rs
//! │   ├── cursor_edge_cases.rs
//! │   └── command_edge_cases.rs
//! ├── integration/          # Integration tests
//! │   ├── buffer_integration.rs
//! │   ├── cursor_integration.rs
//! │   ├── command_integration.rs
//! │   ├── editor_workflow.rs
//! │   └── multi_buffer_workflow.rs
//! ├── property/             # Property-based tests
//! │   ├── buffer_properties.rs
//! │   ├── cursor_properties.rs
//! │   └── command_properties.rs
//! └── lib.rs               # This file
//! ```
//!
//! # Running Tests
//!
//! ```bash
//! # Run all tests
//! cargo test
//!
//! # Run only integration tests
//! cargo test --test integration
//!
//! # Run only property-based tests
//! cargo test --test property
//!
//! # Run with test output
//! cargo test -- --nocapture
//!
//! # Run specific test
//! cargo test test_buffer_insert
//! ```
//!
//! # Test Coverage
//!
//! Generate coverage report:
//! ```bash
//! cargo tarpaulin --workspace --out Html --output-dir coverage/
//! ```

// Unit test modules (edge cases and boundary conditions)
mod unit;

// Integration test modules
mod integration;

// Property-based test modules
mod property;

// Common test utilities and helpers
mod helpers;
