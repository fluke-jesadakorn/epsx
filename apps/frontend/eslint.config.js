const baseConfig = require('../../packages/config/eslint/base.cjs');
const nextjs = require('@next/eslint-plugin-next');

module.exports = [
  ...baseConfig,
  {
    files: ['**/*.ts', '**/*.tsx'],
    plugins: {
      '@next/next': nextjs,
    },
    rules: {
      '@next/next/no-html-link-for-pages': 'error',
      '@next/next/no-img-element': 'warn',
    },
  },
];
