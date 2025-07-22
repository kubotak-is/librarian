<script lang="ts">
  import {
    IconSettings as Settings,
    IconServer as Server,
    IconCopy as Copy,
    IconExternalLink as ExternalLink,
    IconTrash as Trash,
  } from '@tabler/icons-svelte';
  import { notification } from '../../stores';
  import { ConfigAPI } from '../../stores/config';
  import { selectedRepository, currentView } from '../../stores/app';
  import type { RepositoryConfig } from '../../stores/config';

  let { repository }: { repository: RepositoryConfig } = $props();

  let activeTab = $state('connection');
  let deletingRepositoryId = $state<string | null>(null);
  let repositoryToDelete = $state<{ id: string; name: string } | null>(null);

  // MCP接続設定の例（リアクティブ）
  let connectionExamples = $derived({
    'claude-code': {
      title: 'Claude Code',
      description: 'Anthropic の Claude Code エディタ用設定',
      config: `{
  "mcpServers": {
    "librarian-${repository.name}": {
      "command": "curl",
      "args": [
        "-X", "POST",
        "-H", "Content-Type: application/json",
        "-d", "@-",
        "http://localhost:${repository.mcp_server?.port || 9500}/"
      ]
    }
  }
}`,
      configFile: '~/.claude-code/mcp-servers.json',
      get oneLiner() {
        return `claude mcp add -t http librarian-${repository.name} http://localhost:${repository.mcp_server?.port || 9500}`;
      },
    },
    cursor: {
      title: 'Cursor',
      description: 'Cursor エディタ用設定',
      config: `{
  "mcp": {
    "servers": {
      "librarian-${repository.name}": {
        "url": "http://localhost:${repository.mcp_server?.port || 9500}",
        "protocol": "json-rpc"
      }
    }
  }
}`,
      configFile: '~/.cursor/mcp-config.json',
    },
    vscode: {
      title: 'Visual Studio Code',
      description: 'VSCode MCP 拡張機能用設定',
      config: `{
  "mcp.servers": [
    {
      "name": "librarian-${repository.name}",
      "url": "http://localhost:${repository.mcp_server?.port || 9500}",
      "enabled": true
    }
  ]
}`,
      configFile: 'settings.json',
    },
  });

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      notification.set({
        type: 'success',
        message: 'クリップボードにコピーしました',
      });
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
      notification.set({
        type: 'error',
        message: 'クリップボードへのコピーに失敗しました',
      });
    }
  }

  function openExternalLink(url: string) {
    // Tauriの場合はshellを使用
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      import('@tauri-apps/plugin-shell').then(({ open }) => {
        open(url);
      });
    } else {
      window.open(url, '_blank');
    }
  }

  // リポジトリ削除を実行
  async function executeRepositoryDeletion() {
    // ボタンクリック時点でのリポジトリ情報を保存
    const currentRepo = { id: repository.id, name: repository.name };

    try {
      // Tauri環境での確認ダイアログ
      let confirmed = false;

      if (typeof window !== 'undefined' && (window as any).__TAURI__) {
        // Tauriの場合はdialog APIを使用
        const { ask } = await import('@tauri-apps/plugin-dialog');
        confirmed = await ask(`リポジトリ "${currentRepo.name}" を削除しますか？`, {
          title: '確認',
          kind: 'warning',
        });
      } else {
        // ブラウザ環境の場合は通常のconfirm
        confirmed = confirm(
          `リポジトリ "${currentRepo.name}" を削除しますか？この操作は元に戻せません。`
        );
      }

      if (!confirmed) {
        return;
      }

      // 確認後の非同期処理
      await performActualDeletion(currentRepo);
    } catch (error) {
      console.error('Failed to show confirmation dialog:', error);
      notification.set({
        type: 'error',
        message: '確認ダイアログでエラーが発生しました',
      });
    }
  }

  // 実際の削除処理（非同期）
  async function performActualDeletion(currentRepo: { id: string; name: string }) {
    repositoryToDelete = currentRepo;
    deletingRepositoryId = currentRepo.id;

    try {
      const success = await ConfigAPI.removeRepository(currentRepo.id);

      if (success) {
        // 設定を再読み込みして最新の状態を取得
        const config = await ConfigAPI.getCurrentConfig();
        const remainingRepos = config.repositories;

        // 選択されたリポジトリをクリアまたは別のリポジトリに変更
        if (remainingRepos.length > 0) {
          selectedRepository.set(remainingRepos[0]);
        } else {
          selectedRepository.set(null);
        }

        notification.set({
          type: 'success',
          message: `リポジトリ "${currentRepo.name}" を削除しました`,
        });

        // プロンプト画面に切り替える
        currentView.set('prompts');
      } else {
        throw new Error('削除処理が失敗しました');
      }
    } catch (error) {
      console.error('Failed to remove repository:', error);
      notification.set({
        type: 'error',
        message: `リポジトリの削除に失敗しました: ${error}`,
      });
    } finally {
      // 削除処理終了
      deletingRepositoryId = null;
      repositoryToDelete = null;
    }
  }
