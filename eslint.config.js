const sharedConfig = require('./shared/config/eslint.cjs');

module.exports = [
  ...sharedConfig,
  {
    ignores: [
      '**/node_modules/**',
      '**/.next/**',
      '**/out/**',
      '**/dist/**',
      '**/build/**',
      '**/coverage/**',
      '**/public/**',
      '**/*.d.ts',
      '**/.cache/**',
      '**/playwright-report/**',
      '**/test-results/**',
      '**/.turbo/**',
      '**/.vercel/**',
      'apps/**',  // Apps will use their own eslint configs
      '.git/**',
      '.debug/**',
    ],
  },
];
