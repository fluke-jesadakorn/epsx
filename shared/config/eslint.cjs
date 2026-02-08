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
 * - eslint-plugin-react
 */

const js = require('@eslint/js');
const typescript = require('@typescript-eslint/eslint-plugin');
const typescriptParser = require('@typescript-eslint/parser');
const reactHooks = require('eslint-plugin-react-hooks');
const jsxA11y = require('eslint-plugin-jsx-a11y');
const importPlugin = require('eslint-plugin-import');
const security = require('eslint-plugin-security');
const sonarjs = require('eslint-plugin-sonarjs');
const unicorn = require('eslint-plugin-unicorn').default;
const jsdoc = require('eslint-plugin-jsdoc');
const promise = require('eslint-plugin-promise');
const nextjs = require('@next/eslint-plugin-next');
const react = require('eslint-plugin-react');
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
      'react': react,
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
      '@typescript-eslint/no-unsafe-assignment': 'error',
      '@typescript-eslint/no-unsafe-member-access': 'error',
      '@typescript-eslint/no-unsafe-call': 'error',
      '@typescript-eslint/no-unsafe-return': 'error',
      '@typescript-eslint/no-unsafe-argument': 'error',
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      '@typescript-eslint/await-thenable': 'error',
      '@typescript-eslint/require-await': 'error',
      '@typescript-eslint/no-unused-vars': ['error', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_'
      }],
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/strict-boolean-expressions': 'error',
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@typescript-eslint/prefer-nullish-coalescing': 'error',
      '@typescript-eslint/prefer-optional-chain': 'error',
      '@typescript-eslint/no-unnecessary-condition': 'error',
      '@typescript-eslint/no-unnecessary-type-assertion': 'error',
      '@typescript-eslint/prefer-as-const': 'error',
      '@typescript-eslint/no-inferrable-types': 'error',
      '@typescript-eslint/naming-convention': 'off',
      '@typescript-eslint/consistent-type-imports': ['error', { prefer: 'type-imports' }],
      '@typescript-eslint/no-shadow': 'error',
      '@typescript-eslint/no-use-before-define': ['error', { functions: false, classes: true, variables: true }],
      '@typescript-eslint/consistent-type-definitions': ['error', 'interface'],
      '@typescript-eslint/no-confusing-void-expression': ['error', { ignoreArrowShorthand: true }],
      '@typescript-eslint/no-unnecessary-qualifier': 'error',
      '@typescript-eslint/no-unnecessary-type-arguments': 'error',
      '@typescript-eslint/prefer-reduce-type-parameter': 'error',
      '@typescript-eslint/no-useless-empty-export': 'error',

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
      'import/no-cycle': 'error',
      'import/first': 'error',
      'import/newline-after-import': 'error',
      'import/order': 'off',

      // ================================
      // Security (ERROR)
      // ================================
      'security/detect-object-injection': 'off',
      'security/detect-non-literal-regexp': 'error',
      'security/detect-unsafe-regex': 'error',
      'security/detect-buffer-noassert': 'error',
      'security/detect-child-process': 'error',
      'security/detect-disable-mustache-escape': 'error',
      'security/detect-eval-with-expression': 'error',
      'security/detect-no-csrf-before-method-override': 'error',
      'security/detect-non-literal-fs-filename': 'off',
      'security/detect-non-literal-require': 'off',
      'security/detect-possible-timing-attacks': 'off',
      'security/detect-pseudoRandomBytes': 'error',

      // ================================
      // Complexity Limits (Ultra-Strict)
      // ================================
      'complexity': ['error', 12],
      'max-depth': ['error', 3],
      'max-lines-per-function': ['error', { max: 120, skipBlankLines: true, skipComments: true }],
      'max-params': ['error', 3],
      'max-nested-callbacks': ['error', 3],
      'sonarjs/cognitive-complexity': ['error', 20],
      'sonarjs/no-identical-functions': 'error',
      'sonarjs/no-duplicate-string': ['error', { threshold: 4 }],
      'sonarjs/no-collapsible-if': 'error',
      'sonarjs/no-collection-size-mischeck': 'error',
      'sonarjs/no-redundant-jump': 'error',
      'sonarjs/prefer-immediate-return': 'error',

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
      'no-console': 'error',
      'no-debugger': 'error',
      'no-alert': 'error',
      'no-var': 'error',
      'prefer-const': 'error',
      'no-unreachable': 'error',
      'no-unreachable-loop': 'error',
      'eqeqeq': ['error', 'always'],
      'no-unused-vars': 'off',
      'no-undef': 'error',
      'no-redeclare': 'off',
      '@typescript-eslint/no-redeclare': 'error',
      'curly': ['error', 'all'],
      'no-multiple-empty-lines': ['error', { max: 1 }],

      // ================================
      // React Rules
      // ================================
      'react/jsx-uses-react': 'off',
      'react/react-in-jsx-scope': 'off',
      'react/no-array-index-key': 'error',
      'react/self-closing-comp': 'error',

      // ================================
      // Unicorn Rules (Strict)
      // ================================
      'unicorn/no-array-push-push': 'error',
      'unicorn/no-instanceof-array': 'error',
      'unicorn/no-useless-spread': 'error',
      'unicorn/prefer-query-selector': 'error',
      'unicorn/prefer-string-slice': 'error',
      'unicorn/prefer-node-protocol': 'error',
      'unicorn/no-unnecessary-await': 'error',
      'unicorn/prefer-modern-math-apis': 'error',
      'unicorn/no-typeof-undefined': 'error',
      'unicorn/consistent-function-scoping': ['error', { checkArrowFunctions: false }],
      'unicorn/filename-case': ['error', { case: 'kebabCase', ignore: ['README.md', 'Dockerfile'] }],

      // ================================
      // Additional Hygiene
      // ================================
      'no-implicit-coercion': 'error',
      '@typescript-eslint/no-unused-expressions': 'error',
      'no-lonely-if': 'error',
      'no-unneeded-ternary': 'error',
      'object-shorthand': ['error', 'always'],
      'prefer-template': 'error',
    },
  },
  {
    // Disable no-undef for TypeScript files as TS compiler handles this
    files: ['**/*.{ts,tsx}'],
    rules: {
      'no-undef': 'off',
    },
  },
  {
    // Test files globals and overrides
    files: [
      '**/*.spec.ts',
      '**/*.spec.tsx',
      '**/*.test.ts',
      '**/*.test.tsx',
      '**/__test__/**',
      '**/tests/**'
    ],
    languageOptions: {
      globals: {
        ...globals.jest,
        describe: 'readonly',
        it: 'readonly',
        expect: 'readonly',
        beforeEach: 'readonly',
        afterEach: 'readonly',
        beforeAll: 'readonly',
        afterAll: 'readonly',
        jest: 'readonly',
        vi: 'readonly', // For vitest if used
      },
    },
    rules: {
      'no-console': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-argument': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
      'max-lines-per-function': 'off',
      'complexity': 'off',
      'sonarjs/cognitive-complexity': 'off',
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
