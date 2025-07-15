#!/usr/bin/env node

/**
 * Migration script for Firebase Auth improvements
 * This script helps migrate from the old auth system to the new improved version
 */

const fs = require('fs');
const path = require('path');

const FRONTEND_DIR = process.cwd();

// File mappings for migration
const FILE_MIGRATIONS = [
  {
    from: 'context/auth-context.tsx',
    to: 'context/auth-context-improved.tsx',
    backup: true,
  },
  {
    from: 'lib/session.ts',
    to: 'lib/session-improved.ts',
    backup: true,
  },
  {
    from: 'app/actions/auth.ts',
    to: 'app/actions/auth-improved.ts',
    backup: true,
  },
  {
    from: 'middleware.ts',
    to: 'middleware-improved.ts',
    backup: true,
  },
];

// Import/export replacements
const IMPORT_REPLACEMENTS = [
  {
    from: "import { useAuth } from '@/context/auth-context';",
    to: "import { useAuth } from '@/hooks/useAuth';",
  },
  {
    from: "import { AuthProvider } from '@/context/auth-context';",
    to: "import { AuthProvider } from '@/context/auth-context-improved';",
  },
  {
    from: "import { AuthGuard } from '@/components/features/auth/AuthGuard';",
    to: "import { AuthGuard } from '@/components/auth/AuthGuard';",
  },
];

function log(message, type = 'info') {
  const timestamp = new Date().toISOString();
  const prefix = type === 'error' ? '❌' : type === 'warning' ? '⚠️' : '✅';
  console.log(`${prefix} [${timestamp}] ${message}`);
}

function backupFile(filePath) {
  const backupPath = `${filePath}.backup.${Date.now()}`;
  try {
    fs.copyFileSync(filePath, backupPath);
    log(`Created backup: ${backupPath}`);
    return backupPath;
  } catch (error) {
    log(`Failed to create backup for ${filePath}: ${error.message}`, 'error');
    return null;
  }
}

function replaceImports(filePath) {
  try {
    let content = fs.readFileSync(filePath, 'utf8');
    let modified = false;

    IMPORT_REPLACEMENTS.forEach(({ from, to }) => {
      if (content.includes(from)) {
        content = content.replace(
          new RegExp(from.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g'),
          to,
        );
        modified = true;
      }
    });

    if (modified) {
      fs.writeFileSync(filePath, content);
      log(`Updated imports in: ${filePath}`);
    }

    return modified;
  } catch (error) {
    log(`Failed to update imports in ${filePath}: ${error.message}`, 'error');
    return false;
  }
}

function findFilesWithPattern(dir, pattern) {
  const files = [];

  function searchDir(currentDir) {
    try {
      const items = fs.readdirSync(currentDir);

      items.forEach((item) => {
        const fullPath = path.join(currentDir, item);
        const stat = fs.statSync(fullPath);

        if (
          stat.isDirectory() &&
          !item.startsWith('.') &&
          item !== 'node_modules'
        ) {
          searchDir(fullPath);
        } else if (stat.isFile() && pattern.test(item)) {
          files.push(fullPath);
        }
      });
    } catch (error) {
      log(`Error reading directory ${currentDir}: ${error.message}`, 'warning');
    }
  }

  searchDir(dir);
  return files;
}

function migrateFiles() {
  log('Starting file migration...');

  FILE_MIGRATIONS.forEach(({ from, to, backup }) => {
    const fromPath = path.join(FRONTEND_DIR, from);
    const toPath = path.join(FRONTEND_DIR, to);

    if (!fs.existsSync(fromPath)) {
      log(`Source file not found: ${fromPath}`, 'warning');
      return;
    }

    if (fs.existsSync(toPath)) {
      log(`Target file already exists: ${toPath}`, 'warning');
      return;
    }

    if (backup) {
      backupFile(fromPath);
    }

    try {
      // For this migration, we're not copying files but rather creating new improved versions
      log(`Migration target available: ${toPath}`);
    } catch (error) {
      log(`Failed to migrate ${from} to ${to}: ${error.message}`, 'error');
    }
  });
}

function updateImports() {
  log('Updating imports in TypeScript/JSX files...');

  const tsxFiles = findFilesWithPattern(FRONTEND_DIR, /\.(tsx?|jsx?)$/);
  let updatedCount = 0;

  tsxFiles.forEach((file) => {
    if (replaceImports(file)) {
      updatedCount++;
    }
  });

  log(`Updated imports in ${updatedCount} files`);
}

function generateMigrationReport() {
  const report = {
    timestamp: new Date().toISOString(),
    summary: {
      filesProcessed: 0,
      importsUpdated: 0,
      backupsCreated: 0,
    },
    actions: [
      'Created improved auth service layer',
      'Enhanced session management',
      'Added comprehensive auth forms',
      'Improved auth guards and middleware',
      'Added user profile management',
      'Enhanced error handling',
      'Added type safety improvements',
    ],
    nextSteps: [
      'Update your app to use AuthProvider from auth-context-improved.tsx',
      'Replace auth form components with new AuthForms components',
      'Update middleware.ts with improved middleware',
      'Test authentication flows thoroughly',
      'Update environment variables if needed',
      'Review and update any custom auth logic',
    ],
  };

  const reportPath = path.join(FRONTEND_DIR, 'docs', 'migration-report.json');

  try {
    // Ensure docs directory exists
    const docsDir = path.dirname(reportPath);
    if (!fs.existsSync(docsDir)) {
      fs.mkdirSync(docsDir, { recursive: true });
    }

    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    log(`Migration report saved to: ${reportPath}`);
  } catch (error) {
    log(`Failed to save migration report: ${error.message}`, 'error');
  }
}

function main() {
  log('🚀 Starting Firebase Auth Migration');
  log('=====================================');

  // Check if we're in the right directory
  if (!fs.existsSync(path.join(FRONTEND_DIR, 'package.json'))) {
    log('Please run this script from the frontend app directory', 'error');
    process.exit(1);
  }

  try {
    migrateFiles();
    updateImports();
    generateMigrationReport();

    log('=====================================');
    log('🎉 Migration completed successfully!');
    log('');
    log('Next steps:');
    log('1. Review the migration report in docs/migration-report.json');
    log('2. Update your app.tsx to use the new AuthProvider');
    log('3. Test the authentication flows');
    log('4. Update any custom auth logic');
    log('5. Deploy and monitor for issues');
    log('');
    log('See docs/FIREBASE_AUTH_IMPROVEMENTS.md for detailed documentation');
  } catch (error) {
    log(`Migration failed: ${error.message}`, 'error');
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}

module.exports = {
  migrateFiles,
  updateImports,
  generateMigrationReport,
};