</script>

<!-- 設定ビュー -->
<div class="h-full flex flex-col bg-gray-800">
  <!-- ヘッダー -->
  <div class="p-6 border-b border-gray-600/30">
    <div class="flex items-center space-x-2 mb-2">
      <Settings size={20} class="text-gray-400" />
      <h1 class="text-xl font-semibold text-gray-100">設定</h1>
    </div>
    <p class="text-gray-400">
      {repositoryToDelete ? repositoryToDelete.name : repository.name} の MCP サーバー接続設定とガイド
      {#if repositoryToDelete}
        <span class="text-red-400 ml-2">(削除処理中)</span>
      {/if}
    </p>
  </div>

  <!-- タブ -->
  <div class="border-b border-gray-600/30">
    <div class="flex space-x-1 px-6">
      <button
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab ===
        'connection'
          ? 'border-accent-600 text-accent-600'
          : 'border-transparent text-gray-400 hover:text-gray-100'}"
        onclick={() => (activeTab = 'connection')}
      >
        接続設定
      </button>
      <button
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab ===
        'repository'
          ? 'border-accent-600 text-accent-600'
          : 'border-transparent text-gray-400 hover:text-gray-100'}"
        onclick={() => (activeTab = 'repository')}
      >
        リポジトリ情報
      </button>
    </div>
  </div>

  <!-- コンテンツ -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if activeTab === 'connection'}
      <!-- 接続設定タブ -->
      <div class="space-y-6">
        <!-- サーバー状態 -->
        <div class="bg-gray-700 rounded-lg p-4">
          <h3 class="font-medium text-gray-100 mb-3 flex items-center space-x-2">
            <Server size={16} />
            <span>MCP サーバー状態</span>
          </h3>

          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <div class="block text-gray-400 mb-1">状態</div>
              <span
                class="font-mono px-3 py-1 rounded {repository.mcp_server?.status === 'running'
                  ? 'bg-green-600 text-green-100'
                  : 'bg-gray-600 text-gray-100'}"
              >
                {repository.mcp_server?.status || 'stopped'}
              </span>
            </div>
            <div>
              <div class="block text-gray-400 mb-1">ポート</div>
              <span
                class="font-mono px-3 py-1 rounded bg-gray-800 text-gray-100 border border-gray-600/30"
              >
                {repository.mcp_server?.port || 'N/A'}
              </span>
            </div>
          </div>

          {#if repository.mcp_server?.status === 'running'}
            <div
              class="mt-4 p-3 bg-green-900/30 border border-green-600/30 rounded text-sm text-green-300"
            >
              ✅ MCP サーバーが実行中です。以下の設定を使用してクライアントから接続できます。
            </div>
          {:else}
            <div
              class="mt-4 p-3 bg-yellow-900/30 border border-yellow-600/30 rounded text-sm text-yellow-300"
            >
              ⚠️ MCP サーバーが停止しています。上部のツールバーから起動してください。
            </div>
          {/if}
        </div>

        <!-- 接続設定例 -->
        <div class="space-y-4">
          <h3 class="font-medium text-gray-100">クライアント設定例</h3>

          {#each Object.entries(connectionExamples) as [, example]}
            <div class="bg-gray-700 rounded-lg p-4">
              <div class="flex items-start justify-between mb-3">
                <div>
                  <h4 class="font-medium text-gray-100 flex items-center space-x-2">
                    <span>{example.title}</span>
                  </h4>
                  <p class="text-sm text-gray-400 mt-1">{example.description}</p>
                </div>
                <button
                  class="icon-btn"
                  onclick={() => copyToClipboard(example.config)}
                  title="設定をコピー"
                >
                  <Copy size={16} />
                </button>
              </div>

              <div class="mb-3">
                <div class="block text-xs text-gray-400 mb-2">
                  設定ファイル: <code class="font-mono bg-gray-800 px-1 py-0.5 rounded"
                    >{example.configFile}</code
                  >
                </div>

                {#if 'oneLiner' in example && (example.oneLiner || typeof example.oneLiner === 'string')}
                  <div class="mb-3">
                    <div class="block text-xs text-gray-400 mb-2">ワンライナーコマンド:</div>
                    <div
                      class="bg-green-50 border border-green-200 rounded p-3 text-xs font-mono text-green-800 flex items-center justify-between"
                    >
                      <code
                        >{typeof (example as any).oneLiner === 'function'
                          ? (example as any).oneLiner()
                          : (example as any).oneLiner}</code
                      >
                      <button
                        class="ml-2 text-green-600 hover:text-green-800"
                        onclick={() =>
                          copyToClipboard(
                            typeof (example as any).oneLiner === 'function'
                              ? (example as any).oneLiner()
                              : (example as any).oneLiner
                          )}
                        title="コマンドをコピー"
                      >
                        <Copy size={14} />
                      </button>
                    </div>
                  </div>
                {/if}

                <pre
                  class="bg-gray-800 border border-gray-600/30 rounded p-3 text-xs font-mono text-gray-100 overflow-x-auto">
{example.config}
                </pre>
              </div>
            </div>
          {/each}
        </div>

        <!-- 参考リンク -->
        <div class="bg-gray-700 rounded-lg p-4">
          <h3 class="font-medium text-gray-100 mb-3">参考リンク</h3>
          <div class="space-y-2 text-sm">
            <button
              class="flex items-center space-x-2 text-accent hover:underline"
              onclick={() => openExternalLink('https://spec.modelcontextprotocol.io/')}
            >
              <ExternalLink size={14} />
              <span>Model Context Protocol 仕様</span>
            </button>
            <button
              class="flex items-center space-x-2 text-accent hover:underline"
              onclick={() => openExternalLink('https://github.com/anthropics/mcp')}
            >
              <ExternalLink size={14} />
              <span>MCP GitHub リポジトリ</span>
            </button>
          </div>
        </div>
      </div>
    {:else if activeTab === 'repository'}
      <!-- リポジトリ情報タブ -->
      <div class="space-y-6">
        <div class="bg-gray-700 rounded-lg p-4">
          <h3 class="font-medium text-gray-100 mb-3">リポジトリ詳細</h3>

          <div class="grid grid-cols-1 gap-4 text-sm">
            <div>
              <div class="block text-gray-400 mb-1">名前</div>
              <div class="font-mono p-2 bg-gray-800 border border-gray-600/30 rounded">
                {repository.name}
              </div>
            </div>

            <div>
              <div class="block text-gray-400 mb-1">パス</div>
              <div class="font-mono p-2 bg-gray-800 border border-gray-600/30 rounded break-all">
                {repository.path}
              </div>
            </div>

            <div>
              <div class="block text-gray-400 mb-1">ID</div>
              <div class="font-mono p-2 bg-gray-800 border border-gray-600/30 rounded">
                {repository.id}
              </div>
            </div>

            <div>
              <div class="block text-gray-400 mb-1">最終更新</div>
              <div class="font-mono p-2 bg-gray-800 border border-gray-600/30 rounded">
                {new Date(repository.last_updated).toLocaleString('ja-JP')}
              </div>
            </div>

            <div>
              <div class="block text-gray-400 mb-1">状態</div>
              <span
                class="inline-block px-3 py-1 rounded text-sm {repository.is_active
                  ? 'bg-green-100 text-green-800'
                  : 'bg-gray-100 text-gray-800'}"
              >
                {repository.is_active ? 'アクティブ' : '非アクティブ'}
              </span>
            </div>
          </div>
        </div>

        <!-- 危険な操作 -->
        <div class="bg-red-50 border border-red-200 rounded-lg p-4">
          <h3 class="font-medium text-red-800 mb-3 flex items-center space-x-2">
            <Trash size={16} />
            <span>危険な操作</span>
          </h3>

          <p class="text-sm text-red-700 mb-4">
            このリポジトリを削除します。この操作は元に戻すことができません。
          </p>

          <button
            class="px-4 py-2 rounded-md text-sm font-medium transition-colors duration-150 flex items-center space-x-2 {deletingRepositoryId
              ? 'bg-gray-500 cursor-not-allowed'
              : 'bg-red-600 hover:bg-red-700'} text-white"
            onclick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              if (deletingRepositoryId) {
                return;
              }
              executeRepositoryDeletion();
            }}
            disabled={deletingRepositoryId !== null}
          >
            <Trash size={16} />
            <span>{deletingRepositoryId ? '削除中...' : 'リポジトリを削除'}</span>
          </button>
        </div>

        <!-- .agent_library 情報 -->
        <div class="bg-gray-700 rounded-lg p-4">
          <h3 class="font-medium text-gray-100 mb-3">.agent_library 構造</h3>
          <div class="text-sm text-gray-400 space-y-2">
            <div class="font-mono bg-gray-800 p-3 rounded border border-gray-600/30">
              <div class="text-gray-100">{repository.path}/.agent_library/</div>
              <div class="ml-4 text-gray-400">├── agent_index.yml</div>
              <div class="ml-4 text-gray-400">├── *.md (プロンプトファイル)</div>
              <div class="ml-4 text-gray-400">└── README.md (オプション)</div>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
