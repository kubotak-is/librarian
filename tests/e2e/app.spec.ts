import { test, expect } from '@playwright/test';

test.describe('Librarian Application', () => {
  test('should load the main application', async ({ page }) => {
    await page.goto('/');

    // Check if the app loads
    await expect(page).toHaveTitle(/Librarian/);

    // Check for main navigation elements
    await expect(page.getByText('Repository')).toBeVisible();
    await expect(page.getByText('Settings')).toBeVisible();
  });

  test('should display welcome message when no repositories', async ({ page }) => {
    await page.goto('/');

    // Should show empty state
    await expect(page.getByText('リポジトリが見つかりません')).toBeVisible();
    await expect(page.getByText('新しいリポジトリを追加')).toBeVisible();
  });

  test('should navigate to settings page', async ({ page }) => {
    await page.goto('/');

    // Navigate to settings
    await page.getByText('Settings').click();

    // Check settings page content
    await expect(page.getByText('MCP 接続ガイド')).toBeVisible();
    await expect(page.getByText('Claude Code')).toBeVisible();
  });
});
