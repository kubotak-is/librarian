import { test, expect } from '@playwright/test';
import { promises as fs } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

test.describe('Repository Management', () => {
  let tempDir: string;
  let testRepoPath: string;

  test.beforeAll(async () => {
    // Create temporary test repository
    tempDir = await fs.mkdtemp(join(tmpdir(), 'librarian-test-'));
    testRepoPath = join(tempDir, 'test-repo');

    await fs.mkdir(testRepoPath);
    await fs.mkdir(join(testRepoPath, '.agent_library'));

    // Create test agent_index.yml
    const agentIndex = `
name: "Test Repository"
description: "Test repository for E2E testing"
version: "1.0.0"
mcp_endpoints:
  - id: "test_prompt"
    label: "Test Prompt"
    description: "A test prompt for E2E testing"
    prompt_file: "test_prompt.md"
`;
    await fs.writeFile(join(testRepoPath, '.agent_library', 'agent_index.yml'), agentIndex);

    // Create test prompt file
    const promptContent = '# Test Prompt\n\nThis is a test prompt for E2E testing.';
    await fs.writeFile(join(testRepoPath, '.agent_library', 'test_prompt.md'), promptContent);
  });

  test.afterAll(async () => {
    // Cleanup temporary directory
    await fs.rm(tempDir, { recursive: true, force: true });
  });

  test('should add a new repository', async ({ page }) => {
    await page.goto('/');

    // Mock the file dialog to return our test repo path
    await page.addInitScript((repoPath) => {
      (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
        invoke: async (command: string, _args: unknown) => {
          if (command === 'validate_agent_library') {
            return true;
          }
          if (command === 'parse_agent_library') {
            return {
              index: {
                name: 'Test Repository',
                description: 'Test repository for E2E testing',
                version: '1.0.0',
              },
              prompts: [
                {
                  id: 'test_prompt',
                  title: 'Test Prompt',
                  description: 'A test prompt for E2E testing',
                  content: '# Test Prompt\n\nThis is a test prompt for E2E testing.',
                  filePath: `${repoPath}/.agent_library/test_prompt.md`,
                },
              ],
            };
          }
          if (command === 'add_repository_config') {
            return undefined;
          }
          return Promise.resolve();
        },
      };
    }, testRepoPath);

    // Click add repository button
    await page.getByText('新しいリポジトリを追加').click();

    // Should show repository in list
    await expect(page.getByText('Test Repository')).toBeVisible();
  });

  test('should display repository prompts', async ({ page }) => {
    await page.goto('/');

    // Mock repository data
    await page.addInitScript(() => {
      (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
        invoke: async (command: string) => {
          if (command === 'load_app_config') {
            return {
              repositories: [
                {
                  id: 'test-repo',
                  name: 'Test Repository',
                  path: '/test/path',
                  createdAt: '2023-01-01T00:00:00Z',
                  lastUpdated: '2023-01-01T00:00:00Z',
                },
              ],
            };
          }
          if (command === 'parse_agent_library') {
            return {
              prompts: [
                {
                  id: 'test_prompt',
                  title: 'Test Prompt',
                  description: 'A test prompt for E2E testing',
                  content: '# Test Prompt\n\nThis is a test prompt.',
                },
              ],
            };
          }
          return Promise.resolve();
        },
      };
    });

    await page.reload();

    // Click on repository
    await page.getByText('Test Repository').click();

    // Should display prompts
    await expect(page.getByText('Test Prompt')).toBeVisible();
  });

  test('should start MCP server for repository', async ({ page }) => {
    await page.goto('/');

    // Mock MCP server operations
    await page.addInitScript(() => {
      let serverRunning = false;

      (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
        invoke: async (command: string, args?: unknown) => {
          if (command === 'start_repository_mcp_server') {
            serverRunning = true;
            return 'MCP Server started on port 9500';
          }
          if (command === 'get_mcp_server_status') {
            const typedArgs = args as { repositoryId?: string };
            return {
              repository_id: typedArgs?.repositoryId,
              port: serverRunning ? 9500 : null,
              status: serverRunning ? 'running' : 'stopped',
            };
          }
          if (command === 'load_app_config') {
            return {
              repositories: [
                {
                  id: 'test-repo',
                  name: 'Test Repository',
                  path: '/test/path',
                  createdAt: '2023-01-01T00:00:00Z',
                  lastUpdated: '2023-01-01T00:00:00Z',
                },
              ],
            };
          }
          return Promise.resolve();
        },
      };
    });

    await page.reload();

    // Find and click start MCP server button
    await page.getByText('Test Repository').click();
    await page.getByRole('button', { name: /MCP.*開始/ }).click();

    // Should show server running status
    await expect(page.getByText(/running|実行中/)).toBeVisible();
  });
});
