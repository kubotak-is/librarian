<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Sidebar from './Sidebar.svelte';
  import Toolbar from './Toolbar.svelte';
  import MainContent from './MainContent.svelte';
  import { appConfig } from '../stores/config';
  import { selectedRepository, setSelectedRepository, currentView } from '../stores/app';
  import type { RepositoryConfig } from '../stores/config';
  import {
    loadInitialConfig,
    handleRepositorySelect,
    handleAddRepository,
    performPeriodicSync,
  } from './Layout.svelte.ts';

  let sidebarWidth = $state(240); // デフォルト幅
  let isResizing = $state(false);
  let startX = $state(0);
  let startWidth = $state(0);

  // 中央管理する状態 ($stateで明示的にリアクティブに)
  let repositories = $state<RepositoryConfig[]>([]);
  let currentSelectedRepository = $state<RepositoryConfig | null>(null);
  let currentViewValue = $state('prompts');

  // 定期同期タイマー
  let syncInterval: ReturnType<typeof setInterval> | null = null;

  // アプリケーション初期化と状態管理
  onMount(async () => {
    try {
      // 初期設定読み込み
      await initializeConfig();

      // 定期同期開始（Tauriサーバー側の状態を定期的にチェック）
      startPeriodicSync();

      // appConfigストアの変更を監視
      const unsubscribeConfig = appConfig.subscribe((config) => {
        repositories = [...config.repositories];

        // 選択されたリポジトリが更新された場合、最新情報に同期
        if (currentSelectedRepository) {
          const updatedRepo = config.repositories.find(
            (r) => r.id === currentSelectedRepository!.id
          );
          if (updatedRepo) {
            currentSelectedRepository = updatedRepo;
          }
        }
      });

      // selectedRepositoryストアの変更を監視
      const unsubscribeSelected = selectedRepository.subscribe((repo) => {
        currentSelectedRepository = repo;
      });

      // currentViewストアの変更を監視
      const unsubscribeView = currentView.subscribe((view) => {
        currentViewValue = view;
      });

      // クリーンアップ関数を登録
      onDestroy(() => {
        unsubscribeConfig();
        unsubscribeSelected();
        unsubscribeView();
      });
    } catch (error) {
      console.error('Failed to initialize application:', error);
    }
  });

  onDestroy(() => {
    if (syncInterval) {
      clearInterval(syncInterval);
    }
  });

  // 初期設定読み込み
  async function initializeConfig() {
    try {
      const { repositories: updatedRepositories, defaultRepository } = await loadInitialConfig();
      repositories = updatedRepositories;

      if (defaultRepository) {
        await setSelectedRepository(defaultRepository);
      }
    } catch (error) {
      console.error('Failed to load initial configuration:', error);
    } finally {
      // 処理完了
    }
  }

  // 定期同期開始
  function startPeriodicSync() {
    // 30秒ごとに実行中のサーバー状態をチェック
    syncInterval = setInterval(async () => {
      try {
        repositories = await performPeriodicSync(repositories);
      } catch (error) {
        console.error('Failed to perform periodic sync:', error);
      }
    }, 30000);
  }

  // リポジトリ選択処理（Tauriサーバーとの同期付き）
  async function selectRepository(repository: RepositoryConfig) {
    try {
      const result = await handleRepositorySelect(repository);
      repositories = result.repositories;
    } catch (error) {
      console.error('Failed to select repository:', error);
    }
  }

  // リポジトリ追加処理（Layoutで管理）
  async function addRepository() {
    try {
      const result = await handleAddRepository();
      if (result.repositories.length > 0) {
        repositories = result.repositories;
      }

      if (result.newRepository) {
        const { notification } = await import('../stores');
        notification.set({
          type: 'success',
          message: `リポジトリ "${result.newRepository.name}" を追加しました`,
        });
      }
    } catch (error) {
      console.error('Failed to add repository:', error);
      const { notification } = await import('../stores');
      notification.set({
        type: 'error',
        message: `リポジトリの追加に失敗しました: ${error}`,
      });
    }
  }

  // リサイズハンドラー
  function startResize(e: MouseEvent) {
    isResizing = true;
    startX = e.clientX;
    startWidth = sidebarWidth;
    document.body.classList.add('resizing');
    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    e.preventDefault();
  }

  function handleResize(e: MouseEvent) {
    if (!isResizing) return;

    const deltaX = e.clientX - startX;
    const newWidth = startWidth + deltaX;

    // 最小幅200px、最大幅400px
    sidebarWidth = Math.max(200, Math.min(400, newWidth));
  }

  function stopResize() {
    isResizing = false;
    document.body.classList.remove('resizing');
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
  }
</script>

<!-- メインレイアウト: 逆L字構造 -->
<div class="app-container h-screen flex flex-col bg-gray-800 text-gray-100">
  <!-- 上部ツールバー（Traffic Lightボタン付き） -->
  <div class="bg-gray-700/80 backdrop-blur-sm border-b border-gray-600/30 h-14 flex-shrink-0">
    <Toolbar />
  </div>

  <!-- メインコンテンツエリア -->
  <div class="flex flex-1 overflow-hidden">
    <!-- 左側サイドバー -->
    <div
      class="bg-gray-900/90 backdrop-blur-sm border-r border-gray-600/30 flex-shrink-0 relative"
      style="width: {sidebarWidth}px;"
    >
      <Sidebar
        {repositories}
        selectedRepository={currentSelectedRepository}
        onRepositorySelect={selectRepository}
        onAddRepository={addRepository}
      />

      <!-- リサイズハンドル -->
      <div class="resize-handle" onmousedown={startResize} role="button" tabindex="-1"></div>
    </div>

    <!-- 右側メインコンテンツ -->
    <div class="bg-gray-800/95 backdrop-blur-sm flex-1 overflow-hidden">
      <MainContent selectedRepository={currentSelectedRepository} currentView={currentViewValue} />
    </div>
  </div>
</div>

<style>
  .app-container {
    height: 100vh;
    overflow: hidden;
    background: var(--bg-primary);
  }

  :global(.h-screen) {
    height: 100vh;
  }

  .resize-handle {
    position: absolute;
    right: -2px;
    top: 0;
    width: 4px;
    height: 100%;
    cursor: col-resize;
    background: transparent;
    z-index: 10;
  }

  .resize-handle:hover {
    background: #3b82f6;
    opacity: 0.6;
  }

  :global(body) {
    user-select: none;
  }

  :global(body.resizing) {
    cursor: col-resize;
  }
</style>
