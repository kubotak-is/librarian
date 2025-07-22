<script lang="ts">
  import { appConfig } from '../stores/config';

  let selectedTool = 'claude-code';
  let copyStatus = '';

  const tools = [
    { id: 'claude-code', name: 'Claude Code', icon: '🤖' },
    { id: 'cursor', name: 'Cursor', icon: '🎯' },
    { id: 'vscode', name: 'VS Code', icon: '💙' },
    { id: 'cline', name: 'Cline', icon: '🤖' },
  ];

  function getConfigExample(tool: string, repos: any[]) {
    const runningRepos = repos.filter((r) => r.mcp_server?.status === 'running');

    switch (tool) {
      case 'claude-code':
        return {
          title: 'Claude Code設定',
          description: 'Claude Codeでは以下の方法でMCPサーバーを追加できます：',
          config: `方法1: コマンドラインで追加 (推奨)
${runningRepos.map((repo) => `claude mcp add -t http librarian-${repo.name} http://localhost:${repo.mcp_server?.port || 9500}`).join('\n')}

方法2: 設定ファイルに追加 (.mcp.json)
${JSON.stringify(
  runningRepos.reduce((acc, repo) => {
    acc[`librarian-${repo.name}`] = {
      transport: 'http',
      commandOrUrl: `http://localhost:${repo.mcp_server?.port || 9500}`,
    };
    return acc;
  }, {}),
  null,
  2
)}`,
          path: 'コマンド または .mcp.json',
        };

      case 'cursor':
        return {
          title: 'Cursor設定',
          description: 'Cursorの設定で以下のMCPサーバーを追加してください：',
          config: runningRepos
            .map(
              (repo) => `Server Name: librarian-${repo.name}
URL: http://localhost:${repo.mcp_server?.port || 9500}
Type: HTTP MCP Server`
            )
            .join('\n\n'),
          path: 'Cursor Settings > MCP Servers',
        };

      case 'vscode':
        return {
          title: 'VS Code設定',
          description: 'VS CodeのMCP拡張機能で以下を設定してください：',
          config: runningRepos
            .map(
              (repo) => `{
  "name": "librarian-${repo.name}",
  "url": "http://localhost:${repo.mcp_server?.port || 9500}",
  "type": "http"
}`
            )
            .join(',\n'),
          path: 'settings.json > mcp.servers',
        };

      case 'cline':
        return {
          title: 'Cline設定',
          description: 'Clineの設定ファイルに以下を追加してください：',
          config: JSON.stringify(
            {
              mcpServers: runningRepos.reduce((acc, repo) => {
                acc[`librarian-${repo.name}`] = {
                  type: 'http',
                  url: `http://localhost:${repo.mcp_server?.port || 9500}`,
                };
                return acc;
              }, {}),
            },
            null,
            2
          ),
          path: '~/.cline/config.json',
        };

      default:
        return {
          title: '設定例',
          description: '選択されたツールの設定例を表示します',
          config: 'ツールを選択してください',
          path: '',
        };
    }
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copyStatus = '✅ コピーしました！';
      setTimeout(() => (copyStatus = ''), 2000);
    } catch {
      copyStatus = '❌ コピーに失敗しました';
      setTimeout(() => (copyStatus = ''), 2000);
    }
  }

  $: configExample = getConfigExample(selectedTool, $appConfig.repositories);
  $: runningServers = $appConfig.repositories.filter((r) => r.mcp_server?.status === 'running');
</script>

