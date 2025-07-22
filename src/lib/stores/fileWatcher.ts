import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { notification } from '../stores';

export interface FileChangeEvent {
  repository_id: string;
  file_path: string;
  change_type: 'Created' | 'Modified' | 'Deleted' | 'Renamed';
  timestamp: string;
}

export interface WatchedRepository {
  repository_id: string;
  is_watching: boolean;
  last_change?: FileChangeEvent;
}

// ファイル変更イベントのストア
export const fileChangeEvents = writable<FileChangeEvent[]>([]);

// 監視中のリポジトリのストア
export const watchedRepositories = writable<WatchedRepository[]>([]);

// 自動リロード設定
export const autoReloadEnabled = writable(true);

/**
 * ファイル監視 API
 */
export class FileWatcherAPI {
  private static eventListener: (() => void) | null = null;

  /**
   * ファイル変更イベントリスナーを初期化
   */
  static async initializeEventListener() {
    if (this.eventListener) {
      return; // 既に初期化済み
    }

    try {
      const unlisten = await listen<FileChangeEvent>('file-change', (event) => {
        console.log('File change detected:', event.payload);

        // イベントストアに追加
        fileChangeEvents.update((events) => {
          const newEvents = [event.payload, ...events];
          // 最新100件のみ保持
          return newEvents.slice(0, 100);
        });

        // 監視中リポジトリの最終変更を更新
        watchedRepositories.update((repos) =>
          repos.map((repo) =>
            repo.repository_id === event.payload.repository_id
              ? { ...repo, last_change: event.payload }
              : repo
          )
        );

        // 自動リロードが有効な場合、agent_libraryを再読み込み
        autoReloadEnabled.subscribe((enabled) => {
          if (enabled && this.shouldAutoReload(event.payload)) {
            // ファイルパスからリポジトリパスを推定
            const repositoryPath = event.payload.file_path.split('/.agent_library')[0];
            this.reloadAgentLibrary(event.payload.repository_id, repositoryPath);
          }
        });

        // 通知を表示
        this.showChangeNotification(event.payload);
      });

      this.eventListener = unlisten;
    } catch (error) {
      console.error('Failed to initialize file change listener:', error);
    }
  }

  /**
   * イベントリスナーを破棄
   */
  static destroyEventListener() {
    if (this.eventListener) {
      this.eventListener();
      this.eventListener = null;
    }
  }

  /**
   * リポジトリの監視を開始
   */
  static async startWatchingRepository(
    repositoryId: string,
    repositoryPath: string
  ): Promise<void> {
    try {
      await invoke('start_watching_repository', {
        repositoryId,
        repositoryPath,
      });

      // 監視中リポジトリリストを更新
      watchedRepositories.update((repos) => {
        const existingIndex = repos.findIndex((r) => r.repository_id === repositoryId);
        const watchedRepo: WatchedRepository = {
          repository_id: repositoryId,
          is_watching: true,
        };

        if (existingIndex !== -1) {
          repos[existingIndex] = watchedRepo;
        } else {
          repos.push(watchedRepo);
        }
        return repos;
      });

      notification.set({
        type: 'info',
        message: `リポジトリ "${repositoryId}" のファイル監視を開始しました`,
      });
    } catch (error) {
      console.error('Failed to start watching repository:', error);
      notification.set({
        type: 'error',
        message: `ファイル監視の開始に失敗しました (リポジトリ: ${repositoryId}): ${error}`,
      });
      throw error;
    }
  }

  /**
   * リポジトリの監視を停止
   */
  static async stopWatchingRepository(repositoryId: string): Promise<void> {
    try {
      await invoke('stop_watching_repository', { repositoryId });

      // 監視中リポジトリリストを更新
      watchedRepositories.update((repos) =>
        repos.map((repo) =>
          repo.repository_id === repositoryId ? { ...repo, is_watching: false } : repo
        )
      );

      notification.set({
        type: 'info',
        message: `リポジトリ "${repositoryId}" のファイル監視を停止しました`,
      });
    } catch (error) {
      console.error('Failed to stop watching repository:', error);
      notification.set({
        type: 'error',
        message: `ファイル監視の停止に失敗しました (リポジトリ: ${repositoryId}): ${error}`,
      });
      throw error;
    }
  }

