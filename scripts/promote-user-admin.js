#!/usr/bin/env node

/**
 * Cross-platform script to promote a user from email to SuperAdmin role with all access
 * or assign IAM/ACL permission profiles
 * Usage: 
 *   node scripts/promote-user-admin.js promote <email> [reason]
 *   node scripts/promote-user-admin.js assign <email> <profile_id> [options]
 *   node scripts/promote-user-admin.js super-admin <email> [reason]  # Full SuperAdmin with all access
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

function getPromoteCommand(email, reason = 'Admin promotion via script') {
  const isWindows = platform() === 'win32';
  const reasonArg = reason ? `--reason="${reason}"` : '';
  
  if (isWindows) {
    return `cd /d "${CONFIG.backendPath}" && cargo run --bin promote_admin -- --email="${email}" ${reasonArg}`;
  } else {
    return `cd "${CONFIG.backendPath}" && cargo run --bin promote_admin -- --email="${email}" ${reasonArg}`;
  }
}

function getSuperAdminCommand(email, reason = 'Super Admin promotion with full access via script') {
  const isWindows = platform() === 'win32';
  const reasonArg = reason ? `--reason="${reason}"` : '';
  
  if (isWindows) {
    return `cd /d "${CONFIG.backendPath}" && cargo run --bin promote_admin -- --email="${email}" ${reasonArg} --super-admin`;
  } else {
    return `cd "${CONFIG.backendPath}" && cargo run --bin promote_admin -- --email="${email}" ${reasonArg} --super-admin`;
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

function validateProfileId(profileId) {
  return profileId && profileId.length > 0 && /^[a-zA-Z0-9_-]+$/.test(profileId);
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
Usage: node scripts/promote-user-admin.js <command> [arguments]

Commands:
  promote <email> [reason]                      Promote user to SuperAdmin role
  super-admin <email> [reason]                  Promote user to SuperAdmin with ALL ACCESS
  assign <email> <profile_id> [options]        Assign IAM/ACL permission profile

Promote Arguments:
  email     Email address of user to promote to SuperAdmin
  reason    Optional reason for the promotion (default: "Admin promotion via script")

Super-Admin Arguments:
  email     Email address of user to promote to SuperAdmin with ALL ACCESS
  reason    Optional reason for the promotion (default: "Super Admin promotion with full access via script")

Assign Arguments:
  email         Email address of user to assign permissions to
  profile_id    ID of the permission profile to assign

Examples:
  # Promote to SuperAdmin
  node scripts/promote-user-admin.js promote user@example.com
  node scripts/promote-user-admin.js promote user@example.com "Emergency admin access needed"
  
  # Promote to SuperAdmin with ALL ACCESS (recommended)
  node scripts/promote-user-admin.js super-admin user@example.com
  node scripts/promote-user-admin.js super-admin user@example.com "Full system access granted"
  
  # Assign permission profile
  node scripts/promote-user-admin.js assign user@example.com user-premium-002
  
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

  const command = args[0];

  // Check backend exists
  if (!checkBackendExists()) {
    console.error(`❌ Error: Backend not found at ${CONFIG.backendPath}`);
    console.error('Make sure the backend is built and available');
    process.exit(1);
  }

  if (command === 'promote') {
    // Handle promote command
    if (args.length < 2) {
      console.error('❌ Error: promote command requires email');
      console.error('Usage: node scripts/promote-user-admin.js promote <email> [reason]');
      process.exit(1);
    }

    const email = args[1];
    const reason = args[2];

    // Validate inputs
    if (!validateEmail(email)) {
      console.error(`❌ Error: Invalid email format: ${email}`);
      process.exit(1);
    }

    console.log(`🔄 Promoting user ${email} to SuperAdmin...`);
    console.log(`📍 Platform: ${platform()}`);
    if (reason) {
      console.log(`📝 Reason: ${reason}`);
    }

    // Execute promotion
    const cmd = getPromoteCommand(email, reason);
    const result = executeCargoCommand(cmd);

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

  } else if (command === 'super-admin') {
    // Handle super-admin command
    if (args.length < 2) {
      console.error('❌ Error: super-admin command requires email');
      console.error('Usage: node scripts/promote-user-admin.js super-admin <email> [reason]');
      process.exit(1);
    }

    const email = args[1];
    const reason = args[2];

    // Validate inputs
    if (!validateEmail(email)) {
      console.error(`❌ Error: Invalid email format: ${email}`);
      process.exit(1);
    }

    console.log(`🔄 Promoting user ${email} to SuperAdmin with ALL ACCESS...`);
    console.log(`📍 Platform: ${platform()}`);
    if (reason) {
      console.log(`📝 Reason: ${reason}`);
    }

    // Execute super admin promotion
    const cmd = getSuperAdminCommand(email, reason);
    const result = executeCargoCommand(cmd);

    if (result.success) {
      console.log('✅ SuperAdmin promotion with ALL ACCESS successful!');
      console.log('🔑 User now has full system access and all permissions');
      if (result.output) {
        console.log('📋 Output:', result.output);
      }
    } else {
      console.error('❌ SuperAdmin promotion failed!');
      console.error('🚨 Error:', result.error);
      if (result.output) {
        console.error('📋 Output:', result.output);
      }
      process.exit(1);
    }

  } else if (command === 'assign') {
    // Handle assign command
    if (args.length < 3) {
      console.error('❌ Error: assign command requires email and profile_id');
      console.error('Usage: node scripts/promote-user-admin.js assign <email> <profile_id> [options]');
      process.exit(1);
    }

    const email = args[1];
    const profileId = args[2];
    
    // Parse options (simplified version)
    const options = {};
    for (let i = 3; i < args.length; i += 2) {
      if (args[i] === '--reason' && i + 1 < args.length) {
        options.reason = args[i + 1];
      }
    }

    // Validate inputs
    if (!validateEmail(email)) {
      console.error(`❌ Error: Invalid email format: ${email}`);
      process.exit(1);
    }

    if (!validateProfileId(profileId)) {
      console.error(`❌ Error: Invalid profile ID format: ${profileId}`);
      process.exit(1);
    }

    console.log(`🔄 Assigning permission profile to user...`);
    console.log(`📧 User: ${email}`);
    console.log(`🔑 Profile: ${profileId}`);
    console.log(`📍 Platform: ${platform()}`);
    
    if (options.reason) {
      console.log(`📝 Reason: ${options.reason}`);
    }

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
    // Handle legacy usage (backward compatibility) or unknown command
    const email = command;
    const reason = args[1];

    // Check if it looks like an email (legacy usage)
    if (validateEmail(email)) {
      console.log('⚠️  Using legacy format. Consider using: node scripts/promote-user-admin.js super-admin ' + email);
      console.log('⚠️  Note: For full system access, use the "super-admin" command instead of legacy format');
      console.log(`🔄 Promoting user ${email} to SuperAdmin...`);
      console.log(`📍 Platform: ${platform()}`);
      if (reason) {
        console.log(`📝 Reason: ${reason}`);
      }

      // Execute promotion (legacy)
      const cmd = getPromoteCommand(email, reason);
      const result = executeCargoCommand(cmd);

      if (result.success) {
        console.log('✅ User promotion successful!');
        console.log('💡 Tip: Use "super-admin" command for full system access next time');
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
    } else {
      console.error(`❌ Error: Unknown command: ${command}`);
      console.error('Available commands: promote, super-admin, assign');
      console.error('Use --help for more information');
      process.exit(1);
    }
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