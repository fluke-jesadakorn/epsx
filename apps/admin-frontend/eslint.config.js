/**
 * Admin Frontend ESLint Configuration
 * Extends ultra-strict shared configuration with pragmatic adjustments
 */

const sharedConfig = require('../../shared/config/eslint.cjs');
const typescript = require('@typescript-eslint/eslint-plugin');
const reactHooks = require('eslint-plugin-react-hooks');
const sonarjs = require('eslint-plugin-sonarjs');
const security = require('eslint-plugin-security');
const jsdoc = require('eslint-plugin-jsdoc');
const jsxA11y = require('eslint-plugin-jsx-a11y');
const promise = require('eslint-plugin-promise');
const importPlugin = require('eslint-plugin-import');

module.exports = [
  ...sharedConfig,
  {
    // Admin-specific globals and pragmatic rule adjustments
    plugins: {
      '@typescript-eslint': typescript,
      'react-hooks': reactHooks,
      sonarjs: sonarjs,
      security: security,
      jsdoc: jsdoc,
      'jsx-a11y': jsxA11y,
      promise: promise,
      import: importPlugin,
    },
    languageOptions: {
      parserOptions: {
        project: false, // Disable type-aware linting for performance
      },
      globals: {
        // Timer functions
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        setInterval: 'readonly',
        clearInterval: 'readonly',

        // Browser APIs
        performance: 'readonly',
        crypto: 'readonly',
        localStorage: 'readonly',
        alert: 'readonly',
        confirm: 'readonly',
        prompt: 'readonly',
        atob: 'readonly',
        Blob: 'readonly',
        Storage: 'readonly',
        TextEncoder: 'readonly',

        // DOM types
        HTMLElement: 'readonly',
        HTMLDivElement: 'readonly',
        HTMLButtonElement: 'readonly',
        HTMLFormElement: 'readonly',
        HTMLInputElement: 'readonly',
        HTMLTextAreaElement: 'readonly',
        HTMLSpanElement: 'readonly',
        HTMLParagraphElement: 'readonly',
        HTMLHeadingElement: 'readonly',
        HTMLImageElement: 'readonly',
        HTMLTableElement: 'readonly',
        HTMLTableRowElement: 'readonly',
        HTMLTableCellElement: 'readonly',
        HTMLTableSectionElement: 'readonly',
        HTMLTableCaptionElement: 'readonly',
        EventListener: 'readonly',
        CustomEvent: 'readonly',
        MouseEvent: 'readonly',
        KeyboardEvent: 'readonly',
        TouchEvent: 'readonly',
        Node: 'readonly',

        // Fetch/Network types
        RequestInit: 'readonly',

        // Performance API
        PerformanceNavigationTiming: 'readonly',

        // Test framework (Jest/Vitest/Playwright)
        describe: 'readonly',
        test: 'readonly',
        expect: 'readonly',
        beforeEach: 'readonly',
        afterEach: 'readonly',
        beforeAll: 'readonly',
        afterAll: 'readonly',
        it: 'readonly',
        jest: 'readonly',

        // App-specific
        getAuthorizationUrl: 'readonly',
        canManageGroups: 'readonly',
        canManageWeb3Rules: 'readonly',
        // Session storage
        sessionStorage: 'readonly',
        // Error handling
        error: 'readonly',
      },
    },
    rules: {
      // Disable import/no-unresolved - TypeScript handles this better with path aliases
      'import/no-unresolved': 'off',

      // Disable type-aware rules (require project: true which is too slow)
      '@typescript-eslint/strict-boolean-expressions': 'off',
      '@typescript-eslint/prefer-nullish-coalescing': 'off',
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-argument': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
      '@typescript-eslint/no-unnecessary-condition': 'off',
      '@typescript-eslint/no-floating-promises': 'off',
      '@typescript-eslint/no-misused-promises': 'off',
      '@typescript-eslint/await-thenable': 'off',
      '@typescript-eslint/require-await': 'off',
      '@typescript-eslint/prefer-optional-chain': 'off',
      '@typescript-eslint/no-unnecessary-type-assertion': 'off',

      // Relax documentation and complexity rules to warnings
      '@typescript-eslint/explicit-function-return-type': 'warn',
      '@typescript-eslint/explicit-module-boundary-types': 'warn',
      'max-lines-per-function': 'warn',
      'complexity': 'warn',
      'sonarjs/cognitive-complexity': 'warn',
      'sonarjs/no-duplicate-string': 'warn',
      'sonarjs/no-identical-functions': 'warn', // Downgrade - requires careful refactoring
      'jsdoc/require-jsdoc': 'off',
      'jsdoc/require-param': 'off',
      'jsdoc/require-returns': 'off',
      'jsdoc/check-param-names': 'off',
    },
  },
  {
    files: ['**/*.spec.ts', '**/*.test.ts', '**/__test__/**'],
    rules: {
      'no-console': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      'max-lines-per-function': 'off',
      'complexity': 'off',
      'sonarjs/cognitive-complexity': 'off',
      'max-nested-callbacks': 'off',
      'max-depth': 'off',
      'jsdoc/require-*': 'off',
      'no-empty': 'off',
      'no-empty-pattern': 'off', // Playwright fixtures often have empty patterns
      'sonarjs/no-duplicate-string': 'off',
      'promise/prefer-await-to-then': 'off',
      'react-hooks/rules-of-hooks': 'off', // Playwright `use` is not a React hook
    },
  },
  {
    files: ['**/*.js'],
    rules: {
      '@typescript-eslint/no-var-requires': 'off',
    },
  },
  {
    ignores: ['.debug/**'],
  },
];
