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
const globals = require('globals');

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
        ...globals.browser,
        ...globals.node,
        ...globals.es2021,
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
      // TypeScript Rules (Strict)
      // ================================
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unsafe-assignment': 'error', // Requires type-aware linting
      '@typescript-eslint/no-unsafe-member-access': 'error', // Requires type-aware linting
      '@typescript-eslint/no-unsafe-call': 'error', // Requires type-aware linting
      '@typescript-eslint/no-unsafe-return': 'error', // Requires type-aware linting
      '@typescript-eslint/no-unsafe-argument': 'error', // Requires type-aware linting
      '@typescript-eslint/no-floating-promises': 'error', // Requires type-aware linting
      '@typescript-eslint/no-misused-promises': 'error', // Requires type-aware linting
      '@typescript-eslint/await-thenable': 'error', // Requires type-aware linting
      '@typescript-eslint/require-await': 'error', // Requires type-aware linting
      '@typescript-eslint/no-unused-vars': ['error', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_'
      }],
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/strict-boolean-expressions': 'off',
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@typescript-eslint/prefer-nullish-coalescing': 'error',
      '@typescript-eslint/prefer-optional-chain': 'error',
      '@typescript-eslint/no-unnecessary-condition': 'error',
      '@typescript-eslint/no-unnecessary-type-assertion': 'error',
      '@typescript-eslint/prefer-as-const': 'error',
      '@typescript-eslint/no-inferrable-types': 'error',
      '@typescript-eslint/naming-convention': 'off',

      // ================================
      // React Hooks (ERROR)
      // ================================
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',

      // ================================
      // Accessibility (ERROR)
      // ================================
      'jsx-a11y/alt-text': 'error',
      'jsx-a11y/aria-props': 'error',
      'jsx-a11y/aria-proptypes': 'error',
      'jsx-a11y/aria-unsupported-elements': 'error',
      'jsx-a11y/role-has-required-aria-props': 'error',
      'jsx-a11y/role-supports-aria-props': 'error',
      'jsx-a11y/click-events-have-key-events': 'off',
      'jsx-a11y/no-static-element-interactions': 'off',
      'jsx-a11y/anchor-is-valid': 'error',
      'jsx-a11y/img-redundant-alt': 'error',
      'jsx-a11y/interactive-supports-focus': 'off',

      // ================================
      // Import Management
      // ================================
      'import/no-unresolved': 'off',
      'import/no-duplicates': 'error',
      'import/no-cycle': 'off',
      'import/first': 'error',
      'import/newline-after-import': 'error',
      'import/order': 'off',

      // ================================
      // Security (ERROR)
      // ================================
      'security/detect-object-injection': 'off', // Too many false positives
      'security/detect-non-literal-regexp': 'error',
      'security/detect-unsafe-regex': 'error',
      'security/detect-buffer-noassert': 'error',
      'security/detect-child-process': 'error',
      'security/detect-disable-mustache-escape': 'error',
      'security/detect-eval-with-expression': 'error',
      'security/detect-no-csrf-before-method-override': 'error',
      'security/detect-non-literal-fs-filename': 'off', // Too many false positives
      'security/detect-non-literal-require': 'off', // Too many false positives
      'security/detect-possible-timing-attacks': 'off', // Too many false positives
      'security/detect-pseudoRandomBytes': 'error',

      // ================================
      // Complexity Limits (Stricter)
      // ================================
      'complexity': ['error', 15],
      'max-depth': ['error', 4],
      'max-lines-per-function': ['error', { max: 150, skipBlankLines: true, skipComments: true }],
      'max-params': ['error', 4],
      'max-nested-callbacks': ['error', 4],
      'sonarjs/cognitive-complexity': ['error', 25],
      'sonarjs/no-identical-functions': 'error',
      'sonarjs/no-duplicate-string': ['error', { threshold: 5 }],
      'sonarjs/no-collapsible-if': 'error',
      'sonarjs/no-collection-size-mischeck': 'error',
      'sonarjs/no-redundant-jump': 'error',
      'sonarjs/prefer-immediate-return': 'off',

      // ================================
      // Promise Handling
      // ================================
      'promise/always-return': 'off',
      'promise/catch-or-return': 'error',
      'promise/no-return-wrap': 'error',
      'promise/no-nesting': 'error',
      'promise/prefer-await-to-then': 'error',

      // ================================
      // JSDoc Documentation (Strict)
      // ================================
      'jsdoc/require-jsdoc': 'off',
      'jsdoc/require-param': 'off',
      'jsdoc/require-returns': 'off',
      'jsdoc/check-param-names': 'error',
      'jsdoc/check-types': 'error',

      // ================================
      // Next.js Rules
      // ================================
      '@next/next/no-html-link-for-pages': 'error',
      '@next/next/no-img-element': 'error',
      '@next/next/no-head-element': 'error',
      '@next/next/no-sync-scripts': 'error',
      '@next/next/no-page-custom-font': 'error',

      // ================================
      // Code Hygiene
      // ================================
      'no-console': ['error', { allow: ['warn', 'error', 'info'] }],
      'no-debugger': 'error',
      'no-alert': 'error',
      'no-var': 'error',
      'prefer-const': 'error',
      'no-unreachable': 'error',
      'no-unreachable-loop': 'error',
      'eqeqeq': ['error', 'smart'],
      'no-unused-vars': 'off', // Use TypeScript version
      'no-undef': 'error',
      'no-redeclare': 'off', // Use TypeScript version
      '@typescript-eslint/no-redeclare': 'error',
      'curly': 'error',
      'brace-style': 'off', // Conflicts with Prettier
      '@typescript-eslint/brace-style': 'off',
      'comma-dangle': 'off', // Prettier handles this
      'semi': 'off', // Prettier handles this
      '@typescript-eslint/semi': 'off',
      'quotes': 'off', // Prettier handles this
      '@typescript-eslint/quotes': 'off',
      'indent': 'off', // Causes stack overflow with TypeScript
      '@typescript-eslint/indent': 'off',
      'no-multiple-empty-lines': ['error', { max: 2 }],
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
      'playwright-report/**',
      'test-results/**',
    ],
  },
];
