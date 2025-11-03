#!/bin/bash
# AIT42 Editor - Comprehensive Test Runner
# Runs all tests with coverage reporting

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   AIT42 Editor - Test Suite Runner    ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Function to print section headers
print_section() {
    echo -e "\n${YELLOW}>>> $1${NC}\n"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Parse command line arguments
RUN_COVERAGE=false
RUN_BENCHES=false
VERBOSE=false
TEST_FILTER=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --coverage|-c)
            RUN_COVERAGE=true
            shift
            ;;
        --bench|-b)
            RUN_BENCHES=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --filter|-f)
            TEST_FILTER="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: ./scripts/run_tests.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -c, --coverage    Generate coverage report"
            echo "  -b, --bench       Run benchmark tests"
            echo "  -v, --verbose     Show detailed test output"
            echo "  -f, --filter      Run only tests matching filter"
            echo "  -h, --help        Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./scripts/run_tests.sh                    # Run all tests"
            echo "  ./scripts/run_tests.sh --coverage         # Run with coverage"
            echo "  ./scripts/run_tests.sh -f buffer          # Run buffer tests only"
            echo "  ./scripts/run_tests.sh -v -c              # Verbose with coverage"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust toolchain."
    exit 1
fi

print_section "Environment Check"
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Project: AIT42 Editor"
echo "Test filter: ${TEST_FILTER:-<none>}"
print_success "Environment ready"

# Clean previous build artifacts
print_section "Cleaning Build Artifacts"
cargo clean --quiet
print_success "Clean complete"

# Build the project
print_section "Building Project"
if cargo build --all --quiet; then
    print_success "Build successful"
else
    print_error "Build failed"
    exit 1
fi

# Run unit tests
print_section "Running Unit Tests"
UNIT_TEST_CMD="cargo test --lib"

if [ "$VERBOSE" = true ]; then
    UNIT_TEST_CMD="$UNIT_TEST_CMD -- --nocapture"
fi

if [ -n "$TEST_FILTER" ]; then
    UNIT_TEST_CMD="$UNIT_TEST_CMD $TEST_FILTER"
fi

if eval "$UNIT_TEST_CMD"; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# Run integration tests
print_section "Running Integration Tests"
INTEGRATION_TEST_CMD="cargo test --test '*'"

if [ "$VERBOSE" = true ]; then
    INTEGRATION_TEST_CMD="$INTEGRATION_TEST_CMD -- --nocapture"
fi

if [ -n "$TEST_FILTER" ]; then
    INTEGRATION_TEST_CMD="$INTEGRATION_TEST_CMD -- $TEST_FILTER"
fi

if eval "$INTEGRATION_TEST_CMD"; then
    print_success "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

# Run property-based tests
print_section "Running Property-Based Tests"
PROPERTY_TEST_CMD="cargo test --test lib property::"

if [ "$VERBOSE" = true ]; then
    PROPERTY_TEST_CMD="$PROPERTY_TEST_CMD -- --nocapture"
fi

if eval "$PROPERTY_TEST_CMD"; then
    print_success "Property-based tests passed"
else
    print_error "Property-based tests failed"
    exit 1
fi

# Run doc tests
print_section "Running Documentation Tests"
if cargo test --doc --quiet; then
    print_success "Documentation tests passed"
else
    print_error "Documentation tests failed"
    exit 1
fi

# Run benchmarks if requested
if [ "$RUN_BENCHES" = true ]; then
    print_section "Running Benchmarks"
    if [ -d "benches" ]; then
        if cargo bench --quiet; then
            print_success "Benchmarks completed"
        else
            print_error "Benchmarks failed"
            exit 1
        fi
    else
        echo -e "${YELLOW}No benchmarks found${NC}"
    fi
fi

# Generate coverage report if requested
if [ "$RUN_COVERAGE" = true ]; then
    print_section "Generating Coverage Report"

    # Check if tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
        cargo install cargo-tarpaulin
    fi

    COVERAGE_DIR="$PROJECT_ROOT/coverage"
    mkdir -p "$COVERAGE_DIR"

    echo "Generating coverage report (this may take a few minutes)..."

    if cargo tarpaulin \
        --all \
        --out Html \
        --out Json \
        --out Xml \
        --output-dir "$COVERAGE_DIR" \
        --timeout 300 \
        --exclude-files "target/*" \
        --exclude-files "tests/*"; then

        print_success "Coverage report generated"

        # Extract coverage percentage from JSON
        if [ -f "$COVERAGE_DIR/tarpaulin-report.json" ]; then
            COVERAGE=$(python3 -c "
import json
with open('$COVERAGE_DIR/tarpaulin-report.json') as f:
    data = json.load(f)
    covered = data.get('coverage', {}).get('covered', 0)
    total = data.get('coverage', {}).get('total', 1)
    print(f'{(covered/total*100):.2f}')
" 2>/dev/null || echo "N/A")

            echo -e "\n${BLUE}Coverage: ${COVERAGE}%${NC}"

            # Check coverage threshold
            if command -v python3 &> /dev/null; then
                THRESHOLD_MET=$(python3 -c "
try:
    coverage = float('$COVERAGE')
    print('true' if coverage >= 85.0 else 'false')
except:
    print('false')
" 2>/dev/null)

                if [ "$THRESHOLD_MET" = "true" ]; then
                    print_success "Coverage meets threshold (≥85%)"
                else
                    echo -e "${YELLOW}⚠ Coverage below threshold (target: ≥85%)${NC}"
                fi
            fi
        fi

        echo -e "\n${BLUE}Coverage reports available at:${NC}"
        echo "  HTML: file://$COVERAGE_DIR/index.html"
        echo "  JSON: $COVERAGE_DIR/tarpaulin-report.json"
        echo "  XML:  $COVERAGE_DIR/cobertura.xml"

        # Open HTML report (macOS)
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo -e "\n${BLUE}Opening HTML report...${NC}"
            open "$COVERAGE_DIR/index.html"
        fi
    else
        print_error "Coverage generation failed"
        exit 1
    fi
fi

# Test summary
print_section "Test Summary"

TEST_COUNT=$(cargo test --all -- --list --format=terse 2>/dev/null | grep -c "test" || echo "N/A")

echo "Total tests: $TEST_COUNT"
echo "Status: All tests passed ✓"

if [ "$RUN_COVERAGE" = true ]; then
    echo "Coverage: ${COVERAGE}%"
fi

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}   All Tests Completed Successfully!   ${NC}"
echo -e "${GREEN}========================================${NC}"

# Run clippy for additional checks
print_section "Running Clippy (Linter)"
if cargo clippy --all --quiet -- -D warnings; then
    print_success "Clippy checks passed"
else
    echo -e "${YELLOW}⚠ Clippy warnings found${NC}"
fi

# Check formatting
print_section "Checking Code Formatting"
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    echo -e "${YELLOW}⚠ Code formatting issues found. Run 'cargo fmt' to fix.${NC}"
fi

echo ""
echo -e "${BLUE}Test run completed at $(date)${NC}"
echo ""

exit 0
