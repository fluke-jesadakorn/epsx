#!/usr/bin/env node

/**
 * Cross-platform script to promote a user from email to SuperAdmin role
 * Usage: node scripts/promote-user-admin.js <email> [reason]
 */

import { execSync } from 'child_process';
import { readFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import { platform } from 'os';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Configuration
const CONFIG = {
  backendPath: resolve(__dirname, '../apps/backend'),
  cargoToml: resolve(__dirname, '../apps/backend/Cargo.toml'),
};

function validateEmail(email) {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

function checkBackendExists() {
  try {
    readFileSync(CONFIG.cargoToml, 'utf8');
    return existsSync(CONFIG.backendPath);
  } catch {
    return false;
  }
}

function getShellCommand(email, reason = 'Admin promotion via script') {
  const isWindows = platform() === 'win32';
  const reasonArg = reason ? `--reason="${reason}"` : '';
  
  if (isWindows) {
    return `cd /d "${CONFIG.backendPath}" && cargo run --bin promote_admin -- --email="${email}" ${reasonArg}`;
  } else {
    return `cd "${CONFIG.backendPath}" && cargo run --bin promote_admin -- --email="${email}" ${reasonArg}`;
  }
}

function executeCargoCommand(email, reason = 'Admin promotion via script') {
  const cmd = getShellCommand(email, reason);
  
  console.log(`Executing: ${cmd}`);
  
  try {
    const output = execSync(cmd, { 
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
      timeout: 60000, // 60 seconds timeout
      shell: platform() === 'win32' ? 'cmd.exe' : '/bin/bash'
    });
    return { success: true, output };
  } catch (error) {
    return { 
      success: false, 
      error: error.message,
      output: error.stdout || error.stderr || ''
    };
  }
}

function showHelp() {
  console.log(`
Usage: node scripts/promote-user-admin.js <email> [reason]

Arguments:
  email     Email address of user to promote to SuperAdmin
  reason    Optional reason for the promotion (default: "Admin promotion via script")

Examples:
  node scripts/promote-user-admin.js user@example.com
  node scripts/promote-user-admin.js user@example.com "Emergency admin access needed"
  
  # Cross-platform usage:
  npm run promote-admin user@example.com
  pnpm promote-admin user@example.com

Note: This script requires the backend to be built and available at ${CONFIG.backendPath}
Platform: ${platform()}
  `);
}

function main() {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
    showHelp();
    process.exit(0);
  }

  const email = args[0];
  const reason = args[1];

  // Validate inputs
  if (!validateEmail(email)) {
    console.error(`❌ Error: Invalid email format: ${email}`);
    process.exit(1);
  }

  // Check backend exists
  if (!checkBackendExists()) {
    console.error(`❌ Error: Backend not found at ${CONFIG.backendPath}`);
    console.error('Make sure the backend is built and available');
    process.exit(1);
  }

  console.log(`🔄 Promoting user ${email} to SuperAdmin...`);
  console.log(`📍 Platform: ${platform()}`);
  if (reason) {
    console.log(`📝 Reason: ${reason}`);
  }

  // Execute promotion
  const result = executeCargoCommand(email, reason);

  if (result.success) {
    console.log('✅ User promotion successful!');
    if (result.output) {
      console.log('📋 Output:', result.output);
    }
  } else {
    console.error('❌ User promotion failed!');
    console.error('🚨 Error:', result.error);
    if (result.output) {
      console.error('📋 Output:', result.output);
    }
    process.exit(1);
  }
}

// Handle interruption gracefully
process.on('SIGINT', () => {
  console.log('\n⚠️  Operation interrupted by user');
  process.exit(1);
});

process.on('SIGTERM', () => {
  console.log('\n⚠️  Operation terminated');
  process.exit(1);
});

// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  console.error('🚨 Uncaught Exception:', error.message);
  process.exit(1);
});

main();