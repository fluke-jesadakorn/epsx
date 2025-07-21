import { readFileSync } from 'fs';
import { join } from 'path';
import type { Firestore } from 'firebase/firestore';
import type { SeedOptions, SeedResult, PackagePermission } from './types';
import {
  SeedManager,
  IAMSeeder,
  OrganizationSeeder,
  ContentSeeder,
  AnalyticsSeeder,
  NotificationSeeder,
  ConfigurationSeeder
} from './seeders';

/**
 * Load static data from JSON files
 */
export function loadStaticSeedData() {
  const dataPath = join(__dirname, 'data');
  
  try {
    const roles = JSON.parse(readFileSync(join(dataPath, 'roles.json'), 'utf8'));
    const permissions = JSON.parse(readFileSync(join(dataPath, 'permissions.json'), 'utf8'));
    const packagePermissions = JSON.parse(readFileSync(join(dataPath, 'package-permissions.json'), 'utf8'));
    
    return { roles, permissions, packagePermissions };
  } catch (error) {
    console.error('Error loading static seed data:', error);
    throw new Error('Failed to load static seed data files');
  }
}

/**
 * Get package permissions for a specific package level
 */
export function getPackagePermissions(packageLevel: string): PackagePermission[] {
  const { packagePermissions } = loadStaticSeedData();
  return packagePermissions[packageLevel] || [];
}

/**
 * Create and configure all seeders
 */
export function createSeedManager(db: Firestore, options: SeedOptions = { environment: 'development' }): SeedManager {
  const manager = new SeedManager(db, options);
  
  // Add all seeders in dependency order
  manager.addSeeder(new ConfigurationSeeder(db, options));  // First - system configuration
  manager.addSeeder(new IAMSeeder(db, options));           // Second - users and permissions
  manager.addSeeder(new OrganizationSeeder(db, options));  // Third - organizations and preferences
  manager.addSeeder(new ContentSeeder(db, options));       // Fourth - content and media
  manager.addSeeder(new AnalyticsSeeder(db, options));     // Fifth - analytics data
  manager.addSeeder(new NotificationSeeder(db, options));  // Sixth - notifications and templates
  
  return manager;
}

/**
 * Run all seeders for initial project setup
 */
export async function seedInitialProject(
  db: Firestore, 
  options: SeedOptions = { environment: 'development', verbose: true }
): Promise<SeedResult[]> {
  console.log('🌱 Starting Initial Project Seeding...\n');
  console.log(`Environment: ${options.environment}`);
  console.log(`Force mode: ${options.force ? 'enabled' : 'disabled'}`);
  console.log(`Verbose mode: ${options.verbose ? 'enabled' : 'disabled'}\n`);
  
  const manager = createSeedManager(db, options);
  const results = await manager.runAll();
  
  console.log('\n🎉 Initial project seeding completed!');
  console.log('\n📋 What was seeded:');
  console.log('   • System configuration and feature flags');
  console.log('   • IAM system (roles, permissions, users)');
  console.log('   • Organizations and user preferences');
  console.log('   • Sample content and media library');
  console.log('   • Analytics and usage tracking');
  console.log('   • Notification templates and queue');
  
  const successful = results.filter(r => r.success);
  const totalDocuments = successful.reduce((sum, r) => sum + r.count, 0);
  
  console.log(`\n📊 Total: ${totalDocuments} documents seeded across ${successful.length} collections`);
  
  return results;
}

/**
 * Seed only specific collections
 */
export async function seedCollections(
  db: Firestore,
  collections: string[],
  options: SeedOptions = { environment: 'development' }
): Promise<SeedResult[]> {
  const seedOptions = { ...options, collections };
  const manager = createSeedManager(db, seedOptions);
  
  console.log(`🎯 Seeding specific collections: ${collections.join(', ')}\n`);
  
  const results = await manager.runAll();
  
  console.log('\n✅ Selective seeding completed!');
  
  return results;
}

/**
 * Validate seed data before running
 */
export function validateSeedData(): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  try {
    const { roles, permissions, packagePermissions } = loadStaticSeedData();
    
    // Validate roles
    if (!Array.isArray(roles) || roles.length === 0) {
      errors.push('Roles data is invalid or empty');
    }
    
    // Validate permissions
    if (!Array.isArray(permissions) || permissions.length === 0) {
      errors.push('Permissions data is invalid or empty');
    }
    
    // Validate package permissions
    if (!packagePermissions || typeof packagePermissions !== 'object') {
      errors.push('Package permissions data is invalid');
    }
    
    // Check required package levels
    const requiredLevels = ['FREE', 'BRONZE', 'SILVER', 'GOLD', 'ENTERPRISE'];
    for (const level of requiredLevels) {
      if (!packagePermissions[level]) {
        errors.push(`Missing package permissions for level: ${level}`);
      }
    }
    
    // Validate role structure
    for (const role of roles) {
      if (!role.id || !role.name || !Array.isArray(role.permissions)) {
        errors.push(`Invalid role structure: ${role.id || 'unknown'}`);
      }
    }
    
    // Validate permission structure
    for (const permission of permissions) {
      if (!permission.id || !permission.featureId || !permission.permission) {
        errors.push(`Invalid permission structure: ${permission.id || 'unknown'}`);
      }
    }
    
  } catch (error) {
    errors.push(`Failed to load or parse seed data: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
  
  return {
    valid: errors.length === 0,
    errors
  };
}

/**
 * Clear all seeded data (use with caution!)
 */
export async function clearAllSeedData(
  db: Firestore,
  confirm: boolean = false
): Promise<void> {
  if (!confirm) {
    throw new Error('Must explicitly confirm data clearing by passing confirm=true');
  }
  
  console.log('🗑️  Clearing all seed data...');
  
  const collections = [
    'systemSettings', 'featureFlags', 'integrations',
    'roles', 'permissions', 'users', 'userSessions', 'auditLogs',
    'organizations', 'invitations', 'userPreferences',
    'categories', 'content', 'media',
    'usageAnalytics', 'systemMetrics', 'featureUsage', 'usageTracking',
    'emailTemplates', 'notifications', 'messageQueue'
  ];
  
  const manager = createSeedManager(db, { environment: 'development', force: true });
  
  for (const collection of collections) {
    try {
      await manager.runSeeder(collection);
    } catch (error) {
      console.warn(`Warning: Could not clear collection ${collection}:`, error);
    }
  }
  
  console.log('✅ All seed data cleared');
}

// Re-export types and utilities
export * from './types';
export * from './seeders';
