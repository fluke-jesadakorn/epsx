/**
 * Test User Permission Assignment Script
 * Assigns all necessary admin modules and permissions to the test user
 * Can be run standalone or as part of E2E test setup
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { env } from '@/config/env';

const execAsync = promisify(exec);

const TEST_EMAIL = 'jesadakorn.kirtnu@gmail.com';
const BACKEND_URL = env.getBackendUrl();

// All admin modules that should be assigned to test user
const REQUIRED_ADMIN_MODULES = [
  'system_admin',
  'user_management',
  'permission_management', 
  'iam_management',
  'analytics',
  'billing_management',
  'stock_ranking_management',
  'database_management',
  'developer_portal',
  'module_management'
];

// All permissions that should be assigned to test user
const REQUIRED_PERMISSIONS = [
  // System admin permissions
  'admin:read',
  'admin:write',
  'system:manage',
  'system:configure',
  
  // User management permissions
  'user:manage',
  'user:create',
  'user:edit',
  'user:delete',
  'user:read',
  
  // Permission management permissions
  'permission:manage',
  'permission:assign',
  'permission:revoke',
  'permission:read',
  
  // IAM permissions
  'iam:manage',
  'iam:read',
  'iam:write',
  
  // Analytics permissions
  'analytics:read',
  'analytics:export',
  'analytics:configure',
  
  // Billing permissions
  'billing:read',
  'billing:manage',
  'billing:export',
  
  // Stock ranking permissions
  'stock_ranking:manage',
  'stock_ranking:assign',
  'stock_ranking:read',
  
  // Database permissions
  'database:read',
  'database:manage',
  'database:backup',
  
  // Developer portal permissions
  'developer:read',
  'developer:manage',
  'api:read',
  'api:manage'
];

interface AssignmentResult {
  success: boolean;
  method: string;
  message: string;
  error?: string;
}

/**
 * Assign admin modules via API
 */
async function assignModulesViaAPI(email: string, modules: string[]): Promise<AssignmentResult> {
  try {
    console.log('🔄 Attempting to assign modules via API...');
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/assign-modules`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        // Note: In a real scenario, this would use proper authentication
        'Authorization': 'Bearer admin-test-token'
      },
      body: JSON.stringify({
        email: email,
        admin_modules: modules,
        reason: 'E2E test setup - comprehensive admin access required'
      })
    });

    if (response.ok) {
      const result = await response.json();
      return {
        success: true,
        method: 'API',
        message: `Successfully assigned ${modules.length} admin modules via API`
      };
    } else {
      const errorText = await response.text();
      return {
        success: false,
        method: 'API',
        message: 'API assignment failed',
        error: errorText
      };
    }
  } catch (error) {
    return {
      success: false,
      method: 'API',
      message: 'API assignment failed',
      error: error.message
    };
  }
}

/**
 * Assign permissions via API
 */
async function assignPermissionsViaAPI(email: string, permissions: string[]): Promise<AssignmentResult> {
  try {
    console.log('🔄 Attempting to assign permissions via API...');
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/assign-permissions`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': 'Bearer admin-test-token'
      },
      body: JSON.stringify({
        email: email,
        permissions: permissions,
        reason: 'E2E test setup - comprehensive permissions required'
      })
    });

    if (response.ok) {
      return {
        success: true,
        method: 'API',
        message: `Successfully assigned ${permissions.length} permissions via API`
      };
    } else {
      const errorText = await response.text();
      return {
        success: false,
        method: 'API',
        message: 'Permission assignment failed',
        error: errorText
      };
    }
  } catch (error) {
    return {
      success: false,
      method: 'API',
      message: 'Permission assignment failed',
      error: error.message
    };
  }
}

/**
 * Assign modules via database query (fallback method)
 */
async function assignModulesViaDatabase(email: string, modules: string[]): Promise<AssignmentResult> {
  try {
    console.log('🔄 Attempting to assign modules via database...');
    
    const modulesJson = JSON.stringify(modules);
    const updateQuery = `
      UPDATE users 
      SET admin_modules = '${modulesJson}'::jsonb,
          role = 'admin',
          updated_at = NOW()
      WHERE email = '${email}';
    `;
    
    // Execute via psql (assuming PostgreSQL)
    const { stdout, stderr } = await execAsync(
      `psql ${env.DATABASE_URL || 'postgresql://localhost/epsx'} -c "${updateQuery}"`
    );
    
    if (stderr && !stderr.includes('NOTICE')) {
      return {
        success: false,
        method: 'Database',
        message: 'Database assignment failed',
        error: stderr
      };
    }
    
    return {
      success: true,
      method: 'Database',
      message: `Successfully assigned ${modules.length} admin modules via database`
    };
  } catch (error) {
    return {
      success: false,
      method: 'Database',
      message: 'Database assignment failed',
      error: error.message
    };
  }
}

