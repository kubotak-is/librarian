import { writable, get } from 'svelte/store';

// テーマタイプ
export type Theme = 'dark' | 'light' | 'system';

// テーマストア
export const theme = writable<Theme>('dark');

// 現在の実効テーマ（system の場合はシステム設定を参照）
export const currentTheme = writable<'dark' | 'light'>('dark');

// ブラウザ環境判定
const browser = typeof window !== 'undefined';

// テーマ初期化とローカルストレージ連携
export function initializeTheme() {
  if (!browser) return;

  // ローカルストレージから読み込み
  const savedTheme = localStorage.getItem('theme') as Theme;
  if (savedTheme && ['dark', 'light', 'system'].includes(savedTheme)) {
    theme.set(savedTheme);
  }

  // 現在のテーマ値を取得して初期適用
  let currentThemeValue: Theme = 'dark';
  const unsubscribe = theme.subscribe((value) => {
    currentThemeValue = value;
  });
  unsubscribe();

  // 初期テーマを適用
  updateDocumentTheme(currentThemeValue);

  // テーマ変更時の処理
  theme.subscribe((value) => {
    if (!browser) return;

    console.log('Theme subscription triggered:', value);
    localStorage.setItem('theme', value);
    updateDocumentTheme(value);
  });

  // システム設定変更の監視
  if (window.matchMedia) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addListener(() => {
      const currentValue = get(theme);
      if (currentValue === 'system') {
        updateDocumentTheme('system');
      }
    });
  }
}

// ドキュメントにテーマを適用
function updateDocumentTheme(themeValue: Theme) {
  if (!browser) return;

  let actualTheme: 'dark' | 'light';

  if (themeValue === 'system') {
    // システム設定を確認
    actualTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  } else {
    actualTheme = themeValue;
  }

  console.log('Updating document theme:', themeValue, '->', actualTheme);

  // HTMLの data-theme 属性を設定
  document.documentElement.setAttribute('data-theme', actualTheme);

  // currentTheme ストアを更新
  currentTheme.set(actualTheme);

  console.log('Document theme applied:', document.documentElement.getAttribute('data-theme'));
}

// テーマ切り替え関数
export function toggleTheme() {
  const current = get(theme);
  let newTheme: Theme;

  switch (current) {
    case 'dark':
      newTheme = 'light';
      break;
    case 'light':
      newTheme = 'system';
      break;
    case 'system':
      newTheme = 'dark';
      break;
    default:
      newTheme = 'dark';
  }

  console.log('Theme toggle:', current, '->', newTheme);
  theme.set(newTheme);
}

// 特定のテーマに設定
export function setTheme(newTheme: Theme) {
  theme.set(newTheme);
}
