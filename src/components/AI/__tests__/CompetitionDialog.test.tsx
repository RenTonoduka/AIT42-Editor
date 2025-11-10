/**
 * CompetitionDialog - Error Handling Tests
 *
 * Tests for timeout error handling and fallback behavior
 */

import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { CompetitionDialog } from '../CompetitionDialog';
import * as tauriApiModule from '@/services/tauri';

// Mock Tauri API
jest.mock('@/services/tauri', () => ({
  tauriApi: {
    analyzeTaskWithClaudeCode: jest.fn(),
    executeMultiRuntimeCompetition: jest.fn(),
    getWorkspace: jest.fn(),
  },
}));

describe('CompetitionDialog - Error Handling', () => {
  const mockOnClose = jest.fn();
  const mockOnStart = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();

    // Default workspace response
    (tauriApiModule.tauriApi.getWorkspace as jest.Mock).mockResolvedValue({
      path: '/test/project',
      is_git_repo: true,
    });
  });

  it('should handle analysis timeout error gracefully', async () => {
    // Simulate timeout error
    (tauriApiModule.tauriApi.analyzeTaskWithClaudeCode as jest.Mock).mockRejectedValue(
      new Error('Request timed out after 120 seconds')
    );

    const user = userEvent.setup();

    render(
      <CompetitionDialog
        isOpen={true}
        onClose={mockOnClose}
        onStart={mockOnStart}
      />
    );

    // Type task description
    const textarea = screen.getByPlaceholderText(/各ランタイムに実行させるタスク/);
    await user.type(textarea, 'AIT42最適化したいです');

    // Wait for debounced analysis to trigger
    await waitFor(
      () => {
        expect(tauriApiModule.tauriApi.analyzeTaskWithClaudeCode).toHaveBeenCalled();
      },
      { timeout: 3000 }
    );

    // Wait for error state to appear
    await waitFor(
      () => {
        expect(screen.getByText(/自動分析失敗/)).toBeInTheDocument();
      },
      { timeout: 2000 }
    );

    // Verify error message is displayed
    expect(screen.getByText(/Request timed out after 120 seconds/)).toBeInTheDocument();

    // Verify fallback message is shown
    expect(
      screen.getByText(/手動でランタイム配分を設定してCompetitionを開始できます/)
    ).toBeInTheDocument();
  });

  it('should allow starting competition even when analysis fails', async () => {
    // Simulate timeout error
    (tauriApiModule.tauriApi.analyzeTaskWithClaudeCode as jest.Mock).mockRejectedValue(
      new Error('Request timed out after 120 seconds')
    );

    (tauriApiModule.tauriApi.executeMultiRuntimeCompetition as jest.Mock).mockResolvedValue({
      competitionId: 'test-competition-123',
      instanceCount: 3,
      worktreePaths: [],
    });

    const user = userEvent.setup();

    render(
      <CompetitionDialog
        isOpen={true}
        onClose={mockOnClose}
        onStart={mockOnStart}
      />
    );

    // Type task description
    const textarea = screen.getByPlaceholderText(/各ランタイムに実行させるタスク/);
    await user.type(textarea, 'AIT42最適化したいです');

    // Wait for error state
    await waitFor(
      () => {
        expect(screen.getByText(/自動分析失敗/)).toBeInTheDocument();
      },
      { timeout: 3000 }
    );

    // Start competition despite analysis error
    const startButton = screen.getByRole('button', { name: /コンペティション開始/ });
    expect(startButton).not.toBeDisabled();

    await user.click(startButton);

    // Verify competition was started
    await waitFor(() => {
      expect(tauriApiModule.tauriApi.executeMultiRuntimeCompetition).toHaveBeenCalled();
    });

    expect(mockOnStart).toHaveBeenCalledWith(
      'test-competition-123',
      expect.any(Array),
      'AIT42最適化したいです'
    );
  });

  it('should allow closing dialog during analysis', async () => {
    // Simulate long-running analysis
    (tauriApiModule.tauriApi.analyzeTaskWithClaudeCode as jest.Mock).mockImplementation(
      () => new Promise((resolve) => setTimeout(resolve, 5000))
    );

    const user = userEvent.setup();

    render(
      <CompetitionDialog
        isOpen={true}
        onClose={mockOnClose}
        onStart={mockOnStart}
      />
    );

    // Type task description to trigger analysis
    const textarea = screen.getByPlaceholderText(/各ランタイムに実行させるタスク/);
    await user.type(textarea, 'Long running task');

    // Wait for analysis to start
    await waitFor(() => {
      expect(screen.getByText(/Claude Codeがタスクを分析中/)).toBeInTheDocument();
    });

    // Close dialog during analysis
    const closeButton = screen.getByTitle(/閉じる/);
    await user.click(closeButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it('should display success message when analysis completes', async () => {
    // Simulate successful analysis
    (tauriApiModule.tauriApi.analyzeTaskWithClaudeCode as jest.Mock).mockResolvedValue({
      complexityClass: 'Linear',
      recommendedSubtasks: 5,
      recommendedInstances: 3,
      confidence: 0.85,
      reasoning: 'This is a standard CRUD operation with moderate complexity',
    });

    const user = userEvent.setup();

    render(
      <CompetitionDialog
        isOpen={true}
        onClose={mockOnClose}
        onStart={mockOnStart}
      />
    );

    // Type task description
    const textarea = screen.getByPlaceholderText(/各ランタイムに実行させるタスク/);
    await user.type(textarea, 'Implement user authentication');

    // Wait for success state
    await waitFor(
      () => {
        expect(screen.getByText(/分析完了/)).toBeInTheDocument();
      },
      { timeout: 3000 }
    );

    // Verify success message
    expect(screen.getByText(/Linear 複雑度/)).toBeInTheDocument();
    expect(screen.getByText(/推奨インスタンス数: 3/)).toBeInTheDocument();
  });

  it('should not crash on empty error message', async () => {
    // Simulate error with no message
    (tauriApiModule.tauriApi.analyzeTaskWithClaudeCode as jest.Mock).mockRejectedValue(
      new Error()
    );

    const user = userEvent.setup();

    render(
      <CompetitionDialog
        isOpen={true}
        onClose={mockOnClose}
        onStart={mockOnStart}
      />
    );

    // Type task description
    const textarea = screen.getByPlaceholderText(/各インスタンスに実行させるタスク/);
    await user.type(textarea, 'Test task');

    // Wait for error state
    await waitFor(
      () => {
        expect(screen.getByText(/自動分析失敗/)).toBeInTheDocument();
      },
      { timeout: 3000 }
    );

    // Should display fallback error message
    expect(
      screen.getByText(/Claude Code分析に失敗しました/)
    ).toBeInTheDocument();
  });
});
