
const sharedConfig = require('./config/eslint.cjs');

module.exports = [
    ...sharedConfig,
    {
        ignores: [
            'node_modules/**',
            'dist/**',
            'coverage/**'
        ]
    },
    {
        rules: {
            '@next/next/no-html-link-for-pages': 'off',
        }
    }
];
