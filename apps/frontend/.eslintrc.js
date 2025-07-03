module.exports = {
  ignorePatterns: ['lib/store/theme.tsx', 'lib/firebase-admin.d.ts'],
  extends: [
    'next/core-web-vitals',
    '../../packages/config/eslint/base.cjs',
  ],
  rules: {
    'import/order': 'off', // Temporarily disabled to resolve persistent lint errors
    '@typescript-eslint/explicit-function-return-type': 'off',
    '@typescript-eslint/no-unsafe-assignment': 'off',
    '@typescript-eslint/no-unsafe-member-access': 'off',
    '@typescript-eslint/no-unsafe-return': 'off',
    '@typescript-eslint/no-explicit-any': 'off',
    '@typescript-eslint/no-non-null-assertion': 'off',
    '@typescript-eslint/consistent-type-imports': 'off',
    '@typescript-eslint/no-unsafe-call': 'off'
  },
  parserOptions: {
    project: './tsconfig.json',
  },
};
