<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { isLoading, notification } from '../stores';
  import {
    ConfigAPI,
    createRepositoryConfig,
    generateRepositoryId,
    appConfig,
  } from '../stores/config';
  import type { RepositoryConfig } from '../stores/config';
  import { FileWatcherAPI, watchedRepositories } from '../stores/fileWatcher';
  import { onMount, onDestroy } from 'svelte';

  let repositories: RepositoryConfig[] = [];

  let newRepoPath = '';
  let showAddForm = false;

  // appConfigストアを購読
  appConfig.subscribe((config) => {
    repositories = [...config.repositories];
  });

  // アプリ起動時に設定を読み込み
  onMount(async () => {
    try {
      // ファイル変更イベントリスナーを初期化
      await FileWatcherAPI.initializeEventListener();

      const config = await ConfigAPI.loadConfig();

      // アクティブなリポジトリの監視を開始
      for (const repoConfig of config.repositories) {
        if (repoConfig.is_active) {
          try {
            await FileWatcherAPI.startWatchingRepository(repoConfig.id, repoConfig.path);
          } catch (error) {
            console.warn(`Failed to start watching repository ${repoConfig.id}:`, error);
          }
        }
      }
    } catch (error) {
      console.error('Failed to load configuration:', error);
    }
  });

  onDestroy(() => {
    // コンポーネント破棄時にイベントリスナーを破棄
    FileWatcherAPI.destroyEventListener();
  });

  async function selectDirectory() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'リポジトリディレクトリを選択',
      });

      if (selected) {
        newRepoPath = selected as string;
      }
    } catch (error) {
      notification.set({
        type: 'error',
        message: `ディレクトリ選択エラー: ${error}`,
      });
    }
  }

  async function addRepository() {
    if (!newRepoPath.trim()) return;

    isLoading.set(true);

    try {
      // agent_libraryが存在するかチェック
      await invoke('parse_agent_library', {
        repoPath: newRepoPath,
      });

      // 成功した場合、設定を永続化
      const repositoryId = generateRepositoryId();
      const repoName = newRepoPath.split('/').pop() || 'Unknown';
      const newRepoConfig = createRepositoryConfig(repositoryId, repoName, newRepoPath);

      await ConfigAPI.addRepository(newRepoConfig);

      // 新しいリポジトリのファイル監視を開始
      await FileWatcherAPI.startWatchingRepository(repositoryId, newRepoPath);

      notification.set({
        type: 'success',
        message: `リポジトリ "${repoName}" を追加しました`,
      });

      newRepoPath = '';
      showAddForm = false;
    } catch (error) {
      notification.set({
        type: 'error',
        message: `リポジトリの追加に失敗しました: ${error}`,
      });
    } finally {
      isLoading.set(false);
    }
  }

  async function removeRepository(id: string) {
    try {
      // ファイル監視を停止
      await FileWatcherAPI.stopWatchingRepository(id);

      const removed = await ConfigAPI.removeRepository(id);
      if (removed) {
        notification.set({
          type: 'info',
          message: 'リポジトリを削除しました',
        });
      }
    } catch (error) {
      notification.set({
        type: 'error',
        message: `リポジトリの削除に失敗しました: ${error}`,
      });
    }
  }

  async function toggleRepository(repo: RepositoryConfig) {
    try {
      await ConfigAPI.toggleRepositoryActive(repo.id);
      const newIsActive = !repo.is_active;

      // ファイル監視の開始/停止
      if (newIsActive) {
        await FileWatcherAPI.startWatchingRepository(repo.id, repo.path);
      } else {
        await FileWatcherAPI.stopWatchingRepository(repo.id);
      }

      // ConfigAPI.toggleRepositoryActiveがappConfigストアを更新するため、手動更新は不要
    } catch (error) {
      notification.set({
        type: 'error',
        message: `リポジトリの状態変更に失敗しました: ${error}`,
      });
    }
  }

  async function startMcpServer(repo: RepositoryConfig) {
    isLoading.set(true);

    try {
      const result = await invoke('start_repository_mcp_server', {
        repositoryId: repo.id,
        repoPath: repo.path,
      });

      // ポート番号を取得（サーバーステータスをチェックして取得）
      const status = await invoke('get_mcp_server_status', { repositoryId: repo.id });
      const port = (status as any).port || 9500;

      // 設定を永続化（これでappConfigストアも自動更新される）
      await ConfigAPI.updateMcpServerStatus(repo.id, port, 'running');

      notification.set({
        type: 'success',
        message: result as string,
      });
    } catch (error) {
      notification.set({
        type: 'error',
        message: `MCPサーバー起動エラー: ${error}`,
      });
    } finally {
      isLoading.set(false);
    }
  }

  async function stopMcpServer(repo: RepositoryConfig) {
    isLoading.set(true);

    try {
      const result = await invoke('stop_repository_mcp_server', {
        repositoryId: repo.id,
      });

      // 設定を永続化（これでappConfigストアも自動更新される）
      await ConfigAPI.updateMcpServerStatus(repo.id, repo.mcp_server?.port || 9500, 'stopped');

      notification.set({
        type: 'success',
        message: result as string,
      });
    } catch (error) {
      notification.set({
        type: 'error',
        message: `MCPサーバー停止エラー: ${error}`,
      });
    } finally {
      isLoading.set(false);
    }
  }
