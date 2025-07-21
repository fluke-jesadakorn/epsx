/**
 * Example usage of EPSX seeding functionality
 * 
 * This file demonstrates how to use the seeding system in different scenarios
 */

import { initializeApp } from 'firebase/app';
import { getFirestore } from 'firebase/firestore';
import {
  seedInitialProject,
  seedCollections,
  createSeedManager,
  validateSeedData,
  loadStaticSeedData,
  getPackagePermissions,
  type SeedOptions
} from './index';

// Example Firebase configuration
const firebaseConfig = {
  apiKey: process.env.FIREBASE_API_KEY,
  authDomain: process.env.FIREBASE_AUTH_DOMAIN,
  projectId: process.env.FIREBASE_PROJECT_ID,
  storageBucket: process.env.FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.FIREBASE_APP_ID
};

// Initialize Firebase
const app = initializeApp(firebaseConfig);
const db = getFirestore(app);

/**
 * Example 1: Complete project initialization (recommended for new projects)
 */
export async function initializeNewProject() {
  console.log('🚀 Initializing new EPSX project...');
  
  // First, validate the seed data
  const validation = validateSeedData();
  if (!validation.valid) {
    console.error('❌ Seed data validation failed:', validation.errors);
    return;
  }
  
  // Seed all collections
  const options: SeedOptions = {
    environment: 'development',
    force: true, // Overwrite existing data
    verbose: true
  };
  
  try {
    const results = await seedInitialProject(db, options);
    
    // Check for failures
    const failed = results.filter(r => !r.success);
    if (failed.length > 0) {
      console.error('❌ Some collections failed to seed:', failed);
    } else {
      console.log('✅ Project initialized successfully!');
    }
    
    return results;
  } catch (error) {
    console.error('❌ Project initialization failed:', error);
    throw error;
  }
}

/**
 * Example 2: Selective seeding (useful for development and testing)
 */
export async function seedDevelopmentData() {
  console.log('🔧 Seeding development data...');
  
  // Only seed essential collections for development
  const collections = ['iam', 'organizations', 'configuration'];
  
  const options: SeedOptions = {
    environment: 'development',
    force: true,
    verbose: true
  };
  
  try {
    const results = await seedCollections(db, collections, options);
    console.log('✅ Development data seeded successfully!');
    return results;
  } catch (error) {
    console.error('❌ Development seeding failed:', error);
    throw error;
  }
}

/**
 * Example 3: Production seeding (careful approach)
 */
export async function seedProductionData() {
  console.log('🏭 Seeding production data...');
  
  // Production seeding should be more careful
  const options: SeedOptions = {
    environment: 'production',
    force: false, // Don't overwrite existing data
    verbose: false // Less verbose for production
  };
  
  try {
    // Only seed configuration and basic IAM data for production
    const results = await seedCollections(db, ['configuration', 'iam'], options);
    
    console.log('✅ Production seeding completed!');
    return results;
  } catch (error) {
    console.error('❌ Production seeding failed:', error);
    throw error;
  }
}

/**
 * Example 4: Custom seeding with fine-grained control
 */
export async function customSeeding() {
  console.log('⚙️ Custom seeding example...');
  
  const manager = createSeedManager(db, {
    environment: 'development',
    verbose: true
  });
  
  try {
    // Seed collections in specific order
    await manager.runSeeder('configuration');
    console.log('Configuration seeded');
    
    await manager.runSeeder('iam');
    console.log('IAM seeded');
    
    // Add custom logic between seeders if needed
    console.log('Performing custom operations...');
    
    await manager.runSeeder('content');
    console.log('Content seeded');
    
    console.log('✅ Custom seeding completed!');
  } catch (error) {
    console.error('❌ Custom seeding failed:', error);
    throw error;
  }
}

/**
 * Example 5: Working with static data
 */
export function workWithStaticData() {
  console.log('📄 Working with static seed data...');
  
  try {
    // Load all static data
    const { roles, permissions, packagePermissions } = loadStaticSeedData();
    
    console.log(`Loaded ${roles.length} roles`);
    console.log(`Loaded ${permissions.length} permissions`);
    console.log(`Loaded package permissions for ${Object.keys(packagePermissions).length} tiers`);
    
    // Get permissions for specific package level
    const goldPermissions = getPackagePermissions('GOLD');
    console.log(`GOLD tier has ${goldPermissions.length} feature permissions`);
    
    // Example: Find admin role
    const adminRole = roles.find((role: any) => role.id === 'admin');
    if (adminRole) {
      console.log(`Admin role has permissions: ${adminRole.permissions.join(', ')}`);
    }
    
    // Example: List all permission categories
    const categories = [...new Set(permissions.map((p: any) => p.category))];
    console.log(`Permission categories: ${categories.join(', ')}`);
    
  } catch (error) {
    console.error('❌ Failed to load static data:', error);
  }
}

/**
 * Example 6: Testing scenario
 */
export async function seedTestData() {
  console.log('🧪 Seeding test data...');
  
  const options: SeedOptions = {
    environment: 'test',
    force: true, // Always overwrite in tests
    verbose: false // Keep tests quiet
  };
  
  try {
    // For testing, we might only need core functionality
    const results = await seedCollections(db, ['iam', 'organizations'], options);
    
    console.log('✅ Test data seeded successfully!');
    return results;
  } catch (error) {
    console.error('❌ Test seeding failed:', error);
    throw error;
  }
}

/**
 * Example 7: Validation and health checks
 */
export async function validateAndSeed() {
  console.log('🔍 Validating data before seeding...');
  
  // Always validate before seeding
  const validation = validateSeedData();
  if (!validation.valid) {
    console.error('❌ Validation failed:');
    validation.errors.forEach(error => console.error(`   • ${error}`));
    return false;
  }
  
  console.log('✅ Data validation passed!');
  
  // Proceed with seeding
  try {
    const results = await seedInitialProject(db, {
      environment: 'development',
      force: false,
      verbose: true
    });
    
    // Analyze results
    const successful = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);
    const totalDocs = successful.reduce((sum, r) => sum + r.count, 0);
    
    console.log(`\n📊 Seeding Summary:`);
    console.log(`   ✅ Successful: ${successful.length} collections`);
    console.log(`   ❌ Failed: ${failed.length} collections`);
    console.log(`   📄 Total documents: ${totalDocs}`);
    
    if (failed.length > 0) {
      console.log(`\n❌ Failed collections:`);
      failed.forEach(r => console.log(`   • ${r.collection}: ${r.error}`));
    }
    
    return failed.length === 0;
  } catch (error) {
    console.error('❌ Seeding failed:', error);
    return false;
  }
}

// Export examples for use in other modules
export const examples = {
  initializeNewProject,
  seedDevelopmentData,
  seedProductionData,
  customSeeding,
  workWithStaticData,
  seedTestData,
  validateAndSeed
};

// If running this file directly, run a demo
if (require.main === module) {
  console.log('🎯 Running seeding examples...\n');
  
  // Run validation example
  workWithStaticData();
  
  console.log('\n💡 Other examples available:');
  console.log('   • initializeNewProject() - Complete project setup');
  console.log('   • seedDevelopmentData() - Development environment');
  console.log('   • seedProductionData() - Production environment');
  console.log('   • customSeeding() - Fine-grained control');
  console.log('   • seedTestData() - Testing scenarios');
  console.log('   • validateAndSeed() - With validation');
}
