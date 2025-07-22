import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface McpServerConfig {
  port: number;
  status: 'running' | 'stopped' | 'error';
}

export interface RepositoryConfig {
  id: string;
  name: string;
  path: string;
  is_active: boolean;
  last_updated: string;
  mcp_server?: McpServerConfig;
}

export interface AppConfig {
  repositories: RepositoryConfig[];
  last_opened_repository?: string;
  theme: string;
  auto_start_servers: boolean;
}

// アプリケーション設定のストア
export const appConfig = writable<AppConfig>({
  repositories: [],
  theme: 'light',
  auto_start_servers: true,
});

// 設定の読み込み状態
export const configLoading = writable(false);

/**
 * 設定管理API
 */
export class ConfigAPI {
  /**
   * アプリケーション設定を読み込み
   */
  static async loadConfig(): Promise<AppConfig> {
    configLoading.set(true);
    try {
      const config = await invoke<AppConfig>('load_app_config');
      appConfig.set(config);
      return config;
    } catch (error) {
      console.error('Failed to load config:', error);
      throw error;
    } finally {
      configLoading.set(false);
    }
  }

  /**
   * アプリケーション設定を保存
   */
  static async saveConfig(config: AppConfig): Promise<void> {
    try {
      await invoke('save_app_config', { config });
      appConfig.set(config);
    } catch (error) {
      console.error('Failed to save config:', error);
      throw error;
    }
  }

  /**
   * リポジトリを追加
   */
  static async addRepository(repository: RepositoryConfig): Promise<void> {
    try {
      await invoke('add_repository_config', { repository });

      // ストアを更新 (新しい配列参照を作成してリアクティビティを確保)
      appConfig.update((config) => {
        const existingIndex = config.repositories.findIndex((r) => r.id === repository.id);
        let updatedRepositories;

        if (existingIndex !== -1) {
          updatedRepositories = [...config.repositories];
          updatedRepositories[existingIndex] = repository;
        } else {
          updatedRepositories = [...config.repositories, repository];
        }

        return {
          ...config,
          repositories: updatedRepositories,
        };
      });
    } catch (error) {
      console.error('Failed to add repository:', error);
      throw error;
    }
  }

  /**
   * リポジトリを削除（確認済み）
   */
  static async removeRepository(repositoryId: string): Promise<boolean> {
    try {
      const removed = await invoke<boolean>('remove_repository_config', {
        repositoryId,
      });

      if (removed) {
        // ストアを更新
        appConfig.update((config) => {
          return {
            ...config,
            repositories: config.repositories.filter((r) => r.id !== repositoryId),
          };
        });
      }

      return removed;
    } catch (error) {
      console.error('Failed to remove repository:', error);
      throw error;
    }
  }

  /**
   * リポジトリを削除（確認付き）
   */
  static async removeRepositoryWithConfirmation(
    repositoryId: string,
    repositoryName: string
  ): Promise<boolean> {
    // 確認ダイアログを表示
    const confirmed = confirm(
      `リポジトリ "${repositoryName}" を削除しますか？この操作は元に戻せません。`
    );

    if (!confirmed) {
      return false;
    }

    try {
      const removed = await invoke<boolean>('remove_repository_config', {
        repositoryId,
      });

      if (removed) {
        // ストアを更新
        appConfig.update((config) => {
          return {
            ...config,
            repositories: config.repositories.filter((r) => r.id !== repositoryId),
          };
        });
      }

      return removed;
    } catch (error) {
      console.error('Failed to remove repository:', error);
      throw error;
    }
  }

