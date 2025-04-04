import baseConfig from '@epsx/config/eslint/base'

export default [
  ...baseConfig,
  {
    ignores: ['dist/**', 'node_modules/**']
  }
]
