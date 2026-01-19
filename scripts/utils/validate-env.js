// validate-env.js
// This script validates that all required environment variables defined in .env.example are present.
// It loads .env (if present) using dotenv and checks each variable.

const fs = require('fs');
const path = require('path');

// Load .env if exists
try {
  const envPath = path.resolve(__dirname, '../../.env');
  require('dotenv').config({ path: envPath });
} catch (e) {
  // dotenv may not be installed; ignore if not needed
}

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
  console.error('❌ Missing required environment variables:');
  missing.forEach(v => console.error('  -', v));
  process.exit(1);
} else {
  console.log('✅ All required environment variables are set.');
  process.exit(0);
}
