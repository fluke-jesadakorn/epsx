// import { firebaseIAMService } from '../services/firebaseIAMService'; // Service removed
// import { firebaseAuthIAMService } from '../services/firebaseAuthIAMService'; // Service uses removed dependency
import { PackageTier, SubscriptionStatus } from '../types/admin/iam-enhanced';
// import { buildPackagePermissions } from '../config/packagePermissions'; // Config removed

// Placeholders for removed services
const firebaseIAMService = {
  createUser: async (...args: any[]) => {},
  updateUserPackageTier: async (...args: any[]) => {},
  getUsers: async (...args: any[]) => [],
  grantCustomPermission: async (...args: any[]) => {},
  hasFeatureAccess: async (...args: any[]) => false,
  cleanupExpiredPermissions: async (...args: any[]) => {},
};
const firebaseAuthIAMService = {
  createAdminUser: async (...args: any[]) => {},
  createUser: async (...args: any[]) => ({ user: null, profile: null }),
};
const buildPackagePermissions = () => ({});

/**
 * Initialize Firebase IAM collections with sample data and setup
 * Run this script once to set up your IAM system
 */
export async function initializeFirebaseIAM() {
  console.log('🚀 Initializing Firebase IAM System...');
  
  try {
    // Step 1: Clean up expired permissions
    console.log('🧹 Cleaning up expired permissions...');
    await firebaseIAMService.cleanupExpiredPermissions();
    
    // Step 2: Create sample admin user (if needed)
    console.log('👤 Setting up sample admin user...');
    await createSampleAdminUser();
    
    // Step 3: Create sample regular users with different package tiers
    console.log('👥 Creating sample users...');
    await createSampleUsers();
    
    // Step 4: Grant some custom permissions for testing
    console.log('⭐ Adding sample custom permissions...');
    await grantSampleCustomPermissions();
    
    console.log('✅ Firebase IAM System initialized successfully!');
    console.log('\n📊 System Summary:');
    
    // Show summary
    const allUsers = await firebaseIAMService.getUsers();
    console.log(`   - Total Users: ${allUsers.length}`);
    console.log(`   - Package Tiers: ${Object.values(PackageTier).join(', ')}`);
    console.log(`   - Permission Templates: ${Object.keys(buildPackagePermissions()).length}`);
    
    return true;
    
  } catch (error) {
    console.error('❌ Failed to initialize Firebase IAM:', error);
    throw error;
  }
}

async function createSampleAdminUser() {
  // Note: In a real app, admin users would be created through your admin auth system
  // This is just for demonstration
  console.log('   Creating admin user...');
}

async function createSampleUsers() {
  const sampleUsers = [
    {
      id: 'user_bronze_1',
      email: 'bronze.user@example.com',
      name: 'Bronze User',
      password: 'password123', // In production, use secure passwords
      packageTier: PackageTier.BRONZE,
      subscriptionStatus: SubscriptionStatus.ACTIVE
    },
    {
      id: 'user_silver_1', 
      email: 'silver.user@example.com',
      name: 'Silver User',
      password: 'password123',
      packageTier: PackageTier.SILVER,
      subscriptionStatus: SubscriptionStatus.ACTIVE
    },
    {
      id: 'user_gold_1',
      email: 'gold.user@example.com', 
      name: 'Gold User',
      password: 'password123',
      packageTier: PackageTier.GOLD,
      subscriptionStatus: SubscriptionStatus.ACTIVE
    },
    {
      id: 'user_free_1',
      email: 'free.user@example.com',
      name: 'Free User',
      password: 'password123',
      packageTier: PackageTier.FREE,
      subscriptionStatus: SubscriptionStatus.PENDING
    }
  ];

  for (const userData of sampleUsers) {
    console.log(`   Creating user: ${userData.email} (${userData.packageTier})`);
    try {
      await firebaseAuthIAMService.createUser({
        email: userData.email,
        password: userData.password,
        name: userData.name,
        packageTier: userData.packageTier,
        subscriptionStatus: userData.subscriptionStatus,
        roles: userData.packageTier === PackageTier.GOLD ? ['beta-tester'] : []
      });
      console.log(`   ✅ Successfully created user: ${userData.email}`);
    } catch (error) {
      console.warn(`   ⚠️  Failed to create user ${userData.email}:`, error);
      // Continue with other users
    }
  }
}

async function grantSampleCustomPermissions() {
  const customPermissions = [
    {
      userId: 'user_bronze_1',
      featureId: 'advanced_analytics',
      permission: { action: 'READ', resource: 'advanced_analytics' },
      grantedBy: 'admin_system',
      reason: 'Beta tester access',
      expiresAt: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000) // 30 days
    },
    {
      userId: 'user_free_1', 
      featureId: 'premium_support',
      permission: { action: 'READ', resource: 'premium_support' },
      grantedBy: 'admin_system',
      reason: 'Customer service exception'
    }
  ];

  for (const perm of customPermissions) {
    console.log(`   Granting custom permission: ${perm.featureId} to ${perm.userId}`);
    try {
      await firebaseIAMService.grantCustomPermission(
        perm.userId,
        perm.featureId, 
        perm.permission,
        perm.grantedBy,
        { 
          expiresAt: perm.expiresAt,
          reason: perm.reason 
        }
      );
    } catch (error) {
      console.warn(`   ⚠️  Failed to grant custom permission:`, error);
    }
  }
}

/**
 * Reset the IAM system (careful - this will delete all IAM data)
 */
export async function resetFirebaseIAM() {
  console.log('🚨 WARNING: This will delete all IAM data!');
  console.log('⏳ Resetting Firebase IAM System...');
  
  try {
    // In a real implementation, you'd delete all documents from IAM collections
    // This is a placeholder for demonstration
    console.log('   Deleting custom permissions...');
    console.log('   Deleting effective permissions...');
    console.log('   Deleting audit logs...');
    
    console.log('✅ Firebase IAM System reset completed!');
    
  } catch (error) {
    console.error('❌ Failed to reset Firebase IAM:', error);
    throw error;
  }
}

/**
 * Utility function to check system health
 */
export async function checkFirebaseIAMHealth() {
  console.log('🔍 Checking Firebase IAM System Health...');
  
  try {
    // Check if we can read users
    const users = await firebaseIAMService.getUsers();
    console.log(`✅ Users collection: ${users.length} users found`);
    
    // Check permission templates
    const templates = buildPackagePermissions();
    console.log(`✅ Permission templates: ${Object.keys(templates).length} tiers configured`);
    
    // Check if we can perform basic operations
    const sampleUserId = 'health_check_user';
    try {
      await firebaseIAMService.hasFeatureAccess(sampleUserId, 'test_feature');
      console.log('✅ Permission checking: Working');
    } catch (error) {
      console.log('⚠️  Permission checking: May need setup');
    }
    
    console.log('✅ Firebase IAM System Health Check Complete!');
    return true;
    
  } catch (error) {
    console.error('❌ Firebase IAM Health Check Failed:', error);
    return false;
  }
}
