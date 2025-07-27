#!/usr/bin/env node

/**
 * Cross-platform script to assign IAM/ACL permission profiles to users
 * Usage: node scripts/assign-iam-acl.js <email> <profile_id> [options]
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

function validateProfileId(profileId) {
  // Basic validation - profile ID should not be empty and follow a reasonable pattern
  return profileId && profileId.length > 0 && /^[a-zA-Z0-9_-]+$/.test(profileId);
}

function validateDateTime(dateTimeStr) {
  try {
    const date = new Date(dateTimeStr);
    return !isNaN(date.getTime()) && dateTimeStr.includes('T');
  } catch {
    return false;
  }
}

function checkBackendExists() {
  try {
    readFileSync(CONFIG.cargoToml, 'utf8');
    return existsSync(CONFIG.backendPath);
  } catch {
    return false;
  }
}

function getAssignCommand(email, profileId, options = {}) {
  const isWindows = platform() === 'win32';
  
  let args = [`--email="${email}"`, `--profile_id="${profileId}"`];
  
  if (options.reason) {
    args.push(`--reason="${options.reason}"`);
  }
  
  if (options.adminId) {
    args.push(`--admin_id="${options.adminId}"`);
  }
  
  if (options.mergePermissions !== undefined) {
    args.push(`--merge_permissions=${options.mergePermissions}`);
  }
  
  if (options.expiresAt) {
    args.push(`--expires_at="${options.expiresAt}"`);
  }
  
  const argsStr = args.join(' ');
  
  if (isWindows) {
    return `cd /d "${CONFIG.backendPath}" && cargo run --bin assign_iam -- ${argsStr}`;
  } else {
    return `cd "${CONFIG.backendPath}" && cargo run --bin assign_iam -- ${argsStr}`;
  }
}

function getListProfilesCommand() {
  const isWindows = platform() === 'win32';
  
  if (isWindows) {
    return `cd /d "${CONFIG.backendPath}" && cargo run --bin assign_iam -- --list_profiles`;
  } else {
    return `cd "${CONFIG.backendPath}" && cargo run --bin assign_iam -- --list_profiles`;
  }
}

function executeCargoCommand(cmd) {
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
Usage: node scripts/assign-iam-acl.js <command> [arguments]

Commands:
  assign <email> <profile_id> [options]   Assign permission profile to user
  list                                    List available permission profiles

Assign Arguments:
  email         Email address of user to assign permissions to
  profile_id    ID of the permission profile to assign

Assign Options:
  --reason <string>              Reason for the assignment
  --admin-id <string>            Admin user ID performing the assignment
  --merge-permissions <boolean>  Whether to merge with existing permissions (default: true)
  --expires-at <datetime>        Expiration date in ISO 8601 format (e.g., 2024-12-31T23:59:59Z)

Examples:
  # List available profiles
  node scripts/assign-iam-acl.js list
  
  # Assign basic user profile
  node scripts/assign-iam-acl.js assign user@example.com user-basic-001
  
  # Assign premium profile with reason
  node scripts/assign-iam-acl.js assign user@example.com user-premium-002 --reason "Upgrade to premium plan"
  
  # Assign with expiration
  node scripts/assign-iam-acl.js assign user@example.com mod-standard-003 --expires-at "2024-12-31T23:59:59Z"
  
  # Assign without merging permissions
  node scripts/assign-iam-acl.js assign user@example.com admin-full-004 --merge-permissions false

  # Cross-platform usage:
  npm run assign-iam user@example.com user-premium-002
  pnpm assign-iam user@example.com user-premium-002

Note: This script requires the backend to be built and available at ${CONFIG.backendPath}
Platform: ${platform()}
  `);
}

function parseOptions(args) {
  const options = {};
  
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    if (arg === '--reason' && i + 1 < args.length) {
      options.reason = args[i + 1];
      i++; // skip next arg
    } else if (arg === '--admin-id' && i + 1 < args.length) {
      options.adminId = args[i + 1];
      i++; // skip next arg
    } else if (arg === '--merge-permissions' && i + 1 < args.length) {
      options.mergePermissions = args[i + 1].toLowerCase() === 'true';
      i++; // skip next arg
    } else if (arg === '--expires-at' && i + 1 < args.length) {
      options.expiresAt = args[i + 1];
      i++; // skip next arg
    }
  }
  
  return options;
}

function main() {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
    showHelp();
    process.exit(0);
  }

  const command = args[0];

  // Check backend exists
  if (!checkBackendExists()) {
    console.error(`❌ Error: Backend not found at ${CONFIG.backendPath}`);
    console.error('Make sure the backend is built and available');
    process.exit(1);
  }

  if (command === 'list') {
    console.log('📋 Listing available permission profiles...');
    console.log(`📍 Platform: ${platform()}`);
    
    const cmd = getListProfilesCommand();
    const result = executeCargoCommand(cmd);
    
    if (result.success) {
      console.log('✅ Successfully retrieved permission profiles!');
      if (result.output) {
        console.log(result.output);
      }
    } else {
      console.error('❌ Failed to list permission profiles!');
      console.error('🚨 Error:', result.error);
      if (result.output) {
        console.error('📋 Output:', result.output);
      }
      process.exit(1);
    }
    
  } else if (command === 'assign') {
    if (args.length < 3) {
      console.error('❌ Error: assign command requires email and profile_id');
      console.error('Usage: node scripts/assign-iam-acl.js assign <email> <profile_id> [options]');
      process.exit(1);
    }
    
    const email = args[1];
    const profileId = args[2];
    const options = parseOptions(args.slice(3));

    // Validate inputs
    if (!validateEmail(email)) {
      console.error(`❌ Error: Invalid email format: ${email}`);
      process.exit(1);
    }

    if (!validateProfileId(profileId)) {
      console.error(`❌ Error: Invalid profile ID format: ${profileId}`);
      console.error('Profile ID should contain only letters, numbers, underscores, and hyphens');
      process.exit(1);
    }

    if (options.expiresAt && !validateDateTime(options.expiresAt)) {
      console.error(`❌ Error: Invalid expiration date format: ${options.expiresAt}`);
      console.error('Use ISO 8601 format (e.g., 2024-12-31T23:59:59Z)');
      process.exit(1);
    }

    console.log(`🔄 Assigning permission profile to user...`);
    console.log(`📧 User: ${email}`);
    console.log(`🔑 Profile: ${profileId}`);
    console.log(`📍 Platform: ${platform()}`);
    
    if (options.reason) {
      console.log(`📝 Reason: ${options.reason}`);
    }
    
    if (options.adminId) {
      console.log(`👤 Admin: ${options.adminId}`);
    }
    
    if (options.expiresAt) {
      console.log(`⏰ Expires: ${options.expiresAt}`);
    }
    
    console.log(`🔀 Merge permissions: ${options.mergePermissions !== false ? 'true' : 'false'}`);

    // Execute assignment
    const cmd = getAssignCommand(email, profileId, options);
    const result = executeCargoCommand(cmd);

    if (result.success) {
      console.log('✅ Permission assignment successful!');
      if (result.output) {
        console.log('📋 Output:', result.output);
      }
    } else {
      console.error('❌ Permission assignment failed!');
      console.error('🚨 Error:', result.error);
      if (result.output) {
        console.error('📋 Output:', result.output);
      }
      process.exit(1);
    }
    
  } else {
    console.error(`❌ Error: Unknown command: ${command}`);
    console.error('Available commands: assign, list');
    console.error('Use --help for more information');
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