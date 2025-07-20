#!/usr/bin/env tsx

/**
 * Runner script for Firebase IAM initialization
 * This script sets up Firebase Authentication users and IAM permissions
 */

import { initializeFirebaseIAM } from './initializeFirebaseIAM';

async function main() {
  console.log('🚀 Starting Firebase IAM initialization...\n');
  
  try {
    await initializeFirebaseIAM();
    console.log('\n✅ Firebase IAM initialization completed successfully!');
    console.log('\n🔧 Next steps:');
    console.log('1. Test the Firebase Auth IAM integration in your app');
    console.log('2. Check the Firebase IAM Debug Panel to verify setup');
    console.log('3. Try signing in with the created users');
    console.log('4. Test feature access controls based on package tiers');
    
    process.exit(0);
  } catch (error) {
    console.error('\n❌ Firebase IAM initialization failed:', error);
    console.log('\n🔧 Troubleshooting:');
    console.log('1. Check your Firebase configuration');
    console.log('2. Verify Firebase Auth and Firestore are enabled');
    console.log('3. Check your internet connection');
    console.log('4. Use the Firebase IAM Debug Panel for diagnostics');
    
    process.exit(1);
  }
}

// Handle uncaught errors
process.on('unhandledRejection', (error) => {
  console.error('\n💥 Unhandled error:', error);
  process.exit(1);
});

// Run the initialization
main();
