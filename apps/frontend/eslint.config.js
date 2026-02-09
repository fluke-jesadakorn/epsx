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
    files: ['**/*.{ts,tsx}'],
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
      // STRICT POLICY FOR @apps/frontend
      // Already inherits ultra-strict rules from shared config
      // Additional enforcement on specific patterns:
      '@typescript-eslint/strict-boolean-expressions': 'error',
      '@typescript-eslint/prefer-nullish-coalescing': 'error',
    }
  },
  {
    plugins: {
      '@typescript-eslint': typescript,
      '@next/next': nextjs,
      'react-hooks': reactHooks,
    },
    files: ['**/*.{js,jsx}'],
    rules: {
      '@typescript-eslint/strict-boolean-expressions': 'error',
      '@typescript-eslint/prefer-nullish-coalescing': 'error',
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
      '@typescript-eslint/strict-boolean-expressions': 'off',
      '@typescript-eslint/no-unnecessary-condition': 'off',
      '@typescript-eslint/no-misused-promises': 'off',
      '@typescript-eslint/require-await': 'off',
      '@typescript-eslint/no-floating-promises': 'off',
      'max-depth': 'off',
      'max-params': 'off',
      'security/detect-non-literal-regexp': 'off',
      'no-empty-pattern': 'off',
      'react-hooks/rules-of-hooks': 'off',
      '@typescript-eslint/no-confusing-void-expression': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
      'sonarjs/no-collapsible-if': 'off',
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
    files: ['**/polyfills.ts', '**/polyfills.js'],
    rules: {
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
      '@typescript-eslint/no-unnecessary-condition': 'off',
      '@typescript-eslint/strict-boolean-expressions': 'off',
      'complexity': 'off',
      'sonarjs/cognitive-complexity': 'off',
      'max-depth': 'off',
      'no-empty': 'off',
    },
  },
  {
    files: ['next.config.ts', 'next.config.js', 'next.config.mjs'],
    rules: {
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
      '@typescript-eslint/no-var-requires': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      'sonarjs/no-duplicate-string': 'off',
      'import/no-extraneous-dependencies': 'off',
    },
  },
  {
    files: ['eslint.config.js', 'jest.config.js', 'jest.setup.js'],
    rules: {
      '@typescript-eslint/no-var-requires': 'off',
      '@typescript-eslint/no-require-imports': 'off',
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