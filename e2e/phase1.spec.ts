/**
 * AIT42 Editor - Phase 1 E2E Tests
 *
 * Tests complete user workflows:
 * - File navigation
 * - File editing
 * - Tab management
 * - Keyboard shortcuts
 * - Persistence
 */

import { test, expect } from '@playwright/test';

test.describe('Phase 1: File Editing Workflow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');

    // Wait for app to be ready
    await page.waitForSelector('[data-testid="app-loaded"]', { timeout: 10000 });
  });

  test('should display welcome screen on startup', async ({ page }) => {
    // Verify welcome message
    await expect(page.locator('h1')).toContainText('AIT42 Editor');

    // Verify action buttons
    await expect(page.locator('button:has-text("Open Folder")')).toBeVisible();
    await expect(page.locator('button:has-text("New File")')).toBeVisible();
  });

  test('should open folder and display file tree', async ({ page }) => {
    // Click "Open Folder" button
    await page.click('button:has-text("Open Folder")');

    // Select test folder (mock file dialog)
    await page.evaluate(() => {
      window.__TAURI__.invoke('read_directory', { path: '/test/workspace' });
    });

    // Verify file tree is displayed
    await expect(page.locator('[data-testid="file-tree"]')).toBeVisible();

    // Verify at least one file/folder is shown
    const fileItems = page.locator('[data-testid^="file-item-"]');
    await expect(fileItems.first()).toBeVisible();
  });

  test('should expand and collapse directories', async ({ page }) => {
    // Open folder
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    // Find a directory
    const directory = page.locator('[data-testid="file-item-src"]');
    await expect(directory).toBeVisible();

    // Verify collapsed state (no children visible)
    const childrenBefore = await directory.locator('[data-testid^="file-item-"]').count();
    expect(childrenBefore).toBe(0);

    // Click to expand
    await directory.click();

    // Verify expanded state (children visible)
    await page.waitForTimeout(500); // Wait for expansion animation
    const childrenAfter = await directory.locator('[data-testid^="file-item-"]').count();
    expect(childrenAfter).toBeGreaterThan(0);

    // Click again to collapse
    await directory.click();
    await page.waitForTimeout(500);

    // Verify collapsed again
    const childrenFinal = await directory.locator('[data-testid^="file-item-"]').count();
    expect(childrenFinal).toBe(0);
  });

  test('should open file in editor', async ({ page }) => {
    // Open folder
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    // Click on a file
    await page.click('[data-testid="file-item-main.rs"]');

    // Verify editor is shown
    await expect(page.locator('[data-testid="monaco-editor"]')).toBeVisible();

    // Verify tab is created
    await expect(page.locator('.tab:has-text("main.rs")')).toBeVisible();

    // Verify file content is loaded
    const editorContent = await page.locator('[data-testid="monaco-textarea"]').inputValue();
    expect(editorContent.length).toBeGreaterThan(0);
  });

  test('should edit file content', async ({ page }) => {
    // Open file
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');
    await page.click('[data-testid="file-item-main.rs"]');

    // Wait for editor to be ready
    const editor = page.locator('[data-testid="monaco-textarea"]');
    await expect(editor).toBeVisible();

    // Get original content
    const originalContent = await editor.inputValue();

    // Type new content
    await editor.click();
    await page.keyboard.type('\n// New comment');

    // Verify content changed
    const newContent = await editor.inputValue();
    expect(newContent).not.toBe(originalContent);
    expect(newContent).toContain('// New comment');

    // Verify dirty indicator appears
    const tab = page.locator('.tab:has-text("main.rs")');
    await expect(tab.locator('.dirty-indicator')).toBeVisible();
  });

  test('should save file with Cmd+S', async ({ page }) => {
    // Open and edit file
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');
    await page.click('[data-testid="file-item-main.rs"]');

    const editor = page.locator('[data-testid="monaco-textarea"]');
    await editor.click();
    await page.keyboard.type('\n// Modified');

    // Verify dirty state
    const tab = page.locator('.tab:has-text("main.rs")');
    await expect(tab.locator('.dirty-indicator')).toBeVisible();

    // Save with keyboard shortcut
    await page.keyboard.press(process.platform === 'darwin' ? 'Meta+s' : 'Control+s');

    // Wait for save to complete
    await page.waitForTimeout(500);

    // Verify dirty indicator is removed
    await expect(tab.locator('.dirty-indicator')).not.toBeVisible();
  });

  test('should open multiple files in tabs', async ({ page }) => {
    // Open folder
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    // Open first file
    await page.click('[data-testid="file-item-main.rs"]');
    await expect(page.locator('.tab:has-text("main.rs")')).toBeVisible();

    // Open second file
    await page.click('[data-testid="file-item-lib.rs"]');
    await expect(page.locator('.tab:has-text("lib.rs")')).toBeVisible();

    // Open third file
    await page.click('[data-testid="file-item-config.toml"]');
    await expect(page.locator('.tab:has-text("config.toml")')).toBeVisible();

    // Verify all three tabs are visible
    const tabs = page.locator('.tab');
    await expect(tabs).toHaveCount(3);
  });

  test('should switch between tabs', async ({ page }) => {
    // Open multiple files
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    await page.click('[data-testid="file-item-main.rs"]');
    await page.click('[data-testid="file-item-lib.rs"]');

    // Verify lib.rs tab is active
    const libTab = page.locator('.tab:has-text("lib.rs")');
    await expect(libTab).toHaveClass(/active/);

    // Click on main.rs tab
    const mainTab = page.locator('.tab:has-text("main.rs")');
    await mainTab.click();

    // Verify main.rs tab is now active
    await expect(mainTab).toHaveClass(/active/);
    await expect(libTab).not.toHaveClass(/active/);

    // Verify editor content changed
    const editorContent = await page.locator('[data-testid="monaco-textarea"]').inputValue();
    expect(editorContent).toContain('main'); // main.rs content
  });

  test('should close tab', async ({ page }) => {
    // Open file
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');
    await page.click('[data-testid="file-item-main.rs"]');

    // Verify tab exists
    const tab = page.locator('.tab:has-text("main.rs")');
    await expect(tab).toBeVisible();

    // Click close button on tab
    await tab.locator('.close-button').click();

    // Verify tab is removed
    await expect(tab).not.toBeVisible();

    // Verify editor is cleared
    await expect(page.locator('[data-testid="monaco-editor"]')).not.toBeVisible();
  });

  test('should preserve unsaved changes when switching tabs', async ({ page }) => {
    // Open two files
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');
    await page.click('[data-testid="file-item-main.rs"]');
    await page.click('[data-testid="file-item-lib.rs"]');

    // Edit lib.rs
    const editor = page.locator('[data-testid="monaco-textarea"]');
    await editor.click();
    const modification = '\n// Test modification';
    await page.keyboard.type(modification);

    const libContent = await editor.inputValue();

    // Switch to main.rs
    await page.click('.tab:has-text("main.rs")');
    await page.waitForTimeout(300);

    // Switch back to lib.rs
    await page.click('.tab:has-text("lib.rs")');
    await page.waitForTimeout(300);

    // Verify modification is preserved
    const preservedContent = await editor.inputValue();
    expect(preservedContent).toBe(libContent);
    expect(preservedContent).toContain('// Test modification');
  });

  test('should show dirty indicator for multiple modified tabs', async ({ page }) => {
    // Open two files
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');
    await page.click('[data-testid="file-item-main.rs"]');
    await page.click('[data-testid="file-item-lib.rs"]');

    // Edit both files
    let editor = page.locator('[data-testid="monaco-textarea"]');

    // Edit lib.rs
    await editor.click();
    await page.keyboard.type('\n// Mod 1');

    // Switch to main.rs
    await page.click('.tab:has-text("main.rs")');
    await page.waitForTimeout(300);

    // Edit main.rs
    await editor.click();
    await page.keyboard.type('\n// Mod 2');

    // Verify both tabs show dirty indicator
    await expect(page.locator('.tab:has-text("main.rs") .dirty-indicator')).toBeVisible();
    await expect(page.locator('.tab:has-text("lib.rs") .dirty-indicator')).toBeVisible();
  });

  test('should handle very long file names', async ({ page }) => {
    // Open folder
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    // Find and click long filename
    const longFileName = 'very_long_file_name_that_might_overflow_the_tab.rs';
    await page.click(`[data-testid="file-item-${longFileName}"]`);

    // Verify tab is created and visible
    const tab = page.locator(`.tab:has-text("${longFileName}")`);
    await expect(tab).toBeVisible();

    // Verify tab doesn't overflow
    const tabBox = await tab.boundingBox();
    expect(tabBox?.width).toBeLessThan(300); // Reasonable max width
  });

  test('should display correct file icons in tree', async ({ page }) => {
    // Open folder
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    // Verify folder icon
    const folder = page.locator('[data-testid="file-item-src"]');
    await expect(folder.locator('.file-icon')).toContainText('📁');

    // Verify Rust file icon
    const rustFile = page.locator('[data-testid="file-item-main.rs"]');
    await expect(rustFile.locator('.file-icon')).toContainText('🦀');

    // Verify JSON file icon
    const jsonFile = page.locator('[data-testid="file-item-package.json"]');
    await expect(jsonFile.locator('.file-icon')).toContainText('📋');
  });

  test('should handle rapid tab switching', async ({ page }) => {
    // Open multiple files
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    for (let i = 1; i <= 5; i++) {
      await page.click(`[data-testid="file-item-file${i}.rs"]`);
    }

    // Rapidly switch between tabs
    for (let i = 0; i < 10; i++) {
      const tabIndex = (i % 5) + 1;
      await page.click(`.tab:has-text("file${tabIndex}.rs")`);
      await page.waitForTimeout(50); // Minimal delay
    }

    // Verify no crashes and last tab is active
    await expect(page.locator('.tab.active')).toBeVisible();
  });

  test('should maintain scroll position in file tree', async ({ page }) => {
    // Open folder with many files
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    const fileTree = page.locator('[data-testid="file-tree"]');

    // Scroll down
    await fileTree.evaluate((el) => {
      el.scrollTop = 500;
    });

    const scrollPosition = await fileTree.evaluate((el) => el.scrollTop);

    // Open a file
    await page.click('[data-testid="file-item-bottom-file.rs"]');

    // Verify scroll position is maintained
    const newScrollPosition = await fileTree.evaluate((el) => el.scrollTop);
    expect(newScrollPosition).toBe(scrollPosition);
  });

  test('should focus editor after opening file', async ({ page }) => {
    // Open file
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');
    await page.click('[data-testid="file-item-main.rs"]');

    // Verify editor has focus
    const editor = page.locator('[data-testid="monaco-textarea"]');
    await expect(editor).toBeFocused();

    // Verify can type immediately
    await page.keyboard.type('test');
    const content = await editor.inputValue();
    expect(content).toContain('test');
  });

  test('should handle large files efficiently', async ({ page }) => {
    // Open large file
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    const startTime = Date.now();
    await page.click('[data-testid="file-item-large-file.txt"]');

    // Wait for editor to load
    await page.waitForSelector('[data-testid="monaco-editor"]');
    const loadTime = Date.now() - startTime;

    // Should load within reasonable time (< 3 seconds)
    expect(loadTime).toBeLessThan(3000);

    // Verify editor is responsive
    const editor = page.locator('[data-testid="monaco-textarea"]');
    await editor.click();
    await page.keyboard.type('x');

    const content = await editor.inputValue();
    expect(content).toContain('x');
  });

  test('should support keyboard navigation in file tree', async ({ page }) => {
    // Open folder
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    // Focus file tree
    await page.click('[data-testid="file-tree"]');

    // Navigate with arrow keys
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');

    // Open file with Enter
    await page.keyboard.press('Enter');

    // Verify file opened
    await expect(page.locator('[data-testid="monaco-editor"]')).toBeVisible();
  });

  test('should show status bar with file information', async ({ page }) => {
    // Open file
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');
    await page.click('[data-testid="file-item-main.rs"]');

    // Verify status bar shows file info
    const statusBar = page.locator('[data-testid="status-bar"]');
    await expect(statusBar).toBeVisible();

    // Verify language indicator
    await expect(statusBar.locator(':has-text("Rust")')).toBeVisible();

    // Verify encoding
    await expect(statusBar.locator(':has-text("UTF-8")')).toBeVisible();
  });
});

