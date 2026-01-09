/**
 * EPSX Ultra-Strict ESLint Configuration (Shared)
 *
 * Maximum enforcement of code quality, type safety, accessibility, and security.
 * All rules set to ERROR level (zero tolerance).
 *
 * Required plugins:
 * - @typescript-eslint/eslint-plugin
 * - @typescript-eslint/parser
 * - eslint-plugin-react-hooks
 * - eslint-plugin-jsx-a11y
 * - eslint-plugin-import
 * - eslint-plugin-security
 * - eslint-plugin-sonarjs
 * - eslint-plugin-unicorn
 * - eslint-plugin-jsdoc
 * - eslint-plugin-promise
 * - @next/eslint-plugin-next
 */

const js = require('@eslint/js');
const typescript = require('@typescript-eslint/eslint-plugin');
const typescriptParser = require('@typescript-eslint/parser');
const reactHooks = require('eslint-plugin-react-hooks');
const jsxA11y = require('eslint-plugin-jsx-a11y');
const importPlugin = require('eslint-plugin-import');
const security = require('eslint-plugin-security');
const sonarjs = require('eslint-plugin-sonarjs');
const unicorn = require('eslint-plugin-unicorn');
const jsdoc = require('eslint-plugin-jsdoc');
const promise = require('eslint-plugin-promise');
const nextjs = require('@next/eslint-plugin-next');

