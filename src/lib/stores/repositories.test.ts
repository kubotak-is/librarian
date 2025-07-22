import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { repositories, addRepository, removeRepository, updateRepository } from './repositories';
import type { Repository } from '../types';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

describe('repositories store', () => {
  const mockRepository: Repository = {
    id: 'test-repo',
    name: 'Test Repository',
    path: '/test/path',
    createdAt: '2023-01-01T00:00:00Z',
    lastUpdated: '2023-01-01T00:00:00Z',
    mcpServer: {
      port: 9500,
      status: 'stopped',
    },
  };

  beforeEach(() => {
    // Reset store
    repositories.set([]);
    vi.clearAllMocks();
  });

  it('should initialize with empty array', () => {
    expect(get(repositories)).toEqual([]);
  });

  it('should add repository successfully', async () => {
    mockInvoke.mockResolvedValue(undefined);

    await addRepository(mockRepository);

    const currentRepos = get(repositories);
    expect(currentRepos).toHaveLength(1);
    expect(currentRepos[0]).toEqual(mockRepository);
    expect(mockInvoke).toHaveBeenCalledWith('add_repository_config', {
      repository: mockRepository,
    });
  });

  it('should handle add repository error', async () => {
    const errorMessage = 'Failed to add repository';
    mockInvoke.mockRejectedValue(new Error(errorMessage));

    await expect(addRepository(mockRepository)).rejects.toThrow(errorMessage);
    expect(get(repositories)).toHaveLength(0);
  });

  it('should remove repository successfully', async () => {
    // Setup initial state
    repositories.set([mockRepository]);
    mockInvoke.mockResolvedValue(true);

    await removeRepository('test-repo');

    expect(get(repositories)).toHaveLength(0);
    expect(mockInvoke).toHaveBeenCalledWith('remove_repository_config', {
      repositoryId: 'test-repo',
    });
  });

  it('should handle remove repository when not found', async () => {
    repositories.set([mockRepository]);
    mockInvoke.mockResolvedValue(false);

    await removeRepository('nonexistent-repo');

    // Repository should still be there since removal failed
    expect(get(repositories)).toHaveLength(1);
  });

  it('should update repository successfully', async () => {
    repositories.set([mockRepository]);
    mockInvoke.mockResolvedValue(undefined);

    const updates = { name: 'Updated Repository Name' };
    await updateRepository('test-repo', updates);

    const currentRepos = get(repositories);
    expect(currentRepos[0].name).toBe('Updated Repository Name');
    expect(mockInvoke).toHaveBeenCalledWith('add_repository_config', {
      repository: expect.objectContaining(updates),
    });
  });

  it('should handle update repository when not found', async () => {
    repositories.set([mockRepository]);

    await expect(updateRepository('nonexistent-repo', { name: 'New Name' })).rejects.toThrow(
      'Repository not found: nonexistent-repo'
    );
  });

  it('should update MCP server status', async () => {
    repositories.set([mockRepository]);
    mockInvoke.mockResolvedValue(undefined);

    await updateRepository('test-repo', {
      mcpServer: { port: 9501, status: 'running' },
    });

    const currentRepos = get(repositories);
    expect(currentRepos[0].mcpServer?.status).toBe('running');
    expect(currentRepos[0].mcpServer?.port).toBe(9501);
  });
});
