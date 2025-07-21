#!/usr/bin/env node

import { initializeApp } from 'firebase/app';
import { getFirestore } from 'firebase/firestore';
import { Command } from 'commander';
import { seedInitialProject, seedCollections, validateSeedData, clearAllSeedData } from './index';
import type { SeedOptions } from './types';

const program = new Command();

// Default Firebase config (should be overridden with environment variables)
const defaultFirebaseConfig = {
  apiKey: process.env.FIREBASE_API_KEY || 'demo-api-key',
  authDomain: process.env.FIREBASE_AUTH_DOMAIN || 'demo-project.firebaseapp.com',
  projectId: process.env.FIREBASE_PROJECT_ID || 'demo-project',
  storageBucket: process.env.FIREBASE_STORAGE_BUCKET || 'demo-project.appspot.com',
  messagingSenderId: process.env.FIREBASE_MESSAGING_SENDER_ID || '123456789',
  appId: process.env.FIREBASE_APP_ID || '1:123456789:web:abcdef123456'
};

function initializeFirebase() {
  try {
    const app = initializeApp(defaultFirebaseConfig);
    return getFirestore(app);
  } catch (error) {
    console.error('❌ Failed to initialize Firebase:', error);
    console.log('\n💡 Make sure you have set the following environment variables:');
    console.log('   - FIREBASE_API_KEY');
    console.log('   - FIREBASE_AUTH_DOMAIN');
    console.log('   - FIREBASE_PROJECT_ID');
    console.log('   - FIREBASE_STORAGE_BUCKET');
    console.log('   - FIREBASE_MESSAGING_SENDER_ID');
    console.log('   - FIREBASE_APP_ID');
    process.exit(1);
  }
}

program
  .name('epsx-seed')
  .description('EPSX Platform Database Seeding Tool')
  .version('1.0.0');

program
  .command('init')
  .description('Initialize project with all seed data')
  .option('-e, --environment <env>', 'Target environment', 'development')
  .option('-f, --force', 'Force overwrite existing data', false)
  .option('-v, --verbose', 'Verbose output', true)
  .action(async (options) => {
    console.log('🚀 EPSX Platform Database Seeding Tool v1.0.0\n');
    
    const db = initializeFirebase();
    
    const seedOptions: SeedOptions = {
      environment: options.environment as 'development' | 'production' | 'test',
      force: options.force,
      verbose: options.verbose
    };
    
    try {
      const results = await seedInitialProject(db, seedOptions);
      const failed = results.filter(r => !r.success);
      
      if (failed.length > 0) {
        console.log('\n⚠️  Some seeding operations failed:');
        failed.forEach(r => console.log(`   • ${r.collection}: ${r.error}`));
        process.exit(1);
      }
      
      process.exit(0);
    } catch (error) {
      console.error('\n❌ Seeding failed:', error);
      process.exit(1);
    }
  });

program
  .command('seed')
  .description('Seed specific collections')
  .argument('<collections...>', 'Collection names to seed')
  .option('-e, --environment <env>', 'Target environment', 'development')
  .option('-f, --force', 'Force overwrite existing data', false)
  .option('-v, --verbose', 'Verbose output', true)
  .action(async (collections: string[], options: any) => {
    console.log('🎯 EPSX Selective Seeding Tool\n');
    
    const db = initializeFirebase();
    
    const seedOptions: SeedOptions = {
      environment: options.environment as 'development' | 'production' | 'test',
      force: options.force,
      verbose: options.verbose
    };
    
    try {
      const results = await seedCollections(db, collections, seedOptions);
      const failed = results.filter(r => !r.success);
      
      if (failed.length > 0) {
        console.log('\n⚠️  Some seeding operations failed:');
        failed.forEach(r => console.log(`   • ${r.collection}: ${r.error}`));
        process.exit(1);
      }
      
      process.exit(0);
    } catch (error) {
      console.error('\n❌ Seeding failed:', error);
      process.exit(1);
    }
  });

program
  .command('validate')
  .description('Validate seed data files')
  .action(() => {
    console.log('🔍 Validating seed data...\n');
    
    const validation = validateSeedData();
    
    if (validation.valid) {
      console.log('✅ All seed data is valid!');
      process.exit(0);
    } else {
      console.log('❌ Seed data validation failed:\n');
      validation.errors.forEach(error => console.log(`   • ${error}`));
      process.exit(1);
    }
  });

program
  .command('clear')
  .description('Clear all seeded data (DANGEROUS)')
  .option('--confirm', 'Confirm data clearing', false)
  .action(async (options) => {
    if (!options.confirm) {
      console.log('❌ This command will delete all seeded data!');
      console.log('💡 Add --confirm flag to proceed.');
      process.exit(1);
    }
    
    console.log('🗑️  EPSX Data Clearing Tool\n');
    console.log('⚠️  WARNING: This will delete all seeded data!\n');
    
    const db = initializeFirebase();
    
    try {
      await clearAllSeedData(db, true);
      console.log('\n✅ All data cleared successfully');
      process.exit(0);
    } catch (error) {
      console.error('\n❌ Failed to clear data:', error);
      process.exit(1);
    }
  });

program
  .command('list')
  .description('List available collections for seeding')
  .action(() => {
    console.log('📋 Available Collections:\n');
    
    const collections = [
      { name: 'configuration', description: 'System settings, feature flags, and integrations' },
      { name: 'iam', description: 'Roles, permissions, users, and audit logs' },
      { name: 'organizations', description: 'Organizations, invitations, and user preferences' },
      { name: 'content', description: 'Articles, media, and categories' },
      { name: 'analytics', description: 'Usage analytics and system metrics' },
      { name: 'notifications', description: 'Email templates, notifications, and message queue' }
    ];
    
    collections.forEach(collection => {
      console.log(`   • ${collection.name.padEnd(15)} - ${collection.description}`);
    });
    
    console.log('\n💡 Use "epsx-seed seed <collection>" to seed specific collections');
    console.log('💡 Use "epsx-seed init" to seed all collections');
  });

// Parse command line arguments
program.parse(process.argv);

// Show help if no command provided
if (!process.argv.slice(2).length) {
  program.outputHelp();
}
