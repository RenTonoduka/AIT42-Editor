# AIT42 Editor - Phase 1 Test Documentation

## Overview

Comprehensive test suite for AIT42 Editor Phase 1 implementation covering:
- Backend Rust tests
- Frontend unit tests (React/TypeScript)
- Integration tests
- End-to-end tests (Playwright)

**Test Coverage Goals:**
- Backend: 80%+
- Frontend: 70%+
- Integration: Key workflows covered
- E2E: Critical user paths

---

## Test Structure

```
AIT42-Editor/
├── src-tauri/src/commands/
│   └── file_tests.rs          # Backend Rust tests
├── editor/src/components/
│   ├── Sidebar/__tests__/
│   │   └── FileTree.test.tsx  # File tree unit tests
│   └── Editor/__tests__/
│       ├── MonacoEditor.test.tsx  # Editor unit tests
│       └── TabBar.test.tsx        # Tab bar unit tests
├── src/__tests__/
│   └── integration/
│       └── file-operations.test.tsx  # Integration tests
├── e2e/
│   └── phase1.spec.ts         # E2E tests
├── jest.config.js             # Jest configuration
├── playwright.config.ts       # Playwright configuration
└── src/setupTests.ts          # Test setup
```

---

## 1. Backend Tests (Rust)

### File: `src-tauri/src/commands/file_tests.rs`

**Purpose:** Test all Tauri file system commands.

#### Test Categories

##### File Operations
- `test_open_file_success` - Open file and verify content
- `test_open_file_not_found` - Handle missing files
- `test_open_file_with_language_detection` - Detect file language

##### Save Operations
- `test_save_file_success` - Save file content
- `test_save_file_creates_parent_directories` - Create dirs if needed
- `test_save_file_overwrites_existing` - Overwrite files
- `test_save_file_atomic_write` - Atomic write with temp files

##### Directory Operations
- `test_read_directory_success` - List directory contents
- `test_read_directory_not_a_directory` - Error for non-directories
- `test_read_directory_shallow_recursion` - Lazy load children
- `test_directory_sorting` - Directories first, alphabetical

##### Create/Delete
- `test_create_file_success` - Create new file
- `test_create_file_with_parent_dirs` - Create nested paths
- `test_create_directory_success` - Create directory
- `test_create_directory_nested` - Create nested directories
- `test_delete_file_success` - Delete file
- `test_delete_directory_success` - Delete directory with contents
- `test_delete_nonexistent_path` - Error for missing paths

##### Rename/Move
- `test_rename_file_success` - Rename file
- `test_rename_directory_success` - Rename directory
- `test_rename_creates_parent_dirs` - Create dirs on move

##### Edge Cases
- `test_concurrent_file_operations` - Handle concurrent saves
- `test_large_file_handling` - Handle 1MB+ files
- `test_unicode_filenames` - Support Unicode filenames
- `test_special_characters_in_content` - Handle special chars

#### Running Backend Tests

```bash
# Run all Rust tests
npm run test:backend

# Run tests in watch mode
npm run test:backend:watch

# Run specific test
cd src-tauri && cargo test test_open_file_success

# Run with output
cd src-tauri && cargo test -- --nocapture
```

---

## 2. Frontend Unit Tests (Jest + React Testing Library)

### File: `editor/src/components/Sidebar/__tests__/FileTree.test.tsx`

**Purpose:** Test file tree component behavior.

#### Test Coverage

##### Rendering
- Empty directory state
- Directory tree with files/folders
- File icons (Rust, TypeScript, JSON, etc.)
- Folder icons (open/closed)

##### Directory Expansion
- Expand directory on click
- Collapse directory on second click
- Change folder icon when expanded
- Lazy load children

##### File Selection
- Open file on click
- Highlight selected file
- Call `openFile` with correct path

##### Error Handling
- Directory load errors
- File open errors

##### Nested Directories
- Render nested structure
- Apply correct indentation

### File: `editor/src/components/Editor/__tests__/MonacoEditor.test.tsx`

**Purpose:** Test Monaco editor integration.

#### Test Coverage