test.describe('Accessibility', () => {
  test('should be keyboard navigable', async ({ page }) => {
    await page.goto('/');

    // Tab through interactive elements
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Verify focus is visible
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  });

  test('should have proper ARIA labels', async ({ page }) => {
    await page.goto('/');

    // Verify important elements have ARIA labels
    await expect(page.locator('[aria-label="File Explorer"]')).toBeVisible();
    await expect(page.locator('[aria-label="Editor"]')).toBeVisible();
  });

  test('should support screen reader navigation', async ({ page }) => {
    await page.goto('/');

    // Verify landmarks exist
    await expect(page.locator('main')).toBeVisible();
    await expect(page.locator('aside')).toBeVisible();
  });
});

test.describe('Performance', () => {
  test('should load quickly', async ({ page }) => {
    const startTime = Date.now();
    await page.goto('/');
    await page.waitForSelector('[data-testid="app-loaded"]');
    const loadTime = Date.now() - startTime;

    // Should load within 2 seconds
    expect(loadTime).toBeLessThan(2000);
  });

  test('should handle 50+ tabs without performance degradation', async ({ page }) => {
    await page.goto('/');
    await page.click('button:has-text("Open Folder")');
    await page.waitForSelector('[data-testid="file-tree"]');

    // Open 50 tabs
    for (let i = 1; i <= 50; i++) {
      await page.click(`[data-testid="file-item-file${i}.txt"]`);
    }

    // Verify all tabs loaded
    const tabs = page.locator('.tab');
    await expect(tabs).toHaveCount(50);

    // Test tab switching performance
    const startTime = Date.now();
    await page.click('.tab:has-text("file1.txt")');
    await page.waitForTimeout(100);
    const switchTime = Date.now() - startTime;

    // Should switch quickly (< 500ms)
    expect(switchTime).toBeLessThan(500);
  });
});
