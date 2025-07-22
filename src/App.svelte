<script lang="ts">
  import Layout from './lib/components/Layout.svelte';
  import Notification from './lib/components/Notification.svelte';
  import { FileWatcherAPI } from './lib/stores/fileWatcher';
  import { onMount } from 'svelte';

  // アプリケーション初期化
  onMount(async () => {
    // ファイル監視のイベントリスナーを初期化
    await FileWatcherAPI.initializeEventListener();
  });
</script>

<!-- 新しいレイアウト -->
<Layout />

<!-- 通知システム -->
<Notification />
