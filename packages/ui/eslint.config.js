const baseConfig = require('../../packages/config/eslint/base.cjs')

module.exports = [
  ...baseConfig,
  {
    ignores: ['dist/**', 'node_modules/**']
  }
]
