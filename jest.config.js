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
        '<rootDir>/tests/integration/frontend/**/*.test.{ts,tsx}',
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
    {
      displayName: 'admin',
      testEnvironment: 'jsdom',
      testMatch: ['<rootDir>/tests/unit/admin/**/*.test.{ts,tsx}'],
      moduleNameMapper: {
        '^server-only$': '<rootDir>/tests/setup/mocks/server-only.js',
        '^@/shared/(.*)$': '<rootDir>/shared/$1',
        '^@/(.*)$': '<rootDir>/apps/admin-frontend/$1',
      },
      setupFilesAfterEnv: ['<rootDir>/tests/setup/admin.ts'],
      testEnvironmentOptions: { customExportConditions: [''] },
      transform: babelTransform,
      transformIgnorePatterns: transformIgnore,
    },
    {
      displayName: 'shared',
      testEnvironment: 'node',
      testMatch: ['<rootDir>/tests/unit/shared/**/*.test.{ts,tsx}'],
      moduleNameMapper: {
        '^@shared/(.*)$': '<rootDir>/shared/$1',
        '^@/shared/(.*)$': '<rootDir>/shared/$1',
      },
      transform: babelTransform,
      transformIgnorePatterns: transformIgnore,
    },
  ],
}
