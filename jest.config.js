/** @type {import('jest').Config} */

const babelTransform = {
  '^.+\\.(js|jsx|ts|tsx)$': ['babel-jest', { presets: ['next/babel'] }],
}

const transformIgnore = [
  '/node_modules/',
  '^.+\\.module\\.(css|sass|scss)$',
]

module.exports = {
  projects: [
    {
      displayName: 'frontend',
      testEnvironment: 'jsdom',
      testMatch: [
        '<rootDir>/tests/unit/frontend/**/*.test.{ts,tsx}',
      ],
      moduleNameMapper: {
        '^server-only$': '<rootDir>/tests/setup/mocks/server-only.js',
        '^@/shared/(.*)$': '<rootDir>/shared/$1',
        '^@/(.*)$': '<rootDir>/apps/frontend/$1',
      },
      setupFilesAfterEnv: ['<rootDir>/tests/setup/frontend.ts'],
      testEnvironmentOptions: { customExportConditions: [''] },
      transform: babelTransform,
      transformIgnorePatterns: transformIgnore,
    },
  ],
}