/**
 * Assign permissions via database query (fallback method)
 */
async function assignPermissionsViaDatabase(email: string, permissions: string[]): Promise<AssignmentResult> {
  try {
    console.log('🔄 Attempting to assign permissions via database...');
    
    const permissionsJson = JSON.stringify(permissions);
    const updateQuery = `
      UPDATE users 
      SET permissions = '${permissionsJson}'::jsonb,
          updated_at = NOW()
      WHERE email = '${email}';
    `;
    
    const { stdout, stderr } = await execAsync(
      `psql ${env.DATABASE_URL || 'postgresql://localhost/epsx'} -c "${updateQuery}"`
    );
    
    if (stderr && !stderr.includes('NOTICE')) {
      return {
        success: false,
        method: 'Database',
        message: 'Database permission assignment failed',
        error: stderr
      };
    }
    
    return {
      success: true,
      method: 'Database',
      message: `Successfully assigned ${permissions.length} permissions via database`
    };
  } catch (error) {
    return {
      success: false,
      method: 'Database',
      message: 'Database permission assignment failed',
      error: error.message
    };
  }
}

/**
 * Create user if doesn't exist
 */
async function createUserIfNotExists(email: string): Promise<AssignmentResult> {
  try {
    console.log('🔄 Checking if user exists, creating if necessary...');
    
    const createUserQuery = `
      INSERT INTO users (
        id, 
        email, 
        name, 
        role, 
        admin_modules, 
        permissions,
        firebase_uid,
        created_at,
        updated_at
      ) VALUES (
        gen_random_uuid(),
        '${email}',
        'Test Admin User',
        'admin',
        '${JSON.stringify(REQUIRED_ADMIN_MODULES)}'::jsonb,
        '${JSON.stringify(REQUIRED_PERMISSIONS)}'::jsonb,
        'test-admin-uid',
        NOW(),
        NOW()
      ) ON CONFLICT (email) DO UPDATE SET
        admin_modules = '${JSON.stringify(REQUIRED_ADMIN_MODULES)}'::jsonb,
        permissions = '${JSON.stringify(REQUIRED_PERMISSIONS)}'::jsonb,
        role = 'admin',
        updated_at = NOW();
    `;
    
    const { stdout, stderr } = await execAsync(
      `psql ${env.DATABASE_URL || 'postgresql://localhost/epsx'} -c "${createUserQuery}"`
    );
    
    if (stderr && !stderr.includes('NOTICE')) {
      return {
        success: false,
        method: 'Database',
        message: 'User creation/update failed',
        error: stderr
      };
    }
    
    return {
      success: true,
      method: 'Database',
      message: 'User created/updated successfully with full permissions'
    };
  } catch (error) {
    return {
      success: false,
      method: 'Database',
      message: 'User creation/update failed',
      error: error.message
    };
  }
}

/**
 * Main assignment function with multiple fallback methods
 */
