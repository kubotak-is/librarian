<script lang="ts">
  import {
    IconPlayerPlay as Play,
    IconPlayerStop as Square,
    IconList as List,
    IconSettings as Settings,
    IconServer as Server,
    IconCircleCheck as CircleCheck,
    IconCircleX as CircleX,
    IconLoader as Loader,
  } from '@tabler/icons-svelte';
  import { selectedRepository, currentView } from '../stores/app';
  import { notification } from '../stores';
  import { invoke } from '@tauri-apps/api/core';
  import type { RepositoryConfig } from '../stores/config';
  import { ConfigAPI } from '../stores/config';
  import { onMount } from 'svelte';

  onMount(async () => {
    // ツールバー要素にドラッグイベントを手動追加
    const toolbarElement = document.querySelector('[data-tauri-drag-region]');
    if (toolbarElement) {
      toolbarElement.addEventListener('mousedown', async (e) => {
        // ボタン要素でのクリックは除外
        if (e.target && e.target instanceof Element && e.target.closest('button')) return;

        try {
          const { getCurrentWindow } = await import('@tauri-apps/api/window');
          const appWindow = getCurrentWindow();
          await appWindow.startDragging();
          console.log('Window drag started');
        } catch (error) {
          console.error('Failed to start dragging:', error);
        }
      });
    }
  });

  let currentSelectedRepository = $state<RepositoryConfig | null>(null);
  let currentViewValue = $state('prompts');
  let serverStatus = $state<'stopped' | 'starting' | 'running' | 'error'>('stopped');
  let serverPort = $state<number | null>(null);

  // リアクティブな購読
  selectedRepository.subscribe((value) => {
    currentSelectedRepository = value;
    // リポジトリが変更されたらサーバー状態を更新
    updateServerStatus(value);
  });

  function updateServerStatus(repo: RepositoryConfig | null) {
    if (repo?.mcp_server) {
      serverStatus = repo.mcp_server.status as any;
      serverPort = repo.mcp_server.port;
    } else {
      serverStatus = 'stopped';
      serverPort = null;
    }
  }

  currentView.subscribe((value) => {
    currentViewValue = value;
  });

  // MCPサーバーを起動
  async function startMcpServer() {
    if (!currentSelectedRepository) {
      notification.set({
        type: 'error',
        message: 'リポジトリを選択してください',
      });
      return;
    }

    if (!currentSelectedRepository.mcp_server?.port) {
      notification.set({
        type: 'error',
        message: 'リポジトリにポート番号が設定されていません',
      });
      return;
    }

    try {
      serverStatus = 'starting';
      const fixedPort = currentSelectedRepository.mcp_server.port;

      await invoke<string>('start_repository_mcp_server', {
        repositoryId: currentSelectedRepository.id,
        repoPath: currentSelectedRepository.path,
        port: fixedPort,
      });

      serverStatus = 'running';
      serverPort = fixedPort;

      // リポジトリ状態を更新
      if (currentSelectedRepository) {
        currentSelectedRepository.mcp_server = {
          port: fixedPort,
          status: 'running',
        };
        // ストアを更新
        selectedRepository.set(currentSelectedRepository);

        // 設定ファイルに保存
        await ConfigAPI.updateMcpServerStatus(currentSelectedRepository.id, fixedPort, 'running');
      }

      notification.set({
        type: 'success',
        message: `MCPサーバーがポート${fixedPort}で起動しました`,
      });
    } catch (error) {
      serverStatus = 'error';
      console.error('Failed to start MCP server:', error);
      notification.set({
        type: 'error',
        message: `MCPサーバーの起動に失敗しました: ${error}`,
      });
    }
  }

  // MCPサーバーを停止
  async function stopMcpServer() {
    if (!currentSelectedRepository) return;

    try {
      await invoke<string>('stop_repository_mcp_server', {
        repositoryId: currentSelectedRepository.id,
      });

      serverStatus = 'stopped';
      serverPort = null;

      // リポジトリ状態を更新（ポートは保持）
      if (currentSelectedRepository && currentSelectedRepository.mcp_server) {
        currentSelectedRepository.mcp_server.status = 'stopped';
        // ストアを更新
        selectedRepository.set(currentSelectedRepository);

        // 設定ファイルに保存
        await ConfigAPI.updateMcpServerStatus(
          currentSelectedRepository.id,
          currentSelectedRepository.mcp_server.port,
          'stopped'
        );
      }

      notification.set({
        type: 'info',
        message: 'MCPサーバーを停止しました',
      });
    } catch (error) {
      console.error('Failed to stop MCP server:', error);
      notification.set({
        type: 'error',
        message: `MCPサーバーの停止に失敗しました: ${error}`,
      });
    }
  }

  // ビューを変更
  function changeView(view: 'prompts' | 'settings') {
    currentView.set(view);
  }

  // サーバー状態アイコンを取得
  function getServerStatusIcon() {
    switch (serverStatus) {
      case 'running':
        return CircleCheck;
      case 'starting':
        return Loader;
      case 'error':
        return CircleX;
      default:
        return Server;
    }
  }

  // サーバー状態の色を取得
  function getServerStatusColor() {
    switch (serverStatus) {
      case 'running':
        return 'text-green-500';
      case 'starting':
        return 'text-yellow-500';
      case 'error':
        return 'text-red-500';
      default:
        return 'text-gray-500';
    }
  }
