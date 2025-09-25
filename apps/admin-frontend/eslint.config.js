const js = require('@eslint/js');
const typescript = require('@typescript-eslint/eslint-plugin');
const typescriptParser = require('@typescript-eslint/parser');
const nextjs = require('@next/eslint-plugin-next');

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
      },
      globals: {
        console: 'readonly',
        process: 'readonly',
        fetch: 'readonly',
        FormData: 'readonly',
        URLSearchParams: 'readonly',
        URL: 'readonly',
        RequestInit: 'readonly',
        TextEncoder: 'readonly',
        require: 'readonly',
        window: 'readonly',
        React: 'readonly',
        getAuthorizationUrl: 'readonly'
      },
    },
    plugins: {
      '@typescript-eslint': typescript,
      '@next/next': nextjs,
    },
    rules: {
      // TypeScript rules
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      
      // Next.js rules
      '@next/next/no-html-link-for-pages': 'off',
      '@next/next/no-img-element': 'off',
      '@next/next/no-page-custom-font': 'off',
      
      // General rules
      'no-console': 'off',
      'no-var': 'off',
      'object-shorthand': 'off',
      'no-unreachable': 'off',
      'no-unused-vars': 'off',
      'no-undef': 'off',
      'no-redeclare': 'off',
      'no-case-declarations': 'off',
      'no-useless-escape': 'off',
      
      // React rules
      'react/jsx-uses-react': 'off',
      'react/react-in-jsx-scope': 'off',
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
      'out/**',
      'dist/**',
      'build/**',
      'coverage/**',
      'public/**',
      '*.d.ts',
    ],
  },
];