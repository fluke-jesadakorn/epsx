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
      'import/no-unresolved': 'off',
    }
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
      'jsdoc/require-*': 'off',
      'no-empty': 'off',
    },
  },
  {
    files: ['**/*.js'],
    rules: {
      '@typescript-eslint/no-var-requires': 'off',
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