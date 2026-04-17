// validate-env.js
// Validates required environment variables against the merged root env stack.

const fs = require('fs');
const path = require('path');
const { buildRootEnv } = require('./root-env');

const { envName, mergedEnv } = buildRootEnv(process.env);
Object.assign(process.env, mergedEnv);

const envExamplePath = path.resolve(__dirname, '../../.env.example');
if (!fs.existsSync(envExamplePath)) {
  console.error('❌ .env.example file not found at', envExamplePath);
  process.exit(1);
}

const requiredVars = [];
const lines = fs.readFileSync(envExamplePath, 'utf-8').split(/\r?\n/);
for (const line of lines) {
  const trimmed = line.trim();
  if (!trimmed || trimmed.startsWith('#')) continue; // skip comments/empty
  const eqIdx = trimmed.indexOf('=');
  if (eqIdx === -1) continue;
  const key = trimmed.substring(0, eqIdx).trim();
  if (key) requiredVars.push(key);
}

let missing = [];
for (const key of requiredVars) {
  if (!process.env[key]) {
    missing.push(key);
  }
}

if (missing.length > 0) {
  console.error(`❌ Missing required environment variables for ${envName}:`);
  missing.forEach(v => console.error('  -', v));
  process.exit(1);
} else {
  console.log(`✅ All required environment variables are set for ${envName}.`);
  process.exit(0);
}
