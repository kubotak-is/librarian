import { writable } from 'svelte/store';
import type { RepositoryConfig } from './config';

// 現在選択されているリポジトリ
export const selectedRepository = writable<RepositoryConfig | null>(null);

// 現在のビュー（プロンプト一覧、設定など）
export const currentView = writable<'prompts' | 'settings'>('prompts');

// アプリケーションの初期化状態
export const isInitialized = writable(false);

// サイドバーの表示状態（レスポンシブ対応用）
export const sidebarVisible = writable(true);

// 現在のテーマ（システム設定に従う）
export const currentTheme = writable<'light' | 'dark'>('light');

// アプリケーション設定
export interface AppSettings {
  autoStartServers: boolean;
  showNotifications: boolean;
  minimizeToTray: boolean;
}

export const appSettings = writable<AppSettings>({
  autoStartServers: true,
  showNotifications: true,
  minimizeToTray: false,
});

// テーマ検出と監視
if (typeof window !== 'undefined') {
  // 初期テーマを設定
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  currentTheme.set(mediaQuery.matches ? 'dark' : 'light');

  // テーマ変更を監視
  mediaQuery.addEventListener('change', (e) => {
    currentTheme.set(e.matches ? 'dark' : 'light');
  });
}

// ユーティリティ関数

/**
 * サイドバーの表示/非表示を切り替え
 */
export function toggleSidebar(): void {
  sidebarVisible.update((visible) => !visible);
}

/**
 * ビューを変更
 */
export function setCurrentView(view: 'prompts' | 'settings'): void {
  currentView.set(view);
}

/**
 * リポジトリを選択
 */
export async function setSelectedRepository(repository: RepositoryConfig | null): Promise<void> {
  // リポジトリが変更されたらプロンプトビューに戻る
  if (repository) {
    currentView.set('prompts');

    // 最新の設定を読み込んでサーバー状態を同期
    try {
      const { ConfigAPI } = await import('./config');
      const config = await ConfigAPI.getCurrentConfig();
      const updatedRepository = config.repositories.find((r) => r.id === repository.id);

      if (updatedRepository) {
        selectedRepository.set(updatedRepository);
      } else {
        selectedRepository.set(repository);
      }
    } catch (error) {
      console.error('Failed to sync repository state:', error);
      selectedRepository.set(repository);
    }
  } else {
    selectedRepository.set(null);
  }
}

/**
 * アプリケーション設定を更新
 */
export function updateAppSettings(settings: Partial<AppSettings>): void {
  appSettings.update((current) => ({ ...current, ...settings }));
}