  /**
   * 監視中のリポジトリ一覧を取得
   */
  static async getWatchedRepositories(): Promise<string[]> {
    try {
      return await invoke<string[]>('get_watched_repositories');
    } catch (error) {
      console.error('Failed to get watched repositories:', error);
      return [];
    }
  }

  /**
   * agent_libraryを再読み込み
   */
  static async reloadAgentLibrary(repositoryId: string, repositoryPath?: string): Promise<void> {
    try {
      let repoPath = repositoryPath;

      // パスが指定されていない場合は設定から取得
      if (!repoPath) {
        const { ConfigAPI } = await import('./config');
        const config = await ConfigAPI.getCurrentConfig();
        const repository = config.repositories.find((r) => r.id === repositoryId);
        if (!repository) {
          throw new Error(`Repository ${repositoryId} not found`);
        }
        repoPath = repository.path;
      }

      const result = await invoke<string>('reload_agent_library', {
        repositoryId,
        repositoryPath: repoPath,
      });

      console.log('Agent library reloaded:', result);

      notification.set({
        type: 'success',
        message: '設定ファイルの変更を検知し、自動更新しました',
      });
    } catch (error) {
      console.error('Failed to reload agent library:', error);
      notification.set({
        type: 'error',
        message: `自動更新に失敗しました (リポジトリ: ${repositoryId}): ${error}`,
      });
    }
  }

  /**
   * 自動リロードすべきかどうかを判定
   */
  private static shouldAutoReload(event: FileChangeEvent): boolean {
    const importantFiles = [
      'agent_index.yml',
      'agent_index.yaml',
      '.md', // Markdownファイル
    ];

    return importantFiles.some((pattern) =>
      event.file_path.toLowerCase().includes(pattern.toLowerCase())
    );
  }

  /**
   * ファイル変更通知を表示
   */
  private static showChangeNotification(event: FileChangeEvent) {
    const fileName = event.file_path.split('/').pop() || event.file_path;
    const changeTypeText = {
      Created: '作成',
      Modified: '変更',
      Deleted: '削除',
      Renamed: '名前変更',
    }[event.change_type];

    // 重要なファイルの変更のみ通知
    if (this.shouldAutoReload(event)) {
      notification.set({
        type: 'info',
        message: `${fileName} が${changeTypeText}されました`,
      });
    }
  }

  /**
   * 自動リロードの有効/無効を切り替え
   */
  static toggleAutoReload(): void {
    autoReloadEnabled.update((enabled) => !enabled);
  }

  /**
   * ファイル変更履歴をクリア
   */
  static clearChangeHistory(): void {
    fileChangeEvents.set([]);
  }

  /**
   * 特定のリポジトリの監視状態を取得
   */
  static async getRepositoryWatchStatus(repositoryId: string): Promise<boolean> {
    try {
      const watchedRepos = await this.getWatchedRepositories();
      return watchedRepos.includes(repositoryId);
    } catch (error) {
      console.error('Failed to get repository watch status:', error);
      return false;
    }
  }
}

/**
 * ファイル変更イベントをフォーマットする utility 関数
 */
export function formatFileChangeEvent(event: FileChangeEvent): string {
  const date = new Date(event.timestamp);
  const time = date.toLocaleTimeString();
  const fileName = event.file_path.split('/').pop() || event.file_path;

  const changeTypeEmoji = {
    Created: '➕',
    Modified: '✏️',
    Deleted: '🗑️',
    Renamed: '📝',
  }[event.change_type];

  return `${changeTypeEmoji} ${fileName} - ${time}`;
}

/**
 * ファイル変更イベントの統計を取得
 */
export function getChangeEventStats(events: FileChangeEvent[]) {
  const stats = {
    total: events.length,
    created: 0,
    modified: 0,
    deleted: 0,
    renamed: 0,
    byRepository: {} as Record<string, number>,
  };

  events.forEach((event) => {
    // 変更タイプ別の統計
    switch (event.change_type) {
      case 'Created':
        stats.created++;
        break;
      case 'Modified':
        stats.modified++;
        break;
      case 'Deleted':
        stats.deleted++;
        break;
      case 'Renamed':
        stats.renamed++;
        break;
    }

    // リポジトリ別の統計
    stats.byRepository[event.repository_id] = (stats.byRepository[event.repository_id] || 0) + 1;
  });

  return stats;
}
