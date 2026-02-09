/**
 * Admin Frontend ESLint Configuration
 * Extends ultra-strict shared configuration
 */

const sharedConfig = require('../../shared/config/eslint.cjs');

module.exports = [
  ...sharedConfig,
  {
    files: ['**/*.spec.ts', '**/*.test.ts', '**/__test__/**'],
    rules: {
      'no-console': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      'max-lines-per-function': 'off',
      'complexity': 'off',
      // 'sonarjs/cognitive-complexity': 'off',
      'max-nested-callbacks': 'off',
      'max-depth': 'off',
      'jsdoc/require-*': 'off',
      'no-empty': 'off',
      'no-empty-pattern': 'off', // Playwright fixtures often have empty patterns
      'sonarjs/no-duplicate-string': 'off',
      'promise/prefer-await-to-then': 'off',
      'react-hooks/rules-of-hooks': 'off', // Playwright `use` is not a React hook
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
    ignores: ['.debug/**'],
  },
];
