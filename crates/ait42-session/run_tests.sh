#!/bin/bash
# SQLite Migration Test Suite Runner

set -e

echo "========================================"
echo "  AIT42 Session - Test Suite Runner"
echo "========================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test statistics
echo -e "${BLUE}Test Suite Statistics:${NC}"
echo "- Test Files: 6"
echo "- Test Functions: 70+"
echo "- Total Lines: 3,734"
echo "- Benchmark Tests: 10"
echo ""

# Unit Tests
echo -e "${BLUE}Running Unit Tests...${NC}"
echo ""

echo -e "${GREEN}1. Database Connection Tests${NC}"
cargo test -p ait42-session --test db_connection_tests

echo ""
echo -e "${GREEN}2. CRUD Operation Tests${NC}"
cargo test -p ait42-session --test crud_tests

echo ""
echo -e "${GREEN}3. Query and Filtering Tests${NC}"
cargo test -p ait42-session --test query_tests

echo ""
echo -e "${GREEN}4. Transaction and Concurrency Tests${NC}"
cargo test -p ait42-session --test transaction_tests

# Integration Tests
echo ""
echo -e "${BLUE}Running Integration Tests...${NC}"
echo ""

echo -e "${GREEN}5. Data Migration Tests${NC}"
cargo test -p ait42-session --test migration_tests

# All tests
echo ""
echo -e "${BLUE}Running All Tests Together...${NC}"
cargo test -p ait42-session

# Benchmarks (optional)
echo ""
echo -e "${YELLOW}To run benchmarks:${NC}"
echo "  cargo bench -p ait42-session"
echo ""

# Coverage (optional)
echo -e "${YELLOW}To run coverage analysis:${NC}"
echo "  cargo tarpaulin -p ait42-session --out Html"
echo ""

echo -e "${GREEN}All tests completed successfully!${NC}"
