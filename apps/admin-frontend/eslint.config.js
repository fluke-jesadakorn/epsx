import { defineFlatConfig } from 'eslint-define-config'
import baseConfig from '../../packages/config/eslint/base.js'
import nextjs from '@next/eslint-plugin-next'

export default defineFlatConfig([
  {
    ...baseConfig,
    // Admin frontend specific overrides
    files: ['**/*.ts', '**/*.tsx'],
    plugins: {
      ...baseConfig.plugins,
      '@next/next': nextjs,
    },
    rules: {
      ...baseConfig.rules,
      '@next/next/no-html-link-for-pages': 'error',
      '@next/next/no-img-element': 'warn',
    }
  }
])
