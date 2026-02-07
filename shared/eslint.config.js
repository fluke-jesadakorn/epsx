
import sharedConfig from './config/eslint.cjs';

export default [
    ...sharedConfig,
    {
        ignores: [
            'node_modules/**',
            'dist/**',
            'coverage/**'
        ]
    }
];