<div class="space-y-6">
  <div class="text-center">
    <h2 class="text-2xl font-bold text-gray-900 mb-4">🔗 MCP接続ガイド</h2>
    <p class="text-gray-600">各種AIツールでlibrarian MCPサーバーに接続する方法</p>
  </div>

  <!-- サーバー状態表示 -->
  <div class="bg-blue-50 p-4 rounded-lg border border-blue-200">
    <h3 class="text-lg font-medium text-blue-900 mb-3">📊 実行中のMCPサーバー</h3>
    {#if runningServers.length > 0}
      <div class="space-y-2">
        {#each runningServers as repo}
          <div class="flex items-center justify-between bg-white p-3 rounded border">
            <div>
              <span class="font-medium">{repo.name}</span>
              <span class="text-sm text-gray-500 ml-2">{repo.path}</span>
            </div>
            <span class="bg-green-100 text-green-800 px-2 py-1 rounded text-sm">
              Port {repo.mcp_server?.port}
            </span>
          </div>
        {/each}
      </div>
    {:else}
      <p class="text-blue-700">
        現在実行中のMCPサーバーはありません。リポジトリページでMCPサーバーを起動してください。
      </p>
    {/if}
  </div>

  <!-- ツール選択 -->
  <div>
    <h3 class="text-lg font-medium text-gray-900 mb-3">🛠️ 使用するツールを選択</h3>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      {#each tools as tool}
        <button
          onclick={() => (selectedTool = tool.id)}
          class="flex flex-col items-center p-4 border-2 rounded-lg transition-colors {selectedTool ===
          tool.id
            ? 'border-blue-500 bg-blue-50'
            : 'border-gray-200 hover:border-gray-300'}"
        >
          <div class="text-2xl mb-2">{tool.icon}</div>
          <span class="text-sm font-medium">{tool.name}</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- 設定例表示 -->
  {#if runningServers.length > 0}
    <div class="bg-white p-6 rounded-lg border border-gray-200">
      <div class="flex justify-between items-start mb-4">
        <div>
          <h3 class="text-lg font-medium text-gray-900">{configExample.title}</h3>
          <p class="text-sm text-gray-600 mt-1">{configExample.description}</p>
          {#if configExample.path}
            <p class="text-xs text-gray-500 mt-1">
              設定ファイル: <code class="bg-gray-100 px-1 rounded">{configExample.path}</code>
            </p>
          {/if}
        </div>
        <button
          onclick={() => copyToClipboard(configExample.config)}
          class="bg-blue-600 text-white px-3 py-1 rounded text-sm hover:bg-blue-700 transition-colors"
        >
          📋 全てコピー
        </button>
      </div>

      <pre
        class="bg-gray-50 p-4 rounded-md text-sm overflow-x-auto whitespace-pre-wrap border">{configExample.config}</pre>

      <!-- Claude Code専用: 個別コマンドコピー -->
      {#if selectedTool === 'claude-code' && runningServers.length > 0}
        <div class="mt-4 border-t pt-4">
          <h4 class="text-sm font-medium text-gray-700 mb-3">
            🚀 個別コマンド実行（クリックでコピー）
          </h4>
          <div class="space-y-2">
            {#each runningServers as repo}
              <div class="flex items-center justify-between bg-blue-50 p-3 rounded border">
                <div class="flex-1">
                  <span class="text-sm font-medium text-blue-900">{repo.name}</span>
                  <code class="block text-xs text-blue-700 mt-1 font-mono">
                    claude mcp add -t http librarian-{repo.name} http://localhost:{repo.mcp_server
                      ?.port}
                  </code>
                </div>
                <button
                  onclick={() =>
                    copyToClipboard(
                      `claude mcp add -t http librarian-${repo.name} http://localhost:${repo.mcp_server?.port}`
                    )}
                  class="bg-blue-600 text-white px-2 py-1 rounded text-xs hover:bg-blue-700 transition-colors ml-3"
                >
                  📋
                </button>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if copyStatus}
        <p class="text-sm mt-2 {copyStatus.includes('✅') ? 'text-green-600' : 'text-red-600'}">
          {copyStatus}
        </p>
      {/if}
    </div>
  {/if}

  <!-- 接続手順 -->
  <div class="bg-green-50 p-6 rounded-lg border border-green-200">
    <h3 class="text-lg font-medium text-green-900 mb-4">📝 接続手順</h3>
    <div class="space-y-3 text-green-800">
      <div class="flex items-start space-x-3">
        <span
          class="flex-shrink-0 w-6 h-6 bg-green-600 text-white rounded-full flex items-center justify-center text-sm font-bold"
          >1</span
        >
        <p>
          <strong>MCPサーバー起動</strong> - リポジトリページで必要なリポジトリのMCPサーバーを起動
        </p>
      </div>
      <div class="flex items-start space-x-3">
        <span
          class="flex-shrink-0 w-6 h-6 bg-green-600 text-white rounded-full flex items-center justify-center text-sm font-bold"
          >2</span
        >
        <div>
          <p><strong>MCPサーバー追加</strong></p>
          <ul class="text-sm mt-1 space-y-1">
            <li>• <strong>Claude Code:</strong> 個別コマンドを実行するか設定ファイルに追加</li>
            <li>• <strong>その他ツール:</strong> 上記の設定例をコピーして設定</li>
          </ul>
        </div>
      </div>
      <div class="flex items-start space-x-3">
        <span
          class="flex-shrink-0 w-6 h-6 bg-green-600 text-white rounded-full flex items-center justify-center text-sm font-bold"
          >3</span
        >
        <p>
          <strong>ツール再起動</strong> - 設定を反映するため、AIツールを再起動（Claude Codeのコマンド追加は即座に反映）
        </p>
      </div>
      <div class="flex items-start space-x-3">
        <span
          class="flex-shrink-0 w-6 h-6 bg-green-600 text-white rounded-full flex items-center justify-center text-sm font-bold"
          >4</span
        >
        <p><strong>接続確認</strong> - ツール内でMCPサーバーが認識されていることを確認</p>
      </div>
    </div>
  </div>

  <!-- 参考リンク -->
  <div class="bg-gray-50 p-6 rounded-lg border border-gray-200">
    <h3 class="text-lg font-medium text-gray-900 mb-4">🔗 参考リンク</h3>
    <div class="space-y-2 text-sm">
      <a
        href="https://modelcontextprotocol.io/"
        target="_blank"
        class="text-blue-600 hover:text-blue-800 block"
      >
        📖 Model Context Protocol 公式ドキュメント
      </a>
      <a
        href="https://docs.anthropic.com/en/docs/build-with-claude/computer-use"
        target="_blank"
        class="text-blue-600 hover:text-blue-800 block"
      >
        🤖 Claude Code MCP設定ガイド
      </a>
      <a
        href="https://docs.cursor.com/"
        target="_blank"
        class="text-blue-600 hover:text-blue-800 block"
      >
        🎯 Cursor ドキュメント
      </a>
      <a
        href="https://code.visualstudio.com/docs"
        target="_blank"
        class="text-blue-600 hover:text-blue-800 block"
      >
        💙 VS Code 拡張機能ガイド
      </a>
    </div>
  </div>
</div>
