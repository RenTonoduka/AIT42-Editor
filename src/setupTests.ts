/**
 * Test Setup - Runs before all tests
 */

import '@testing-library/jest-dom';
import { TextEncoder, TextDecoder } from 'util';

// Polyfills
global.TextEncoder = TextEncoder;
global.TextDecoder = TextDecoder as any;

// Mock Tauri API
const mockTauri = {
  invoke: jest.fn(),
  listen: jest.fn(),
  emit: jest.fn(),
};

jest.mock('@tauri-apps/api/tauri', () => mockTauri);

// Mock window.__TAURI__
(global as any).window = {
  __TAURI__: {
    invoke: mockTauri.invoke,
    listen: mockTauri.listen,
    emit: mockTauri.emit,
  },
};

// Mock Monaco Editor
jest.mock('@monaco-editor/react', () => ({
  __esModule: true,
  default: ({ onMount, value, onChange }: any) => {
    // Simulate Monaco editor
    if (onMount) {
      const mockEditor = {
        getValue: () => value,
        setValue: (newValue: string) => onChange?.(newValue),
        focus: jest.fn(),
        updateOptions: jest.fn(),
        addCommand: jest.fn(),
      };

      const mockMonaco = {
        editor: {
          defineTheme: jest.fn(),
          setTheme: jest.fn(),
        },
        KeyMod: {
          CtrlCmd: 2048,
        },
        KeyCode: {
          KeyS: 49,
        },
      };

      onMount(mockEditor, mockMonaco);
    }

    return (
      <div data-testid="monaco-editor">
        <textarea
          data-testid="monaco-textarea"
          value={value}
          onChange={(e) => onChange?.(e.target.value)}
        />
      </div>
    );
  },
}));

// Suppress console errors in tests
const originalError = console.error;
beforeAll(() => {
  console.error = (...args: any[]) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Not implemented: HTMLFormElement.prototype.submit')
    ) {
      return;
    }
    originalError.call(console, ...args);
  };
});

afterAll(() => {
  console.error = originalError;
});
