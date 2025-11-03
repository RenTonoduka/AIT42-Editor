/**
 * TabBar Component Tests
 *
 * Test coverage:
 * - Tab display
 * - Tab switching
 * - Tab closing
 * - Dirty indicator
 * - Empty state
 */

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { TabBar } from '../TabBar';
import { useEditor } from '../../../hooks/useEditor';

// Mock dependencies
jest.mock('../../../hooks/useEditor');
jest.mock('../Tab', () => ({
  Tab: ({ tab, isActive, onActivate }: any) => (
    <div
      data-testid={`tab-${tab.id}`}
      className={isActive ? 'tab active' : 'tab'}
      onClick={onActivate}
    >
      {tab.title}
      {tab.isDirty && <span className="dirty-indicator">●</span>}
    </div>
  ),
}));

describe('TabBar', () => {
  const mockSetActiveTab = jest.fn();
  const mockCloseTab = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render null when no tabs are open', () => {
      (useEditor as jest.Mock).mockReturnValue({
        tabs: [],
        activeTabId: null,
        setActiveTab: mockSetActiveTab,
      });

      const { container } = render(<TabBar />);

      expect(container.firstChild).toBeNull();
    });

    it('should render single tab', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: 'fn main() {}',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      expect(screen.getByTestId('tab-tab-1')).toBeInTheDocument();
      expect(screen.getByText('main.rs')).toBeInTheDocument();
    });

    it('should render multiple tabs', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: 'fn main() {}',
          isDirty: false,
        },
        {
          id: 'tab-2',
          title: 'lib.rs',
          path: '/test/lib.rs',
          content: 'pub fn test() {}',
          isDirty: false,
        },
        {
          id: 'tab-3',
          title: 'config.toml',
          path: '/test/config.toml',
          content: '[package]',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      expect(screen.getByTestId('tab-tab-1')).toBeInTheDocument();
      expect(screen.getByTestId('tab-tab-2')).toBeInTheDocument();
      expect(screen.getByTestId('tab-tab-3')).toBeInTheDocument();
    });

    it('should display tab titles correctly', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'VeryLongFileName.tsx',
          path: '/test/VeryLongFileName.tsx',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      expect(screen.getByText('VeryLongFileName.tsx')).toBeInTheDocument();
    });
  });

  describe('Tab Activation', () => {
    it('should mark active tab', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: false,
        },
        {
          id: 'tab-2',
          title: 'lib.rs',
          path: '/test/lib.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      const activeTab = screen.getByTestId('tab-tab-1');
      const inactiveTab = screen.getByTestId('tab-tab-2');

      expect(activeTab).toHaveClass('active');
      expect(inactiveTab).not.toHaveClass('active');
    });

    it('should switch tabs on click', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: false,
        },
        {
          id: 'tab-2',
          title: 'lib.rs',
          path: '/test/lib.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      const tab2 = screen.getByTestId('tab-tab-2');
      fireEvent.click(tab2);

      expect(mockSetActiveTab).toHaveBeenCalledWith('tab-2');
    });

    it('should handle clicking already active tab', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      const tab1 = screen.getByTestId('tab-tab-1');
      fireEvent.click(tab1);

      expect(mockSetActiveTab).toHaveBeenCalledWith('tab-1');
    });
  });

  describe('Dirty Indicator', () => {
    it('should show dirty indicator for modified tab', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: 'fn main() {}',
          isDirty: true,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      expect(screen.getByText('●')).toBeInTheDocument();
    });

    it('should not show dirty indicator for unmodified tab', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: 'fn main() {}',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      expect(screen.queryByText('●')).not.toBeInTheDocument();
    });

    it('should show dirty indicator for multiple dirty tabs', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: true,
        },
        {
          id: 'tab-2',
          title: 'lib.rs',
          path: '/test/lib.rs',
          content: '',
          isDirty: true,
        },
        {
          id: 'tab-3',
          title: 'config.toml',
          path: '/test/config.toml',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      const dirtyIndicators = screen.getAllByText('●');
      expect(dirtyIndicators).toHaveLength(2);
    });
  });

  describe('Tab Order', () => {
    it('should maintain tab order', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'a.rs',
          path: '/test/a.rs',
          content: '',
          isDirty: false,
        },
        {
          id: 'tab-2',
          title: 'b.rs',
          path: '/test/b.rs',
          content: '',
          isDirty: false,
        },
        {
          id: 'tab-3',
          title: 'c.rs',
          path: '/test/c.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      const { container } = render(<TabBar />);

      const tabElements = container.querySelectorAll('[data-testid^="tab-"]');
      expect(tabElements[0]).toHaveAttribute('data-testid', 'tab-tab-1');
      expect(tabElements[1]).toHaveAttribute('data-testid', 'tab-tab-2');
      expect(tabElements[2]).toHaveAttribute('data-testid', 'tab-tab-3');
    });
  });

  describe('Dynamic Tab Updates', () => {
    it('should update when tabs change', () => {
      const initialTabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs: initialTabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      const { rerender } = render(<TabBar />);

      expect(screen.getByTestId('tab-tab-1')).toBeInTheDocument();

      // Add new tab
      const updatedTabs = [
        ...initialTabs,
        {
          id: 'tab-2',
          title: 'lib.rs',
          path: '/test/lib.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs: updatedTabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      rerender(<TabBar />);

      expect(screen.getByTestId('tab-tab-1')).toBeInTheDocument();
      expect(screen.getByTestId('tab-tab-2')).toBeInTheDocument();
    });

    it('should update when active tab changes', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: false,
        },
        {
          id: 'tab-2',
          title: 'lib.rs',
          path: '/test/lib.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      const { rerender } = render(<TabBar />);

      let activeTab = screen.getByTestId('tab-tab-1');
      expect(activeTab).toHaveClass('active');

      // Change active tab
      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-2',
        setActiveTab: mockSetActiveTab,
      });

      rerender(<TabBar />);

      activeTab = screen.getByTestId('tab-tab-2');
      expect(activeTab).toHaveClass('active');
    });

    it('should update when tab becomes dirty', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      const { rerender } = render(<TabBar />);

      expect(screen.queryByText('●')).not.toBeInTheDocument();

      // Make tab dirty
      tabs[0].isDirty = true;

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      rerender(<TabBar />);

      expect(screen.getByText('●')).toBeInTheDocument();
    });
  });

  describe('Empty State Transitions', () => {
    it('should transition from empty to populated', () => {
      (useEditor as jest.Mock).mockReturnValue({
        tabs: [],
        activeTabId: null,
        setActiveTab: mockSetActiveTab,
      });

      const { container, rerender } = render(<TabBar />);

      expect(container.firstChild).toBeNull();

      // Add tab
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      rerender(<TabBar />);

      expect(screen.getByTestId('tab-tab-1')).toBeInTheDocument();
    });

    it('should transition from populated to empty', () => {
      const tabs = [
        {
          id: 'tab-1',
          title: 'main.rs',
          path: '/test/main.rs',
          content: '',
          isDirty: false,
        },
      ];

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-1',
        setActiveTab: mockSetActiveTab,
      });

      const { container, rerender } = render(<TabBar />);

      expect(screen.getByTestId('tab-tab-1')).toBeInTheDocument();

      // Remove all tabs
      (useEditor as jest.Mock).mockReturnValue({
        tabs: [],
        activeTabId: null,
        setActiveTab: mockSetActiveTab,
      });

      rerender(<TabBar />);

      expect(container.firstChild).toBeNull();
    });
  });

  describe('Many Tabs', () => {
    it('should handle many tabs efficiently', () => {
      const tabs = Array.from({ length: 50 }, (_, i) => ({
        id: `tab-${i}`,
        title: `file-${i}.rs`,
        path: `/test/file-${i}.rs`,
        content: '',
        isDirty: i % 3 === 0, // Every third tab is dirty
      }));

      (useEditor as jest.Mock).mockReturnValue({
        tabs,
        activeTabId: 'tab-25',
        setActiveTab: mockSetActiveTab,
      });

      render(<TabBar />);

      expect(screen.getByTestId('tab-tab-0')).toBeInTheDocument();
      expect(screen.getByTestId('tab-tab-25')).toBeInTheDocument();
      expect(screen.getByTestId('tab-tab-49')).toBeInTheDocument();
    });
  });
});
