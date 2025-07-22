<script lang="ts">
  import { repositories, prompts, currentView } from '../stores';

  function navigateTo(view: 'repositories' | 'settings') {
    currentView.set(view);
  }

  $: runningServers = $repositories.filter((r) => r.mcpServer?.status === 'running');
</script>

<div class="space-y-8">
  <!-- ヘッダー -->
  <div class="text-center">
    <h1 class="text-4xl font-bold text-gray-900 mb-4">📚 librarian</h1>
    <p class="text-xl text-gray-600">MCP Agent Library Manager</p>
    <p class="text-gray-500 mt-2">AI エージェント用プロンプトライブラリの管理ツール</p>
  </div>

  <!-- 統計情報 -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
      <div class="flex items-center">
        <div class="flex-shrink-0">
          <div class="text-2xl">📁</div>
        </div>
        <div class="ml-4">
          <p class="text-sm font-medium text-gray-500">登録リポジトリ</p>
          <p class="text-2xl font-semibold text-gray-900">{$repositories.length}</p>
        </div>
      </div>
    </div>

    <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
      <div class="flex items-center">
        <div class="flex-shrink-0">
          <div class="text-2xl">📝</div>
        </div>
        <div class="ml-4">
          <p class="text-sm font-medium text-gray-500">利用可能プロンプト</p>
          <p class="text-2xl font-semibold text-gray-900">{$prompts.length}</p>
        </div>
      </div>
    </div>

    <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
      <div class="flex items-center">
        <div class="flex-shrink-0">
          <div class="text-2xl">{runningServers.length > 0 ? '🟢' : '🔴'}</div>
        </div>
        <div class="ml-4">
          <p class="text-sm font-medium text-gray-500">実行中MCPサーバー</p>
          <p class="text-2xl font-semibold text-gray-900">{runningServers.length}</p>
        </div>
      </div>
    </div>
  </div>

  <!-- 実行中サーバー情報 -->
  {#if runningServers.length > 0}
    <div class="bg-green-50 p-6 rounded-lg border border-green-200">
      <h2 class="text-lg font-medium text-green-900 mb-4">🟢 実行中MCPサーバー</h2>
      <div class="space-y-3">
        {#each runningServers as repo}
          <div class="flex items-center justify-between bg-white p-3 rounded border">
            <div>
              <span class="font-medium text-green-900">{repo.name}</span>
              <span class="text-sm text-green-700 ml-2">Port {repo.mcpServer?.port}</span>
            </div>
            <span class="bg-green-100 text-green-800 px-2 py-1 rounded text-sm">運用中</span>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="bg-gray-50 p-6 rounded-lg border border-gray-200">
      <h2 class="text-lg font-medium text-gray-900 mb-4">📌 まずはじめに</h2>
      <div class="space-y-4">
        <p class="text-gray-600">MCPサーバーが起動していません。以下の手順で開始してください：</p>
        <div class="flex flex-col sm:flex-row gap-4">
          <button
            onclick={() => navigateTo('repositories')}
            class="flex-1 bg-blue-600 text-white px-4 py-3 rounded-lg hover:bg-blue-700 transition-colors font-medium"
          >
            📁 リポジトリを登録
          </button>
          <button
            onclick={() => navigateTo('settings')}
            class="flex-1 bg-green-600 text-white px-4 py-3 rounded-lg hover:bg-green-700 transition-colors font-medium"
          >
            🔗 接続ガイドを見る
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- 開始ガイド -->
  <div class="bg-blue-50 p-6 rounded-lg border border-blue-200">
    <h2 class="text-lg font-medium text-blue-900 mb-4">🚀 使用手順</h2>
    <div class="space-y-3 text-blue-800">
      <div class="flex items-start space-x-3">
        <span
          class="flex-shrink-0 w-6 h-6 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-bold"
          >1</span
        >
        <p><strong>リポジトリを登録</strong> - .agent_library フォルダを含むGitリポジトリを追加</p>
      </div>
      <div class="flex items-start space-x-3">
        <span
          class="flex-shrink-0 w-6 h-6 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-bold"
          >2</span
        >
        <p>
          <strong>MCPサーバーを起動</strong> - リポジトリページで各リポジトリのMCPサーバーを個別に起動
        </p>
      </div>
      <div class="flex items-start space-x-3">
        <span
          class="flex-shrink-0 w-6 h-6 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-bold"
          >3</span
        >
        <p>
          <strong>AIツールに接続</strong> - 接続ガイドを参考にClaude Code等のツールでMCPサーバーに接続
        </p>
      </div>
      <div class="flex items-start space-x-3">
        <span
          class="flex-shrink-0 w-6 h-6 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-bold"
          >4</span
        >
        <p>
          <strong>プロンプトを活用</strong> - AIツールから登録されたプロンプトライブラリにアクセス
        </p>
      </div>
    </div>
  </div>
</div>
