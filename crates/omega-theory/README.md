# omega-theory

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../LICENSE)
[![Rust Version: 1.91+](https://img.shields.io/badge/rust-1.91+-orange.svg)](https://www.rust-lang.org)

Ω-Theory complexity analysis framework for AIT42 Editor task decomposition.

## Overview

`omega-theory` provides tools for analyzing task complexity using mathematical concepts related to "Omega" (Ω):

- **Big Omega (Ω) Notation**: Asymptotic lower bounds for algorithm complexity (v1.6.0 ✅)
- **Prime Omega (Ω)**: Number-theoretic function counting prime factors (v1.7.0 🔜)
- **Chaitin's Omega (Ω)**: Algorithmic information theory and halting probability (v1.7.0 🔜)

## Current Features (v1.6.0)

### ComplexityClass Enum

Classify tasks into 6 computational complexity tiers:

```rust
use omega_theory::ComplexityClass;

let task = ComplexityClass::Linear;
println!("Complexity: {}", task); // "Ω(n)"

// Get recommended subtask decomposition
let subtasks = task.to_subtask_range();
assert_eq!(subtasks, 3..=5);
```

### Complexity Tiers

| Class | Notation | Description | Subtasks |
|-------|----------|-------------|----------|
| `Constant` | Ω(1) | Simple config changes, trivial operations | 1 |
| `Logarithmic` | Ω(log n) | Binary search, tree operations | 2-3 |
| `Linear` | Ω(n) | Standard CRUD operations, single-pass processing | 3-5 |
| `Linearithmic` | Ω(n log n) | Sorting, indexing, efficient algorithms | 4-6 |
| `Quadratic` | Ω(n²) | Nested loops, matrix operations | 5-10 |
| `Exponential` | Ω(2^n) | Combinatorial problems, brute-force search | 8-15 |

### Integration with Task Master AI

```rust
use omega_theory::ComplexityClass;
use serde_json;

#[derive(serde::Serialize, serde::Deserialize)]
struct Task {
    id: String,
    description: String,
    complexity: ComplexityClass,
}

let task = Task {
    id: "1.2.3".to_string(),
    description: "Implement authentication".to_string(),
    complexity: ComplexityClass::Linear,
};

// Serialize for Task Master AI
let json = serde_json::to_string(&task)?;
```

## Roadmap

### v1.7.0 (Planned)

- **LLM-based Classification**: Automatic complexity inference from task descriptions using Anthropic API
- **Prime Omega Function**: `Ω(n)` counting prime factors with multiplicity
- **Chaitin's Omega**: Halting probability approximation for algorithmic randomness

### Future Versions

- Multi-dimensional complexity analysis (time vs. space vs. cognitive)
- Historical accuracy tracking for LLM predictions
- Integration with AIT42 multi-agent system for adaptive decomposition

## Usage Example

```rust
use omega_theory::ComplexityClass;

// v1.6.0: Placeholder classification (always returns Linear)
let complexity = ComplexityClass::from_description(
    "Migrate authentication system to OAuth 2.0"
);

assert_eq!(complexity, ComplexityClass::Linear); // Will improve in v1.7.0

// Get recommended decomposition
let range = complexity.to_subtask_range();
println!("Recommended subtasks: {} to {}", range.start(), range.end());

// Display in mathematical notation
println!("Task complexity: {}", complexity); // "Ω(n)"
```

## Architecture

```text
omega-theory/
├── src/
│   ├── lib.rs                 # Public API and re-exports
│   ├── big_omega.rs          # ✅ v1.6.0: ComplexityClass enum
│   ├── prime_omega.rs        # 🔜 v1.7.0: Prime factorization
│   └── chaitins_omega.rs     # 🔜 v1.7.0: Algorithmic information
└── tests/
    └── complexity_tests.rs   # Integration tests (30 test cases)
```

## Testing

```bash
# Run all tests
cargo test -p omega-theory

# Run with coverage
cargo test -p omega-theory --coverage

# Lint with clippy
cargo clippy -p omega-theory -- -D warnings
```

**Test Coverage**: 100% (30/30 tests passing)

## Quality Metrics

- ✅ Zero compiler warnings
- ✅ Zero clippy warnings (pedantic mode)
- ✅ 100% test coverage
- ✅ All doctests passing
- ✅ Comprehensive integration tests
- ✅ Full documentation with examples

## Dependencies

- `serde` - Serialization/deserialization support
- `serde_json` - JSON encoding for Task Master AI integration
- `thiserror` - Error handling (used in future versions)

**Zero runtime dependencies** for core functionality.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

**Version**: 1.6.0 (Foundation)
**Author**: AIT42 Team
**Status**: Production-ready for basic complexity classification
