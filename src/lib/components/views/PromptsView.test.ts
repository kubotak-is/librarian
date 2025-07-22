import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import PromptsView from './PromptsView.svelte';
import type { Repository, Prompt } from '../../types';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

describe('PromptsView Component', () => {
  const mockRepository: Repository = {
    id: 'test-repo',
    name: 'Test Repository',
    path: '/test/path',
    createdAt: '2023-01-01T00:00:00Z',
    lastUpdated: '2023-01-01T00:00:00Z',
    mcpServer: {
      port: 9500,
      status: 'running',
    },
  };

  const mockPrompts: Prompt[] = [
    {
      id: 'prompt1',
      title: 'Test Prompt 1',
      description: 'First test prompt',
      content: '# Test Prompt 1\n\nThis is the first test prompt.',
      filePath: '/test/prompt1.md',
    },
    {
      id: 'prompt2',
      title: 'Test Prompt 2',
      description: 'Second test prompt',
      content: '# Test Prompt 2\n\nThis is the second test prompt.',
      filePath: '/test/prompt2.md',
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue({ prompts: mockPrompts });
  });

  it('should render repository name', async () => {
    render(PromptsView, { props: { repository: mockRepository } });
    expect(screen.getByText('Test Repository')).toBeInTheDocument();
  });

  it('should load and display prompts', async () => {
    render(PromptsView, { props: { repository: mockRepository } });

    await waitFor(() => {
      expect(screen.getByText('Test Prompt 1')).toBeInTheDocument();
      expect(screen.getByText('Test Prompt 2')).toBeInTheDocument();
    });

    expect(mockInvoke).toHaveBeenCalledWith('parse_agent_library', {
      repoPath: '/test/path',
    });
  });

  it('should filter prompts based on search query', async () => {
    render(PromptsView, { props: { repository: mockRepository } });

    await waitFor(() => {
      expect(screen.getByText('Test Prompt 1')).toBeInTheDocument();
    });

    const searchInput = screen.getByPlaceholderText('プロンプトを検索...');
    await fireEvent.input(searchInput, { target: { value: 'First' } });

    // Wait for debounced search
    await waitFor(
      () => {
        expect(screen.getByText('Test Prompt 1')).toBeInTheDocument();
        expect(screen.queryByText('Test Prompt 2')).not.toBeInTheDocument();
      },
      { timeout: 500 }
    );
  });

  it('should enter edit mode when edit button is clicked', async () => {
    render(PromptsView, { props: { repository: mockRepository } });

    await waitFor(() => {
      expect(screen.getByText('Test Prompt 1')).toBeInTheDocument();
    });

    const prompt1Button = screen.getByText('Test Prompt 1');
    await fireEvent.click(prompt1Button);

    const editButton = screen.getByText('編集');
    await fireEvent.click(editButton);

    expect(screen.getByText('保存')).toBeInTheDocument();
    expect(screen.getByText('キャンセル')).toBeInTheDocument();
  });

  it('should save prompt when save button is clicked', async () => {
    mockInvoke.mockResolvedValueOnce({ prompts: mockPrompts }).mockResolvedValueOnce(undefined); // For save operation

    render(PromptsView, { props: { repository: mockRepository } });

    await waitFor(() => {
      expect(screen.getByText('Test Prompt 1')).toBeInTheDocument();
    });

    // Select prompt and enter edit mode
    const prompt1Button = screen.getByText('Test Prompt 1');
    await fireEvent.click(prompt1Button);

    const editButton = screen.getByText('編集');
    await fireEvent.click(editButton);

    // Modify content
    const textarea = screen.getByDisplayValue('# Test Prompt 1\n\nThis is the first test prompt.');
    await fireEvent.input(textarea, {
      target: { value: '# Modified Prompt\n\nThis is modified content.' },
    });

    // Save
    const saveButton = screen.getByText('保存');
    await fireEvent.click(saveButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('save_prompt_file', {
        repoPath: '/test/path',
        promptId: 'prompt1',
        content: '# Modified Prompt\n\nThis is modified content.',
      });
    });
  });

  it('should cancel edit mode when cancel button is clicked', async () => {
    render(PromptsView, { props: { repository: mockRepository } });

    await waitFor(() => {
      expect(screen.getByText('Test Prompt 1')).toBeInTheDocument();
    });

    // Enter edit mode
    const prompt1Button = screen.getByText('Test Prompt 1');
    await fireEvent.click(prompt1Button);

    const editButton = screen.getByText('編集');
    await fireEvent.click(editButton);

    // Cancel
    const cancelButton = screen.getByText('キャンセル');
    await fireEvent.click(cancelButton);

    expect(screen.getByText('編集')).toBeInTheDocument();
    expect(screen.queryByText('保存')).not.toBeInTheDocument();
  });

  it('should display error message when loading fails', async () => {
    const errorMessage = 'Failed to load prompts';
    mockInvoke.mockRejectedValue(new Error(errorMessage));

    render(PromptsView, { props: { repository: mockRepository } });

    await waitFor(() => {
      expect(screen.getByText(/エラーが発生しました/)).toBeInTheDocument();
    });
  });

  it('should show empty state when no prompts exist', async () => {
    mockInvoke.mockResolvedValue({ prompts: [] });

    render(PromptsView, { props: { repository: mockRepository } });

    await waitFor(() => {
      expect(screen.getByText(/プロンプトが見つかりません/)).toBeInTheDocument();
    });
  });
});
