
import sharedConfig from './config/eslint.cjs';

console.log('Shared sharedConfig type:', Array.isArray(sharedConfig) ? 'array' : typeof sharedConfig);
console.log('Shared sharedConfig length:', Array.isArray(sharedConfig) ? sharedConfig.length : 'N/A');

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
