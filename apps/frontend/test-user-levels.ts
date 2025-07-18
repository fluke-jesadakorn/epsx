// Test file to verify user level system alignment
import { UserLevel } from '../admin-frontend/types/admin/userLevels';
import type { UserLevelType } from './app/constants/packages';
import { LEVEL_CONFIGS } from './app/constants/packages';

// Test type compatibility between admin and frontend
console.log('Testing user level system alignment...');

// Test that all admin levels are available in frontend
const adminLevels = Object.values(UserLevel);
console.log('Admin levels:', adminLevels);

// Test that frontend levels include all admin levels
const frontendLevels = Object.keys(LEVEL_CONFIGS) as UserLevelType[];
console.log('Frontend levels:', frontendLevels);

// Check if all admin levels are supported in frontend
adminLevels.forEach(adminLevel => {
  const isSupported = frontendLevels.includes(adminLevel as UserLevelType);
  console.log(`${adminLevel}: ${isSupported ? '✅ Supported' : '❌ Not supported'}`);
});

// Test level configurations
console.log('\nLevel configurations:');
frontendLevels.forEach(level => {
  const config = LEVEL_CONFIGS[level];
  console.log(`${level}:`, {
    name: config.name,
    numericLevel: config.numericLevel,
    rankingLimit: config.rankingLimit,
    color: config.color
  });
});

console.log('\n✅ User level system alignment test complete!');