</script>

<div class="space-y-6">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-bold text-gray-900">リポジトリ管理</h2>
    <button
      onclick={() => (showAddForm = !showAddForm)}
      class="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-colors"
    >
      + リポジトリを追加
    </button>
  </div>

  {#if showAddForm}
    <div class="bg-gray-50 p-6 rounded-lg border">
      <h3 class="text-lg font-medium text-gray-900 mb-4">新しいリポジトリを追加</h3>
      <div class="space-y-4">
        <div class="flex gap-4">
          <input
            bind:value={newRepoPath}
            placeholder="リポジトリのパスを入力、または下のボタンで選択..."
            class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent outline-none"
          />
          <button
            onclick={selectDirectory}
            class="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-colors flex items-center gap-2"
          >
            📁 ディレクトリ選択
          </button>
        </div>
        <div class="flex gap-4 justify-end">
          <button
            onclick={() => (showAddForm = false)}
            class="bg-gray-600 text-white px-4 py-2 rounded-md hover:bg-gray-700 transition-colors"
          >
            キャンセル
          </button>
          <button
            onclick={addRepository}
            disabled={!newRepoPath.trim() || $isLoading}
            class="bg-green-600 text-white px-4 py-2 rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {$isLoading ? '追加中...' : '追加'}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <div class="space-y-4">
    {#each repositories as repo (repo.id)}
      <div class="bg-white p-6 rounded-lg border border-gray-200 shadow-sm">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <div class="flex items-center">
              <input
                type="checkbox"
                checked={repo.is_active}
                onchange={() => toggleRepository(repo)}
                class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
            </div>
            <div>
              <h3 class="text-lg font-medium text-gray-900">{repo.name}</h3>
              <p class="text-sm text-gray-500">{repo.path}</p>
              {#if repo.last_updated}
                <p class="text-xs text-gray-400">
                  最終更新: {new Date(repo.last_updated).toLocaleString()}
                </p>
              {/if}
            </div>
          </div>

          <div class="flex items-center space-x-3">
            <span
              class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {repo.is_active
                ? 'bg-green-100 text-green-800'
                : 'bg-gray-100 text-gray-800'}"
            >
              {repo.is_active ? '有効' : '無効'}
            </span>

            {#if repo.mcp_server}
              <span
                class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {repo
                  .mcp_server.status === 'running'
                  ? 'bg-blue-100 text-blue-800'
                  : repo.mcp_server.status === 'error'
                    ? 'bg-red-100 text-red-800'
                    : 'bg-gray-100 text-gray-800'}"
              >
                MCP: {repo.mcp_server.status === 'running'
                  ? `Port ${repo.mcp_server.port}`
                  : repo.mcp_server.status}
              </span>
            {/if}

            <!-- ファイル監視状態表示 -->
            {#if repo.is_active}
              {@const watchedRepo = $watchedRepositories.find((w) => w.repository_id === repo.id)}
              {#if watchedRepo && watchedRepo.is_watching}
                <span
                  class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800"
                  title="ファイル変更を監視中"
                >
                  👁️ 監視中
                </span>
                {#if watchedRepo.last_change}
                  <span
                    class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800"
                    title="最終変更: {new Date(watchedRepo.last_change.timestamp).toLocaleString()}"
                  >
                    ✨ 更新あり
                  </span>
                {/if}
              {/if}
            {/if}

            <div class="flex items-center space-x-2">
              {#if repo.is_active}
                {#if !repo.mcp_server || repo.mcp_server.status === 'stopped'}
                  <button
                    onclick={() => startMcpServer(repo)}
                    disabled={$isLoading}
                    class="text-xs bg-green-600 text-white px-3 py-1 rounded hover:bg-green-700 disabled:opacity-50 transition-colors"
                    title="MCPサーバーを起動"
                  >
                    ▶️ MCP起動
                  </button>
                {:else if repo.mcp_server.status === 'running'}
                  <button
                    onclick={() => stopMcpServer(repo)}
                    disabled={$isLoading}
                    class="text-xs bg-red-600 text-white px-3 py-1 rounded hover:bg-red-700 disabled:opacity-50 transition-colors"
                    title="MCPサーバーを停止"
                  >
                    ⏹️ MCP停止
                  </button>
                {/if}
              {/if}

              <button
                onclick={() => removeRepository(repo.id)}
                class="text-red-600 hover:text-red-800 p-1 rounded transition-colors"
                title="リポジトリを削除"
              >
                🗑️
              </button>
            </div>
          </div>
        </div>
      </div>
    {:else}
      <div class="text-center py-12">
        <div class="text-gray-400 text-lg mb-2">📁</div>
        <p class="text-gray-500">まだリポジトリが登録されていません</p>
        <p class="text-sm text-gray-400">「+ リポジトリを追加」ボタンから開始してください</p>
      </div>
    {/each}
  </div>
</div>