##### Theme
- Define Cursor dark theme on mount
- Apply correct theme colors
- Set syntax highlighting rules

##### Configuration
- Font ligatures enabled
- Minimap enabled
- Tab size = 2
- Auto layout

##### Content Changes
- Update content on change
- Handle empty content
- Ignore undefined values

##### Save Functionality
- Register Cmd/Ctrl+S keybinding
- Call `saveFile` on save command
- Don't save if no active tab

##### Special Cases
- Large files (100KB+)
- Unicode characters
- Special characters (newlines, tabs)

### File: `editor/src/components/Editor/__tests__/TabBar.test.tsx`

**Purpose:** Test tab bar and tab management.

#### Test Coverage

##### Rendering
- Null when no tabs
- Single tab
- Multiple tabs
- Long filenames

##### Tab Activation
- Mark active tab
- Switch tabs on click
- Handle clicking active tab

##### Dirty Indicator
- Show for modified tabs
- Hide for unmodified tabs
- Multiple dirty tabs

##### Dynamic Updates
- Add new tabs
- Change active tab
- Tab becomes dirty

---

## 3. Integration Tests

### File: `src/__tests__/integration/file-operations.test.tsx`

**Purpose:** Test complete workflows involving multiple components.

#### Test Scenarios

##### Opening Files
```typescript
it('should open file from tree and display in editor')
```
1. Load file tree
2. Click file
3. Verify tab created
4. Verify content displayed

##### Editing Files
```typescript
it('should edit and mark tab as dirty')
```
1. Open file
2. Edit content
3. Verify dirty indicator

##### Tab Switching
```typescript
it('should preserve edits when switching tabs')
```
1. Open multiple files
2. Edit first file
3. Switch to second file
4. Switch back
5. Verify edits preserved

##### Saving Files
```typescript
it('should save file on Cmd+S')
```
1. Open and edit file
2. Press Cmd+S
3. Verify save called
4. Verify dirty cleared

##### Complete Workflow
```typescript
it('should complete full file editing workflow')
```
Full end-to-end scenario from tree to save.

---

## 4. E2E Tests (Playwright)

### File: `e2e/phase1.spec.ts`

**Purpose:** Test real user interactions in browser.

#### Test Suites

##### File Editing Workflow
- Display welcome screen
- Open folder and display tree
- Expand/collapse directories
- Open file in editor
- Edit file content
- Save with Cmd+S
- Multiple tabs
- Tab switching
- Close tab
- Preserve unsaved changes

##### Edge Cases
- Long filenames
- Correct file icons
- Rapid tab switching
- Scroll position in tree
- Focus management
- Large files

##### Accessibility
- Keyboard navigation
- ARIA labels
- Screen reader support

##### Performance
- App load time < 2s
- Handle 50+ tabs
- Tab switching < 500ms

#### Running E2E Tests

```bash
# Run all E2E tests
npm run test:e2e

# Run with UI
npm run test:e2e:ui

# Debug mode
npm run test:e2e:debug

# Run specific test
npx playwright test e2e/phase1.spec.ts

# Run in specific browser
npx playwright test --project=chromium
npx playwright test --project=firefox
npx playwright test --project=webkit
```

---

## Test Execution

### Run All Tests

```bash
# Install dependencies
npm install

# Run all tests
npm run test:all

# Run frontend unit tests
npm test

# Run tests in watch mode
npm run test:watch

# Run with coverage
npm run test:coverage

# Run backend tests
npm run test:backend

# Run E2E tests
npm run test:e2e
```

### Coverage Reports

After running `npm run test:coverage`, view coverage:

```bash
# Open HTML coverage report
open coverage/index.html
```

**Coverage Targets:**
- Statements: 70%
- Branches: 70%
- Functions: 70%
- Lines: 70%

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18

      - name: Install dependencies
        run: npm ci

      - name: Run unit tests
        run: npm test

      - name: Run E2E tests
        run: npm run test:e2e

      - name: Run backend tests
        run: npm run test:backend

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/coverage-final.json
```

---

## Debugging Tests

### Jest Tests

```bash
# Debug specific test
node --inspect-brk node_modules/.bin/jest --runInBand FileTree.test.tsx