  /**
   * MCPサーバーの状態を更新
   */
  static async updateMcpServerStatus(
    repositoryId: string,
    port: number,
    status: 'running' | 'stopped' | 'error'
  ): Promise<void> {
    try {
      await invoke('update_repository_mcp_status', {
        repositoryId,
        port,
        status,
      });

      // ストアを更新 (新しい配列参照を作成してリアクティビティを確保)
      appConfig.update((config) => {
        const updatedRepositories = config.repositories.map((r) => {
          if (r.id === repositoryId) {
            return {
              ...r,
              mcp_server: { port, status },
              last_updated: new Date().toISOString(),
            };
          }
          return r;
        });

        return {
          ...config,
          repositories: updatedRepositories,
        };
      });
    } catch (error) {
      console.error('Failed to update MCP server status:', error);
      throw error;
    }
  }

  /**
   * リポジトリをアクティブ/非アクティブに切り替え
   */
  static async toggleRepositoryActive(repositoryId: string): Promise<void> {
    try {
      const config = await this.getCurrentConfig();
      const repository = config.repositories.find((r) => r.id === repositoryId);

      if (repository) {
        repository.is_active = !repository.is_active;
        repository.last_updated = new Date().toISOString();
        await this.saveConfig(config);
      }
    } catch (error) {
      console.error('Failed to toggle repository active state:', error);
      throw error;
    }
  }

  /**
   * 現在の設定を取得
   */
  static async getCurrentConfig(): Promise<AppConfig> {
    try {
      // まずTauriから最新の設定を読み込み
      const config = await invoke<AppConfig>('load_app_config');
      appConfig.set(config);
      return config;
    } catch (error) {
      console.error('Failed to load config in getCurrentConfig:', error);
      // フォールバック: ストアから取得
      return new Promise((resolve) => {
        const unsubscribe = appConfig.subscribe((config) => {
          unsubscribe();
          resolve(config);
        });
      });
    }
  }

  /**
   * アクティブなリポジトリを取得
   */
  static async getActiveRepositories(): Promise<RepositoryConfig[]> {
    const config = await this.getCurrentConfig();
    return config.repositories.filter((r) => r.is_active);
  }

  /**
   * 実行中のMCPサーバーを取得
   */
  static async getRunningServers(): Promise<RepositoryConfig[]> {
    const config = await this.getCurrentConfig();
    return config.repositories.filter((r) => r.mcp_server?.status === 'running');
  }

  /**
   * テーマを変更
   */
  static async changeTheme(theme: string): Promise<void> {
    try {
      const config = await this.getCurrentConfig();
      config.theme = theme;
      await this.saveConfig(config);
    } catch (error) {
      console.error('Failed to change theme:', error);
      throw error;
    }
  }

  /**
   * 自動開始設定を変更
   */
  static async toggleAutoStartServers(): Promise<void> {
    try {
      const config = await this.getCurrentConfig();
      config.auto_start_servers = !config.auto_start_servers;
      await this.saveConfig(config);
    } catch (error) {
      console.error('Failed to toggle auto start servers:', error);
      throw error;
    }
  }

  /**
   * 指定されたパスに.agent_libraryディレクトリが存在するかを確認
   */
  static async validateAgentLibrary(path: string): Promise<boolean> {
    try {
      const result = await invoke<boolean>('validate_agent_library', { path });
      return result;
    } catch (error) {
      console.error('Failed to validate agent library:', error);
      return false;
    }
  }

  /**
   * 最後に開いたリポジトリを更新
   */
  static async updateLastOpenedRepository(repositoryId: string): Promise<void> {
    try {
      const config = await this.getCurrentConfig();
      config.last_opened_repository = repositoryId;
      await this.saveConfig(config);
    } catch (error) {
      console.error('Failed to update last opened repository:', error);
      throw error;
    }
  }
}

/**
 * リポジトリ設定を作成するヘルパー関数
 */
export function createRepositoryConfig(id: string, name: string, path: string): RepositoryConfig {
  return {
    id,
    name,
    path,
    is_active: true,
    last_updated: new Date().toISOString(),
  };
}

/**
 * 一意なリポジトリIDを生成
 */
export function generateRepositoryId(): string {
  return `repo_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}
