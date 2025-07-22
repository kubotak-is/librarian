import type { RepositoryConfig } from '../stores/config';
import { ConfigAPI } from '../stores/config';
import { setSelectedRepository } from '../stores/app';

export interface LayoutState {
  repositories: RepositoryConfig[];
  currentSelectedRepository: RepositoryConfig | null;
  currentViewValue: string;
  isLoading: boolean;
}

/**
 * 初期設定を読み込み、全リポジトリのTauriサーバー状態を同期
 */
export async function loadInitialConfig(): Promise<{
  repositories: RepositoryConfig[];
  defaultRepository: RepositoryConfig | null;
}> {
  try {
    const config = await ConfigAPI.loadConfig();

    // 全リポジトリのTauriサーバー状態を同期
    console.log('Syncing all repository states from Tauri...');
    for (const repo of config.repositories) {
      await syncRepositoryStateFromTauri(repo.id);
    }

    // 最新の設定を再読み込みして状態を反映
    const updatedConfig = await ConfigAPI.getCurrentConfig();

    // デフォルトのリポジトリを決定
    let defaultRepository: RepositoryConfig | null = null;
    if (updatedConfig.repositories.length > 0) {
      defaultRepository = updatedConfig.last_opened_repository
        ? updatedConfig.repositories.find((r) => r.id === updatedConfig.last_opened_repository) ||
          updatedConfig.repositories[0]
        : updatedConfig.repositories[0];
    }

    return {
      repositories: [...updatedConfig.repositories],
      defaultRepository,
    };
  } catch (error) {
    console.error('Failed to load initial configuration:', error);
    throw error;
  }
}

/**
 * Tauriサーバー側から最新の状態を同期
 */
export async function syncRepositoryStateFromTauri(repositoryId: string): Promise<void> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const status = await invoke('get_mcp_server_status', { repositoryId });

    if (status && typeof status === 'object') {
      const { port, status: serverStatus } = status as {
        port: number;
        status: 'running' | 'stopped' | 'error';
      };
      if (port && serverStatus) {
        console.log(`Repository ${repositoryId}: port=${port}, status=${serverStatus}`);
        await ConfigAPI.updateMcpServerStatus(repositoryId, port, serverStatus);
      } else {
        // サーバーが起動していない場合は現在のポートを保持してstopped状態に設定
        console.log(`Repository ${repositoryId}: no active server, setting to stopped`);
        const config = await ConfigAPI.getCurrentConfig();
        const repository = config.repositories.find((r) => r.id === repositoryId);
        const currentPort = repository?.mcp_server?.port || 9500;
        await ConfigAPI.updateMcpServerStatus(repositoryId, currentPort, 'stopped');
      }
    } else {
      // ステータス取得に失敗した場合も現在のポートを保持してstopped状態に設定
      console.log(`Repository ${repositoryId}: failed to get status, setting to stopped`);
      const config = await ConfigAPI.getCurrentConfig();
      const repository = config.repositories.find((r) => r.id === repositoryId);
      const currentPort = repository?.mcp_server?.port || 9500;
      await ConfigAPI.updateMcpServerStatus(repositoryId, currentPort, 'stopped');
    }
  } catch (error) {
    console.warn(`Failed to sync repository state for ${repositoryId}:`, error);
    // エラーの場合も現在のポートを保持してstopped状態に設定
    try {
      const config = await ConfigAPI.getCurrentConfig();
      const repository = config.repositories.find((r) => r.id === repositoryId);
      const currentPort = repository?.mcp_server?.port || 9500;
      await ConfigAPI.updateMcpServerStatus(repositoryId, currentPort, 'stopped');
    } catch (updateError) {
      console.error(`Failed to update repository status to stopped:`, updateError);
    }
  }
}

/**
 * リポジトリ選択処理（Tauriサーバーとの同期付き）
 */
export async function handleRepositorySelect(repository: RepositoryConfig): Promise<{
  repositories: RepositoryConfig[];
  updatedRepository: RepositoryConfig | null;
}> {
  try {
    // Tauriサーバー側の最新状態を同期
    await syncRepositoryStateFromTauri(repository.id);

    // 最新の設定を再読み込みして状態を反映
    const updatedConfig = await ConfigAPI.getCurrentConfig();

    // 更新されたリポジトリ情報を取得
    const updatedRepository = updatedConfig.repositories.find((r) => r.id === repository.id);
    if (updatedRepository) {
      // 選択状態を更新
      await setSelectedRepository(updatedRepository);

      // 最後に開いたリポジトリとして保存
      await ConfigAPI.updateLastOpenedRepository(updatedRepository.id);
    }

    return {
      repositories: [...updatedConfig.repositories],
      updatedRepository: updatedRepository || null,
    };
  } catch (error) {
    console.error('Failed to select repository:', error);
    throw error;
  }
}

/**
 * リポジトリ追加処理
 */
export async function handleAddRepository(): Promise<{
  repositories: RepositoryConfig[];
  newRepository: RepositoryConfig | null;
}> {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selectedPath = await open({
      title: 'リポジトリを選択',
      directory: true,
      multiple: false,
    });

    if (!selectedPath) {
      return {
        repositories: [],
        newRepository: null,
      };
    }

    // .agent_library の存在確認
    const agentLibraryExists = await ConfigAPI.validateAgentLibrary(selectedPath);

    if (!agentLibraryExists) {
      throw new Error('.agent_library ディレクトリが見つかりません');
    }

    // リポジトリ名を生成（パスの最後の部分）
    const name = selectedPath.split('/').pop() || 'Unknown Repository';
    const id = `repo_${Date.now()}`;

    // 固定ポート番号を生成（9500から開始）
    const config = await ConfigAPI.getCurrentConfig();
    const existingPorts = config.repositories
      .filter((r) => r.mcp_server?.port)
      .map((r) => r.mcp_server!.port);

    console.log('既存のポート:', existingPorts);

    let port = 9500;
    while (existingPorts.includes(port)) {
      port++;
    }

    console.log('新しいポート:', port);

    // 新しいリポジトリを追加（固定ポート付き）
    const newRepository: RepositoryConfig = {
      id,
      name,
      path: selectedPath,
      is_active: true,
      last_updated: new Date().toISOString(),
      mcp_server: {
        port: port,
        status: 'stopped',
      },
    };

    await ConfigAPI.addRepository(newRepository);

    // 全リポジトリ状態を再同期
    const { repositories } = await loadInitialConfig();

    // 新しいリポジトリを選択
    await setSelectedRepository(newRepository);

    return {
      repositories,
      newRepository,
    };
  } catch (error) {
    console.error('Failed to add repository:', error);
    throw error;
  }
}

/**
 * 定期同期処理
 */
export async function performPeriodicSync(
  repositories: RepositoryConfig[]
): Promise<RepositoryConfig[]> {
  let hasChanges = false;
  for (const repo of repositories) {
    if (repo.mcp_server?.status === 'running') {
      await syncRepositoryStateFromTauri(repo.id);
      hasChanges = true;
    }
  }

  // 変更があった場合は最新の設定を再読み込み
  if (hasChanges) {
    const updatedConfig = await ConfigAPI.getCurrentConfig();
    return [...updatedConfig.repositories];
  }

  return repositories;
}