async function assignTestUserPermissions(email: string): Promise<void> {
  console.log('🚀 Starting test user permission assignment');
  console.log(`📧 Target user: ${email}`);
  console.log(`🔧 Backend URL: ${BACKEND_URL}`);
  console.log(`📦 Admin modules to assign: ${REQUIRED_ADMIN_MODULES.length}`);
  console.log(`🔐 Permissions to assign: ${REQUIRED_PERMISSIONS.length}`);
  console.log('='.repeat(60));

  let successCount = 0;
  const results: AssignmentResult[] = [];

  // Method 1: Try API assignment for modules
  console.log('\n📡 Method 1: API Assignment');
  const moduleApiResult = await assignModulesViaAPI(email, REQUIRED_ADMIN_MODULES);
  results.push(moduleApiResult);
  if (moduleApiResult.success) {
    successCount++;
    console.log(`✅ ${moduleApiResult.message}`);
  } else {
    console.log(`❌ ${moduleApiResult.message}: ${moduleApiResult.error}`);
  }

  // Method 2: Try API assignment for permissions
  const permissionApiResult = await assignPermissionsViaAPI(email, REQUIRED_PERMISSIONS);
  results.push(permissionApiResult);
  if (permissionApiResult.success) {
    successCount++;
    console.log(`✅ ${permissionApiResult.message}`);
  } else {
    console.log(`❌ ${permissionApiResult.message}: ${permissionApiResult.error}`);
  }

  // Method 3: Fallback to database if API failed
  if (!moduleApiResult.success || !permissionApiResult.success) {
    console.log('\n🗄️ Method 2: Database Fallback');
    
    // Try creating/updating user with all permissions at once
    const userCreationResult = await createUserIfNotExists(email);
    results.push(userCreationResult);
    if (userCreationResult.success) {
      successCount++;
      console.log(`✅ ${userCreationResult.message}`);
    } else {
      console.log(`❌ ${userCreationResult.message}: ${userCreationResult.error}`);
      
      // Try individual database assignments if bulk creation failed
      if (!moduleApiResult.success) {
        const moduleDbResult = await assignModulesViaDatabase(email, REQUIRED_ADMIN_MODULES);
        results.push(moduleDbResult);
        if (moduleDbResult.success) {
          successCount++;
          console.log(`✅ ${moduleDbResult.message}`);
        } else {
          console.log(`❌ ${moduleDbResult.message}: ${moduleDbResult.error}`);
        }
      }
      
      if (!permissionApiResult.success) {
        const permissionDbResult = await assignPermissionsViaDatabase(email, REQUIRED_PERMISSIONS);
        results.push(permissionDbResult);
        if (permissionDbResult.success) {
          successCount++;
          console.log(`✅ ${permissionDbResult.message}`);
        } else {
          console.log(`❌ ${permissionDbResult.message}: ${permissionDbResult.error}`);
        }
      }
    }
  }

  // Print summary
  console.log('\n📊 ASSIGNMENT SUMMARY:');
  console.log('='.repeat(60));
  console.log(`✅ Successful operations: ${successCount}`);
  console.log(`❌ Failed operations: ${results.length - successCount}`);
  console.log(`📈 Success rate: ${((successCount / results.length) * 100).toFixed(2)}%`);

  console.log('\n📋 DETAILED RESULTS:');
  results.forEach((result, index) => {
    const statusIcon = result.success ? '✅' : '❌';
    console.log(`${statusIcon} ${index + 1}. ${result.method}: ${result.message}`);
    if (result.error) {
      console.log(`   Error: ${result.error}`);
    }
  });

  if (successCount > 0) {
    console.log('\n🎉 Permission assignment completed with some success!');
    console.log('💡 If some assignments failed, the E2E tests will attempt automatic retry');
  } else {
    console.log('\n⚠️ All permission assignments failed!');
    console.log('🔧 Manual intervention may be required');
    console.log('📝 Please check:');
    console.log('   - Backend server is running');
    console.log('   - Database is accessible');
    console.log('   - Environment variables are set correctly');
  }
}

/**
 * Verify assigned permissions
 */
async function verifyAssignedPermissions(email: string): Promise<void> {
  try {
    console.log('\n🔍 Verifying assigned permissions...');
    
    // Try to get user info via API
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users?email=${email}`, {
      headers: {
        'Authorization': 'Bearer admin-test-token'
      }
    });
    
    if (response.ok) {
      const userData = await response.json();
      console.log('✅ User data retrieved successfully');
      console.log(`📦 Admin modules: ${JSON.stringify(userData.admin_modules || [])}`);
      console.log(`🔐 Permissions: ${JSON.stringify(userData.permissions || [])}`);
    } else {
      console.log('⚠️ Could not verify permissions via API');
    }
  } catch (error) {
    console.log(`⚠️ Permission verification failed: ${error.message}`);
  }
}

// Export functions for use in tests
export {
  assignTestUserPermissions,
  assignModulesViaAPI,
  assignPermissionsViaAPI,
  verifyAssignedPermissions,
  REQUIRED_ADMIN_MODULES,
  REQUIRED_PERMISSIONS,
  TEST_EMAIL
};

// Run script if called directly
if (require.main === module) {
  assignTestUserPermissions(TEST_EMAIL)
    .then(() => verifyAssignedPermissions(TEST_EMAIL))
    .catch(console.error);
}