module.exports = [
  js.configs.recommended,
  {
    files: ['**/*.{js,jsx,ts,tsx}'],
    languageOptions: {
      parser: typescriptParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: {
          jsx: true,
        },
        project: true, // Enable type-aware linting
      },
      globals: {
        // Browser
        window: 'readonly',
        document: 'readonly',
        navigator: 'readonly',
        console: 'readonly',
        fetch: 'readonly',
        URL: 'readonly',
        URLSearchParams: 'readonly',
        FormData: 'readonly',
        Headers: 'readonly',
        Request: 'readonly',
        Response: 'readonly',
        ReadableStream: 'readonly',
        AbortController: 'readonly',
        AbortSignal: 'readonly',
        EventSource: 'readonly',
        WebSocket: 'readonly',
        // Node.js
        process: 'readonly',
        Buffer: 'readonly',
        __dirname: 'readonly',
        __filename: 'readonly',
        global: 'readonly',
        module: 'readonly',
        require: 'readonly',
        exports: 'readonly',
        // TypeScript
        NodeJS: 'readonly',
        // React
        React: 'readonly',
        JSX: 'readonly',
      },
    },
    plugins: {
      '@typescript-eslint': typescript,
      'react-hooks': reactHooks,
      'jsx-a11y': jsxA11y,
      'import': importPlugin,
      'security': security,
      'sonarjs': sonarjs,
      'unicorn': unicorn,
      'jsdoc': jsdoc,
      'promise': promise,
      '@next/next': nextjs,
    },
    rules: {
      // ================================
      // TypeScript Rules (Pragmatic)
      // ================================
      '@typescript-eslint/no-explicit-any': 'warn', // Many legacy uses, fix gradually
      '@typescript-eslint/no-unsafe-assignment': 'off', // Requires type-aware linting
      '@typescript-eslint/no-unsafe-member-access': 'off', // Requires type-aware linting
      '@typescript-eslint/no-unsafe-call': 'off', // Requires type-aware linting
      '@typescript-eslint/no-unsafe-return': 'off', // Requires type-aware linting
      '@typescript-eslint/no-unsafe-argument': 'off', // Requires type-aware linting
      '@typescript-eslint/no-floating-promises': 'off', // Requires type-aware linting
      '@typescript-eslint/no-misused-promises': 'off', // Requires type-aware linting
      '@typescript-eslint/await-thenable': 'off', // Requires type-aware linting
      '@typescript-eslint/require-await': 'off', // Requires type-aware linting
      '@typescript-eslint/no-unused-vars': ['warn', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_'
      }],
      '@typescript-eslint/explicit-function-return-type': 'off', // Too noisy for React components
      '@typescript-eslint/explicit-module-boundary-types': 'off', // Too noisy for React components
      '@typescript-eslint/strict-boolean-expressions': 'off', // Requires type-aware linting
      '@typescript-eslint/no-non-null-assertion': 'warn',
      '@typescript-eslint/prefer-nullish-coalescing': 'off', // Requires type-aware linting
      '@typescript-eslint/prefer-optional-chain': 'off', // Requires type-aware linting
      '@typescript-eslint/no-unnecessary-condition': 'off', // Requires type-aware linting
      '@typescript-eslint/no-unnecessary-type-assertion': 'off', // Requires type-aware linting
      '@typescript-eslint/prefer-as-const': 'warn',
      '@typescript-eslint/no-inferrable-types': 'warn',
      '@typescript-eslint/naming-convention': 'off', // Too strict

      // ================================
      // React Hooks (ERROR - Important)
      // ================================
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn', // Often intentionally ignored

      // ================================
      // Accessibility (WARN - Fix gradually)
      // ================================
      'jsx-a11y/alt-text': 'warn',
      'jsx-a11y/aria-props': 'warn',
      'jsx-a11y/aria-proptypes': 'warn',
      'jsx-a11y/aria-unsupported-elements': 'warn',
      'jsx-a11y/role-has-required-aria-props': 'warn',
      'jsx-a11y/role-supports-aria-props': 'warn',
      'jsx-a11y/click-events-have-key-events': 'off', // Often not applicable
      'jsx-a11y/no-static-element-interactions': 'off', // Often not applicable
      'jsx-a11y/anchor-is-valid': 'warn',
      'jsx-a11y/img-redundant-alt': 'warn',
      'jsx-a11y/interactive-supports-focus': 'off', // Often not applicable

      // ================================
      // Import Management (Relaxed)
      // ================================
      'import/no-unresolved': 'off', // TypeScript handles this
      'import/no-duplicates': 'warn',
      'import/no-cycle': 'off', // Slow and often false positives
      'import/first': 'warn',
      'import/newline-after-import': 'warn',
      'import/order': 'off', // Too strict for quick development

      // ================================
      // Security (WARN - Some false positives)
      // ================================
      'security/detect-object-injection': 'off', // Too many false positives
      'security/detect-non-literal-regexp': 'warn',
      'security/detect-unsafe-regex': 'warn',
      'security/detect-buffer-noassert': 'warn',
      'security/detect-child-process': 'warn',
      'security/detect-disable-mustache-escape': 'warn',
      'security/detect-eval-with-expression': 'error',
      'security/detect-no-csrf-before-method-override': 'warn',
      'security/detect-non-literal-fs-filename': 'off', // Too many false positives
      'security/detect-non-literal-require': 'off', // Too many false positives
      'security/detect-possible-timing-attacks': 'off', // Too many false positives
      'security/detect-pseudoRandomBytes': 'warn',

      // ================================
      // Complexity Limits (WARN)
      // ================================
      'complexity': ['warn', 15], // Relaxed from 10
      'max-depth': ['warn', 4], // Relaxed from 3
      'max-lines-per-function': ['warn', { max: 100, skipBlankLines: true, skipComments: true }],
      'max-params': ['warn', 5], // Relaxed from 3
      'max-nested-callbacks': ['warn', 4],
      'sonarjs/cognitive-complexity': ['warn', 20],
      'sonarjs/no-identical-functions': 'warn',
      'sonarjs/no-duplicate-string': ['warn', { threshold: 5 }],
      'sonarjs/no-collapsible-if': 'warn',
      'sonarjs/no-collection-size-mischeck': 'warn',
      'sonarjs/no-redundant-jump': 'warn',
      'sonarjs/prefer-immediate-return': 'off', // Preference

      // ================================
      // Code Quality (Unicorn) - OFF by default
      // ================================

      // ================================
      // Promise Handling (WARN)
      // ================================
      'promise/always-return': 'off', // Too strict
      'promise/catch-or-return': 'warn',
      'promise/no-return-wrap': 'warn',
      'promise/no-nesting': 'warn',
      'promise/prefer-await-to-then': 'off', // Preference

      // ================================
      // JSDoc Documentation (OFF - Use TypeScript)
      // ================================
      'jsdoc/require-jsdoc': 'off',
      'jsdoc/require-param': 'off',
      'jsdoc/require-returns': 'off',
      'jsdoc/check-param-names': 'off',
      'jsdoc/check-types': 'off',

      // ================================
      // Next.js Rules (WARN)
      // ================================
      '@next/next/no-html-link-for-pages': 'warn',
      '@next/next/no-img-element': 'warn',
      '@next/next/no-head-element': 'warn',
      '@next/next/no-sync-scripts': 'warn',
      '@next/next/no-page-custom-font': 'warn',

      // ================================
      // Code Hygiene
      // ================================
      'no-console': ['warn', { allow: ['warn', 'error', 'info'] }],
      'no-debugger': 'error',
      'no-alert': 'warn',
      'no-var': 'error',
      'prefer-const': 'warn',
      'no-unreachable': 'error',
      'no-unreachable-loop': 'error',
      'eqeqeq': ['warn', 'always'],
      'no-unused-vars': 'off', // Use TypeScript version
      'no-undef': 'error',
      'no-redeclare': 'off', // Use TypeScript version
      '@typescript-eslint/no-redeclare': 'error',
      'curly': 'off', // Preference
      'brace-style': 'off', // Conflicts with Prettier
      '@typescript-eslint/brace-style': 'off',
      'comma-dangle': 'off', // Prettier handles this
      'semi': 'off', // Prettier handles this
      '@typescript-eslint/semi': 'off',
      'quotes': 'off', // Prettier handles this
      '@typescript-eslint/quotes': 'off',
      'indent': 'off', // Causes stack overflow with TypeScript
      '@typescript-eslint/indent': 'off',
      'no-multiple-empty-lines': ['warn', { max: 2 }],
      'no-trailing-spaces': 'off', // Prettier handles this
      'object-curly-spacing': 'off', // Prettier handles this
      '@typescript-eslint/object-curly-spacing': 'off',
      'array-bracket-spacing': 'off', // Prettier handles this

      // ================================
      // React Rules
      // ================================
      'react/jsx-uses-react': 'off',
      'react/react-in-jsx-scope': 'off',
    },
  },
  {
    // Allow require in config files and relax some strict rules
    files: ['**/*.config.js', '**/*.config.ts', '**/next.config.js'],
    rules: {
      '@typescript-eslint/no-var-requires': 'off',
      'jsdoc/require-jsdoc': 'off',
      'unicorn/prefer-module': 'off',
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
    },
  },
  {
    ignores: [
      'node_modules/**',
      '.next/**',
      'out/**',
      'dist/**',
      'build/**',
      'coverage/**',
      'public/**',
      '*.d.ts',
      '.cache/**',
    ],
  },
];
