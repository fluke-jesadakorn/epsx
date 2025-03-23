const fs = require('fs');
const path = require('path');

// Ensure source config is compiled
const srcConfigPath = path.resolve(__dirname, '../src/eslint/base.ts');
const distDir = path.resolve(__dirname, '../dist/eslint');

// Make sure dist directory exists
if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

// Read the source config file
const sourceContent = fs.readFileSync(srcConfigPath, 'utf8');

// Extract the config object definition
const configMatch = sourceContent.match(/const config = ({[\s\S]*?});/);
if (!configMatch) {
  throw new Error('Could not find config object in source file');
}

// Parse the config, replacing the plugins array
let configStr = configMatch[1];
const outputPath = path.resolve(distDir, 'base.js');

fs.writeFileSync(outputPath, `module.exports = ${configStr};`);