# Run single test
npm test -- FileTree.test.tsx

# Update snapshots
npm test -- -u
```

### Playwright Tests

```bash
# Debug mode (opens browser)
npm run test:e2e:debug

# Run with UI
npm run test:e2e:ui

# Headed mode
npx playwright test --headed

# Trace on failure
npx playwright test --trace on
```

### Rust Tests

```bash
# Debug output
cd src-tauri && cargo test -- --nocapture

# Run specific test
cd src-tauri && cargo test test_name

# Show test list
cd src-tauri && cargo test -- --list
```

---

## Test Best Practices

### Unit Tests
1. **Isolation:** Each test should be independent
2. **Mocking:** Mock external dependencies (Tauri API, Monaco)
3. **AAA Pattern:** Arrange, Act, Assert
4. **Descriptive Names:** Clear test names describing behavior
5. **Fast:** Unit tests should run in milliseconds

### Integration Tests
1. **Real Interactions:** Test actual component interactions
2. **User Perspective:** Test what users see/do
3. **Setup/Teardown:** Clean state between tests
4. **Meaningful Assertions:** Verify complete workflows

### E2E Tests
1. **Critical Paths:** Focus on key user journeys
2. **Stable Selectors:** Use data-testid attributes
3. **Wait Strategies:** Use proper waits, not arbitrary timeouts
4. **Error Handling:** Test error scenarios
5. **Performance:** Monitor load times

### Backend Tests
1. **Temp Files:** Use tempfile crate for file operations
2. **Cleanup:** Always clean up test files
3. **Error Cases:** Test both success and failure paths
4. **Concurrency:** Test concurrent operations
5. **Edge Cases:** Unicode, large files, special chars

---

## Common Issues & Solutions

### Issue: Monaco Editor Mock Not Working

**Solution:** Ensure setupTests.ts is loaded before tests:

```typescript
// jest.config.js
setupFilesAfterEnv: ['<rootDir>/src/setupTests.ts']
```

### Issue: Tauri API Mock Fails

**Solution:** Mock window.__TAURI__ in setupTests.ts:

```typescript
(global as any).window = {
  __TAURI__: {
    invoke: mockInvoke,
    listen: mockListen,
    emit: mockEmit,
  },
};
```

### Issue: E2E Tests Timeout

**Solution:** Increase timeout in playwright.config.ts:

```typescript
timeout: 30000,
expect: { timeout: 5000 }
```

### Issue: Backend Tests Fail on CI

**Solution:** Ensure temp directory permissions:

```rust
let temp_dir = tempfile::tempdir()
    .expect("Failed to create temp dir");
```

---

## Test Metrics

### Current Coverage

| Component | Coverage | Target |
|-----------|----------|--------|
| Backend (Rust) | TBD | 80% |
| Frontend | TBD | 70% |
| Integration | TBD | Key workflows |
| E2E | TBD | Critical paths |

### Test Counts

- Backend Tests: 30+
- Frontend Unit Tests: 80+
- Integration Tests: 10+
- E2E Tests: 25+

**Total: 145+ tests**

---

## Next Steps

1. **Run Initial Tests:**
   ```bash
   npm install
   npm run test:all
   ```

2. **Review Coverage:**
   ```bash
   npm run test:coverage
   open coverage/index.html
   ```

3. **Fix Failures:** Address any failing tests

4. **Improve Coverage:** Add tests for uncovered code

5. **CI Integration:** Set up GitHub Actions

---

## Resources

- [Jest Documentation](https://jestjs.io/)
- [React Testing Library](https://testing-library.com/react)
- [Playwright Documentation](https://playwright.dev/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tauri Testing](https://tauri.app/v1/guides/testing/)

---

## Maintenance

**Review tests when:**
- Adding new features
- Modifying existing functionality
- Refactoring components
- Fixing bugs (add regression test)

**Update coverage when:**
- Coverage drops below threshold
- New critical paths added
- Edge cases discovered

---

*Last Updated: 2025-11-03*
*Test Suite Version: 1.0.0*
