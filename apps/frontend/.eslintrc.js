module.exports = {
  root: true,
  extends: [
    '../../packages/config/dist/eslint/base',
    'next/core-web-vitals'
  ],
  parserOptions: {
    project: ['./tsconfig.json'],
    tsconfigRootDir: __dirname,
  },
  ignorePatterns: ["lib/store/*"],
  settings: {
    'import/resolver': {
      typescript: {
        project: './tsconfig.json',
      },
    },
    next: {
      rootDir: '.',
    },
  },
  rules: {
    'no-console': ['warn', { allow: ['warn', 'error', 'info', 'debug'] }],
    'no-case-declarations': 'off',
    'no-useless-catch': 'off',
    '@next/next/no-html-link-for-pages': 'error',
    '@next/next/no-img-element': 'error',
    'jsx-a11y/alt-text': ['warn', { elements: ['img'], img: ['Image'] }],
  },
  overrides: [
    {
      files: ['app/**/*.ts?(x)', 'pages/**/*.ts?(x)'],
      rules: {
        'import/no-default-export': 'off',
      },
    },
  ],
};
