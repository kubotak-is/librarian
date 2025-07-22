import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  loadInitialConfig,
  syncRepositoryStateFromTauri,
  handleRepositorySelect,
  handleAddRepository,
  performPeriodicSync,
} from './Layout.svelte.ts';
import type { RepositoryConfig } from '../stores/config';

// モックの設定
vi.mock('../stores/config', () => ({
  ConfigAPI: {
    loadConfig: vi.fn(),
    getCurrentConfig: vi.fn(),
    updateMcpServerStatus: vi.fn(),
    validateAgentLibrary: vi.fn(),
    addRepository: vi.fn(),
    updateLastOpenedRepository: vi.fn(),
  },
}));

vi.mock('../stores/app', () => ({
  setSelectedRepository: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

import { ConfigAPI } from '../stores/config';
import { setSelectedRepository } from '../stores/app';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

describe('Layout.svelte.ts', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  const mockRepository: RepositoryConfig = {
    id: 'repo_1',
    name: 'Test Repository',
    path: '/test/path',
    is_active: true,
    last_updated: '2025-01-01T00:00:00.000Z',
    mcp_server: {
      port: 9500,
      status: 'stopped',
    },
  };

  describe('loadInitialConfig', () => {
    it('全リポジトリの状態を同期して設定を返す', async () => {
      const mockConfig = {
        repositories: [mockRepository],
        last_opened_repository: 'repo_1',
      };

      vi.mocked(ConfigAPI.loadConfig).mockResolvedValue(mockConfig);
      vi.mocked(ConfigAPI.getCurrentConfig).mockResolvedValue(mockConfig);
      vi.mocked(invoke).mockResolvedValue({ port: 9500, status: 'stopped' });

      const result = await loadInitialConfig();

      expect(result.repositories).toEqual([mockRepository]);
      expect(result.defaultRepository).toEqual(mockRepository);
      expect(ConfigAPI.loadConfig).toHaveBeenCalled();
      expect(invoke).toHaveBeenCalledWith('get_mcp_server_status', { repositoryId: 'repo_1' });
    });

    it('デフォルトリポジトリが指定されていない場合は最初のリポジトリを選択', async () => {
      const mockConfig = {
        repositories: [mockRepository],
        last_opened_repository: null,
      };

      vi.mocked(ConfigAPI.loadConfig).mockResolvedValue(mockConfig);
      vi.mocked(ConfigAPI.getCurrentConfig).mockResolvedValue(mockConfig);
      vi.mocked(invoke).mockResolvedValue({ port: 9500, status: 'stopped' });

      const result = await loadInitialConfig();

      expect(result.defaultRepository).toEqual(mockRepository);
    });
  });

  describe('syncRepositoryStateFromTauri', () => {
    it('サーバーが動作している場合は状態を更新', async () => {
      vi.mocked(invoke).mockResolvedValue({ port: 9501, status: 'running' });

      await syncRepositoryStateFromTauri('repo_1');

      expect(invoke).toHaveBeenCalledWith('get_mcp_server_status', { repositoryId: 'repo_1' });
      expect(ConfigAPI.updateMcpServerStatus).toHaveBeenCalledWith('repo_1', 9501, 'running');
    });

    it('サーバーが動作していない場合はstopped状態に設定', async () => {
      vi.mocked(invoke).mockResolvedValue({ port: null, status: null });

      await syncRepositoryStateFromTauri('repo_1');

      expect(ConfigAPI.updateMcpServerStatus).toHaveBeenCalledWith('repo_1', 9500, 'stopped');
    });

    it('エラーが発生した場合はstopped状態に設定', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Connection failed'));

      await syncRepositoryStateFromTauri('repo_1');

      expect(ConfigAPI.updateMcpServerStatus).toHaveBeenCalledWith('repo_1', 9500, 'stopped');
    });
  });

  describe('handleRepositorySelect', () => {
    it('リポジトリを選択して状態を同期', async () => {
      const updatedConfig = {
        repositories: [{ ...mockRepository, mcp_server: { port: 9500, status: 'running' } }],
      };

      vi.mocked(invoke).mockResolvedValue({ port: 9500, status: 'running' });
      vi.mocked(ConfigAPI.getCurrentConfig).mockResolvedValue(updatedConfig);

      const result = await handleRepositorySelect(mockRepository);

      expect(result.repositories).toEqual(updatedConfig.repositories);
      expect(result.updatedRepository).toEqual(updatedConfig.repositories[0]);
      expect(setSelectedRepository).toHaveBeenCalledWith(updatedConfig.repositories[0]);
      expect(ConfigAPI.updateLastOpenedRepository).toHaveBeenCalledWith('repo_1');
    });
  });

  describe('handleAddRepository', () => {
    it('新しいリポジトリを追加', async () => {
      const selectedPath = '/new/repo/path';
      const mockConfig = { repositories: [] };

      vi.mocked(open).mockResolvedValue(selectedPath);
      vi.mocked(ConfigAPI.validateAgentLibrary).mockResolvedValue(true);
      vi.mocked(ConfigAPI.getCurrentConfig).mockResolvedValue(mockConfig);
      vi.mocked(ConfigAPI.loadConfig).mockResolvedValue({ repositories: [] });

      const result = await handleAddRepository();

      expect(open).toHaveBeenCalledWith({
        title: 'リポジトリを選択',
        directory: true,
        multiple: false,
      });
      expect(ConfigAPI.validateAgentLibrary).toHaveBeenCalledWith(selectedPath);
      expect(ConfigAPI.addRepository).toHaveBeenCalled();
      expect(result.newRepository).toBeTruthy();
      expect(result.newRepository?.name).toBe('path');
      expect(result.newRepository?.path).toBe(selectedPath);
    });

    it('.agent_libraryが存在しない場合はエラーを投げる', async () => {
      const selectedPath = '/invalid/repo/path';

      vi.mocked(open).mockResolvedValue(selectedPath);
      vi.mocked(ConfigAPI.validateAgentLibrary).mockResolvedValue(false);

      await expect(handleAddRepository()).rejects.toThrow(
        '.agent_library ディレクトリが見つかりません'
      );
    });

    it('ダイアログがキャンセルされた場合はnullを返す', async () => {
      vi.mocked(open).mockResolvedValue(null);

      const result = await handleAddRepository();

      expect(result.repositories).toEqual([]);
      expect(result.newRepository).toBeNull();
    });
  });

  describe('performPeriodicSync', () => {
    it('実行中のリポジトリのみ同期', async () => {
      const runningRepository = {
        ...mockRepository,
        mcp_server: { port: 9500, status: 'running' as const },
      };
      const stoppedRepository = {
        ...mockRepository,
        id: 'repo_2',
        mcp_server: { port: 9501, status: 'stopped' as const },
      };
      const repositories = [runningRepository, stoppedRepository];
      const updatedConfig = { repositories };

      vi.mocked(invoke).mockResolvedValue({ port: 9500, status: 'running' });
      vi.mocked(ConfigAPI.getCurrentConfig).mockResolvedValue(updatedConfig);

      const result = await performPeriodicSync(repositories);

      expect(invoke).toHaveBeenCalledTimes(1); // 実行中のリポジトリのみ
      expect(invoke).toHaveBeenCalledWith('get_mcp_server_status', { repositoryId: 'repo_1' });
      expect(result).toEqual(repositories);
    });

    it('変更がない場合は元の配列を返す', async () => {
      const repositories = [mockRepository];

      const result = await performPeriodicSync(repositories);

      expect(invoke).not.toHaveBeenCalled();
      expect(result).toBe(repositories);
    });
  });
});
