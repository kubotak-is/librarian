import { writable } from 'svelte/store';

// アプリケーション状態
export const currentView = writable<'home' | 'repositories' | 'prompts' | 'settings'>('home');

// MCP サーバー状態
export const mcpServerRunning = writable<boolean>(false);
export const mcpServerPort = writable<number>(9500);

// リポジトリ関連
export interface Repository {
  id: string;
  name: string;
  path: string;
  isActive: boolean;
  lastUpdated?: Date;
  mcpServer?: {
    port: number;
    status: 'stopped' | 'starting' | 'running' | 'error';
    error?: string;
  };
}

export const repositories = writable<Repository[]>([]);

// MCP サーバー管理
export interface McpServerInfo {
  repositoryId: string;
  port: number;
  status: 'stopped' | 'starting' | 'running' | 'error';
  error?: string;
}

export const mcpServers = writable<McpServerInfo[]>([]);

// プロンプト関連
export interface Prompt {
  id: string;
  name: string;
  title: string;
  description: string;
  category: string;
  repository: string;
  content?: string;
}

export const prompts = writable<Prompt[]>([]);

// UI状態
export const isLoading = writable<boolean>(false);
export const notification = writable<{
  type: 'success' | 'error' | 'info';
  message: string;
} | null>(null);