</script>

<!-- ツールバー -->
<div class="h-full flex items-center justify-between px-4" data-tauri-drag-region>
  <!-- 左側: リポジトリ情報 -->
  <div class="flex items-center space-x-3">
    <!-- リポジトリ情報 -->
    <div class="flex items-center space-x-2">
      {#if currentSelectedRepository}
        {#snippet statusIcon()}
          {@const StatusIcon = getServerStatusIcon()}
          <StatusIcon
            size={16}
            class={getServerStatusColor() + (serverStatus === 'starting' ? ' animate-spin' : '')}
          />
        {/snippet}

        {@render statusIcon()}
        <span class="text-sm font-medium text-primary">{currentSelectedRepository.name}</span>
        {#if serverPort}
          <span class="text-xs text-secondary bg-primary px-2 py-1 rounded">:{serverPort}</span>
        {/if}
      {:else}
        <span class="text-sm text-secondary">リポジトリを選択してください</span>
      {/if}
    </div>
  </div>

  <!-- 中央: アクションボタン -->
  <div class="flex items-center space-x-2">
    <!-- MCPサーバー制御 -->
    {#if currentSelectedRepository}
      {#if serverStatus === 'stopped' || serverStatus === 'error'}
        <button
          class="inline-flex items-center space-x-2 px-5 py-2.5 text-sm font-medium rounded-lg transition-colors duration-200
                 {serverStatus === 'error'
            ? 'bg-red-600 text-white border border-red-600 hover:bg-red-700'
            : 'bg-gray-600 text-white border border-gray-600 hover:bg-gray-700'}"
          onclick={startMcpServer}
          disabled={false}
          title="MCPサーバーを起動"
        >
          <Play size={16} />
          <span class="font-medium">Start Server</span>
        </button>
      {:else if serverStatus === 'running'}
        <button
          class="inline-flex items-center space-x-2 px-5 py-2.5 text-sm font-medium rounded-lg transition-colors duration-200
                 bg-green-600 text-white border border-green-600 hover:bg-green-700"
          onclick={stopMcpServer}
          title="MCPサーバーを停止"
        >
          <Square size={16} />
          <span class="font-medium">Stop Server</span>
        </button>
      {:else}
        <button
          class="btn opacity-50 cursor-not-allowed flex items-center space-x-2 px-5 py-2.5"
          disabled
          title="サーバー起動中..."
        >
          <Loader size={16} class="animate-spin" />
          <span class="font-medium">Starting...</span>
        </button>
      {/if}
    {/if}
  </div>

  <!-- 右側: ビュー切り替え -->
  <div class="flex items-center space-x-2">
    <button
      class="flex flex-col items-center justify-center px-4 py-3 font-medium min-w-16 text-center transition-colors duration-200
             {currentViewValue === 'prompts'
        ? 'text-gray-100 bg-gray-600/70'
        : 'text-gray-400 hover:text-gray-100 hover:bg-gray-600/50'}"
      onclick={() => changeView('prompts')}
      title="プロンプト一覧"
    >
      <div class="icon flex justify-center mb-1">
        <List size={16} />
      </div>
      <span class="text-xs">プロンプト</span>
    </button>

    <button
      class="flex flex-col items-center justify-center px-4 py-3 font-medium min-w-16 text-center transition-colors duration-200
             {currentViewValue === 'settings'
        ? 'text-gray-100 bg-gray-600/70'
        : 'text-gray-400 hover:text-gray-100 hover:bg-gray-600/50'}"
      onclick={() => changeView('settings')}
      title="設定・接続ガイド"
    >
      <div class="icon flex justify-center mb-1">
        <Settings size={16} />
      </div>
      <span class="text-xs">設定</span>
    </button>
  </div>
</div>
