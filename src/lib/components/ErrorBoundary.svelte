<script lang="ts">
  import { IconAlertTriangle as AlertTriangle, IconRefresh as Refresh } from '@tabler/icons-svelte';

  export let fallback = '予期しないエラーが発生しました';
  export let showRetry = true;
  export let onRetry: (() => void) | null = null;

  let error: Error | null = null;
  let errorDetails = '';
  let showDetails = false;

  function handleError(event: ErrorEvent) {
    error = new Error(event.message);
    errorDetails = `
エラーメッセージ: ${event.message}
ファイル: ${event.filename}
行番号: ${event.lineno}
カラム: ${event.colno}
スタックトレース: ${event.error?.stack || 'なし'}
    `.trim();
    console.error('Uncaught error caught by ErrorBoundary:', error);
  }

  function handleUnhandledRejection(event: PromiseRejectionEvent) {
    error = new Error(`Promise rejection: ${event.reason}`);
    errorDetails = `
エラーメッセージ: ${event.reason}
タイプ: Promise Rejection
タイムスタンプ: ${new Date().toISOString()}
    `.trim();
    console.error('Unhandled promise rejection caught by ErrorBoundary:', event.reason);
  }

  function retry() {
    error = null;
    errorDetails = '';
    showDetails = false;
    if (onRetry) {
      onRetry();
    } else {
      // デフォルトの再試行: ページリロード
      window.location.reload();
    }
  }

  function toggleDetails() {
    showDetails = !showDetails;
  }
</script>

<svelte:window
  on:error={(e) => handleError(e as unknown as ErrorEvent)}
  on:unhandledrejection={(e) => handleUnhandledRejection(e as unknown as PromiseRejectionEvent)}
/>

{#if error}
  <div
    class="error-boundary fixed inset-0 bg-gray-900 bg-opacity-90 flex items-center justify-center z-50"
  >
    <div class="bg-gray-800 border border-red-500 rounded-lg p-6 max-w-md w-full mx-4">
      <!-- エラーアイコンとタイトル -->
      <div class="flex items-center space-x-3 mb-4">
        <AlertTriangle size={24} class="text-red-500" />
        <h2 class="text-lg font-semibold text-gray-100">エラーが発生しました</h2>
      </div>

      <!-- エラーメッセージ -->
      <p class="text-gray-300 mb-4">{fallback}</p>

      <!-- アクションボタン -->
      <div class="flex space-x-3 mb-4">
        {#if showRetry}
          <button
            class="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors duration-150"
            onclick={retry}
          >
            <Refresh size={16} />
            <span>再試行</span>
          </button>
        {/if}

        <button
          class="px-4 py-2 bg-gray-600 text-gray-100 rounded-md hover:bg-gray-700 transition-colors duration-150"
          onclick={toggleDetails}
        >
          {showDetails ? '詳細を隠す' : '詳細を表示'}
        </button>
      </div>

      <!-- エラー詳細 -->
      {#if showDetails}
        <div class="bg-gray-900 border border-gray-600 rounded p-3">
          <h4 class="text-sm font-medium text-gray-100 mb-2">エラー詳細:</h4>
          <pre class="text-xs text-gray-300 whitespace-pre-wrap break-words">{errorDetails}</pre>
        </div>
      {/if}

      <!-- フッター -->
      <div class="mt-4 pt-4 border-t border-gray-600">
        <p class="text-xs text-gray-400">
          この問題が続く場合は、アプリケーションを再起動してください。
        </p>
      </div>
    </div>
  </div>
{:else}
  <slot />
{/if}

<style>
  .error-boundary {
    backdrop-filter: blur(4px);
  }
</style>
