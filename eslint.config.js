import eslint from '@eslint/js';
import tseslint from '@typescript-eslint/eslint-plugin';
import svelteeslint from 'eslint-plugin-svelte';
import globals from 'globals';
import tsparser from '@typescript-eslint/parser';
import svelteparser from 'svelte-eslint-parser';

export default [
  {
    ignores: ['src-tauri/target/**', 'dist/**', 'node_modules/**'],
  },
  eslint.configs.recommended,
  {
    files: ['**/*.{js,mjs,cjs,ts,tsx}'],
    languageOptions: {
      parser: tsparser,
      parserOptions: {
        ecmaVersion: 2020,
        sourceType: 'module',
      },
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.es2017,
      },
    },
    plugins: {
      '@typescript-eslint': tseslint,
    },
    rules: {
      ...tseslint.configs.recommended.rules,
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      'no-undef': 'off',
    },
  },
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parser: svelteparser,
      parserOptions: {
        parser: tsparser,
        extraFileExtensions: ['.svelte'],
      },
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    plugins: {
      '@typescript-eslint': tseslint,
      svelte: svelteeslint,
    },
    rules: {
      ...svelteeslint.configs.recommended.rules,
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      'no-undef': 'off',
      'svelte/no-unused-class-name': 'off',
    },
  },
];
