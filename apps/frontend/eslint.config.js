const sharedConfig = require('../../shared/config/eslint.cjs');
const typescript = require('@typescript-eslint/eslint-plugin');
const nextjs = require('@next/eslint-plugin-next');
const reactHooks = require('eslint-plugin-react-hooks');
const globals = require('globals');

module.exports = [
  ...sharedConfig,
  {
    plugins: {
      '@typescript-eslint': typescript,
      '@next/next': nextjs,
      'react-hooks': reactHooks,
    },
    // Add any specific overrides for frontend app here if needed
    files: ['**/*.{js,jsx,ts,tsx}'],
    languageOptions: {
      parserOptions: {
        project: true, // Enable type-aware linting
      },
      globals: {
        ...globals.browser,
        ...globals.node,
        React: 'readonly',
        JSX: 'readonly',
      }
    },
    rules: {
      // specific overrides for frontend can go here, but strict shared default is preferred.
      // 'import/no-unresolved': 'off',
    }
  },
  {
    files: ['**/*.spec.ts', '**/*.test.ts', '**/__test__/**', '**/e2e/**'],
    rules: {
      'no-console': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      'max-lines-per-function': 'off',
      'complexity': 'off',
      'sonarjs/cognitive-complexity': 'off',
      'sonarjs/no-duplicate-string': 'off',
      'max-nested-callbacks': 'off',
      'jsdoc/require-*': 'off',
      'no-empty': 'off',
      '@typescript-eslint/no-floating-promises': 'off',
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
      '@typescript-eslint/no-unsafe-argument': 'off',
      '@typescript-eslint/await-thenable': 'off',
      '@typescript-eslint/prefer-nullish-coalescing': 'off',
    },
  },
  {
    files: ['**/*.js'],
    rules: {
      '@typescript-eslint/no-var-requires': 'off',
    },
  },
  {
    files: ['playwright.config.ts'],
    rules: {
      'sonarjs/no-duplicate-string': 'off',
    },
  },
  {
    files: ['next.config.ts', 'next.config.js', 'next.config.mjs'],
    rules: {
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-var-requires': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      'import/no-extraneous-dependencies': 'off',
    },
  },
  {
    ignores: [
      'node_modules/**',
      '.next/**',
      '.debug/**',
      'out/**',
      'dist/**',
      'build/**',
      'coverage/**',
      'public/**',
      '*.d.ts',
    ],
  },
];